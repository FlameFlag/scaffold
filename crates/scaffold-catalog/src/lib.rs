use std::path::Path;

use miette::Diagnostic;
use serde::Deserialize;
use thiserror::Error;

use scaffold_diagnostic::SourceDiagnostic;
use scaffold_dsl as dsl;

mod action;
mod archive;
mod package;
mod schema;
#[cfg(test)]
mod tests;
mod tool;

pub use action::{Action, Phase};
pub use archive::ArchiveAction;
pub use package::PackageAction;
pub use tool::Tool;

pub use schema::catalog_schema;

#[derive(Debug, Error, Diagnostic)]
pub enum CatalogError {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Diagnostic(Box<SourceDiagnostic>),
    #[error(transparent)]
    Dsl(#[from] dsl::DslError),
    #[error("catalog: {0}")]
    Invalid(String),
}

#[derive(Debug, Deserialize)]
pub struct Catalog {
    pub tools: Vec<Tool>,
}

impl Catalog {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, CatalogError> {
        let document = dsl::catalog_document_from_path(path)?;
        Self::from_document(document)
    }

    pub fn from_value(value: serde_json::Value) -> Result<Self, CatalogError> {
        schema::validate_catalog_value(&value).map_err(CatalogError::Invalid)?;
        Self::from_validated_value(value)
    }

    fn from_document(document: dsl::CatalogDocument) -> Result<Self, CatalogError> {
        if let Err(message) = schema::validate_catalog_value(&document.value) {
            return Err(CatalogError::Diagnostic(Box::new(
                catalog_validation_diagnostic(&document, message),
            )));
        }
        Self::from_value(document.value)
    }

    fn from_validated_value(value: serde_json::Value) -> Result<Self, CatalogError> {
        let mut catalog: Self = serde_path_to_error::deserialize(value)
            .map_err(|err| CatalogError::Invalid(err.to_string()))?;
        catalog.apply_defaults()?;
        Ok(catalog)
    }

    fn apply_defaults(&mut self) -> Result<(), CatalogError> {
        if self.tools.is_empty() {
            return Err(CatalogError::Invalid(
                "catalog must contain at least one tool".to_owned(),
            ));
        }

        for tool in &mut self.tools {
            tool.apply_defaults()?;
        }

        Ok(())
    }
}

fn catalog_validation_diagnostic(
    document: &dsl::CatalogDocument,
    message: String,
) -> SourceDiagnostic {
    let span = catalog_error_span(document, &message);
    SourceDiagnostic::catalog_validation(
        &document.source_name,
        document.source_text.clone(),
        span.offset,
        span.len,
        format!("Catalog validation failed: {message}"),
    )
}

fn catalog_error_span(document: &dsl::CatalogDocument, message: &str) -> dsl::SourceSpan {
    let fallback = document
        .value_spans
        .first()
        .copied()
        .unwrap_or(dsl::SourceSpan {
            offset: 0,
            len: document.source_text.len().max(1),
        });
    let Some(path) = catalog_error_path(message) else {
        return fallback;
    };
    let tool_index = tool_index_from_path(path);
    if let Some(index) = tool_index
        && let Some(tool_span) = catalog_tool_span(document, index)
    {
        if let Some(field) = catalog_field_from_path(path)
            && let Some(field_span) = catalog_field_span(document, tool_span, field)
        {
            return field_span;
        }
        return tool_span;
    }
    fallback
}

fn catalog_error_path(message: &str) -> Option<&str> {
    let message = message.strip_prefix('$')?;
    let end = message.find(char::is_whitespace).unwrap_or(message.len());
    Some(&message[..end])
}

fn tool_index_from_path(path: &str) -> Option<usize> {
    path.strip_prefix(".tools[")
        .and_then(|rest| rest.split_once(']'))
        .and_then(|(index, _)| index.parse::<usize>().ok())
}

fn catalog_field_from_path(path: &str) -> Option<&str> {
    path.strip_prefix(".tools[")
        .and_then(|rest| rest.split_once("]."))
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split(['.', '[']).next())
        .filter(|field| !field.is_empty())
}

fn catalog_tool_span(document: &dsl::CatalogDocument, index: usize) -> Option<dsl::SourceSpan> {
    if document.implicit_tools {
        return document.value_spans.get(index).copied();
    }
    let catalog_span = document.value_spans.first().copied()?;
    nth_catalog_tool_span(&document.source_text, catalog_span, index)
}

fn catalog_field_span(
    document: &dsl::CatalogDocument,
    tool_span: dsl::SourceSpan,
    field: &str,
) -> Option<dsl::SourceSpan> {
    let field_form = find_field_form_span(&document.source_text, tool_span, field);
    field_form.or_else(|| find_helper_field_span(&document.source_text, tool_span, field))
}

fn find_field_form_span(
    source: &str,
    parent: dsl::SourceSpan,
    field: &str,
) -> Option<dsl::SourceSpan> {
    let bounded = bounded_source(source, parent)?;
    let patterns = [
        format!("(field '{field}"),
        format!("(field \"{field}\""),
        format!("(field {field}"),
    ];
    patterns.iter().find_map(|pattern| {
        bounded
            .find(pattern)
            .map(|relative| list_span_at(source, parent.offset + relative))
    })
}

fn find_helper_field_span(
    source: &str,
    parent: dsl::SourceSpan,
    field: &str,
) -> Option<dsl::SourceSpan> {
    let bounded = bounded_source(source, parent)?;
    let helpers = match field {
        "depends" => &["depends"][..],
        "before" => &["install/before"][..],
        "after" => &["install/after"][..],
        "platforms" => &["tool/platforms"][..],
        "uninstall" => &["uninstall", "uninstall/paths"][..],
        "checks" => &["check", "host/check"][..],
        "bins" => &["tool/append-bins", "bin", "bin/version"][..],
        "meta" => &["meta"][..],
        "passthru" => &["passthru"][..],
        _ => &[][..],
    };
    helpers.iter().find_map(|helper| {
        let pattern = format!("({helper}");
        bounded
            .find(&pattern)
            .map(|relative| list_span_at(source, parent.offset + relative))
    })
}

fn nth_catalog_tool_span(
    source: &str,
    parent: dsl::SourceSpan,
    index: usize,
) -> Option<dsl::SourceSpan> {
    let bounded = bounded_source(source, parent)?;
    bounded
        .match_indices("(tool")
        .filter(|(offset, _)| list_head_matches(bounded, *offset, "tool"))
        .nth(index)
        .map(|(offset, _)| list_span_at(source, parent.offset + offset))
}

fn list_head_matches(source: &str, offset: usize, head: &str) -> bool {
    let after_open = offset + 1;
    let end = after_open + head.len();
    source.get(after_open..end) == Some(head)
        && source
            .as_bytes()
            .get(end)
            .is_some_and(|byte| byte.is_ascii_whitespace() || *byte == b')')
}

fn bounded_source(source: &str, span: dsl::SourceSpan) -> Option<&str> {
    source.get(span.offset..span.offset.saturating_add(span.len))
}

fn list_span_at(source: &str, absolute_offset: usize) -> dsl::SourceSpan {
    let relative_offset = absolute_offset.min(source.len());
    let bytes = source.as_bytes();
    let len = scan_list_len(bytes, relative_offset);
    dsl::SourceSpan {
        offset: absolute_offset,
        len,
    }
}

fn scan_list_len(bytes: &[u8], mut index: usize) -> usize {
    let start = index;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            index += 1;
            continue;
        }
        match byte {
            b'"' => {
                in_string = true;
                index += 1;
            }
            b';' => {
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            b'(' => {
                depth += 1;
                index += 1;
            }
            b')' => {
                depth = depth.saturating_sub(1);
                index += 1;
                if depth == 0 {
                    break;
                }
            }
            _ => index += 1,
        }
    }
    index.saturating_sub(start).max(1)
}
