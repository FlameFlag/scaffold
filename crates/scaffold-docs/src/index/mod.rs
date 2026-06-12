mod keywords;
mod model;
mod render;
mod source;

pub use model::{DocEntry, DocIndex, DocKind, DocParam, SourceDocs, SourcePosition, SourceRange};
pub use render::{markdown_for_entry, snippet_for_signature};
pub use source::{source_docs, source_docs_with_definitions};

pub use model::WorkspaceDocIndex;

#[cfg(test)]
mod tests;
