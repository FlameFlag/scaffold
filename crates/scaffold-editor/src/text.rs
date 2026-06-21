use line_index::{LineIndex, WideEncoding, WideLineCol};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextPosition {
    pub line: u32,
    pub character: u32,
}

#[must_use]
pub fn utf16_len(text: &str) -> u32 {
    WideEncoding::Utf16.measure(text) as u32
}

#[must_use]
pub fn utf16_position_at_byte_offset(text: &str, offset: usize) -> TextPosition {
    let index = LineIndex::new(text);
    let offset = previous_char_boundary(text, offset.min(text.len()));
    let line_col = index.line_col((offset as u32).into());
    let position = index
        .to_wide(WideEncoding::Utf16, line_col)
        .expect("line/column from valid offset converts to UTF-16");
    TextPosition {
        line: position.line,
        character: position.col,
    }
}

pub(crate) fn byte_offset_at_utf16_position(
    text: &str,
    line: u32,
    character: u32,
) -> Option<usize> {
    let index = LineIndex::new(text);
    let line_range = index.line(line)?;
    let line_col = index.to_utf8(
        WideEncoding::Utf16,
        WideLineCol {
            line,
            col: character,
        },
    )?;
    let offset = u32::from(index.offset(line_col)?) as usize;
    Some(offset.min(line_content_end(text, u32::from(line_range.end()) as usize)))
}

fn line_content_end(text: &str, line_end: usize) -> usize {
    if text.as_bytes().get(line_end.saturating_sub(1)) == Some(&b'\n') {
        line_end - 1
    } else {
        line_end
    }
}

fn previous_char_boundary(text: &str, mut offset: usize) -> usize {
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

pub(crate) fn skip_string(text: &str, mut offset: usize) -> usize {
    offset += 1;
    let mut escaped = false;
    while offset < text.len() {
        let Some(ch) = text[offset..].chars().next() else {
            return offset;
        };
        offset += ch.len_utf8();
        if escaped {
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else if ch == '"' {
            return offset;
        }
    }
    text.len()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceSpanKind {
    Symbol,
    String,
    Comment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SourceSpan<'a> {
    pub(crate) kind: SourceSpanKind,
    pub(crate) text: &'a str,
    byte_start: usize,
}

impl SourceSpan<'_> {
    pub(crate) fn start_utf16(self, line: &str) -> u32 {
        utf16_len(&line[..self.byte_start])
    }

    pub(crate) fn len_utf16(self) -> u32 {
        utf16_len(self.text)
    }
}

pub(crate) fn source_spans_line(line: &str) -> impl Iterator<Item = SourceSpan<'_>> {
    let mut char_indices = line.char_indices().peekable();
    let mut finished = false;
    std::iter::from_fn(move || {
        if finished {
            return None;
        }
        while let Some((byte_index, ch)) = char_indices.next() {
            if ch == ';' {
                finished = true;
                return Some(SourceSpan {
                    kind: SourceSpanKind::Comment,
                    text: &line[byte_index..],
                    byte_start: byte_index,
                });
            }
            if ch == '"' {
                let end = skip_string(line, byte_index);
                while char_indices.peek().is_some_and(|(index, _)| *index < end) {
                    let _advanced = char_indices.next();
                }
                return Some(SourceSpan {
                    kind: SourceSpanKind::String,
                    text: &line[byte_index..end],
                    byte_start: byte_index,
                });
            }
            if !is_symbol_start(ch) {
                continue;
            }

            let start = byte_index;
            let mut end = byte_index + ch.len_utf8();
            while let Some((next_index, next)) = char_indices.peek().copied() {
                if !is_symbol_continue(next) {
                    break;
                }
                let _advanced = char_indices.next();
                end = next_index + next.len_utf8();
            }

            return Some(SourceSpan {
                kind: SourceSpanKind::Symbol,
                text: &line[start..end],
                byte_start: start,
            });
        }
        None
    })
}

pub(crate) const fn is_symbol_start(ch: char) -> bool {
    !ch.is_whitespace() && !matches!(ch, '(' | ')' | '[' | ']' | '"' | '\'' | '`' | ',' | ';')
}

pub(crate) const fn is_symbol_continue(ch: char) -> bool {
    is_symbol_start(ch)
}

pub(crate) fn clean_signature_parameter(name: &str) -> &str {
    name.trim_matches(&['[', ']'][..])
}

pub(crate) fn signature_parameter_names(signature: &str) -> impl Iterator<Item = &str> {
    signature
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split_whitespace()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_scheme_strings_with_escaped_quotes() {
        let text = r#""demo \"quoted\"" tail"#;

        assert_eq!(skip_string(text, 0), r#""demo \"quoted\"""#.len());
    }

    #[test]
    fn skips_unclosed_string_to_text_end() {
        let text = r#""demo"#;

        assert_eq!(skip_string(text, 0), text.len());
    }

    #[test]
    fn classifies_scheme_symbol_boundaries() {
        assert!(is_symbol_start("café".chars().next().expect("char")));
        assert!(!is_symbol_start('('));
        assert!(!is_symbol_start(';'));
        assert!(!is_symbol_continue('"'));
    }

    #[test]
    fn scans_scheme_source_spans() {
        assert_eq!(
            source_spans_line(r#"(local-helper "demo") ; comment"#)
                .map(|span| (
                    span.kind,
                    span.text,
                    span.start_utf16(r#"(local-helper "demo") ; comment"#)
                ))
                .collect::<Vec<_>>(),
            vec![
                (SourceSpanKind::Symbol, "local-helper", 1),
                (SourceSpanKind::String, r#""demo""#, 14),
                (SourceSpanKind::Comment, "; comment", 22),
            ]
        );
    }

    #[test]
    fn parses_signature_parameter_names() {
        assert_eq!(
            signature_parameter_names("(demo name [action] field ...)")
                .map(clean_signature_parameter)
                .collect::<Vec<_>>(),
            vec!["demo", "name", "action", "field", "..."]
        );
    }

    #[test]
    fn maps_utf16_positions_to_byte_offsets() {
        let text = "λ\n(define café 1)\n";

        assert_eq!(byte_offset_at_utf16_position(text, 0, 0), Some(0));
        assert_eq!(
            byte_offset_at_utf16_position(text, 1, utf16_len("(define ")),
            text.find("café")
        );
    }

    #[test]
    fn clamps_utf16_positions_to_line_content_end() {
        let text = "demo\nnext";

        assert_eq!(byte_offset_at_utf16_position(text, 0, 99), Some(4));
    }

    #[test]
    fn maps_byte_offsets_to_utf16_positions() {
        let text = "λ\n(define café 1)\n";

        assert_eq!(
            utf16_position_at_byte_offset(text, text.find("café").expect("symbol")),
            TextPosition {
                line: 1,
                character: utf16_len("(define ")
            }
        );
    }

    #[test]
    fn byte_offset_position_uses_previous_char_boundary() {
        let text = "λ";

        assert_eq!(
            utf16_position_at_byte_offset(text, 1),
            TextPosition {
                line: 0,
                character: 0
            }
        );
    }
}
