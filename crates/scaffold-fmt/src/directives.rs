#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FormatSegment<'a> {
    Enabled(&'a str),
    Disabled(&'a str),
}

const FMT_IGNORE_FILE: &str = "scaffold-fmt: ignore-file";
const FMT_IGNORE_NEXT: &str = "scaffold-fmt: ignore-next";
const FMT_OFF: &str = "scaffold-fmt: off";
const FMT_ON: &str = "scaffold-fmt: on";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FormatDirective {
    IgnoreFile,
    IgnoreNext,
    Off,
    On,
}

pub(super) fn has_ignore_file_directive(text: &str) -> bool {
    text.lines()
        .any(|line| directive_text(line) == Some(FormatDirective::IgnoreFile))
}

pub(super) fn format_segments(text: &str) -> Vec<FormatSegment<'_>> {
    let mut segments = Vec::new();
    let mut cursor = 0;
    let mut disabled_start = None;

    for (line_start, line) in lines_with_offsets(text) {
        if line_start < cursor {
            continue;
        }
        let line_end = line_start + line.len();
        match directive_text(line) {
            Some(FormatDirective::IgnoreNext) if disabled_start.is_none() => {
                if cursor < line_start {
                    segments.push(FormatSegment::Enabled(&text[cursor..line_start]));
                }
                let end = next_form_end(text, line_end).unwrap_or(line_end);
                segments.push(FormatSegment::Disabled(&text[line_start..end]));
                cursor = end;
            }
            Some(FormatDirective::Off) if disabled_start.is_none() => {
                if cursor < line_start {
                    segments.push(FormatSegment::Enabled(&text[cursor..line_start]));
                }
                disabled_start = Some(line_start);
            }
            Some(FormatDirective::On) => {
                if let Some(start) = disabled_start.take() {
                    segments.push(FormatSegment::Disabled(&text[start..line_end]));
                    cursor = line_end;
                }
            }
            _ => {}
        }
    }

    if let Some(start) = disabled_start {
        segments.push(FormatSegment::Disabled(&text[start..]));
    } else if cursor < text.len() {
        segments.push(FormatSegment::Enabled(&text[cursor..]));
    }

    if segments.is_empty() {
        segments.push(FormatSegment::Enabled(text));
    }
    segments
}

fn lines_with_offsets(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    text.split_inclusive('\n').map(move |line| {
        let start = offset;
        offset += line.len();
        (start, line)
    })
}

fn directive_text(line: &str) -> Option<FormatDirective> {
    let trimmed = line.trim_start();
    let comment = trimmed.strip_prefix(';')?.trim();
    if comment == FMT_IGNORE_FILE {
        Some(FormatDirective::IgnoreFile)
    } else if comment == FMT_IGNORE_NEXT {
        Some(FormatDirective::IgnoreNext)
    } else if comment == FMT_OFF {
        Some(FormatDirective::Off)
    } else if comment == FMT_ON {
        Some(FormatDirective::On)
    } else {
        None
    }
}

fn next_form_end(text: &str, offset: usize) -> Option<usize> {
    let offset = skip_ws_and_line_comments(text, offset);
    if offset >= text.len() {
        return Some(offset);
    }
    let end = skip_form(text, offset);
    Some(include_trailing_newline(text, end))
}

fn skip_ws_and_line_comments(text: &str, mut offset: usize) -> usize {
    loop {
        while offset < text.len() {
            let Some(ch) = text[offset..].chars().next() else {
                return offset;
            };
            if !ch.is_whitespace() {
                break;
            }
            offset += ch.len_utf8();
        }
        if text[offset..].starts_with(';') {
            offset = skip_line_comment(text, offset);
            continue;
        }
        return offset;
    }
}

fn skip_form(text: &str, offset: usize) -> usize {
    let Some(ch) = text[offset..].chars().next() else {
        return offset;
    };
    match ch {
        '(' => skip_list(text, offset, ')'),
        '[' => skip_list(text, offset, ']'),
        '"' => skip_string(text, offset),
        ';' => skip_line_comment(text, offset),
        '\'' | '`' => skip_prefixed_form(text, offset + ch.len_utf8()),
        ',' if text[offset..].starts_with(",@") => skip_prefixed_form(text, offset + 2),
        ',' => skip_prefixed_form(text, offset + ch.len_utf8()),
        _ => skip_atom(text, offset),
    }
}

fn skip_prefixed_form(text: &str, offset: usize) -> usize {
    let offset = skip_ws_and_line_comments(text, offset);
    skip_form(text, offset)
}

fn skip_list(text: &str, open_offset: usize, initial_close: char) -> usize {
    let mut closes = vec![initial_close];
    let mut offset = open_offset + 1;
    while offset < text.len() {
        let Some(ch) = text[offset..].chars().next() else {
            return offset;
        };
        match ch {
            '(' => {
                closes.push(')');
                offset += ch.len_utf8();
            }
            '[' => {
                closes.push(']');
                offset += ch.len_utf8();
            }
            '"' => offset = skip_string(text, offset),
            ';' => offset = skip_line_comment(text, offset),
            _ if closes.last().copied() == Some(ch) => {
                offset += ch.len_utf8();
                let _closed = closes.pop();
                if closes.is_empty() {
                    return offset;
                }
            }
            _ => offset += ch.len_utf8(),
        }
    }
    text.len()
}

fn skip_string(text: &str, mut offset: usize) -> usize {
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

fn skip_line_comment(text: &str, mut offset: usize) -> usize {
    while offset < text.len() {
        let Some(ch) = text[offset..].chars().next() else {
            return offset;
        };
        offset += ch.len_utf8();
        if ch == '\n' {
            return offset;
        }
    }
    text.len()
}

fn skip_atom(text: &str, mut offset: usize) -> usize {
    while offset < text.len() {
        let Some(ch) = text[offset..].chars().next() else {
            return offset;
        };
        if ch.is_whitespace() || matches!(ch, '(' | ')' | '[' | ']' | '"' | '\'' | '`' | ',' | ';')
        {
            return offset;
        }
        offset += ch.len_utf8();
    }
    text.len()
}

fn include_trailing_newline(text: &str, offset: usize) -> usize {
    if text[offset..].starts_with("\r\n") {
        offset + 2
    } else if text[offset..].starts_with('\n') || text[offset..].starts_with('\r') {
        offset + 1
    } else {
        offset
    }
}
