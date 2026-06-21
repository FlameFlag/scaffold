use super::*;

fn docs_args() -> DocsArgs {
    DocsArgs {
        query: Vec::new(),
        all: false,
        search: None,
        group: None,
        source: None,
        limit: None,
        output: None,
        format: None,
    }
}

#[test]
fn docs_default_renders_group_overview_not_full_reference() {
    let rendered = render_docs(&docs_args()).expect("render docs");

    assert!(rendered.starts_with("Scaffold Docs"));
    assert!(rendered.contains("scaffold docs tool"));
    assert!(rendered.contains("scaffold docs --search \"ctlg tool\""));
    assert!(rendered.contains("scaffold docs --source src/dsl/std/catalog/tool.scm"));
    assert!(rendered.contains("Catalog"));
    assert!(!rendered.contains("# Scaffold Scheme Reference"));
}

#[test]
fn docs_query_renders_exact_entry() {
    let mut args = docs_args();
    args.query = vec!["tool".to_owned()];

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("tool\n===="));
    assert!(rendered.contains("(tool name action field ...)"));
    assert!(rendered.contains("Create a catalog tool object."));
    assert!(rendered.contains("parameter"));
    assert!(rendered.contains("summary"));
    assert!(rendered.contains("field"));
    assert!(rendered.contains("value"));
    assert!(!rendered.contains("  group   Catalog"));
}

#[test]
fn docs_query_renders_no_documentation_fallback_for_sparse_entry() {
    let mut args = docs_args();
    args.query = vec!["subject".to_owned()];

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("subject\n======="));
    assert!(rendered.contains("No documentation provided."));
    assert!(rendered.contains("Details"));
    assert!(rendered.contains("src/dsl/std/core/doc.scm:"));
}

#[test]
fn docs_query_exact_entry_is_case_insensitive() {
    let mut args = docs_args();
    args.query = vec!["TOOL/PATH".to_owned()];

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("tool/path\n========="));
    assert!(!rendered.contains("Search results for"));
}

#[test]
fn docs_query_does_not_expose_hidden_exact_entries() {
    let mut args = docs_args();
    args.query = vec!["action".to_owned()];

    let rendered = render_docs(&args).expect("render docs");

    assert!(!rendered.starts_with("action\n======"));
    assert!(!rendered.contains("(action ...)"));
    assert!(rendered.contains("Search results for `action`"));
    assert!(rendered.contains("package"));
    assert!(!rendered.contains("src/dsl/std/catalog/action.scm:11"));
}

#[test]
fn docs_query_fuzzy_searches_when_exact_entry_is_missing() {
    let mut args = docs_args();
    args.query = vec!["ctlg".to_owned(), "tool".to_owned()];

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.contains("Search results for `ctlg tool`"));
    assert!(rendered.contains("catalog/tool"));
    assert!(!rendered.contains("signature"));
    assert!(!rendered.contains("(catalog/tool name action field ...)"));
}

#[test]
fn docs_search_flag_always_renders_search_results() {
    let mut args = docs_args();
    args.search = Some("tool".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.contains("Search results for `tool`"));
    assert!(rendered.contains("catalog/tool"));
    assert!(!rendered.starts_with("tool\n===="));
}

#[test]
fn docs_query_rejects_unrelated_fuzzy_noise() {
    let mut args = docs_args();
    args.query = vec!["zzzzzzz".to_owned()];

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(rendered, "No reference entries matched `zzzzzzz`.\n");

    args.query = vec!["no-such-query".to_owned()];

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(rendered, "No reference entries matched `no-such-query`.\n");
}

#[test]
fn docs_search_empty_results_suggest_close_symbol_typos() {
    let mut args = docs_args();
    args.search = Some("catalgo".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No reference entries matched `catalgo`."));
    assert!(rendered.contains("Possible matches for `catalgo`"));
    assert!(rendered.contains("catalog"));
    assert!(rendered.contains("scaffold docs catalog"));
}

#[test]
fn docs_search_empty_markdown_results_suggest_close_symbol_typos() {
    let mut args = docs_args();
    args.search = Some("sourcpath".to_owned());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No reference entries matched `sourcpath`."));
    assert!(rendered.contains("## Possible matches for `sourcpath`"));
    assert!(rendered.contains("`source/path`"));
    assert!(rendered.contains("- `scaffold docs source/path`"));
    assert!(!rendered.contains("┌"));
}

#[test]
fn docs_markdown_search_escapes_reflected_backtick_queries() {
    let mut args = docs_args();
    args.search = Some("zzzz`query".to_owned());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(rendered, "No reference entries matched `` zzzz`query ``.\n");
    assert!(!rendered.contains("`zzzz`query`"));
}

#[test]
fn docs_query_rejects_empty_positional_query() {
    let mut args = docs_args();
    args.query = vec!["   ".to_owned()];

    let err = render_docs(&args).expect_err("empty query should fail");

    assert_eq!(err.to_string(), "docs query cannot be empty");
}

#[test]
fn docs_browse_flags_reject_empty_values() {
    let mut args = docs_args();
    args.search = Some("   ".to_owned());

    let err = render_docs(&args).expect_err("empty search should fail");
    assert_eq!(err.to_string(), "--search cannot be empty");

    args = docs_args();
    args.group = Some("   ".to_owned());

    let err = render_docs(&args).expect_err("empty group should fail");
    assert_eq!(err.to_string(), "--group cannot be empty");

    args = docs_args();
    args.source = Some("   ".to_owned());

    let err = render_docs(&args).expect_err("empty source should fail");
    assert_eq!(err.to_string(), "--source cannot be empty");
}

#[test]
fn docs_group_lists_entries_in_one_group() {
    let mut args = docs_args();
    args.group = Some("Catalog".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("Catalog docs"));
    assert!(rendered.contains("tool"));
    assert!(!rendered.contains("Scaffold Docs"));
}

#[test]
fn docs_group_title_uses_canonical_group_label() {
    let mut args = docs_args();
    args.group = Some("language".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("Language docs"));
    assert!(!rendered.starts_with("language docs"));
}

#[test]
fn docs_markdown_group_titles_escape_inline_markup() {
    assert_eq!(
        doc_group_markdown_title("Bad [Group] | Plus+"),
        "Bad \\[Group\\] | Plus\\+ docs"
    );
}

#[test]
fn docs_group_typo_suggests_matching_groups_without_full_overview() {
    let mut args = docs_args();
    args.group = Some("Catlog".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No documentation group named `Catlog`."));
    assert!(rendered.contains("Did you mean:"));
    assert!(rendered.contains("Catalog"));
    assert!(rendered.contains("scaffold docs --group Catalog"));
    assert!(!rendered.contains("Scaffold Docs"));
}

#[test]
fn docs_group_suggestions_quote_shell_arguments_with_spaces() {
    let mut args = docs_args();
    args.group = Some("mactools".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No documentation group named `mactools`."));
    assert!(rendered.contains("macOS tools"));
    assert!(rendered.contains("scaffold docs --group 'macOS tools'"));
    assert!(!rendered.contains("scaffold docs --group macOS tools"));
}

#[test]
fn docs_group_unrelated_short_query_does_not_suggest_noise() {
    let mut args = docs_args();
    args.group = Some("Nope".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(rendered, "No documentation group named `Nope`.\n");
    assert!(!rendered.contains("Download helpers"));
}

#[test]
fn docs_markdown_group_errors_escape_reflected_backticks() {
    let mut args = docs_args();
    args.group = Some("zzzz`group".to_owned());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(rendered, "No documentation group named `` zzzz`group ``.\n");
    assert!(!rendered.contains("`zzzz`group`"));
}

#[test]
fn docs_try_commands_shell_quote_only_when_needed() {
    assert_eq!(shell_arg("catalog/tool"), "catalog/tool");
    assert_eq!(shell_arg("macOS tools"), "'macOS tools'");
    assert_eq!(shell_arg("author's tool"), "\"author's tool\"");
}

#[test]
fn docs_markdown_try_commands_use_safe_code_spans() {
    assert_eq!(
        markdown_try_command("scaffold docs bad`name"),
        "- `` scaffold docs bad`name ``\n"
    );
}

#[test]
fn docs_markdown_code_spans_use_longer_delimiters_when_needed() {
    assert_eq!(markdown_code_span("catalog/tool"), "`catalog/tool`");
    assert_eq!(markdown_code_span("bad`query"), "`` bad`query ``");
    assert_eq!(markdown_code_span("bad``query"), "``` bad``query ```");
}

#[test]
fn docs_source_shows_recorded_location() {
    let mut args = docs_args();
    args.source = Some("tool".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("Source for `tool`"));
    assert!(rendered.contains("field"));
    assert!(rendered.contains("value"));
    assert!(rendered.contains("source"));
    assert!(rendered.contains("src/dsl/std/catalog/tool.scm"));
    assert!(rendered.contains("signature"));
    assert!(rendered.contains("(tool name action field ...)"));
}

#[test]
fn docs_query_text_includes_effect_and_capabilities() {
    let mut args = docs_args();
    args.query = vec!["source/path".to_owned()];

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.contains("effect"));
    assert!(rendered.contains("context-read-only"));
    assert!(rendered.contains("requires capability"));
    assert!(rendered.contains("scaffold.workspace"));
}

#[test]
fn docs_source_lookup_is_case_insensitive() {
    let mut args = docs_args();
    args.source = Some("Tool".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.contains("src/dsl/std/catalog/tool.scm"));
    assert!(rendered.contains("(tool name action field ...)"));
    assert!(!rendered.contains("No docs for"));
}

#[test]
fn docs_source_lists_entries_from_source_file() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/tool.scm".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("Docs from source `src/dsl/std/catalog/tool.scm`"));
    assert!(rendered.contains("tool"));
    assert!(rendered.contains("catalog/tool"));
    assert!(rendered.contains("Create a catalog tool object."));
    assert!(!rendered.contains("No documented symbol named"));
}

#[test]
fn docs_source_accepts_recorded_source_location() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/tool.scm:16".to_owned());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("## Docs from source `src/dsl/std/catalog/tool.scm`\n\n"));
    assert!(rendered.contains("| `tool`"));
    assert!(rendered.contains("| `tool/append-bins`"));
    assert!(!rendered.contains("No documented symbol named"));
}

#[test]
fn docs_source_accepts_any_line_in_known_source_file() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/tool.scm:999".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("Docs from source `src/dsl/std/catalog/tool.scm`"));
    assert!(rendered.contains("tool/append-bins"));
    assert!(!rendered.contains("No documented symbol named"));
}

#[test]
fn docs_source_accepts_line_and_column_in_known_source_file() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/tool.scm:16:1".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("Docs from source `src/dsl/std/catalog/tool.scm`"));
    assert!(rendered.contains("tool/append-bins"));
    assert!(!rendered.contains("No documented symbol named"));
}

#[test]
fn docs_source_missing_source_path_reports_source_not_symbol() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/missing.scm:1:1".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(
        rendered,
        "No documented source matched `src/dsl/std/catalog/missing.scm:1:1`.\n"
    );

    args.source = Some("catalog/missing".to_owned());
    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No documented symbol named `catalog/missing`."));
}

#[test]
fn docs_markdown_source_missing_source_path_reports_source_not_symbol() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/tool.scm:not-a-line".to_owned());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(
        rendered,
        "No documented source matched `src/dsl/std/catalog/tool.scm:not-a-line`.\n"
    );
}

#[test]
fn docs_source_typo_reports_missing_symbol_then_suggestions() {
    let mut args = docs_args();
    args.source = Some("Catlog".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No documented symbol named `Catlog`."));
    assert!(rendered.contains("Possible matches for `Catlog`"));
    assert!(rendered.contains("catalog/tool"));
    assert!(rendered.contains("Try:\n  scaffold docs --source catalog"));
    assert!(!rendered.contains("No docs for"));
}

#[test]
fn docs_source_empty_results_suggest_close_symbol_typos() {
    let mut args = docs_args();
    args.source = Some("catlgtool".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No documented symbol named `catlgtool`."));
    assert!(rendered.contains("Possible matches for `catlgtool`"));
    assert!(rendered.contains("catalog/tool"));
    assert!(rendered.contains("Create a raw catalog tool list for macro-oriented helpers."));
    assert!(rendered.contains("Try:\n  scaffold docs --source catalog/tool"));
}

#[test]
fn docs_source_unrelated_symbol_does_not_dump_noise() {
    let mut args = docs_args();
    args.source = Some("nope".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(rendered, "No documented symbol named `nope`.\n");
}

#[test]
fn docs_source_does_not_expose_hidden_exact_entries() {
    let mut args = docs_args();
    args.source = Some("action".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No documented symbol named `action`."));
    assert!(!rendered.contains("(action ...)"));
    assert!(!rendered.contains("src/dsl/std/catalog/action.scm:11"));
}

#[test]
fn docs_markdown_source_errors_escape_reflected_backticks() {
    let mut args = docs_args();
    args.source = Some("zzzz`source".to_owned());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert_eq!(rendered, "No documented symbol named `` zzzz`source ``.\n");
    assert!(!rendered.contains("`zzzz`source`"));
}

#[test]
fn docs_markdown_source_empty_results_suggest_close_symbol_typos() {
    let mut args = docs_args();
    args.source = Some("catlgtool".to_owned());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("No documented symbol named `catlgtool`."));
    assert!(rendered.contains("## Possible matches for `catlgtool`"));
    assert!(rendered.contains("| `catalog/tool`"));
    assert!(rendered.contains("Create a raw catalog tool list for macro-oriented helpers."));
    assert!(rendered.contains("## Try\n\n- `scaffold docs --source catalog/tool`"));
}

#[test]
fn docs_format_markdown_keeps_browse_overview() {
    let mut args = docs_args();
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("# Scaffold Docs"));
    assert!(rendered.contains("| Group"));
    assert!(rendered.contains("`Catalog`"));
    assert!(!rendered.contains("┌"));
    assert!(!rendered.starts_with("# Scaffold Scheme Reference"));
}

#[test]
fn docs_format_markdown_renders_search_markdown() {
    let mut args = docs_args();
    args.search = Some("ctlg tool".to_owned());
    args.limit = Some(1);
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("## Search results for `ctlg tool`"));
    assert!(rendered.contains("| Symbol"));
    assert!(rendered.contains("`catalog/tool`"));
    assert!(!rendered.contains("┌"));
}

#[test]
fn docs_search_matches_documented_examples() {
    let mut args = docs_args();
    args.search = Some("ripgrep".to_owned());
    args.limit = Some(5);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.contains("catalog/tool"));
    assert!(!rendered.contains("No reference entries matched"));
}

#[test]
fn docs_search_matches_source_locations() {
    let mut args = docs_args();
    args.search = Some("src/dsl/std/catalog/tool.scm:16:1".to_owned());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("Search results for `src/dsl/std/catalog/tool.scm:16:1`"));
    assert!(rendered.contains("tool"));
    assert!(!rendered.contains("No reference entries matched"));
}

#[test]
fn docs_format_markdown_renders_exact_entry_markdown_with_title() {
    let mut args = docs_args();
    args.query = vec!["tool".to_owned()];
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("## `tool`"));
    assert!(rendered.contains("```scheme\n(tool name action field ...)\n```"));
    assert!(rendered.contains("| Field  | Value"));
    assert!(rendered.contains("| Source | `src/dsl/std/catalog/tool.scm:"));
    assert!(!rendered.contains("┌"));
}

#[test]
fn docs_format_markdown_renders_no_documentation_fallback() {
    let mut args = docs_args();
    args.query = vec!["subject".to_owned()];
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("## `subject`"));
    assert!(rendered.contains("No documentation provided."));
    assert!(rendered.contains("| Source | `src/dsl/std/core/doc.scm:"));
}

#[test]
fn docs_format_markdown_renders_source_markdown() {
    let mut args = docs_args();
    args.source = Some("tool".to_owned());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("## `tool` source"));
    assert!(rendered.contains("| Field     | Value"));
    assert!(rendered.contains("| Source    | `src/dsl/std/catalog/tool.scm:"));
    assert!(rendered.contains("| Signature | `(tool name action field ...)`"));
}

#[test]
fn docs_format_json_renders_overview_json() {
    let mut args = docs_args();
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("overview json");

    assert_eq!(value["mode"], "overview");
    assert!(value["entry_count"].as_u64().is_some_and(|count| count > 0));
    assert!(
        value["groups"]
            .as_array()
            .is_some_and(|groups| { groups.iter().any(|group| group["name"] == "Catalog") })
    );
}

#[test]
fn docs_format_json_renders_exact_entry_json() {
    let mut args = docs_args();
    args.query = vec!["TOOL".to_owned()];
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("entry json");

    assert_eq!(value["mode"], "entry");
    assert_eq!(value["query"], "TOOL");
    assert_eq!(value["entry"]["name"], "tool");
    assert_eq!(value["entry"]["group"], "Catalog");
    assert_eq!(value["entry"]["kind"], "function");
    assert!(value["entry"]["markdown"].is_null());
    assert!(value["entry"]["raw_markdown"].is_null());
    assert!(
        value["entry"]["rendered_markdown"]
            .as_str()
            .is_some_and(|markdown| {
                markdown.contains("```scheme\n(tool name action field ...)\n```")
                    && markdown.contains("**Parameters**")
            })
    );
    assert!(
        value["entry"]["range"]["length"]
            .as_u64()
            .is_some_and(|length| length > 0)
    );
    assert!(
        value["entry"]["params"]
            .as_array()
            .is_some_and(|params| { params.iter().any(|param| param["name"] == "name") })
    );
}

#[test]
fn docs_format_json_renders_search_json() {
    let mut args = docs_args();
    args.search = Some("ctlg tool".to_owned());
    args.limit = Some(3);
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("search json");

    assert_eq!(value["mode"], "search");
    assert_eq!(value["query"], "ctlg tool");
    assert_eq!(value["limit"], 3);
    assert_eq!(value["count"], 3);
    assert_eq!(value["entries"][0]["name"], "catalog/tool");
    assert_eq!(
        value["entries"][0]["markdown"],
        "Prefer `tool` for ordinary catalog entries. Use `catalog/tool` when writing extension macros that need to splice fields directly into the raw catalog shape before Scaffold normalizes it."
    );
    assert_eq!(
        value["entries"][0]["raw_markdown"],
        value["entries"][0]["markdown"]
    );
    assert!(
        value["entries"][0]["rendered_markdown"]
            .as_str()
            .is_some_and(|markdown| {
                markdown.contains("```scheme\n(catalog/tool name action field ...)\n```")
                    && markdown.contains("**Example**")
            })
    );
}

#[test]
fn docs_format_json_search_suggestions_use_entries_field() {
    let mut args = docs_args();
    args.search = Some("catlgtool".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("search json");

    assert_eq!(value["mode"], "search");
    assert_eq!(value["query"], "catlgtool");
    assert_eq!(value["count"], 0);
    assert_eq!(value["entries"].as_array().map(Vec::len), Some(0));
    assert_eq!(value["suggestions"][0]["name"], "catalog/tool");
    assert!(value["suggestions"][0].get("group").is_some());
}

#[test]
fn docs_format_json_renders_group_json() {
    let mut args = docs_args();
    args.group = Some("Catalog".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("group json");

    assert_eq!(value["mode"], "group");
    assert_eq!(value["query"], "Catalog");
    assert!(value["count"].as_u64().is_some_and(|count| count > 0));
    assert!(
        value["entries"]
            .as_array()
            .is_some_and(|entries| { entries.iter().any(|entry| entry["name"] == "tool") })
    );
}

#[test]
fn docs_format_json_group_suggestions_use_group_name_field() {
    let mut args = docs_args();
    args.group = Some("Catlog".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("missing group json");

    assert_eq!(value["mode"], "group");
    assert_eq!(value["count"], 0);
    assert_eq!(value["suggestions"][0]["name"], "Catalog");
    assert!(value["suggestions"][0].get("group").is_none());
}

#[test]
fn docs_format_json_group_unrelated_short_query_has_no_suggestions() {
    let mut args = docs_args();
    args.group = Some("Nope".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("missing group json");

    assert_eq!(value["mode"], "group");
    assert_eq!(value["count"], 0);
    assert_eq!(value["suggestions"].as_array().map(Vec::len), Some(0));
}

#[test]
fn docs_format_json_renders_source_json() {
    let mut args = docs_args();
    args.source = Some("source/path".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("source json");

    assert_eq!(value["mode"], "source");
    assert_eq!(value["query"], "source/path");
    assert_eq!(value["entry"]["name"], "source/path");
    assert_eq!(value["entry"]["effect"], "context-read-only");
    assert_eq!(
        value["entry"]["requires_capability"],
        json!(["scaffold.workspace"])
    );
    assert_eq!(
        value["entry"]["returns"],
        "A path string, or `#f` when no source path is available."
    );
    assert!(
        value["entry"]["source_location"]
            .as_str()
            .is_some_and(|source| source.contains("src/dsl/std/workspace.scm"))
    );
    assert!(
        value["entry"]["range"]["length"]
            .as_u64()
            .is_some_and(|length| length > 0)
    );
}

#[test]
fn docs_format_json_renders_source_file_entries() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/tool.scm".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("source json");

    assert_eq!(value["mode"], "source");
    assert_eq!(value["query"], "src/dsl/std/catalog/tool.scm");
    assert_eq!(value["source"], "src/dsl/std/catalog/tool.scm");
    assert_eq!(
        value["count"].as_u64(),
        value["entries"]
            .as_array()
            .map(|entries| entries.len() as u64)
    );
    assert!(
        value["entries"]
            .as_array()
            .is_some_and(|entries| entries.iter().any(|entry| entry["name"] == "tool"))
    );
    assert!(value.get("entry").is_none());
}

#[test]
fn docs_format_json_preserves_source_location_query() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/tool.scm:999:1".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("source json");

    assert_eq!(value["mode"], "source");
    assert_eq!(value["query"], "src/dsl/std/catalog/tool.scm:999:1");
    assert_eq!(value["source"], "src/dsl/std/catalog/tool.scm");
    assert!(
        value["entries"]
            .as_array()
            .is_some_and(|entries| entries.iter().any(|entry| entry["name"] == "tool"))
    );
}

#[test]
fn docs_format_json_missing_source_includes_match_count_and_limit() {
    let mut args = docs_args();
    args.source = Some("Catlog".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("missing source json");

    assert_eq!(value["mode"], "source");
    assert_eq!(value["missing_kind"], "symbol");
    assert!(value["entry"].is_null());
    assert_eq!(value["limit"], 10);
    assert_eq!(
        value["count"].as_u64(),
        value["matches"]
            .as_array()
            .map(|matches| matches.len() as u64)
    );
}

#[test]
fn docs_format_json_missing_source_path_reports_source_kind() {
    let mut args = docs_args();
    args.source = Some("src/dsl/std/catalog/missing.scm:1:1".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("missing source json");

    assert_eq!(value["mode"], "source");
    assert_eq!(value["query"], "src/dsl/std/catalog/missing.scm:1:1");
    assert_eq!(value["missing_kind"], "source");
    assert!(value["entry"].is_null());
    assert_eq!(value["count"], 0);
    assert_eq!(value["matches"].as_array().map(Vec::len), Some(0));
}

#[test]
fn docs_format_json_missing_source_suggests_close_symbol_typos() {
    let mut args = docs_args();
    args.source = Some("catlgtool".to_owned());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("missing source json");

    assert_eq!(value["mode"], "source");
    assert!(value["entry"].is_null());
    assert_eq!(value["count"], 0);
    assert_eq!(value["matches"].as_array().map(Vec::len), Some(0));
    assert_eq!(value["suggestions"][0]["name"], "catalog/tool");
    assert_eq!(
        value["suggestions"][0]["summary"],
        "Create a raw catalog tool list for macro-oriented helpers."
    );
}

#[test]
fn docs_all_exports_markdown_by_default() {
    let mut args = docs_args();
    args.all = true;

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("# Scaffold Scheme Reference"));
    assert!(rendered.contains("## Contents"));
    assert!(!rendered.starts_with("{\n"));
}

#[test]
fn docs_all_respects_json_format() {
    let mut args = docs_args();
    args.all = true;
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");
    let value: Value = serde_json::from_str(&rendered).expect("reference json");

    assert_eq!(value["title"], "Scaffold Scheme Reference");
    assert!(
        value["entries"]
            .as_array()
            .is_some_and(|entries| { entries.iter().any(|entry| entry["name"] == "tool") })
    );
    assert!(!rendered.starts_with("# Scaffold Scheme Reference"));
}

#[test]
fn docs_output_json_extension_exports_json() {
    let mut args = docs_args();
    args.output = Some("reference.json".into());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("{\n"));
    assert!(rendered.contains("\"title\": \"Scaffold Scheme Reference\""));
    assert!(!rendered.starts_with("# Scaffold Scheme Reference"));
}

#[test]
fn docs_output_markdown_extension_exports_markdown() {
    let mut args = docs_args();
    args.output = Some("reference.md".into());

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("# Scaffold Scheme Reference"));
}

#[test]
fn docs_explicit_format_overrides_output_extension() {
    let mut args = docs_args();
    args.output = Some("reference.json".into());
    args.format = Some(DocsFormat::Markdown);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("# Scaffold Scheme Reference"));

    args.output = Some("reference.md".into());
    args.format = Some(DocsFormat::Json);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("{\n"));
    assert!(rendered.contains("\"title\": \"Scaffold Scheme Reference\""));
}

#[test]
fn docs_unknown_output_extension_requires_explicit_format() {
    let mut args = docs_args();
    args.output = Some("reference.out".into());

    let err = render_docs(&args).expect_err("unknown extension should fail");

    assert!(err.to_string().contains("cannot infer docs format"));
    assert!(err.to_string().contains("--format"));
}

#[test]
fn docs_output_without_extension_requires_explicit_format() {
    let mut args = docs_args();
    args.output = Some("reference".into());

    let err = render_docs(&args).expect_err("missing extension should fail");

    assert!(err.to_string().contains("cannot infer docs format"));
    assert!(err.to_string().contains(".json"));
}

#[test]
fn docs_export_options_reject_browse_selectors() {
    let mut args = docs_args();
    args.output = Some("reference.md".into());
    args.query = vec!["   ".to_owned()];

    let err = render_docs(&args).expect_err("export with query should fail");

    assert!(err.to_string().contains("cannot be combined"));
}

#[test]
fn docs_browse_selectors_reject_mixed_modes() {
    let mut args = docs_args();
    args.query = vec!["tool".to_owned()];
    args.group = Some("Catalog".to_owned());

    let err = render_docs(&args).expect_err("mixed selectors should fail");

    assert!(err.to_string().contains("cannot be combined"));
}

#[test]
fn docs_limit_is_rejected_when_it_would_be_ignored() {
    let mut args = docs_args();
    args.limit = Some(5);

    let err = render_docs(&args).expect_err("overview limit should fail");
    assert_eq!(
        err.to_string(),
        "--limit only applies to reference search; use a query or --search"
    );

    args = docs_args();
    args.group = Some("Catalog".to_owned());
    args.limit = Some(5);

    let err = render_docs(&args).expect_err("group limit should fail");
    assert_eq!(
        err.to_string(),
        "--limit only applies to reference search; it cannot be combined with --group or --source"
    );

    args = docs_args();
    args.output = Some("reference.md".into());
    args.limit = Some(5);

    let err = render_docs(&args).expect_err("export limit should fail");
    assert_eq!(
        err.to_string(),
        "--limit only applies to reference search; full reference exports ignore it"
    );
}

#[test]
fn docs_search_limit_rejects_out_of_range_values() {
    let mut args = docs_args();
    args.search = Some("tool".to_owned());
    args.limit = Some(0);

    let err = render_docs(&args).expect_err("zero search limit should fail");
    assert_eq!(
        err.to_string(),
        format!("--limit must be between 1 and {MAX_SEARCH_LIMIT}")
    );

    args.limit = Some(MAX_SEARCH_LIMIT + 1);

    let err = render_docs(&args).expect_err("oversized search limit should fail");
    assert_eq!(
        err.to_string(),
        format!("--limit must be between 1 and {MAX_SEARCH_LIMIT}")
    );
}

#[test]
fn docs_search_accepts_max_limit() {
    let mut args = docs_args();
    args.search = Some("tool".to_owned());
    args.limit = Some(MAX_SEARCH_LIMIT);

    let rendered = render_docs(&args).expect("render docs");

    assert!(rendered.starts_with("Search results for `tool`"));
}
