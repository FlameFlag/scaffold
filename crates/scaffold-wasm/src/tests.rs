use super::*;

#[test]
fn formats_scheme_text_for_wasm_consumers() {
    assert_eq!(
        format_scaffold_scheme("(define x 1)(define y 2)").expect("format"),
        "(define x 1)\n\n(define y 2)\n"
    );
}

#[test]
fn preserves_formatter_directives() {
    let source = "; scaffold-fmt: ignore-file\n(define x 1)(define y 2)";

    assert_eq!(format_scaffold_scheme(source).expect("format"), source);
}

#[test]
fn returns_json_syntax_diagnostics() {
    let diagnostics = diagnose_scaffold_scheme("(define x 1");

    assert!(diagnostics.contains("scaffold::dsl::syntax"));
    assert!(diagnostics.contains("expected closing delimiter"));
    assert!(!diagnostics.contains("scaffold::dsl::missing-doc"));
}

#[test]
fn returns_json_missing_doc_diagnostics() {
    let diagnostics = diagnose_scaffold_scheme("(define (local-helper value) value)");
    let diagnostics: serde_json::Value =
        serde_json::from_str(&diagnostics).expect("diagnostics json");

    assert_eq!(diagnostics[0]["code"], "scaffold::dsl::missing-doc");
    assert!(
        diagnostics[0]["message"]
            .as_str()
            .expect("message")
            .contains("Document `local-helper`")
    );
    assert_eq!(diagnostics[0]["data"]["line"], 0);
    assert_eq!(diagnostics[0]["data"]["name"], "local-helper");
}

#[test]
fn returns_missing_doc_stub_for_quick_fixes() {
    assert_eq!(
        missing_doc_stub_scaffold_scheme("local-helper", "  "),
        concat!(
            "  (doc-next\n",
            "    (summary \"Describe `local-helper`.\"))\n\n"
        )
    );
}

#[test]
fn accepts_documented_definitions_for_wasm_diagnostics() {
    let diagnostics = diagnose_scaffold_scheme(
        "(doc 'local-helper (summary \"Docs\"))\n(define (local-helper value) value)",
    );

    assert_eq!(diagnostics, "[]");
}

#[test]
fn returns_json_semantic_tokens() {
    let tokens = semantic_tokens_scaffold_scheme(
        "(define (local-helper value) value)\n(if #t (tool #:name \"demo\") (local-helper \"demo\"))\n(doc 'local-helper (summary \"Docs\"))",
    );

    assert!(tokens.contains("\"token_type\":\"keyword\""));
    assert!(tokens.contains("\"defaultLibrary\""));
    assert!(tokens.contains("\"documentation\""));
    assert!(tokens.contains("\"token_type\":\"parameter\""));
    assert!(tokens.contains("local-helper"));
}

#[test]
fn semantic_tokens_use_shared_reference_for_keywords_and_stdlib() {
    let tokens = serde_json::from_str::<Vec<serde_json::Value>>(&semantic_tokens_scaffold_scheme(
        "(if #t (tool \"demo\" (required)) (begin))",
    ))
    .expect("semantic tokens parse");

    let keyword = tokens
        .iter()
        .find(|token| token["text"] == "if")
        .expect("if token");
    assert_eq!(keyword["token_type"], "keyword");
    assert_eq!(keyword["modifiers"], serde_json::json!(["defaultLibrary"]));

    let stdlib = tokens
        .iter()
        .find(|token| token["text"] == "tool")
        .expect("tool token");
    assert_eq!(stdlib["token_type"], "function");
    assert_eq!(stdlib["modifiers"], serde_json::json!(["defaultLibrary"]));
}

#[test]
fn returns_reference_backed_completions() {
    let completions = completion_items_scaffold_scheme();

    assert!(completions.contains("\"label\":\"tool\""));
    assert!(completions.contains("\"label\":\"path/exists?\""));
    assert!(completions.contains("\"label\":\"path/join\""));
    assert!(completions.contains("\"label\":\"command/path\""));
    assert!(completions.contains("\"label\":\"workspace/path\""));
    assert!(completions.contains("\"kind\":\"function\""));
    assert!(completions.contains("Create a catalog tool object"));
}

#[test]
fn returns_reference_backed_hover_markdown() {
    let hover = hover_scaffold_scheme("tool");

    assert!(hover.contains("```scheme"));
    assert!(hover.contains("Create a catalog tool object"));
    assert!(hover_scaffold_scheme("not-real").is_empty());
}

#[test]
fn returns_reference_backed_signature_help() {
    let help = signature_help_scaffold_scheme("catalog/tool");

    assert!(help.contains("\"label\":\"(catalog/tool name action field ...)\""));
    assert!(help.contains("Create a raw catalog tool list for macro-oriented helpers."));
    assert!(help.contains("\"label\":\"name\""));
}

#[test]
fn returns_workspace_aware_signature_help() {
    let workspace_json = serde_json::json!([
        {
            "uri": "file:///workspace/acme.scm",
            "text": concat!(
                "(library (acme tools)\n",
                "  (export acme-tool)\n",
                "  (doc 'acme-tool\n",
                "    (signature \"(acme-tool name [mode])\")\n",
                "    (summary \"Acme.\")\n",
                "    (param 'name \"Name docs.\")))",
            )
        }
    ])
    .to_string();
    let help = signature_help_scaffold_scheme_for_document(
        "(import (acme tools))\n(acme-tool \"demo\")",
        "acme-tool",
        &workspace_json,
    );

    assert!(help.contains("\"label\":\"(acme-tool name [mode])\""));
    assert!(help.contains("\"label\":\"name\""));
    assert!(help.contains("Name docs."));
    assert!(help.contains("\"label\":\"[mode]\""));
}

#[test]
fn returns_reference_entries() {
    let entries = reference_entries_scaffold_scheme();

    assert!(entries.contains("\"name\":\"tool\""));
    assert!(entries.contains("\"name\":\"path/exists?\""));
    assert!(entries.contains("\"name\":\"path/join\""));
    assert!(entries.contains("\"name\":\"host/matches?\""));
    assert!(entries.contains("\"name\":\"workspace/path\""));
    assert!(entries.contains("\"group\":\"Catalog\""));
}

#[test]
fn returns_reference_capability_contracts() {
    let capabilities = reference_capabilities_scaffold_scheme();

    assert!(capabilities.contains("\"library\":\"(scaffold fs)\""));
    assert!(capabilities.contains("\"effect\":\"host-read-only\""));
    assert!(capabilities.contains("\"catalog\":\"available\""));
    assert!(capabilities.contains("\"wasm\":\"reference-only\""));
}

#[test]
fn returns_reference_catalog_schema() {
    let schema = reference_catalog_schema_scaffold_scheme();

    assert!(schema.contains("\"title\":\"Scaffold Catalog\""));
    assert!(schema.contains("\"name\":\"tool\""));
    assert!(schema.contains("\"action_type\""));
    assert!(schema.contains("\"cycle_checked\":true"));
}

#[test]
fn returns_document_reference_symbols_for_documented_entries() {
    let symbols = document_reference_symbols_scaffold_scheme(
        "(doc 'local-helper (signature \"(local-helper value)\") (summary \"Docs\"))\n(define (local-helper value) value)",
    );

    assert!(symbols.contains("\"name\":\"local-helper\""));
    assert!(symbols.contains("\"detail\":\"(local-helper value)\""));
    assert!(!symbols.contains("\"name\":\"define\""));
}

#[test]
fn returns_workspace_reference_locations() {
    let workspace = serde_json::json!([
        {
            "uri": "file:///workspace/b.scm",
            "text": "(local-helper 1)\n\"local-helper\"",
        },
        {
            "uri": "file:///workspace/a.scm",
            "text": "(define (local-helper value) value)",
        },
    ])
    .to_string();
    let locations = reference_locations_scaffold_scheme("local-helper", &workspace);

    assert!(locations.contains("\"uri\":\"file:///workspace/a.scm\""));
    assert!(locations.contains("\"uri\":\"file:///workspace/b.scm\""));
    assert!(!locations.contains("\"line\":1"));
    assert!(locations.find("file:///workspace/a.scm") < locations.find("file:///workspace/b.scm"));
}

#[test]
fn returns_symbol_at_utf16_position() {
    let text = "(define café 1)";

    assert_eq!(symbol_at_scaffold_scheme(text, 0, 10), "café");
    assert_eq!(symbol_at_scaffold_scheme(text, 1, 0), "");
}

#[test]
fn returns_form_context_for_signature_help() {
    let context = form_context_scaffold_scheme("(tool \"demo\" (required) #:name \"rg\")", 0, 26);

    assert!(context.contains("\"head\":\"tool\""));
    assert!(context.contains("\"active_argument\":2"));
}

#[test]
fn form_context_ignores_strings_and_comments() {
    assert_eq!(form_context_scaffold_scheme("\"(tool demo\"", 0, 7), "null");
    assert_eq!(form_context_scaffold_scheme("; (tool demo)", 0, 8), "null");
}

#[test]
fn discovers_nested_definitions_and_docs() {
    let source = concat!(
        "(begin\n",
        "  (define (nested-helper value) value)\n",
        "  (doc 'nested-helper (summary \"Docs\")))",
    );
    let symbols = document_reference_symbols_scaffold_scheme(source);
    let tokens = semantic_tokens_scaffold_scheme(source);

    assert!(symbols.contains("\"name\":\"nested-helper\""));
    assert!(tokens.contains("nested-helper"));
}

#[test]
fn returns_reference_backed_inlay_hints() {
    let hints = inlay_hints_scaffold_scheme(
        "(tool \"demo\" (required))\n\"tool ignored\"\n; (tool \"ignored\")",
        0,
        0,
        10,
        0,
    );

    assert!(hints.contains("\"label\":\"name:\""));
    assert!(hints.contains("\"label\":\"action:\""));
    assert!(!hints.contains("ignored"));
}

#[test]
fn returns_workspace_aware_completion_hover_semantics_and_definition() {
    let workspace_json = serde_json::json!([
        {
            "uri": "file:///workspace/acme.scm",
            "text": "(library (acme tools) (export acme-tool) (doc 'acme-tool (signature \"(acme-tool name)\") (summary \"Acme.\")))"
        }
    ])
    .to_string();
    let text = "(import (acme tools))\n(acme-tool \"demo\")";

    assert!(
        completion_items_scaffold_scheme_for_document(text, &workspace_json)
            .contains("\"label\":\"acme-tool\"")
    );
    assert!(
        hover_scaffold_scheme_for_document(text, "acme-tool", &workspace_json).contains("Acme.")
    );
    assert!(
        semantic_tokens_scaffold_scheme_for_document(text, &workspace_json).contains("acme-tool")
    );
    assert!(
        definition_scaffold_scheme(text, "file:///workspace/main.scm", 1, 2, &workspace_json)
            .contains("file:///workspace/acme.scm")
    );
}

#[test]
fn local_definition_uses_document_uri() {
    let text = concat!(
        "(doc 'local-helper (summary \"Docs\"))\n",
        "(define (local-helper value) value)\n",
        "(local-helper 1)"
    );

    let definition = definition_scaffold_scheme(text, "file:///workspace/main.scm", 2, 2, "[]");

    assert!(definition.contains("file:///workspace/main.scm"));
    assert!(!definition.contains("<open document>"));
}

#[test]
fn undocumented_local_definition_has_definition_location() {
    let text = "(define (local-helper value) value)\n(local-helper 1)";
    let definition = definition_scaffold_scheme(text, "file:///workspace/main.scm", 1, 2, "[]");

    assert!(definition.contains("file:///workspace/main.scm"));
    assert!(definition.contains("\"line\":0"));
}
