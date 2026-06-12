use std::collections::HashMap;

use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, Diagnostic, NumberOrString, Position, Range, TextEdit, Url,
    WorkspaceEdit,
};

use crate::diagnostics;
use crate::document::Document;

pub(super) fn missing_doc_action(
    uri: &Url,
    document: &Document,
    diagnostic: &Diagnostic,
) -> Option<CodeAction> {
    let code = diagnostic.code.as_ref()?;
    if !matches!(
        code,
        NumberOrString::String(value) if value == diagnostics::MISSING_DOC_CODE
    ) {
        return None;
    }
    let name = diagnostic
        .data
        .as_ref()
        .and_then(|data| data.get("name"))
        .and_then(|value| value.as_str())?;
    let line = diagnostic.range.start.line;
    let indent = line_indent(document.text(), line);
    let new_text = scaffold_editor::actions::missing_doc_stub(name, &indent);
    let edit = TextEdit {
        range: Range::new(Position::new(line, 0), Position::new(line, 0)),
        new_text,
    };
    let mut changes = HashMap::new();
    let _previous = changes.insert(uri.clone(), vec![edit]);
    Some(CodeAction {
        title: format!("Add doc stub for `{name}`"),
        kind: Some(CodeActionKind::QUICKFIX),
        diagnostics: Some(vec![diagnostic.clone()]),
        edit: Some(WorkspaceEdit {
            changes: Some(changes),
            ..Default::default()
        }),
        ..Default::default()
    })
}

fn line_indent(text: &str, line: u32) -> String {
    text.lines()
        .nth(line as usize)
        .map(|line| {
            line.chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    #[test]
    fn builds_doc_stub_with_definition_indent() {
        assert_eq!(
            scaffold_editor::actions::missing_doc_stub("local-helper", "  "),
            concat!(
                "  (doc-next\n",
                "    (summary \"Describe `local-helper`.\"))\n\n"
            )
        );
    }
}
