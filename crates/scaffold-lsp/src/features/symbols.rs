#[cfg(test)]
use tower_lsp::lsp_types::{Position, Range};
use tower_lsp::lsp_types::{SymbolInformation, SymbolTag};

use crate::docs::DocIndex;
use scaffold_editor::reference as editor_reference;
#[cfg(test)]
use scaffold_editor::symbols as editor_symbols;

pub fn workspace_symbols(index: &DocIndex, query: &str) -> Vec<SymbolInformation> {
    editor_reference::workspace_symbols(index.visible_entries(), query)
        .into_iter()
        .filter_map(|symbol| {
            #[allow(deprecated)]
            let information = SymbolInformation {
                name: symbol.name,
                kind: super::reference_symbol_kind(symbol.kind),
                tags: symbol.deprecated.then_some(vec![SymbolTag::DEPRECATED]),
                deprecated: None,
                location: super::location_from_parts(
                    &symbol.uri,
                    symbol.line,
                    symbol.start,
                    symbol.length,
                )?,
                container_name: symbol.group,
            };
            Some(information)
        })
        .collect()
}

#[cfg(test)]
pub fn symbol_ranges(text: &str, symbol: &str) -> Vec<Range> {
    editor_symbols::symbol_ranges(text, symbol)
        .into_iter()
        .map(|range| {
            Range::new(
                Position::new(range.line, range.start),
                Position::new(range.line, range.start + range.length),
            )
        })
        .collect()
}
