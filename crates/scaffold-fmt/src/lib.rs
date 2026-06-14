mod directives;
mod error;
mod form;
mod parser;
mod render;

use std::path::Path;

use directives::{FormatSegment, format_segments, has_ignore_file_directive};
pub use error::FormatError;
use parser::Parser;
use render::render_form;

pub type Result<T> = std::result::Result<T, FormatError>;

pub const DEFAULT_WIDTH: usize = 88;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub width: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            width: DEFAULT_WIDTH,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatMode {
    Check,
    Write,
}

pub fn format_text(text: &str) -> Result<String> {
    format_text_with_options(text, FormatOptions::default())
}

pub fn format_text_with_options(text: &str, options: FormatOptions) -> Result<String> {
    if has_ignore_file_directive(text) {
        return Ok(text.to_owned());
    }
    let segments = format_segments(text);
    if segments.len() > 1 {
        let mut rendered = String::new();
        for segment in segments {
            match segment {
                FormatSegment::Enabled(source) if source.trim().is_empty() => {
                    rendered.push_str(source);
                }
                FormatSegment::Enabled(source) => {
                    rendered.push_str(&format_forms(source, options)?);
                }
                FormatSegment::Disabled(source) => rendered.push_str(source),
            }
        }
        return Ok(rendered);
    }

    format_forms(text, options)
}

fn format_forms(text: &str, options: FormatOptions) -> Result<String> {
    let forms = Parser::new(text).parse_all()?;
    let mut rendered = String::new();

    for (index, form) in forms.iter().enumerate() {
        render_form(form, 0, options.width, &mut rendered);
        if index + 1 == forms.len() {
            rendered.push('\n');
        } else {
            rendered.push_str("\n\n");
        }
    }

    Ok(rendered)
}

pub fn format_path(path: impl AsRef<Path>, mode: FormatMode) -> Result<bool> {
    format_path_with_options(path, mode, FormatOptions::default())
}

pub fn format_path_with_options(
    path: impl AsRef<Path>,
    mode: FormatMode,
    options: FormatOptions,
) -> Result<bool> {
    let path = path.as_ref();
    let source = std::fs::read_to_string(path)?;
    let formatted = format_text_with_options(&source, options)?;
    if formatted == source {
        return Ok(false);
    }
    if mode == FormatMode::Write {
        std::fs::write(path, formatted)?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_short_forms_on_one_line() {
        assert_eq!(
            format_text("(tool \"rg\" 'fast)\n").unwrap(),
            "(tool \"rg\" 'fast)\n"
        );
    }

    #[test]
    fn breaks_long_forms() {
        let formatted = format_text(
            "(tool #:name \"demo\" #:bins '(\"demo\" \"democtl\") #:action (cargo #:crate \"demo\" #:locked? #t))",
        )
        .unwrap();

        assert!(formatted.contains("\n  #:name"));
        assert!(formatted.ends_with('\n'));
    }

    #[test]
    fn honors_custom_width() {
        assert_format_with_options(
            "(tool #:name \"demo\" #:bins '(\"demo\" \"democtl\"))",
            FormatOptions { width: 120 },
            "(tool #:name \"demo\" #:bins '(\"demo\" \"democtl\"))\n",
        );
        assert_format_with_options(
            "(tool #:name \"demo\" #:bins '(\"demo\" \"democtl\"))",
            FormatOptions { width: 30 },
            "(tool\n  #:name\n  \"demo\"\n  #:bins\n  '(\"demo\" \"democtl\"))\n",
        );
    }

    #[test]
    fn formats_top_level_forms_with_blank_lines() {
        assert_format("(define x 1)(define y 2)", "(define x 1)\n\n(define y 2)\n");
    }

    #[test]
    fn separates_doc_form_from_documented_definition() {
        assert_eq!(
            format_text("(doc 'x (summary \"X\"))(define x 1)(doc 'y (summary \"Y\"))(define y 2)")
                .unwrap(),
            concat!(
                "(doc 'x (summary \"X\"))\n\n",
                "(define x 1)\n\n",
                "(doc 'y (summary \"Y\"))\n\n",
                "(define y 2)\n",
            )
        );
    }

    #[test]
    fn separates_library_definition_blocks() {
        assert_eq!(
            format_text(
                "(library (demo) (export x y) (import (rnrs)) (doc 'x (summary \"X\")) (define x 1) (define y 2))"
            )
            .unwrap(),
            concat!(
                "(library\n",
                "  (demo)\n",
                "  (export x y)\n",
                "  (import (rnrs))\n\n",
                "  (doc 'x (summary \"X\"))\n\n",
                "  (define x 1)\n\n",
                "  (define y 2))\n",
            )
        );
    }

    #[test]
    fn moves_library_doc_before_matching_definition() {
        assert_eq!(
            format_text(
                "(library (demo) (export x) (import (rnrs)) (define x 1) (doc 'x (summary \"X\")))"
            )
            .unwrap(),
            concat!(
                "(library\n",
                "  (demo)\n",
                "  (export x)\n",
                "  (import (rnrs))\n\n",
                "  (doc 'x (summary \"X\"))\n\n",
                "  (define x 1))\n",
            )
        );
    }

    #[test]
    fn preserves_line_comments_as_forms() {
        assert_eq!(
            format_text("; hello\n(define x 1)").unwrap(),
            "; hello\n\n(define x 1)\n"
        );
    }

    #[test]
    fn rejects_unbalanced_lists() {
        assert!(matches!(
            format_text("(define x 1").unwrap_err(),
            FormatError::UnclosedList { expected: ')' }
        ));
    }

    #[test]
    fn ignores_file_when_directed() {
        let source = "; scaffold-fmt: ignore-file\n(define x 1)(define y 2)";

        assert_eq!(format_text(source).unwrap(), source);
    }

    #[test]
    fn preserves_format_off_regions() {
        let source = concat!(
            "(define x 1)(define y 2)\n",
            "; scaffold-fmt: off\n",
            "(define     hand-aligned       '(1  2  3))\n",
            "; scaffold-fmt: on\n",
            "(define z 3)(define q 4)"
        );
        let formatted = format_text(source).unwrap();

        assert!(formatted.starts_with("(define x 1)\n\n(define y 2)\n"));
        assert!(formatted.contains("(define     hand-aligned       '(1  2  3))\n"));
        assert!(formatted.ends_with("(define z 3)\n\n(define q 4)\n"));
    }

    #[test]
    fn preserves_next_form_when_directed() {
        let source = concat!(
            "(define x 1)(define y 2)\n",
            "; scaffold-fmt: ignore-next\n",
            "(define     hand-aligned       '(1  2  3))\n",
            "(define z 3)(define q 4)",
        );

        let formatted = format_text(source).unwrap();

        assert!(formatted.starts_with("(define x 1)\n\n(define y 2)\n"));
        assert!(formatted.contains("; scaffold-fmt: ignore-next\n"));
        assert!(formatted.contains("(define     hand-aligned       '(1  2  3))\n"));
        assert!(formatted.ends_with("(define z 3)\n\n(define q 4)\n"));
    }

    #[test]
    fn formats_path_in_check_mode_without_writing() {
        let (_root, path) = temp_path("format-path-check.scm");
        std::fs::write(&path, "(define x 1)(define y 2)").expect("write fixture");

        let changed = format_path(&path, FormatMode::Check).expect("format check");

        assert!(changed);
        assert_eq!(
            std::fs::read_to_string(&path).expect("read original"),
            "(define x 1)(define y 2)"
        );
    }

    #[test]
    fn formats_path_in_write_mode() {
        let (_root, path) = temp_path("format-path-write.scm");
        std::fs::write(&path, "(define x 1)(define y 2)").expect("write fixture");

        let changed = format_path(&path, FormatMode::Write).expect("format write");

        assert!(changed);
        assert_eq!(
            std::fs::read_to_string(&path).expect("read formatted"),
            "(define x 1)\n\n(define y 2)\n"
        );
    }

    fn assert_format(source: &str, expected: &str) {
        assert_format_with_options(source, FormatOptions::default(), expected);
    }

    fn assert_format_with_options(source: &str, options: FormatOptions, expected: &str) {
        let formatted = format_text_with_options(source, options).expect("format source");
        assert_eq!(formatted, expected);
        assert_eq!(
            format_text_with_options(&formatted, options).expect("format formatted source"),
            formatted,
            "formatter should be idempotent"
        );
        let _forms = Parser::new(&formatted)
            .parse_all()
            .expect("formatted source should parse");
    }

    fn temp_path(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().expect("temp dir");
        let path = dir.path().join(name);
        (dir, path)
    }
}
