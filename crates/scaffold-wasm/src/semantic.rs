use crate::editor_semantic::{
    ReferenceInfo, ReferenceKind, SemanticReferenceIndex, SemanticToken, semantic_tokens,
};
use crate::reference::{ReferenceDocument, ReferenceKind as WasmReferenceKind};
use crate::syntax::definition_names;

pub(crate) fn document_semantic_tokens(
    reference: &ReferenceDocument,
    text: &str,
) -> Vec<SemanticToken> {
    semantic_tokens(reference, text, definition_names(text))
}

impl SemanticReferenceIndex for ReferenceDocument {
    fn reference_info(&self, symbol: &str) -> Option<ReferenceInfo> {
        let entry = self.entries.iter().find(|entry| entry.name == symbol)?;
        Some(ReferenceInfo {
            kind: match entry.kind {
                WasmReferenceKind::Function => ReferenceKind::Function,
                WasmReferenceKind::Keyword => ReferenceKind::Keyword,
            },
            source: entry.source.clone(),
            deprecated: entry.deprecated.is_some(),
        })
    }
}
