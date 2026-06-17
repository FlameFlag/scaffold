use std::fmt;

use miette::{Diagnostic, LabeledSpan, NamedSource, Severity, SourceCode, SourceSpan};
use thiserror::Error;

#[derive(Debug, Clone, Error)]
#[error("{message}")]
pub struct SourceDiagnostic {
    src: NamedSource<String>,
    message: String,
    code: &'static str,
    severity: Severity,
    labels: Vec<LabeledSpan>,
    help: String,
}

impl Diagnostic for SourceDiagnostic {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(self.code))
    }

    fn severity(&self) -> Option<Severity> {
        Some(self.severity)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(&self.help))
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.src)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(self.labels.clone().into_iter()))
    }
}

impl SourceDiagnostic {
    fn new(
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        code: &'static str,
        severity: Severity,
        message: impl Into<String>,
        labels: Vec<LabeledSpan>,
        help: impl Into<String>,
    ) -> Self {
        Self {
            src: NamedSource::new(source_name.into(), source_text.into()),
            message: message.into(),
            code,
            severity,
            labels,
            help: help.into(),
        }
    }

    pub fn syntax(
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        offset: usize,
        len: usize,
        message: impl Into<String>,
    ) -> Self {
        let source_name = source_name.into();
        Self::new(
            source_name,
            source_text,
            "scaffold::dsl::syntax",
            Severity::Error,
            message,
            vec![primary_label(offset, len, "syntax error starts here")],
            "fix the Scheme syntax before Scaffold can evaluate this file",
        )
    }

    pub fn eval(
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        offset: usize,
        len: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            source_name,
            source_text,
            "scaffold::dsl::eval",
            Severity::Error,
            message,
            vec![primary_label(
                offset,
                len,
                "evaluation failed while processing this form",
            )],
            "check the imported libraries, binding names, argument counts, and value shapes",
        )
    }

    pub fn catalog_validation(
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        offset: usize,
        len: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            source_name,
            source_text,
            "scaffold::catalog::validation",
            Severity::Error,
            message,
            vec![primary_label(
                offset,
                len,
                "invalid catalog data was produced here",
            )],
            "fix the catalog object shape, field names, or tool relationships",
        )
    }

    pub fn missing_doc(
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        offset: usize,
        len: usize,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            source_name,
            source_text,
            "scaffold::dsl::missing-doc",
            Severity::Warning,
            message,
            vec![primary_label(
                offset,
                len,
                "definition has no matching documentation entry",
            )],
            "add a `(doc-next ...)` form near the definition or hide generated/internal helpers",
        )
    }

    pub fn duplicate_definition(
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        name: &str,
        first_offset: usize,
        first_len: usize,
        duplicate_offset: usize,
        duplicate_len: usize,
    ) -> Self {
        Self::new(
            source_name,
            source_text,
            "scaffold::dsl::duplicate-definition",
            Severity::Error,
            format!("`{name}` is defined more than once"),
            vec![
                primary_label(duplicate_offset, duplicate_len, "duplicate definition"),
                secondary_label(first_offset, first_len, "first definition is here"),
            ],
            "rename one definition or remove the duplicate binding",
        )
    }

    pub fn duplicate_doc(
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        name: &str,
        first_offset: usize,
        first_len: usize,
        duplicate_offset: usize,
        duplicate_len: usize,
    ) -> Self {
        Self::new(
            source_name,
            source_text,
            "scaffold::dsl::duplicate-doc",
            Severity::Warning,
            format!("`{name}` has more than one documentation entry"),
            vec![
                primary_label(duplicate_offset, duplicate_len, "duplicate doc entry"),
                secondary_label(first_offset, first_len, "first doc entry is here"),
            ],
            "merge the documentation into one `(doc ...)` form so generated references are deterministic",
        )
    }

    pub fn missing_doc_summary(
        source_name: impl Into<String>,
        source_text: impl Into<String>,
        name: &str,
        offset: usize,
        len: usize,
    ) -> Self {
        Self::new(
            source_name,
            source_text,
            "scaffold::dsl::doc-missing-summary",
            Severity::Warning,
            format!("documentation for `{name}` is missing a summary"),
            vec![primary_label(
                offset,
                len,
                "doc entry needs a `(summary ...)` field",
            )],
            "add a concise `(summary \"...\")` field so completions and generated docs have useful text",
        )
    }

    #[must_use]
    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    #[must_use]
    pub const fn code_str(&self) -> &'static str {
        self.code
    }

    #[must_use]
    pub const fn severity_label(&self) -> &'static str {
        match self.severity {
            Severity::Advice => "advice",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }

    #[must_use]
    pub fn help_text(&self) -> &str {
        &self.help
    }
}

fn primary_label(offset: usize, len: usize, label: impl Into<String>) -> LabeledSpan {
    LabeledSpan::new_primary_with_span(Some(label.into()), source_span(offset, len))
}

fn secondary_label(offset: usize, len: usize, label: impl Into<String>) -> LabeledSpan {
    LabeledSpan::new_with_span(Some(label.into()), source_span(offset, len))
}

fn source_span(offset: usize, len: usize) -> SourceSpan {
    (offset, len.max(1)).into()
}

#[cfg(test)]
mod tests {
    use super::SourceDiagnostic;

    #[test]
    fn exposes_stable_code_severity_and_help_text() {
        let diagnostic = SourceDiagnostic::syntax("test.scm", "(bad", 0, 1, "bad syntax");

        assert_eq!(diagnostic.code_str(), "scaffold::dsl::syntax");
        assert_eq!(diagnostic.severity_label(), "error");
        assert!(diagnostic.help_text().contains("fix the Scheme syntax"));
        assert!(diagnostic.is_error());
    }
}
