use tower_lsp::lsp_types::{Documentation, InlayHintLabel, InsertTextFormat, Position, Range};

use super::semantic::{MOD_DEFAULT_LIBRARY, MOD_DOCUMENTATION, TOKEN_FUNCTION, TOKEN_KEYWORD};
use super::*;
use crate::docs::DocIndex;

#[test]
fn includes_scaffold_catalog_helpers_from_doc_index() {
    let index = DocIndex::scaffold();
    let labels = completion_items(&index)
        .into_iter()
        .map(|item| item.label)
        .collect::<Vec<_>>();

    assert!(labels.contains(&"tool".to_owned()));
    assert!(labels.contains(&"arr/append-list".to_owned()));
    assert!(labels.contains(&"path/exists?".to_owned()));
    assert!(labels.contains(&"path/join".to_owned()));
    assert!(labels.contains(&"command/path".to_owned()));
    assert!(labels.contains(&"workspace/path".to_owned()));
}

#[test]
fn returns_hover_for_indexed_symbol() {
    let index = DocIndex::scaffold();

    assert!(hover_for_symbol(&index, "tool").is_some());
    assert!(hover_for_symbol(&index, "does-not-exist").is_none());
}

#[test]
fn completion_items_include_markdown_docs_and_snippets() {
    let index = DocIndex::scaffold();
    let item = completion_items(&index)
        .into_iter()
        .find(|item| item.label == "tool")
        .expect("tool completion");

    assert_eq!(item.insert_text_format, Some(InsertTextFormat::SNIPPET));
    assert!(item.documentation.is_some());
}

#[test]
fn returns_signature_help_for_indexed_symbol() {
    let index = DocIndex::scaffold();
    let help = signature_help_for_symbol(&index, "tool", 0).expect("signature help");

    assert!(help.signatures[0].label.contains("tool"));
}

#[test]
fn signature_help_includes_parameter_docs() {
    let index = DocIndex::with_language_keywords()
        .merged_with_document("doc-entry.scm", include_str!("../fixtures/doc-entry.scm"));
    let help = signature_help_for_symbol(&index, "demo", 0).expect("signature help");
    let params = help.signatures[0].parameters.as_ref().expect("parameters");

    assert_eq!(
        params[0].documentation,
        Some(Documentation::String("Name for the demo.".to_owned()))
    );
}

#[test]
fn signature_help_selects_active_argument() {
    let index = DocIndex::with_language_keywords().merged_with_document(
        "two-param-doc.scm",
        concat!(
            "(doc 'demo\n",
            "  (signature \"(demo name value)\")\n",
            "  (summary \"Create a demo.\")\n",
            "  (param 'name \"Name for the demo.\")\n",
            "  (param 'value \"Demo value.\"))",
        ),
    );
    let help = signature_help_for_symbol(&index, "demo", 1).expect("signature help");

    assert_eq!(help.active_parameter, Some(1));
    assert_eq!(help.signatures[0].active_parameter, Some(1));
}

#[test]
fn emits_semantic_tokens_for_indexed_symbols() {
    let index = DocIndex::scaffold();
    let tokens = semantic_tokens(
        &index,
        concat!(
            include_str!("../fixtures/semantic-tokens.scm"),
            "\n(tool #:name \"parameter-demo\")\n",
        ),
    );

    assert!(tokens.data.iter().any(|token| token.token_type == 0));
    assert!(tokens.data.iter().any(|token| token.token_type == 2));
    assert!(tokens.data.iter().any(|token| token.token_type == 3));
    assert!(tokens.data.iter().any(|token| token.token_type == 4));
}

#[test]
fn semantic_tokens_distinguish_keyword_std_doc_and_user_functions() {
    let index = DocIndex::scaffold();
    let tokens = semantic_tokens(
        &index,
        concat!(
            "(import (rnrs) (scaffold catalog))\n",
            "(define (local-helper value) value)\n",
            "(doc 'local-helper (summary \"Local helper.\"))\n",
            "(tool \"demo\" (required))\n",
            "(local-helper \"demo\")\n",
        ),
    );

    assert!(tokens.data.iter().any(|token| {
        token.token_type == TOKEN_KEYWORD && token.token_modifiers_bitset & MOD_DEFAULT_LIBRARY != 0
    }));
    assert!(tokens.data.iter().any(|token| {
        token.token_type == TOKEN_FUNCTION
            && token.token_modifiers_bitset & MOD_DEFAULT_LIBRARY != 0
            && token.token_modifiers_bitset & MOD_DOCUMENTATION == 0
    }));
    assert!(tokens.data.iter().any(|token| {
        token.token_type == TOKEN_FUNCTION && token.token_modifiers_bitset & MOD_DOCUMENTATION != 0
    }));
    assert!(
        tokens
            .data
            .iter()
            .any(|token| token.token_type == TOKEN_FUNCTION && token.token_modifiers_bitset == 0)
    );
}

#[test]
fn semantic_tokens_keep_user_symbols_in_files_with_scaffold_keywords() {
    let index = DocIndex::scaffold();
    let tokens = semantic_tokens(
        &index,
        concat!(
            "(tool #:name \"demo\")\n",
            "(define (local-helper value) value)\n",
            "(local-helper 1)\n",
        ),
    );

    assert!(
        tokens
            .data
            .iter()
            .any(|token| token.token_type == TOKEN_FUNCTION && token.token_modifiers_bitset == 0)
    );
}

#[test]
fn emits_doc_driven_parameter_inlay_hints() {
    let index = DocIndex::scaffold();
    let hints = inlay_hints(
        &index,
        include_str!("../fixtures/inlay-hints.scm"),
        Range::new(Position::new(0, 0), Position::new(20, 0)),
    );
    let labels = hints
        .iter()
        .filter_map(|hint| match &hint.label {
            InlayHintLabel::String(label) => Some(label.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(labels.contains(&"name:"));
    assert!(labels.contains(&"action:"));
    assert!(labels.contains(&"field:"));
}

#[test]
fn emits_inlay_hints_in_files_with_scaffold_keywords() {
    let text = concat!(
        "(tool #:name \"demo\")\n",
        "(doc 'local-helper\n",
        "  (signature \"(local-helper value)\")\n",
        "  (summary \"Docs.\")\n",
        "  (param 'value \"Input value.\"))\n",
        "(local-helper 1)\n",
    );
    let index = DocIndex::with_language_keywords().merged_with_document("keyword-inlay.scm", text);
    let hints = inlay_hints(
        &index,
        text,
        Range::new(Position::new(0, 0), Position::new(20, 0)),
    );
    let has_value_label = hints
        .iter()
        .filter_map(|hint| match &hint.label {
            InlayHintLabel::String(label) => Some(label.as_str()),
            _ => None,
        })
        .any(|label| label == "value:");

    assert!(has_value_label);
}

#[test]
fn returns_workspace_symbols_from_doc_index() {
    let index = DocIndex::scaffold();
    let symbols = workspace_symbols(&index, "nix profile");
    let has_profile_symbol = symbols
        .iter()
        .any(|symbol| symbol.name == "nix/profile-package");

    assert!(has_profile_symbol);
    assert!(
        symbols
            .iter()
            .all(|symbol| symbol.location.uri.scheme() == "file")
    );
}

#[test]
fn finds_symbol_reference_ranges_without_strings_or_comments() {
    let text = include_str!("../fixtures/references.scm");
    let ranges = symbol_ranges(text, "local-helper");
    let string_line = line_containing(text, "\"local-helper in a string\"");
    let comment_line = line_containing(text, "; local-helper in a comment");

    assert!(ranges.len() >= 3);
    assert!(!ranges.iter().any(|range| range.start.line == string_line));
    assert!(!ranges.iter().any(|range| range.start.line == comment_line));
}

fn line_containing(text: &str, needle: &str) -> u32 {
    text.lines()
        .position(|line| line.contains(needle))
        .expect("fixture line exists") as u32
}
