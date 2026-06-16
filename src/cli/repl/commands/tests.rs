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
fn repl_doc_search_matches_known_docs() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("tool").expect("tool docs");

    assert!(doc_entry_matches(entry, "catalog"));
    assert!(doc_entry_matches(entry, "tool"));
    assert!(!doc_entry_matches(entry, "definitely-not-in-docs"));
}

#[test]
fn repl_doc_summary_includes_signature_and_summary() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("tool").expect("tool docs");
    let line = doc_entry_summary_line(entry);

    assert!(line.contains("(tool name action field ...)"));
    assert!(line.contains("Create a catalog tool object."));
}

#[test]
fn repl_help_markdown_uses_list_layout() {
    let markdown = repl_help_markdown();

    assert!(markdown.starts_with("## REPL commands"));
    assert!(markdown.contains("- `:doc NAME` - Show reference docs for one symbol."));
    assert!(!markdown.contains("| Command |"));
}

#[test]
fn repl_doc_groups_markdown_uses_list_layout() {
    let docs = DocIndex::scaffold();
    let markdown = doc_groups_markdown(&docs);

    assert!(markdown.starts_with("## Documentation groups"));
    assert!(markdown.contains("- `Language` - "));
    assert!(!markdown.contains("| Group |"));
}

#[test]
fn repl_doc_entries_markdown_uses_compact_list_layout() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("tool").expect("tool docs");
    let markdown = doc_entries_markdown("Search results", &[entry]);

    assert!(markdown.starts_with("## Search results"));
    assert!(markdown.contains("- `tool`  \n  `(tool name action field ...)`"));
    assert!(markdown.contains("Catalog - Create a catalog tool object."));
    assert!(!markdown.contains("| Symbol |"));
}

#[test]
fn repl_doc_entry_markdown_uses_reference_body_once() {
    let docs = DocIndex::scaffold();
    let entry = docs.get("catalog/tool").expect("catalog/tool docs");
    let markdown = doc_entry_markdown(entry);

    assert!(markdown.starts_with("## `catalog/tool`"));
    assert!(markdown.contains("```scheme\n(catalog/tool name action field ...)\n```"));
    assert_eq!(
        markdown
            .matches("Create a raw catalog tool list for macro-oriented helpers.")
            .count(),
        1
    );
    assert!(markdown.contains("Prefer `tool` for ordinary catalog entries."));
    assert!(markdown.contains("- Source: `src/dsl/std/catalog/root.scm:"));
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
    assert!(markdown.contains("- Source: `scheme keyword`"));
}

#[test]
fn repl_doc_search_uses_fuzzy_ranking() {
    let docs = DocIndex::scaffold();
    let matches = doc_search_results(&docs, "ctlg tool", 20);

    assert!(matches.iter().any(|entry| entry.name == "catalog/tool"));
}
