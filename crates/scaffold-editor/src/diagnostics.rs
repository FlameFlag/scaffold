pub const SYNTAX_CODE: &str = "scaffold::dsl::syntax";
pub const MISSING_DOC_CODE: &str = "scaffold::dsl::missing-doc";

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Diagnostic {
    pub message: String,
    pub offset: usize,
    pub length: usize,
    pub severity: DiagnosticSeverity,
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<DiagnosticData>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DiagnosticData {
    pub name: String,
    pub line: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxIssue {
    pub message: String,
    pub offset: usize,
    pub length: usize,
}

#[must_use]
pub fn missing_doc_message(name: &str) -> String {
    format!(
        "Document `{name}` with `(doc-next ...)` so editor help and generated references stay useful."
    )
}

#[must_use]
pub fn document_diagnostics(text: &str, syntax_issue: Option<SyntaxIssue>) -> Vec<Diagnostic> {
    match syntax_issue {
        Some(issue) => syntax_diagnostics(issue),
        None => missing_doc_diagnostics(text),
    }
}

#[must_use]
pub fn syntax_diagnostics(issue: SyntaxIssue) -> Vec<Diagnostic> {
    vec![Diagnostic {
        message: issue.message,
        offset: issue.offset,
        length: issue.length.max(1),
        severity: DiagnosticSeverity::Error,
        code: SYNTAX_CODE,
        data: None,
    }]
}

#[must_use]
pub fn missing_doc_diagnostics(text: &str) -> Vec<Diagnostic> {
    let documented = crate::syntax::documented_symbols(text)
        .into_iter()
        .collect::<std::collections::HashSet<_>>();
    crate::syntax::definitions(text)
        .into_iter()
        .filter(|definition| !documented.contains(&definition.name))
        .map(|definition| Diagnostic {
            message: missing_doc_message(&definition.name),
            offset: definition.offset,
            length: crate::utf16_len(&definition.name) as usize,
            severity: DiagnosticSeverity::Warning,
            code: MISSING_DOC_CODE,
            data: Some(DiagnosticData {
                name: definition.name,
                line: definition.line,
            }),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_syntax_diagnostic_before_doc_warnings() {
        let diagnostics = document_diagnostics(
            "(define (local-helper value) value)",
            Some(SyntaxIssue {
                message: "parse failed".to_owned(),
                offset: 3,
                length: 1,
            }),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, SYNTAX_CODE);
        assert_eq!(diagnostics[0].severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn emits_missing_doc_diagnostics() {
        let diagnostics = missing_doc_diagnostics("(define (local-helper value) value)");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, MISSING_DOC_CODE);
        assert_eq!(diagnostics[0].length, "local-helper".len());
        assert_eq!(
            diagnostics[0].data,
            Some(DiagnosticData {
                name: "local-helper".to_owned(),
                line: 0,
            })
        );
    }

    #[test]
    fn accepts_doc_next_for_missing_doc_diagnostics() {
        let diagnostics = missing_doc_diagnostics(
            "(doc-next (summary \"Docs.\"))\n(define (local-helper value) value)",
        );

        assert!(diagnostics.is_empty());
    }
}
