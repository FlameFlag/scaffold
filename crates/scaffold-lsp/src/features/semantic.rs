use std::collections::HashSet;

use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensLegend,
};

use crate::docs::{DocIndex, DocKind};
use scaffold_editor::semantic::{
    self as editor_semantic, ReferenceInfo, ReferenceKind, SemanticReferenceIndex,
};
use scaffold_editor::syntax::definition_names;

pub(super) const TOKEN_FUNCTION: u32 = 0;
pub(super) const TOKEN_KEYWORD: u32 = 1;
pub(super) const TOKEN_STRING: u32 = 2;
pub(super) const TOKEN_COMMENT: u32 = 3;
pub(super) const TOKEN_PARAMETER: u32 = 4;

pub(super) const MOD_DEFAULT_LIBRARY: u32 = 1 << 0;
pub(super) const MOD_DOCUMENTATION: u32 = 1 << 1;
pub(super) const MOD_DEPRECATED: u32 = 1 << 2;

pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::FUNCTION,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::STRING,
            SemanticTokenType::COMMENT,
            SemanticTokenType::PARAMETER,
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DEFAULT_LIBRARY,
            SemanticTokenModifier::DOCUMENTATION,
            SemanticTokenModifier::DEPRECATED,
            SemanticTokenModifier::DEFINITION,
        ],
    }
}

pub fn semantic_tokens(index: &DocIndex, text: &str) -> SemanticTokens {
    let user_symbols = user_defined_symbols(text);
    let mut previous_line = 0;
    let mut previous_start = 0;
    let reference_index = DocSemanticIndex(index);
    let data = editor_semantic::semantic_tokens(&reference_index, text, user_symbols)
        .into_iter()
        .map(|token| {
            let delta_line = token.line - previous_line;
            let delta_start = if delta_line == 0 {
                token.start - previous_start
            } else {
                token.start
            };
            previous_line = token.line;
            previous_start = token.start;
            SemanticToken {
                delta_line,
                delta_start,
                length: token.length,
                token_type: token_type_index(token.token_type),
                token_modifiers_bitset: token_modifier_bitset(&token.modifiers),
            }
        })
        .collect();

    SemanticTokens {
        result_id: None,
        data,
    }
}

fn token_type_index(token_type: &str) -> u32 {
    if token_type == editor_semantic::TOKEN_KEYWORD {
        TOKEN_KEYWORD
    } else if token_type == editor_semantic::TOKEN_STRING {
        TOKEN_STRING
    } else if token_type == editor_semantic::TOKEN_COMMENT {
        TOKEN_COMMENT
    } else if token_type == editor_semantic::TOKEN_PARAMETER {
        TOKEN_PARAMETER
    } else {
        TOKEN_FUNCTION
    }
}

fn token_modifier_bitset(modifiers: &[&str]) -> u32 {
    modifiers.iter().fold(0, |bitset, modifier| {
        if *modifier == editor_semantic::MOD_DEFAULT_LIBRARY {
            bitset | MOD_DEFAULT_LIBRARY
        } else if *modifier == editor_semantic::MOD_DOCUMENTATION {
            bitset | MOD_DOCUMENTATION
        } else if *modifier == editor_semantic::MOD_DEPRECATED {
            bitset | MOD_DEPRECATED
        } else {
            bitset
        }
    })
}

struct DocSemanticIndex<'a>(&'a DocIndex);

impl SemanticReferenceIndex for DocSemanticIndex<'_> {
    fn reference_info(&self, symbol: &str) -> Option<ReferenceInfo> {
        let entry = self.0.get(symbol)?;
        Some(ReferenceInfo {
            kind: match entry.kind {
                DocKind::Keyword => ReferenceKind::Keyword,
                DocKind::Function => ReferenceKind::Function,
            },
            source: entry.source.clone(),
            deprecated: entry.deprecated.is_some(),
        })
    }
}

fn user_defined_symbols(text: &str) -> HashSet<String> {
    definition_names(text).into_iter().collect()
}
