use lexpr::{
    datum::{Datum, Ref},
    parse::{KeywordSyntax, Options, Parser},
};

pub use crate::TextPosition;

use crate::utf16_len;

#[must_use]
pub fn parse_datums(text: &str) -> Vec<Datum> {
    let mut parser = Parser::from_str_custom(
        text,
        Options::new().with_keyword_syntax(KeywordSyntax::Octothorpe),
    );
    parser.datum_iter().map_while(Result::ok).collect()
}

#[must_use]
pub fn parses_completely(text: &str) -> bool {
    let mut parser = Parser::from_str_custom(
        text,
        Options::new().with_keyword_syntax(KeywordSyntax::Octothorpe),
    );
    parser.datum_iter().all(|datum| datum.is_ok())
}

#[must_use]
pub fn list_items(datum: Ref<'_>) -> Option<Vec<Ref<'_>>> {
    let mut items = datum.list_iter()?;
    Some(items.by_ref().collect())
}

#[must_use]
pub fn symbol_text(datum: Ref<'_>) -> Option<&str> {
    datum.value().as_symbol()
}

#[must_use]
pub fn string_text(datum: Ref<'_>) -> Option<&str> {
    datum.value().as_str()
}

#[must_use]
pub fn span_start(text: &str, datum: Ref<'_>) -> TextPosition {
    let start = datum.span().start();
    let byte_offset = byte_offset_at(text, start.line(), start.column());
    let line_start = line_start_offset(text, start.line());
    TextPosition {
        line: start.line().saturating_sub(1) as u32,
        character: utf16_len(&text[line_start..byte_offset]),
    }
}

#[must_use]
pub fn utf16_offset_at_span_start(text: &str, datum: Ref<'_>) -> usize {
    let start = datum.span().start();
    utf16_len(&text[..byte_offset_at(text, start.line(), start.column())]) as usize
}

fn byte_offset_at(text: &str, one_based_line: usize, byte_column: usize) -> usize {
    let line_start = line_start_offset(text, one_based_line);
    (line_start + byte_column).min(text.len())
}

fn line_start_offset(text: &str, one_based_line: usize) -> usize {
    if one_based_line <= 1 {
        return 0;
    }
    let mut line = 1;
    for (offset, ch) in text.char_indices() {
        if ch == '\n' {
            line += 1;
            if line == one_based_line {
                return offset + ch.len_utf8();
            }
        }
    }
    text.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scaffold_keyword_arguments() {
        let datums = parse_datums("(tool #:name \"demo\")");

        assert_eq!(datums.len(), 1);
    }

    #[test]
    fn reports_incomplete_parse() {
        assert!(parses_completely("(tool #:name \"demo\")"));
        assert!(!parses_completely("(tool #:name \"demo\""));
    }

    #[test]
    fn converts_span_positions_to_utf16() {
        let text = "(define café 1)";
        let datums = parse_datums(text);
        let items = list_items(datums[0].as_ref()).expect("list");
        let position = span_start(text, items[1]);

        assert_eq!(position.line, 0);
        assert_eq!(position.character, utf16_len("(define "));
    }
}
