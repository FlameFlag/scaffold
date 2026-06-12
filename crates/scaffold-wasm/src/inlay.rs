use crate::editor_inlay::{InlayEntry, InlayIndex, InlayParam};
use crate::reference::{ReferenceDocument, ReferenceEntry};

impl InlayEntry for ReferenceEntry {
    fn signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }

    fn params(&self) -> Vec<InlayParam<'_>> {
        self.params
            .iter()
            .map(|param| InlayParam {
                name: &param.name,
                summary: &param.summary,
            })
            .collect()
    }
}

struct ReferenceInlayIndex<'a>(&'a ReferenceDocument);

impl InlayIndex for ReferenceInlayIndex<'_> {
    type Entry<'a>
        = &'a ReferenceEntry
    where
        Self: 'a;

    fn entry<'a>(&'a self, symbol: &str) -> Option<Self::Entry<'a>> {
        self.0.entries.iter().find(|entry| entry.name == symbol)
    }
}

#[derive(Debug, serde::Serialize)]
pub(crate) struct InlayHint {
    line: u32,
    start: u32,
    label: String,
    tooltip: String,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TextRange {
    pub(crate) start_line: u32,
    pub(crate) start_character: u32,
    pub(crate) end_line: u32,
    pub(crate) end_character: u32,
}

pub(crate) fn inlay_hints(
    reference: &ReferenceDocument,
    text: &str,
    range: TextRange,
) -> Vec<InlayHint> {
    crate::editor_inlay::inlay_hints(&ReferenceInlayIndex(reference), text, range.into())
        .into_iter()
        .map(|hint| InlayHint {
            line: hint.line,
            start: hint.start,
            label: hint.label,
            tooltip: hint.tooltip.unwrap_or_default(),
        })
        .collect()
}

impl From<TextRange> for crate::editor_inlay::TextRange {
    fn from(range: TextRange) -> Self {
        Self {
            start_line: range.start_line,
            start_character: range.start_character,
            end_line: range.end_line,
            end_character: range.end_character,
        }
    }
}
