use super::DocEntry;

#[must_use]
pub fn snippet_for_signature(signature: &str) -> Option<String> {
    scaffold_editor::reference::snippet_for_signature(signature)
}

#[must_use]
pub fn markdown_for_entry(entry: &DocEntry) -> String {
    scaffold_editor::reference::markdown_for_entry(entry)
}
