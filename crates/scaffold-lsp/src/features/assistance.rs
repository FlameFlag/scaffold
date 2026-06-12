use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemTag, Documentation, Hover, HoverContents, InsertTextFormat,
    MarkupContent, MarkupKind, ParameterInformation, ParameterLabel, SignatureHelp,
    SignatureInformation,
};

use crate::docs::DocIndex;
use scaffold_editor::reference as editor_reference;

#[must_use]
pub fn completion_items(index: &DocIndex) -> Vec<CompletionItem> {
    editor_reference::completion_items(index.visible_entries())
        .into_iter()
        .map(|item| {
            let filter_text = item.label.clone();
            CompletionItem {
                label: item.label,
                kind: Some(super::completion_kind(item.kind)),
                detail: item.detail,
                documentation: (!item.documentation.trim().is_empty()).then_some({
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: item.documentation,
                    })
                }),
                sort_text: Some(item.sort_text),
                filter_text: Some(filter_text),
                tags: item
                    .deprecated
                    .then_some(vec![CompletionItemTag::DEPRECATED]),
                insert_text: item.insert_text_is_snippet.then_some(item.insert_text),
                insert_text_format: item
                    .insert_text_is_snippet
                    .then_some(InsertTextFormat::SNIPPET),
                ..Default::default()
            }
        })
        .collect()
}

pub fn hover_for_symbol(index: &DocIndex, symbol: &str) -> Option<Hover> {
    let markdown = editor_reference::hover_markdown(index.entries(), symbol)?;
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: markdown,
        }),
        range: None,
    })
}

pub fn signature_help_for_symbol(
    index: &DocIndex,
    symbol: &str,
    active_argument: u32,
) -> Option<SignatureHelp> {
    let help = editor_reference::signature_help(index.entries(), symbol)?;
    let parameters = help
        .parameters
        .into_iter()
        .map(|param| ParameterInformation {
            label: ParameterLabel::Simple(param.label),
            documentation: param.documentation.map(Documentation::String),
        })
        .collect::<Vec<_>>();
    let active_parameter = if parameters.is_empty() {
        0
    } else {
        active_argument.min(parameters.len().saturating_sub(1) as u32)
    };
    Some(SignatureHelp {
        signatures: vec![SignatureInformation {
            label: help.label,
            documentation: Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: help.documentation,
            })),
            parameters: Some(parameters),
            active_parameter: Some(active_parameter),
        }],
        active_signature: Some(0),
        active_parameter: Some(active_parameter),
    })
}
