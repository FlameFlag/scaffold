pub(crate) fn symbol_at_position(text: &str, line: u32, character: u32) -> Option<String> {
    crate::editor_symbols::symbol_at_position(text, line, character)
}
