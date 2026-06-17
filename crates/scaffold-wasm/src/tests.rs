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
    let tokens = semantic_tokens_scaffold_scheme(include_str!("fixtures/semantic-tokens.scm"));

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

    let workspace_hover = hover_scaffold_scheme("source/path");
    assert!(workspace_hover.contains("context-read-only"));
    assert!(workspace_hover.contains("`scaffold.workspace`"));
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
            "text": include_str!("fixtures/workspace-signature-acme.scm")
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
    assert!(entries.contains("\"effect\":\"context-read-only\""));
    assert!(entries.contains("\"requires_capability\":[\"scaffold.workspace\"]"));
}

#[test]
fn returns_ranked_reference_search_entries() {
    let entries = serde_json::from_str::<Vec<serde_json::Value>>(
        &search_reference_entries_scaffold_scheme("ctlg tool", 20),
    )
    .expect("search entries json");

    assert_eq!(entries[0]["name"], "catalog/tool");
    assert_eq!(entries[0]["raw_markdown"], entries[0]["markdown"]);
    assert!(
        entries[0]["rendered_markdown"]
            .as_str()
            .is_some_and(|markdown| {
                markdown.contains("```scheme\n(catalog/tool name action field ...)\n```")
                    && markdown.contains("**Example**")
            })
    );
    assert!(
        search_reference_entries_scaffold_scheme("zzzzzzz", 20)
            .parse::<serde_json::Value>()
            .expect("empty search json")
            .as_array()
            .expect("empty search array")
            .is_empty()
    );
}

#[test]
fn returns_reference_entry_suggestions_for_close_symbol_typos() {
    let suggestions = serde_json::from_str::<Vec<serde_json::Value>>(
        &suggest_reference_entries_scaffold_scheme("catlgtool", 5),
    )
    .expect("suggestion entries json");

    assert_eq!(suggestions[0]["name"], "catalog/tool");
    assert!(
        suggest_reference_entries_scaffold_scheme("zzzzzzz", 5)
            .parse::<serde_json::Value>()
            .expect("empty suggestions json")
            .as_array()
            .expect("empty suggestions array")
            .is_empty()
    );
}

#[test]
fn reference_search_limit_is_clamped_for_wasm_consumers() {
    let oversized = serde_json::from_str::<Vec<serde_json::Value>>(
        &search_reference_entries_scaffold_scheme("tool", 999),
    )
    .expect("oversized search json");
    let max = serde_json::from_str::<Vec<serde_json::Value>>(
        &search_reference_entries_scaffold_scheme("tool", 100),
    )
    .expect("max-limit search json");
    let zero = serde_json::from_str::<Vec<serde_json::Value>>(
        &search_reference_entries_scaffold_scheme("tool", 0),
    )
    .expect("zero-limit search json");

    assert_eq!(oversized, max);
    assert!(oversized.len() <= 100);
    assert_eq!(zero.len(), 1);
}

#[test]
fn returns_workspace_aware_ranked_reference_search_entries() {
    let workspace_json = serde_json::json!([
        {
            "uri": "file:///workspace/acme.scm",
            "text": "(library (acme tools) (export acme-tool) (doc 'acme-tool (signature \"(acme-tool name)\") (summary \"Acme.\")))"
        }
    ])
    .to_string();
    let entries =
        search_reference_entries_scaffold_scheme_for_workspace("acme", &workspace_json, 20);

    assert!(entries.contains("\"name\":\"acme-tool\""));
    assert!(entries.contains("\"raw_markdown\""));
    assert!(entries.contains("\"rendered_markdown\""));
    assert!(entries.contains("(acme-tool name)"));
}

#[test]
fn returns_workspace_aware_reference_entry_suggestions() {
    let workspace_json = serde_json::json!([
        {
            "uri": "file:///workspace/acme.scm",
            "text": "(library (acme tools) (export acme-widget) (doc 'acme-widget (signature \"(acme-widget name)\") (summary \"Acme.\")))"
        }
    ])
    .to_string();
    let suggestions = serde_json::from_str::<Vec<serde_json::Value>>(
        &suggest_reference_entries_scaffold_scheme_for_workspace("acmewidget", &workspace_json, 5),
    )
    .expect("workspace suggestion entries json");

    assert_eq!(suggestions[0]["name"], "acme-widget");
}

#[test]
fn document_reference_entries_use_language_group_fallback() {
    let entries = serde_json::from_str::<Vec<serde_json::Value>>(
        &reference_entries_scaffold_scheme_for_document(
            "(doc 'local-helper (summary \"Docs\"))\n(define (local-helper value) value)",
            "file:///workspace/main.scm",
            "[]",
        ),
    )
    .expect("reference entries json");
    let helper = entries
        .iter()
        .find(|entry| entry["name"] == "local-helper")
        .expect("local helper entry");

    assert_eq!(helper["group"], "Language");
}

#[test]
fn embedded_reference_json_matches_generator() {
    let embedded: serde_json::Value =
        serde_json::from_str(include_str!("reference.json")).expect("embedded reference json");
    let embedded_min: serde_json::Value =
        serde_json::from_str(include_str!("reference.min.json")).expect("minified reference json");
    let generated: serde_json::Value = serde_json::from_str(
        &scaffold_docs::scaffold_reference_json().expect("generated reference json"),
    )
    .expect("generated reference json parses");

    assert_eq!(embedded, generated);
    assert_eq!(embedded_min, embedded);
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
    let source = include_str!("fixtures/nested-definitions.scm");
    let symbols = document_reference_symbols_scaffold_scheme(source);
    let tokens = semantic_tokens_scaffold_scheme(source);

    assert!(symbols.contains("\"name\":\"nested-helper\""));
    assert!(tokens.contains("nested-helper"));
}

#[test]
fn returns_reference_backed_inlay_hints() {
    let hints = inlay_hints_scaffold_scheme(
        include_str!("fixtures/inlay-hints-ignore-strings-comments.scm"),
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
fn readme_lists_current_wasm_exports() {
    let readme = include_str!("../README.md");
    let listed_exports = readme
        .split_once("Current export:\n\n")
        .expect("current export section")
        .1
        .lines()
        .take_while(|line| line.starts_with("- `"))
        .map(|line| {
            line.strip_prefix("- `")
                .and_then(|line| line.strip_suffix('`'))
                .expect("export bullet")
        })
        .collect::<Vec<_>>();
    let listed_export_names = listed_exports
        .iter()
        .map(|export| export.split_once('(').expect("function signature").0)
        .collect::<Vec<_>>();
    let rust_export_names = include_str!("lib.rs")
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("#[wasm_bindgen(js_name = ")
                .and_then(|line| line.strip_suffix(")]"))
        })
        .collect::<Vec<_>>();

    assert_eq!(listed_export_names, rust_export_names);

    assert_eq!(
        listed_exports,
        vec![
            "formatScaffoldScheme(text: string): string",
            "diagnoseScaffoldScheme(text: string): string",
            "missingDocStubScaffoldScheme(name: string, indent: string): string",
            "semanticTokensScaffoldScheme(text: string): string",
            "semanticTokensScaffoldSchemeForDocument(text: string, workspaceJson: string): string",
            "completionItemsScaffoldScheme(): string",
            "completionItemsScaffoldSchemeForDocument(text: string, workspaceJson: string): string",
            "hoverScaffoldScheme(symbol: string): string",
            "hoverScaffoldSchemeForDocument(text: string, symbol: string, workspaceJson: string): string",
            "signatureHelpScaffoldScheme(symbol: string): string",
            "signatureHelpScaffoldSchemeForDocument(text: string, symbol: string, workspaceJson: string): string",
            "referenceEntriesScaffoldScheme(): string",
            "searchReferenceEntriesScaffoldScheme(query: string, limit: number): string",
            "suggestReferenceEntriesScaffoldScheme(query: string, limit: number): string",
            "referenceCapabilitiesScaffoldScheme(): string",
            "referenceCatalogSchemaScaffoldScheme(): string",
            "referenceEntriesScaffoldSchemeForWorkspace(workspaceJson: string): string",
            "searchReferenceEntriesScaffoldSchemeForWorkspace(query: string, workspaceJson: string, limit: number): string",
            "suggestReferenceEntriesScaffoldSchemeForWorkspace(query: string, workspaceJson: string, limit: number): string",
            "referenceEntriesScaffoldSchemeForDocument(text: string, uri: string, workspaceJson: string): string",
            "symbolAtScaffoldScheme(text: string, line: number, character: number): string",
            "formContextScaffoldScheme(text: string, line: number, character: number): string",
            "referenceLocationsScaffoldScheme(symbol: string, workspaceJson: string): string",
            "documentReferenceSymbolsScaffoldScheme(text: string): string",
            "inlayHintsScaffoldScheme(text: string, startLine: number, startCharacter: number, endLine: number, endCharacter: number): string",
            "inlayHintsScaffoldSchemeForDocument(text: string, workspaceJson: string, startLine: number, startCharacter: number, endLine: number, endCharacter: number): string",
            "definitionScaffoldScheme(text: string, uri: string, line: number, character: number, workspaceJson: string): string",
            "workspaceSymbolsScaffoldScheme(query: string, workspaceJson: string): string",
        ]
    );
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
    let text = include_str!("fixtures/local-definition.scm");

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
