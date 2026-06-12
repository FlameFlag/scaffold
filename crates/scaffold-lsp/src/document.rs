use ropey::Rope;
use scaffold_editor::symbols::FormContext;
use tower_lsp::lsp_types::{Position, Range};

#[derive(Debug, Clone)]
pub struct Document {
    text: String,
    rope: Rope,
}

impl Document {
    pub fn new(text: String) -> Self {
        let rope = Rope::from_str(&text);
        Self { text, rope }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn full_range(&self) -> Range {
        let last_line = self.rope.len_lines().saturating_sub(1);
        let line_start = self.rope.line_to_char(last_line);
        let line_end = self.rope.len_chars();
        let end_character = self
            .rope
            .char_to_utf16_cu(line_end)
            .saturating_sub(self.rope.char_to_utf16_cu(line_start));
        Range::new(
            Position::new(0, 0),
            Position::new(last_line as u32, end_character as u32),
        )
    }

    pub fn word_at(&self, position: Position) -> Option<String> {
        scaffold_editor::symbols::symbol_at_position(&self.text, position.line, position.character)
    }

    pub fn form_context_before(&self, position: Position) -> Option<FormContext> {
        scaffold_editor::symbols::form_context_at_position(
            &self.text,
            position.line,
            position.character,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_symbol_at_lsp_position() {
        let text = include_str!("fixtures/document-symbols.scm");
        let document = Document::new(text.to_owned());

        assert_eq!(
            document.word_at(position_of(text, "tool")).as_deref(),
            Some("tool")
        );
    }

    #[test]
    fn handles_utf16_positions() {
        let text = include_str!("fixtures/document-symbols.scm");
        let document = Document::new(text.to_owned());

        assert_eq!(
            document.word_at(position_of(text, "café")).as_deref(),
            Some("café")
        );
        assert_eq!(
            document.word_at(position_of(text, "tool")).as_deref(),
            Some("tool")
        );
    }

    #[test]
    fn finds_list_head_before_cursor() {
        let text = include_str!("fixtures/document-symbols.scm");
        let document = Document::new(text.to_owned());

        assert_eq!(
            document
                .form_context_before(position_after(text, "\"demo\""))
                .map(|context| context.head),
            Some("tool".to_owned())
        );
    }

    #[test]
    fn ignores_symbols_in_strings_and_comments() {
        let text = "\"tool\"\n; tool\n(tool)";
        let document = Document::new(text.to_owned());

        assert_eq!(document.word_at(position_of(text, "\"tool\"")), None);
        assert_eq!(document.word_at(position_of(text, "; tool")), None);
        assert_eq!(
            document.word_at(position_of(text, "tool)")).as_deref(),
            Some("tool")
        );
    }

    fn position_of(text: &str, needle: &str) -> Position {
        byte_offset_to_position(text, text.find(needle).expect("needle exists"))
    }

    fn position_after(text: &str, needle: &str) -> Position {
        let start = text.find(needle).expect("needle exists");
        byte_offset_to_position(text, start + needle.len())
    }

    fn byte_offset_to_position(text: &str, offset: usize) -> Position {
        let before = &text[..offset];
        let line = before.lines().count().saturating_sub(1) as u32;
        let line_start = before.rfind('\n').map_or(0, |index| index + 1);
        let character = text[line_start..offset].encode_utf16().count() as u32;
        Position::new(line, character)
    }
}
