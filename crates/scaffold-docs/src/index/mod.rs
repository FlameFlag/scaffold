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
    search_doc_entries, search_reference_entries, suggest_doc_entries, suggest_reference_entries,
};
pub use source::{source_docs, source_docs_with_definitions};

pub use model::WorkspaceDocIndex;

#[cfg(test)]
mod tests;
