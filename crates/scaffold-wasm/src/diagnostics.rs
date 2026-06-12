use crate::editor_diagnostics;

pub(crate) fn diagnose_text(text: &str) -> String {
    serde_json::to_string(&editor_diagnostics::document_diagnostics(
        text,
        syntax_issue(text),
    ))
    .expect("diagnostics serialize")
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
