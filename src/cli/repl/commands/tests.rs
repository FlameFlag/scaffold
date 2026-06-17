use super::*;

#[test]
fn repl_command_parser_splits_command_and_rest() {
    assert_eq!(split_repl_command(":doc tool"), (":doc", "tool"));
    assert_eq!(
        split_repl_command("  :search package install  "),
        (":search", "package install")
    );
    assert_eq!(split_repl_command(":groups"), (":groups", ""));
}

#[test]
fn repl_doc_summary_table_stays_compact() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("tool").expect("tool docs");
    let line = doc_entry_table_row(entry);

    assert!(!line.contains("(tool name action field ...)"));
    assert!(line.contains("Create a catalog tool object."));
}

#[test]
fn repl_help_markdown_uses_table_layout() {
    let markdown = repl_help_markdown();

    assert!(markdown.starts_with("## REPL commands"));
    assert!(markdown.contains("| Command"));
    assert!(markdown.contains("`:doc NAME`"));
    assert!(markdown.contains("`:group GROUP`"));
    assert!(markdown.contains("`:source SYMBOL_OR_SOURCE`"));
    assert!(markdown.contains("Show reference docs for one symbol."));
    assert!(
        markdown
            .contains("Search reference docs, examples, source paths or locations, and metadata.")
    );
    assert!(markdown.contains("list docs from a source file"));
}

#[test]
fn repl_doc_groups_markdown_uses_table_layout() {
    let docs = DocIndex::scaffold();
    let markdown = doc_groups_markdown(&docs);

    assert!(markdown.starts_with("## Documentation groups"));
    assert!(markdown.contains("| Group"));
    assert!(markdown.contains("`Language`"));
    assert!(markdown.contains("| Entries"));
}

#[test]
fn repl_doc_group_title_uses_canonical_group_label() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("begin").expect("begin docs");

    assert_eq!(doc_group_title(entry), "Documentation group `Language`");
}

#[test]
fn repl_doc_group_suggestions_use_table_layout() {
    let docs = DocIndex::scaffold();
    let groups = crate::cli::docs::search_doc_groups(&docs, "Catlog", 5);
    let markdown = doc_group_suggestions_markdown("Did you mean", &groups);

    assert!(markdown.starts_with("## Did you mean"));
    assert!(markdown.contains("| Group"));
    assert!(markdown.contains("`Catalog`"));
    assert!(markdown.contains("| Entries"));
    assert!(markdown.contains("## Try"));
    assert!(markdown.contains("- `:group Catalog`"));
}

#[test]
fn repl_doc_entries_markdown_uses_table_layout() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("tool").expect("tool docs");
    let markdown = doc_entries_markdown("Search results", &[entry]);

    assert!(markdown.starts_with("## Search results"));
    assert!(markdown.contains("| Symbol"));
    assert!(markdown.contains("`tool`"));
    assert!(!markdown.contains("`(tool name action field ...)`"));
    assert!(markdown.contains("Create a catalog tool object."));
}

#[test]
fn repl_doc_entry_markdown_uses_reference_body_once() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("source/path").expect("source/path docs");
    let markdown = doc_entry_markdown(entry);

    assert!(markdown.starts_with("## `source/path`"));
    assert!(markdown.contains("```scheme\nsource/path\n```"));
    assert_eq!(
        markdown
            .matches("Path of the Scheme source currently being evaluated.")
            .count(),
        1
    );
    assert!(markdown.contains("| effect"));
    assert!(markdown.contains("`context-read-only`"));
    assert!(markdown.contains("| requires capability"));
    assert!(markdown.contains("`scaffold.workspace`"));
    assert!(markdown.contains("| source"));
    assert!(markdown.contains("`src/dsl/std/workspace.scm:"));
}

#[test]
fn repl_doc_entry_markdown_renders_no_documentation_fallback() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("subject").expect("subject docs");
    let markdown = doc_entry_markdown(entry);

    assert!(markdown.starts_with("## `subject`"));
    assert!(markdown.contains("No documentation provided."));
    assert!(markdown.contains("### Details"));
    assert!(markdown.contains("`src/dsl/std/core/doc.scm:"));
}

#[test]
fn repl_doc_lookup_is_case_insensitive() {
    let docs = DocIndex::scaffold();
    let entry = crate::cli::docs::get_doc_entry(&docs, "TOOL/PATH").expect("tool/path docs");
    let markdown = doc_entry_markdown(entry);

    assert!(markdown.starts_with("## `tool/path`"));
}

#[test]
fn repl_doc_source_markdown_uses_table_layout() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("tool").expect("tool docs");
    let markdown = scaffold_docs::source_markdown_for_entry(entry).expect("source markdown");

    assert!(markdown.starts_with("## `tool` source"));
    assert!(markdown.contains("| Field     | Value"));
    assert!(markdown.contains("| Source    | `src/dsl/std/catalog/tool.scm:"));
    assert!(markdown.contains("| Signature | `(tool name action field ...)`"));
    assert!(!markdown.contains("Source:"));
}

#[test]
fn repl_doc_source_accepts_source_file_locations() {
    let docs = DocIndex::scaffold();
    let (source, entries) =
        crate::cli::docs::get_doc_source_entries(&docs, "src/dsl/std/catalog/tool.scm:16:1")
            .expect("source entries");
    let markdown = doc_entries_markdown(
        &format!("Docs from source {}", markdown_code_span(source)),
        &entries,
    );

    assert!(markdown.starts_with("## Docs from source `src/dsl/std/catalog/tool.scm`"));
    assert!(markdown.contains("| Symbol"));
    assert!(markdown.contains("`tool`"));
    assert!(markdown.contains("`tool/append-bins`"));
}

#[test]
fn repl_doc_entry_markdown_deduplicates_keyword_summary() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("begin").expect("begin docs");
    let markdown = doc_entry_markdown(entry);

    assert_eq!(
        markdown
            .matches("Evaluate expressions in order and return the final value.")
            .count(),
        1
    );
    assert!(markdown.contains("| source"));
    assert!(markdown.contains("`scheme keyword`"));
}

#[test]
fn repl_doc_search_uses_fuzzy_ranking() {
    let docs = DocIndex::scaffold();
    let matches = search_doc_entries(&docs, "ctlg tool", 20);

    assert!(matches.iter().any(|entry| entry.name == "catalog/tool"));
}

#[test]
fn repl_doc_search_matches_documented_examples() {
    let docs = DocIndex::scaffold();
    let matches = search_doc_entries(&docs, "ripgrep", 20);

    assert!(matches.iter().any(|entry| entry.name == "catalog/tool"));
}

#[test]
fn repl_missing_doc_possible_matches_use_specific_title() {
    let docs = DocIndex::scaffold();
    let markdown =
        doc_possible_matches_markdown(&docs, "Catlog", ":doc").expect("possible matches");

    assert!(markdown.starts_with("## Possible matches for `Catlog`"));
    assert!(markdown.contains("`catalog/tool`"));
    assert!(markdown.contains("## Try"));
    assert!(markdown.contains("- `:doc catalog`"));
    assert!(!markdown.contains("Search results for"));
}

#[test]
fn repl_missing_doc_possible_matches_use_typo_suggestions() {
    let docs = DocIndex::scaffold();
    let markdown =
        doc_possible_matches_markdown(&docs, "catlgtool", ":doc").expect("possible matches");

    assert!(markdown.starts_with("## Possible matches for `catlgtool`"));
    assert!(markdown.contains("`catalog/tool`"));
    assert!(markdown.contains("Create a raw catalog tool list for macro-oriented helpers."));
    assert!(markdown.contains("- `:doc catalog/tool`"));
}

#[test]
fn repl_missing_source_possible_matches_suggest_source_command() {
    let docs = DocIndex::scaffold();
    let markdown =
        doc_possible_matches_markdown(&docs, "catlgtool", ":source").expect("possible matches");

    assert!(markdown.starts_with("## Possible matches for `catlgtool`"));
    assert!(markdown.contains("`catalog/tool`"));
    assert!(markdown.contains("- `:source catalog/tool`"));
    assert!(!markdown.contains("- `:doc catalog/tool`"));
}

#[test]
fn repl_missing_doc_possible_matches_omit_unrelated_noise() {
    let docs = DocIndex::scaffold();

    assert!(doc_possible_matches_markdown(&docs, "nope", ":doc").is_none());
}
