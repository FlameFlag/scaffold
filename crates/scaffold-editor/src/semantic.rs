use std::collections::HashSet;

use crate::text::{SourceSpanKind, source_spans_line};

pub const TOKEN_FUNCTION: &str = "function";
pub const TOKEN_KEYWORD: &str = "keyword";
pub const TOKEN_STRING: &str = "string";
pub const TOKEN_COMMENT: &str = "comment";
pub const TOKEN_PARAMETER: &str = "parameter";

pub const MOD_DEFAULT_LIBRARY: &str = "defaultLibrary";
pub const MOD_DOCUMENTATION: &str = "documentation";
pub const MOD_DEPRECATED: &str = "deprecated";

const DOCUMENTATION_FORMS: &[&str] = &[
    "deprecated",
    "doc",
    "doc-next",
    "effect",
    "example",
    "extern-doc",
    "group",
    "hidden",
    "markdown",
    "moduledoc",
    "param",
    "returns",
    "requires-capability",
    "see",
    "signature",
    "since",
    "stability",
    "summary",
    "typedoc",
];

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SemanticToken {
    pub text: String,
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub token_type: &'static str,
    pub modifiers: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceKind {
    Function,
    Keyword,
}

#[derive(Debug, Clone)]
pub struct ReferenceInfo {
    pub kind: ReferenceKind,
    pub source: Option<String>,
    pub deprecated: bool,
}

pub trait SemanticReferenceIndex {
    fn reference_info(&self, symbol: &str) -> Option<ReferenceInfo>;
}

pub fn semantic_tokens<I>(
    index: &I,
    text: &str,
    user_symbols: impl IntoIterator<Item = String>,
) -> Vec<SemanticToken>
where
    I: SemanticReferenceIndex,
{
    let user_symbols = user_symbols.into_iter().collect::<HashSet<_>>();
    let mut tokens = Vec::new();
    for (line_index, line) in text.lines().enumerate() {
        scan_semantic_line(index, line_index as u32, line, &user_symbols, &mut tokens);
    }
    tokens.sort_by_key(|token| (token.line, token.start));
    tokens
}

fn scan_semantic_line<I>(
    index: &I,
    line_index: u32,
    line: &str,
    user_symbols: &HashSet<String>,
    output: &mut Vec<SemanticToken>,
) where
    I: SemanticReferenceIndex,
{
    for span in source_spans_line(line) {
        match span.kind {
            SourceSpanKind::Comment => {
                output.push(SemanticToken {
                    text: span.text.to_owned(),
                    line: line_index,
                    start: span.start_utf16(line),
                    length: span.len_utf16(),
                    token_type: TOKEN_COMMENT,
                    modifiers: Vec::new(),
                });
                return;
            }
            SourceSpanKind::String => {
                output.push(SemanticToken {
                    text: span.text.to_owned(),
                    line: line_index,
                    start: span.start_utf16(line),
                    length: span.len_utf16(),
                    token_type: TOKEN_STRING,
                    modifiers: Vec::new(),
                });
            }
            SourceSpanKind::Symbol => {
                if let Some((token_type, modifiers)) =
                    classify_symbol(index, span.text, user_symbols)
                {
                    output.push(SemanticToken {
                        text: span.text.to_owned(),
                        line: line_index,
                        start: span.start_utf16(line),
                        length: span.len_utf16(),
                        token_type,
                        modifiers,
                    });
                }
            }
        }
    }
}

fn classify_symbol<I>(
    index: &I,
    symbol: &str,
    user_symbols: &HashSet<String>,
) -> Option<(&'static str, Vec<&'static str>)>
where
    I: SemanticReferenceIndex,
{
    if symbol.starts_with("#:") {
        return Some((TOKEN_PARAMETER, Vec::new()));
    }
    if DOCUMENTATION_FORMS.contains(&symbol) {
        let mut modifiers = vec![MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION];
        if index
            .reference_info(symbol)
            .is_some_and(|entry| entry.deprecated)
        {
            modifiers.push(MOD_DEPRECATED);
        }
        return Some((TOKEN_FUNCTION, modifiers));
    }
    if user_symbols.contains(symbol) {
        return Some((TOKEN_FUNCTION, Vec::new()));
    }
    let entry = index.reference_info(symbol)?;
    let token_type = match entry.kind {
        ReferenceKind::Keyword => TOKEN_KEYWORD,
        ReferenceKind::Function => TOKEN_FUNCTION,
    };
    let mut modifiers = Vec::new();
    if entry_is_default_library(&entry) {
        modifiers.push(MOD_DEFAULT_LIBRARY);
    }
    if entry.deprecated {
        modifiers.push(MOD_DEPRECATED);
    }
    Some((token_type, modifiers))
}

fn entry_is_default_library(entry: &ReferenceInfo) -> bool {
    entry.source.as_deref().is_some_and(|source| {
        source == "scheme keyword"
            || source.starts_with("src/dsl/std/")
            || source.starts_with("src/extensions/")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct EmptyIndex;

    impl SemanticReferenceIndex for EmptyIndex {
        fn reference_info(&self, _symbol: &str) -> Option<ReferenceInfo> {
            None
        }
    }

    #[test]
    fn classifies_strings_comments_and_user_symbols() {
        let tokens = semantic_tokens(
            &EmptyIndex,
            "(local-helper \"demo\") ; comment",
            ["local-helper".to_owned()],
        );

        assert_eq!(tokens[0].token_type, TOKEN_FUNCTION);
        assert_eq!(tokens[1].token_type, TOKEN_STRING);
        assert_eq!(tokens[2].token_type, TOKEN_COMMENT);
    }

    #[test]
    fn documentation_forms_are_default_library_documentation() {
        let tokens = semantic_tokens(&EmptyIndex, "(doc 'x)", []);

        let doc = tokens
            .iter()
            .find(|token| token.text == "doc")
            .expect("doc token");
        assert_eq!(doc.token_type, TOKEN_FUNCTION);
        assert_eq!(doc.modifiers, vec![MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION]);
    }

    #[test]
    fn classifies_keyword_parameters() {
        let tokens = semantic_tokens(&EmptyIndex, "(tool #:name \"demo\")", []);

        let parameter = tokens
            .iter()
            .find(|token| token.text == "#:name")
            .expect("parameter token");
        assert_eq!(parameter.token_type, TOKEN_PARAMETER);
    }
}
