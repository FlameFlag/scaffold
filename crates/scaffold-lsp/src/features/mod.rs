mod assistance;
mod inlay;
mod semantic;
mod symbols;

use std::path::Path;

use scaffold_docs::{DocEntry, DocKind, SourceRange};
use scaffold_editor::reference::ReferenceKind;
use tower_lsp::lsp_types::{CompletionItemKind, Location, Position, Range, SymbolKind, Url};

pub use assistance::{completion_items, hover_for_symbol, signature_help_for_symbol};
pub use inlay::inlay_hints;
pub use semantic::{semantic_tokens, semantic_tokens_legend};
#[cfg(test)]
pub use symbols::symbol_ranges;
pub use symbols::workspace_symbols;

pub(super) const fn completion_kind(kind: ReferenceKind) -> CompletionItemKind {
    match kind {
        ReferenceKind::Keyword => CompletionItemKind::KEYWORD,
        ReferenceKind::Function => CompletionItemKind::FUNCTION,
    }
}

pub(super) const fn symbol_kind(entry: &DocEntry) -> SymbolKind {
    match entry.kind {
        DocKind::Keyword => SymbolKind::KEY,
        DocKind::Function => SymbolKind::FUNCTION,
    }
}

pub(super) const fn reference_symbol_kind(kind: ReferenceKind) -> SymbolKind {
    match kind {
        ReferenceKind::Keyword => SymbolKind::KEY,
        ReferenceKind::Function => SymbolKind::FUNCTION,
    }
}

pub(super) fn location(entry: &DocEntry) -> Option<Location> {
    let source = entry.source.as_ref()?;
    let range = lsp_range(entry.range?);
    let uri = source_uri(source)?;
    Some(Location { uri, range })
}

pub(super) fn location_from_parts(
    source: &str,
    line: u32,
    start: u32,
    length: u32,
) -> Option<Location> {
    Some(Location {
        uri: source_uri(source)?,
        range: Range::new(
            Position::new(line, start),
            Position::new(line, start + length),
        ),
    })
}

pub(super) fn lsp_range(range: SourceRange) -> Range {
    Range::new(
        Position::new(range.start.line, range.start.character),
        Position::new(range.end.line, range.end.character),
    )
}

fn source_uri(source: &str) -> Option<Url> {
    if let Ok(uri) = Url::parse(source) {
        return Some(uri);
    }

    let path = Path::new(source);
    if path.is_absolute() {
        return Url::from_file_path(path).ok();
    }

    Url::from_file_path(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).ok()
}

#[cfg(test)]
mod tests;
