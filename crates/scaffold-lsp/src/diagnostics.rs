use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range};

use scaffold_editor::diagnostics as editor_diagnostics;
pub(crate) use scaffold_editor::diagnostics::MISSING_DOC_CODE;

#[cfg(test)]
pub fn syntax_diagnostics(_source_name: &str, text: &str) -> Vec<Diagnostic> {
    syntax_issue(text)
        .map(editor_diagnostics::syntax_diagnostics)
        .unwrap_or_default()
        .into_iter()
        .map(|diagnostic| lsp_diagnostic(text, diagnostic))
        .collect()
}

pub fn document_diagnostics(source_name: &str, text: &str) -> Vec<Diagnostic> {
    let _ = source_name;
    editor_diagnostics::document_diagnostics(text, syntax_issue(text))
        .into_iter()
        .map(|diagnostic| lsp_diagnostic(text, diagnostic))
        .collect()
}

#[cfg(test)]
pub fn doc_diagnostics(_source_name: &str, text: &str) -> Vec<Diagnostic> {
    if syntax_issue(text).is_some() {
        return Vec::new();
    }
    editor_diagnostics::missing_doc_diagnostics(text)
        .into_iter()
        .map(|diagnostic| lsp_diagnostic(text, diagnostic))
        .collect()
}

fn syntax_issue(text: &str) -> Option<editor_diagnostics::SyntaxIssue> {
    scaffold_fmt::format_text(text)
        .err()
        .map(|error| editor_diagnostics::SyntaxIssue {
            message: error.to_string(),
            offset: error.primary_offset(),
            length: 1,
        })
}

fn lsp_diagnostic(text: &str, diagnostic: editor_diagnostics::Diagnostic) -> Diagnostic {
    Diagnostic {
        range: offset_range(text, diagnostic.offset, diagnostic.length),
        severity: Some(match diagnostic.severity {
            editor_diagnostics::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
            editor_diagnostics::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
        }),
        code: Some(NumberOrString::String(diagnostic.code.to_owned())),
        source: Some("scaffold".to_owned()),
        message: diagnostic.message,
        data: diagnostic
            .data
            .map(|data| serde_json::to_value(data).expect("diagnostic data serializes")),
        ..Default::default()
    }
}

fn offset_range(text: &str, offset: usize, len: usize) -> Range {
    Range::new(
        byte_offset_to_position(text, offset),
        byte_offset_to_position(text, offset.saturating_add(len.max(1))),
    )
}

fn byte_offset_to_position(text: &str, offset: usize) -> Position {
    let offset = offset.min(text.len());
    let line_start = text[..offset].rfind('\n').map_or(0, |index| index + 1);
    let line = text[..offset].bytes().filter(|byte| *byte == b'\n').count() as u32;
    let character = text[line_start..offset].encode_utf16().count() as u32;
    Position::new(line, character)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_no_diagnostics_for_valid_scheme() {
        assert!(
            document_diagnostics(
                "valid-syntax.scm",
                include_str!("fixtures/valid-syntax.scm")
            )
            .is_empty()
        );
    }

    #[test]
    fn returns_diagnostic_for_invalid_scheme() {
        let diagnostics = syntax_diagnostics(
            "invalid-syntax.scm",
            include_str!("fixtures/invalid-syntax.fixture"),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].source.as_deref(), Some("scaffold"));
        assert_eq!(diagnostics[0].range.start.line, 0);
    }

    #[test]
    fn warns_for_undocumented_definitions() {
        let diagnostics = doc_diagnostics(
            "undocumented-definition.scm",
            include_str!("fixtures/undocumented-definition.fixture"),
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].code,
            Some(NumberOrString::String(MISSING_DOC_CODE.to_owned()))
        );
        assert!(diagnostics[0].message.contains("local-helper"));
    }

    #[test]
    fn warns_for_undocumented_definitions_in_files_with_scaffold_keywords() {
        let diagnostics = doc_diagnostics(
            "keyword-arguments.scm",
            "(tool #:name \"demo\")\n(define (local-helper value) value)",
        );

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].data,
            Some(serde_json::json!({
                "name": "local-helper",
                "line": 1,
            }))
        );
    }

    #[test]
    fn accepts_documented_definitions() {
        let diagnostics = doc_diagnostics("doc-entry.scm", include_str!("fixtures/doc-entry.scm"));

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn syntax_diagnostics_use_wasm_safe_formatter_parser() {
        let cases = [
            ("plain forms", "(define x 1)\n(doc 'x (summary \"X\"))"),
            ("line comment", "; comment\n(define x 1)"),
            ("quote shorthand", "(define x '(1 2 3))"),
            ("quasiquote shorthand", "(define x `(1 ,y ,@z))"),
            ("square brackets", "(define x [list 1 2])"),
            ("keyword argument", "(tool #:name \"demo\")"),
            ("vector literal", "(define x #(1 2 3))"),
            ("character literal", "(define x #\\a)"),
            ("block comment", "#| comment |#\n(define x 1)"),
            ("datum comment", "#;(define ignored (broken)\n(define x 1)"),
            ("unclosed list", "(define x 1"),
            ("unterminated string", "(define x \"demo)"),
            ("unexpected close", "(define x 1))"),
        ];

        for (label, source) in cases {
            let native_lsp_ok = syntax_diagnostics(label, source).is_empty();
            let formatter_ok = scaffold_fmt::format_text(source).is_ok();
            assert_eq!(
                native_lsp_ok, formatter_ok,
                "{label}: native LSP ok={native_lsp_ok}, formatter ok={formatter_ok}",
            );
        }
    }
}
