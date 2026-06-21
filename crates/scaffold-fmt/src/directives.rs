use super::parser::Parser;

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
    match comment {
        FMT_IGNORE_FILE => Some(FormatDirective::IgnoreFile),
        FMT_IGNORE_NEXT => Some(FormatDirective::IgnoreNext),
        FMT_OFF => Some(FormatDirective::Off),
        FMT_ON => Some(FormatDirective::On),
        _ => None,
    }
}

fn next_form_end(text: &str, offset: usize) -> Option<usize> {
    Parser::parse_next_form_end(text, offset)
        .ok()
        .flatten()
        .map(|end| include_trailing_newline(text, end))
}

fn include_trailing_newline(text: &str, offset: usize) -> usize {
    match text.as_bytes()[offset..] {
        [b'\r', b'\n', ..] => offset + 2,
        [b'\n' | b'\r', ..] => offset + 1,
        _ => offset,
    }
}
