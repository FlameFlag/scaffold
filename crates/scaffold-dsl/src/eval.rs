use std::path::PathBuf;

use scheme_rs::{
    env::{ImportPolicy, TopLevelEnvironment},
    exceptions::Exception,
    runtime::Runtime,
    syntax::Syntax,
    value::Value,
};

use scaffold_diagnostic::SourceDiagnostic;
use scaffold_scheme::{parse_error_offset, parse_source};

use super::json::{is_top_level_syntax_definition, top_level_forms, value_is_null, value_to_json};
use super::libraries::{load_bundled_libraries, load_user_libraries};
use super::stdlib::{
    define_context_libraries, define_core_libraries, define_scheme_libraries, import_policy,
};
use super::{CatalogDocument, DslError, Result, SourceSpan, host, workspace};

#[derive(Clone, Debug)]
pub(crate) struct DslEvalContext {
    pub(super) host: host::Host,
    pub(super) workspace_root: Option<PathBuf>,
    pub(super) source_path: Option<PathBuf>,
    pub(super) mode: DslEvalMode,
}

impl DslEvalContext {
    pub(super) const fn new(workspace_root: Option<PathBuf>, source_path: Option<PathBuf>) -> Self {
        Self {
            host: host::Host::current(),
            workspace_root,
            source_path,
            mode: DslEvalMode::Catalog,
        }
    }

    #[allow(dead_code)]
    pub(crate) const fn with_mode(mut self, mode: DslEvalMode) -> Self {
        self.mode = mode;
        self
    }
}

impl Default for DslEvalContext {
    fn default() -> Self {
        Self::new(None, None)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum DslEvalMode {
    Catalog,
    Test,
    Editor,
    Wasm,
}

impl DslEvalMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Catalog => "catalog",
            Self::Test => "test",
            Self::Editor => "editor",
            Self::Wasm => "wasm",
        }
    }

    const fn allows_evaluation(self) -> bool {
        matches!(self, Self::Catalog | Self::Test)
    }
}

pub struct DslSession {
    env: TopLevelEnvironment,
    import_policy: ImportPolicy,
}

impl DslSession {
    pub(crate) fn with_context(
        extension_dirs: &[PathBuf],
        default_imports: bool,
        context: DslEvalContext,
    ) -> Result<Self> {
        if !context.mode.allows_evaluation() {
            return Err(DslError::Eval(format!(
                "Scaffold Scheme evaluation is not available in {} mode",
                context.mode.as_str()
            )));
        }

        let runtime = Runtime::new();
        let bundled_libraries = load_bundled_libraries()?;
        let user_libraries = load_user_libraries(extension_dirs)?;
        define_core_libraries(&runtime).map_err(scheme_error)?;
        runtime
            .def_lib(&host::host_library_source(&context.host))
            .map_err(scheme_error)?;
        runtime
            .def_lib(&workspace::workspace_library_source(
                context.workspace_root.as_deref(),
                context.source_path.as_deref(),
            ))
            .map_err(scheme_error)?;
        define_context_libraries(&runtime).map_err(scheme_error)?;
        define_scheme_libraries(&runtime, &bundled_libraries).map_err(scheme_error)?;
        define_scheme_libraries(&runtime, &user_libraries).map_err(scheme_error)?;

        let env = TopLevelEnvironment::new_repl(&runtime);
        let session = Self {
            env,
            import_policy: import_policy(&bundled_libraries, &user_libraries),
        };
        if default_imports {
            session.import_default_libraries()?;
        }
        Ok(session)
    }

    pub fn eval_json(
        &self,
        text: &str,
        source_name: Option<&str>,
    ) -> Result<Vec<serde_json::Value>> {
        self.eval_values(text, source_name)?
            .into_iter()
            .enumerate()
            .filter_map(|(index, value)| {
                if value.is_undefined() || value_is_null(&value) {
                    return None;
                }
                match value_to_json(value, &format!("$[{index}]")) {
                    Ok(value) if json_is_scaffold_doc(&value) => None,
                    result => Some(result),
                }
            })
            .collect()
    }

    pub(crate) fn eval_values(&self, text: &str, source_name: Option<&str>) -> Result<Vec<Value>> {
        let source_name = source_name.unwrap_or("<unknown>");
        let syntax = parse_syntax(text, source_name)?;
        let forms = top_level_forms(&syntax)?;
        forms
            .iter()
            .enumerate()
            .filter_map(|(index, form)| {
                self.eval_form_value(text, source_name, index, form)
                    .transpose()
            })
            .collect()
    }

    fn eval_form_value(
        &self,
        text: &str,
        source_name: &str,
        index: usize,
        form: &Syntax,
    ) -> Result<Option<Value>> {
        if is_top_level_doc_form(form) {
            return Ok(None);
        }
        let values = self
            .env
            .eval_sexpr(self.import_policy.clone(), form.clone())
            .map_err(|err| eval_diagnostic(err, source_name, text, form))?;
        match values.as_slice() {
            [] => Ok(None),
            [value] if value.is_undefined() => Ok(None),
            [value] if value_is_null(value) && is_top_level_syntax_definition(form) => Ok(None),
            [value] => Ok(Some(value.clone())),
            values => Err(DslError::Shape {
                path: format!("$[{index}]"),
                message: format!("expected one value, got {}", values.len()),
            }),
        }
    }

    fn import_default_libraries(&self) -> Result<()> {
        for import_set in ["(library (rnrs))", "(library (scaffold catalog))"] {
            let import_set = import_set
                .parse()
                .map_err(|err| DslError::Eval(format!("invalid default import set: {err}")))?;
            self.env.import(import_set).map_err(scheme_error)?;
        }
        Ok(())
    }
}

#[cfg(any(test, feature = "test-support"))]
pub(super) fn values_from_str_with_extension_dirs(
    text: &str,
    source_name: Option<&str>,
    extension_dirs: &[PathBuf],
) -> Result<Vec<serde_json::Value>> {
    values_from_str_with_context(
        text,
        source_name,
        extension_dirs,
        DslEvalContext::default().with_mode(DslEvalMode::Test),
    )
}

pub(super) fn values_from_str_with_context(
    text: &str,
    source_name: Option<&str>,
    extension_dirs: &[PathBuf],
    context: DslEvalContext,
) -> Result<Vec<serde_json::Value>> {
    let session = DslSession::with_context(extension_dirs, false, context)?;
    let syntax = parse_syntax(text, source_name.unwrap_or("<unknown>"))?;
    let forms = top_level_forms(&syntax)?;
    let source_name = source_name.unwrap_or("<unknown>");
    forms
        .iter()
        .enumerate()
        .filter_map(|(index, form)| {
            session
                .eval_form_json(text, source_name, index, form)
                .transpose()
        })
        .collect()
}

pub(super) fn catalog_document_from_str_with_context(
    text: String,
    source_name: String,
    extension_dirs: &[PathBuf],
    context: DslEvalContext,
) -> Result<CatalogDocument> {
    let session = DslSession::with_context(extension_dirs, false, context)?;
    let syntax = parse_syntax(&text, &source_name)?;
    let forms = top_level_forms(&syntax)?;
    let form_spans = top_level_source_spans(&text, forms);
    let mut values = Vec::new();
    let mut value_spans = Vec::new();

    for (index, form) in forms.iter().enumerate() {
        if let Some(value) = session.eval_form_json(&text, &source_name, index, form)? {
            values.push(value);
            value_spans.push(form_spans.get(index).copied().unwrap_or(SourceSpan {
                offset: form_source_start(&text, form.span().offset),
                len: form_source_len(&text, form_source_start(&text, form.span().offset)),
            }));
        }
    }

    let implicit_tools = !matches!(values.as_slice(), [value] if value.get("tools").is_some());
    Ok(CatalogDocument {
        value: catalog_value_from_values(values)?,
        source_name,
        source_text: text,
        value_spans,
        implicit_tools,
    })
}

pub(super) fn catalog_value_from_values(
    values: Vec<serde_json::Value>,
) -> Result<serde_json::Value> {
    match values.as_slice() {
        [] => Err(DslError::Shape {
            path: "$".to_owned(),
            message: "expected at least one catalog value".to_owned(),
        }),
        [value] if value.get("tools").is_some() => Ok(value.clone()),
        _ => Ok(serde_json::json!({ "tools": values })),
    }
}

fn json_is_scaffold_doc(value: &serde_json::Value) -> bool {
    value
        .get("scaffold:kind")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|kind| kind == "doc")
}

impl DslSession {
    fn eval_form_json(
        &self,
        text: &str,
        source_name: &str,
        index: usize,
        form: &Syntax,
    ) -> Result<Option<serde_json::Value>> {
        self.eval_form_value(text, source_name, index, form)?
            .map(|value| value_to_json(value, &format!("$[{index}]")))
            .transpose()
            .map(|value| value.filter(|value| !json_is_scaffold_doc(value)))
    }
}

fn is_top_level_doc_form(form: &Syntax) -> bool {
    form.as_list()
        .and_then(|items| items.first())
        .and_then(Syntax::as_ident)
        .is_some_and(|ident| {
            ident == "doc"
                || ident == "doc-next"
                || ident == "extern-doc"
                || ident == "moduledoc"
                || ident == "typedoc"
        })
}

fn parse_syntax(text: &str, source_name: &str) -> Result<Syntax> {
    parse_source(text, source_name).map_err(|err| {
        SourceDiagnostic::syntax(
            source_name,
            text.to_owned(),
            parse_error_offset(&err, text),
            1,
            format!("Scheme syntax failed: {err}"),
        )
        .into()
    })
}

fn eval_diagnostic(error: Exception, source_name: &str, text: &str, form: &Syntax) -> DslError {
    SourceDiagnostic::eval(
        source_name,
        text.to_owned(),
        form.span().offset,
        form_source_len(text, form.span().offset),
        format!("Scheme evaluation failed: {error}"),
    )
    .into()
}

fn form_source_len(text: &str, offset: usize) -> usize {
    text.get(offset..)
        .unwrap_or_default()
        .find('\n')
        .unwrap_or_else(|| text.len().saturating_sub(offset))
        .max(1)
}

fn top_level_source_spans(text: &str, forms: &[Syntax]) -> Vec<SourceSpan> {
    let starts = forms
        .iter()
        .map(|form| form_source_start(text, form.span().offset))
        .collect::<Vec<_>>();
    starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let end = starts.get(index + 1).copied().unwrap_or(text.len());
            let len = text
                .get(*start..end)
                .map(str::trim_end)
                .map_or_else(|| end.saturating_sub(*start), str::len)
                .max(1);
            SourceSpan {
                offset: *start,
                len,
            }
        })
        .collect()
}

fn form_source_start(text: &str, offset: usize) -> usize {
    if offset > 0
        && text
            .as_bytes()
            .get(offset - 1)
            .is_some_and(is_open_delimiter)
    {
        offset - 1
    } else {
        offset
    }
}

const fn is_open_delimiter(byte: &u8) -> bool {
    matches!(byte, b'(' | b'[')
}

fn scheme_error(error: Exception) -> DslError {
    DslError::Eval(error.to_string())
}
