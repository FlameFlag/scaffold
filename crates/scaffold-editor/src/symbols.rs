use crate::text::{SourceSpanKind, byte_offset_at_utf16_position, source_spans_line};

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct SymbolRange {
    pub line: u32,
    pub start: u32,
    pub length: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize)]
pub struct SymbolLocation {
    pub uri: String,
    pub line: u32,
    pub start: u32,
    pub length: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FormContext {
    pub head: String,
    pub active_argument: u32,
}

#[must_use]
pub fn symbol_ranges(text: &str, target: &str) -> Vec<SymbolRange> {
    if target.is_empty() {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        for span in symbol_spans_line(line) {
            if span.text == target {
                ranges.push(SymbolRange {
                    line: line_index as u32,
                    start: span.start,
                    length: span.length,
                });
            }
        }
    }
    ranges
}

pub fn reference_locations<'a>(
    documents: impl IntoIterator<Item = (&'a str, &'a str)>,
    symbol: &str,
) -> Vec<SymbolLocation> {
    documents
        .into_iter()
        .flat_map(|(uri, text)| {
            symbol_ranges(text, symbol)
                .into_iter()
                .map(move |range| SymbolLocation {
                    uri: uri.to_owned(),
                    line: range.line,
                    start: range.start,
                    length: range.length,
                })
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[must_use]
pub fn symbol_at_position(text: &str, line: u32, character: u32) -> Option<String> {
    let line_text = text.lines().nth(line as usize)?;
    symbol_spans_line(line_text)
        .into_iter()
        .find(|span| character >= span.start && character <= span.start + span.length)
        .map(|span| span.text.to_owned())
}

#[must_use]
pub fn form_context_at_position(text: &str, line: u32, character: u32) -> Option<FormContext> {
    form_context_at_offset(text, byte_offset_at_utf16_position(text, line, character)?)
}

#[must_use]
pub fn form_context_at_offset(text: &str, cursor: usize) -> Option<FormContext> {
    let cursor = cursor.min(text.len());
    let mut stack = Vec::new();
    let _match_offset = scan_code(text, 0, cursor, |ch, offset| {
        if is_open_delimiter(ch) {
            stack.push(offset);
        } else if is_close_delimiter(ch) {
            let _closed = stack.pop();
        }
        None
    });

    let open_offset = stack.last().copied()?;
    read_form_context(text, open_offset, cursor)
}

pub struct SymbolSpan<'a> {
    pub text: &'a str,
    pub start: u32,
    pub length: u32,
}

#[must_use]
pub fn symbol_spans_line(line: &str) -> Vec<SymbolSpan<'_>> {
    source_spans_line(line)
        .filter(|span| span.kind == SourceSpanKind::Symbol)
        .map(|span| SymbolSpan {
            text: span.text,
            start: span.start_utf16(line),
            length: span.len_utf16(),
        })
        .collect()
}

fn read_form_context(text: &str, open_offset: usize, cursor: usize) -> Option<FormContext> {
    let head_start = skip_whitespace(text, open_offset + 1, cursor);
    let head_end = read_token_end(text, head_start, cursor);
    if head_start == head_end {
        return None;
    }
    Some(FormContext {
        head: text[head_start..head_end].to_owned(),
        active_argument: active_argument_index(text, head_end, cursor),
    })
}

fn active_argument_index(text: &str, body_offset: usize, cursor: usize) -> u32 {
    let mut offset = body_offset;
    let mut argument = 0;
    loop {
        let Some(expression_end) = next_expression_end(text, offset, cursor) else {
            return argument;
        };
        if cursor <= next_char_offset(text, expression_end).unwrap_or(expression_end) {
            return argument;
        }
        argument += 1;
        offset = next_char_offset(text, expression_end).unwrap_or(expression_end);
    }
}

fn next_expression_end(text: &str, offset: usize, cursor: usize) -> Option<usize> {
    let start = skip_whitespace(text, offset, cursor);
    (start < cursor).then(|| skip_expression(text, start, cursor))
}

fn skip_expression(text: &str, offset: usize, cursor: usize) -> usize {
    if let Some(skipped_offset) = skipped_code_offset(text, offset, cursor) {
        return skipped_offset;
    }
    match char_at(text, offset) {
        Some(ch) if is_open_delimiter(ch) => skip_list(text, offset, cursor),
        Some(_) => skip_atom(text, offset, cursor),
        None => offset,
    }
}

fn skip_atom(text: &str, offset: usize, cursor: usize) -> usize {
    let mut end = offset;
    while end < cursor {
        let Some(ch) = char_at(text, end) else {
            break;
        };
        if ch.is_whitespace() || is_close_delimiter(ch) {
            break;
        }
        end = next_char_offset(text, end).unwrap_or(text.len());
    }
    previous_char_offset(text, end).unwrap_or(offset)
}

fn skip_list(text: &str, open_offset: usize, cursor: usize) -> usize {
    let Some(open) = char_at(text, open_offset) else {
        return open_offset;
    };
    let mut closes = vec![matching_close(open)];
    scan_code(text, open_offset + open.len_utf8(), cursor, |ch, offset| {
        if is_open_delimiter(ch) {
            closes.push(matching_close(ch));
        } else if closes.last().copied() == Some(ch) {
            let _closed = closes.pop();
            if closes.is_empty() {
                return Some(offset);
            }
        }
        None
    })
    .unwrap_or_else(|| previous_char_offset(text, cursor).unwrap_or(open_offset))
}

fn scan_code(
    text: &str,
    start_offset: usize,
    cursor: usize,
    mut visit: impl FnMut(char, usize) -> Option<usize>,
) -> Option<usize> {
    let limit = cursor.min(text.len());
    let mut offset = start_offset.min(limit);
    while offset < limit {
        if let Some(skipped_offset) = skipped_code_offset(text, offset, cursor) {
            offset = next_char_offset(text, skipped_offset).unwrap_or(limit);
            continue;
        }
        let ch = char_at(text, offset)?;
        if let Some(result) = visit(ch, offset) {
            return Some(result);
        }
        offset = next_char_offset(text, offset).unwrap_or(limit);
    }
    None
}

fn skipped_code_offset(text: &str, offset: usize, cursor: usize) -> Option<usize> {
    match char_at(text, offset)? {
        '"' => Some(skip_string_to_cursor(text, offset, cursor)),
        ';' => Some(skip_line_comment(text, offset, cursor)),
        _ => None,
    }
}

fn skip_string_to_cursor(text: &str, offset: usize, cursor: usize) -> usize {
    previous_char_offset(text, crate::text::skip_string(text, offset).min(cursor)).unwrap_or(offset)
}

fn skip_line_comment(text: &str, mut offset: usize, cursor: usize) -> usize {
    let limit = cursor.min(text.len());
    while offset < limit {
        if char_at(text, offset) == Some('\n') {
            return offset;
        }
        offset = next_char_offset(text, offset).unwrap_or(limit);
    }
    previous_char_offset(text, limit).unwrap_or(offset)
}

fn skip_whitespace(text: &str, mut offset: usize, cursor: usize) -> usize {
    let limit = cursor.min(text.len());
    while offset < limit && char_at(text, offset).is_some_and(char::is_whitespace) {
        offset = next_char_offset(text, offset).unwrap_or(limit);
    }
    offset
}

fn read_token_end(text: &str, mut offset: usize, cursor: usize) -> usize {
    let limit = cursor.min(text.len());
    while offset < limit && char_at(text, offset).is_some_and(|ch| !is_delimiter(ch)) {
        offset = next_char_offset(text, offset).unwrap_or(limit);
    }
    offset
}

const fn is_delimiter(ch: char) -> bool {
    ch.is_whitespace() || is_open_delimiter(ch) || is_close_delimiter(ch)
}

const fn is_open_delimiter(ch: char) -> bool {
    matches!(ch, '(' | '[')
}

const fn is_close_delimiter(ch: char) -> bool {
    matches!(ch, ')' | ']')
}

const fn matching_close(open: char) -> char {
    if open == '[' { ']' } else { ')' }
}

fn char_at(text: &str, offset: usize) -> Option<char> {
    text.get(offset..)?.chars().next()
}

fn next_char_offset(text: &str, offset: usize) -> Option<usize> {
    let ch = char_at(text, offset)?;
    Some((offset + ch.len_utf8()).min(text.len()))
}

fn previous_char_offset(text: &str, offset: usize) -> Option<usize> {
    text.get(..offset.min(text.len()))?
        .char_indices()
        .last()
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_strings_and_comments_when_finding_ranges() {
        let ranges = symbol_ranges(
            include_str!("fixtures/reference-ranges.scm"),
            "local-helper",
        );

        assert_eq!(
            ranges
                .into_iter()
                .map(|range| range.line)
                .collect::<Vec<_>>(),
            vec![0, 3]
        );
    }

    #[test]
    fn collects_sorted_deduped_reference_locations() {
        let documents = vec![
            (
                "file:///b.scm",
                include_str!("fixtures/reference-document-b.scm"),
            ),
            ("file:///a.scm", "(define (local-helper value) value)"),
            ("file:///a.scm", "(define (local-helper value) value)"),
        ];

        let locations = reference_locations(documents, "local-helper");

        assert_eq!(
            locations
                .into_iter()
                .map(|location| (location.uri, location.line, location.start, location.length))
                .collect::<Vec<_>>(),
            vec![
                ("file:///a.scm".to_owned(), 0, 9, 12),
                ("file:///b.scm".to_owned(), 0, 1, 12),
            ]
        );
    }

    #[test]
    fn extracts_symbols_at_utf16_positions() {
        let text = include_str!("fixtures/utf16-symbol-filtering.scm");

        assert_eq!(symbol_at_position(text, 0, 8).as_deref(), Some("café"));
        assert_eq!(symbol_at_position(text, 1, 1), None);
        assert_eq!(symbol_at_position(text, 2, 2), None);
    }

    #[test]
    fn reads_form_context_with_active_argument() {
        let text = "(tool \"demo\" (required) #:name \"rg\")";

        assert_eq!(
            form_context_at_position(text, 0, 26),
            Some(FormContext {
                head: "tool".to_owned(),
                active_argument: 2,
            })
        );
    }

    #[test]
    fn ignores_forms_inside_strings_and_comments() {
        assert_eq!(form_context_at_position("\"(tool demo\"", 0, 7), None);
        assert_eq!(form_context_at_position("; (tool demo)", 0, 8), None);
    }
}
