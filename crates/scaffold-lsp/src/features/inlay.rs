use tower_lsp::lsp_types::{
    InlayHint, InlayHintKind, InlayHintLabel, InlayHintTooltip, Position, Range,
};

use crate::docs::{DocEntry, DocIndex};
use scaffold_editor::inlay as editor_inlay;
use scaffold_editor::inlay::{InlayEntry, InlayIndex, InlayParam};

struct DocInlayEntry<'a>(&'a DocEntry);

impl InlayEntry for DocInlayEntry<'_> {
    fn signature(&self) -> Option<&str> {
        self.0.signature.as_deref()
    }

    fn params(&self) -> Vec<InlayParam<'_>> {
        self.0
            .params
            .iter()
            .map(|param| InlayParam {
                name: &param.name,
                summary: &param.summary,
            })
            .collect()
    }
}

struct DocInlayIndex<'a>(&'a DocIndex);

impl InlayIndex for DocInlayIndex<'_> {
    type Entry<'a>
        = DocInlayEntry<'a>
    where
        Self: 'a;

    fn entry<'a>(&'a self, symbol: &str) -> Option<Self::Entry<'a>> {
        self.0.get(symbol).map(DocInlayEntry)
    }
}

pub fn inlay_hints(index: &DocIndex, text: &str, range: Range) -> Vec<InlayHint> {
    editor_inlay::inlay_hints(&DocInlayIndex(index), text, text_range(range))
        .into_iter()
        .map(|hint| InlayHint {
            position: Position::new(hint.line, hint.start),
            label: InlayHintLabel::String(hint.label),
            kind: Some(InlayHintKind::PARAMETER),
            text_edits: None,
            tooltip: hint.tooltip.map(InlayHintTooltip::String),
            padding_left: None,
            padding_right: Some(true),
            data: None,
        })
        .collect()
}

const fn text_range(range: Range) -> editor_inlay::TextRange {
    editor_inlay::TextRange {
        start_line: range.start.line,
        start_character: range.start.character,
        end_line: range.end.line,
        end_character: range.end.character,
    }
}
