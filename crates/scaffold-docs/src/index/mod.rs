mod keywords;
mod model;
mod render;
mod search;
mod source;

pub use model::{DocEntry, DocIndex, DocKind, DocParam, SourceDocs, SourcePosition, SourceRange};
pub use render::{
    EntryDetail, EntryDocumentation, detailed_markdown_for_entry, entry_count_label,
    entry_documentation, entry_summary_markdown_table, group_count_label, group_markdown_table,
    markdown_for_entry, rendered_markdown_for_entry, snippet_for_signature,
    source_markdown_for_entry, titled_markdown_for_entry,
};
pub use search::{
    normalize_reference_query_token, search_doc_entries, search_reference_entries,
    source_path_from_location_query, suggest_doc_entries, suggest_reference_entries,
};
pub use source::{source_docs, source_docs_with_definitions};

pub use model::WorkspaceDocIndex;

fn join_text(items: impl IntoIterator<Item = impl AsRef<str>>, separator: &str) -> String {
    let mut output = String::new();
    let mut first = true;
    for item in items {
        if first {
            first = false;
        } else {
            output.push_str(separator);
        }
        output.push_str(item.as_ref());
    }
    output
}

#[cfg(test)]
mod tests;
