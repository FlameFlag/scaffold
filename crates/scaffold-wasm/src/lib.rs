use wasm_bindgen::prelude::*;

mod diagnostics;

use scaffold_editor::actions as editor_actions;
use scaffold_editor::diagnostics as editor_diagnostics;
use scaffold_editor::inlay as editor_inlay;
use scaffold_editor::reference as editor_reference;
use scaffold_editor::semantic as editor_semantic;
use scaffold_editor::symbols as editor_symbols;
use scaffold_editor::syntax;
use scaffold_fmt as fmt;

mod inlay;
mod navigation;
mod reference;
mod semantic;

use inlay::TextRange;
use reference::{
    reference_document, reference_index_for_document, reference_index_for_source,
    reference_index_for_workspace,
};

fn json_string(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).expect("WASM export payload serializes")
}

#[wasm_bindgen(js_name = formatScaffoldScheme)]
pub fn format_scaffold_scheme(text: &str) -> Result<String, JsValue> {
    fmt::format_text(text).map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen(js_name = diagnoseScaffoldScheme)]
#[must_use]
pub fn diagnose_scaffold_scheme(text: &str) -> String {
    diagnostics::diagnose_text(text)
}

#[wasm_bindgen(js_name = missingDocStubScaffoldScheme)]
#[must_use]
pub fn missing_doc_stub_scaffold_scheme(name: &str, indent: &str) -> String {
    editor_actions::missing_doc_stub(name, indent)
}

#[wasm_bindgen(js_name = semanticTokensScaffoldScheme)]
#[must_use]
pub fn semantic_tokens_scaffold_scheme(text: &str) -> String {
    json_string(&semantic::document_semantic_tokens(
        &reference_document(),
        text,
    ))
}

#[wasm_bindgen(js_name = semanticTokensScaffoldSchemeForDocument)]
#[must_use]
pub fn semantic_tokens_scaffold_scheme_for_document(text: &str, workspace_json: &str) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&semantic::document_semantic_tokens(
        &reference_index_for_document(text, &workspace),
        text,
    ))
}

#[wasm_bindgen(js_name = completionItemsScaffoldScheme)]
#[must_use]
pub fn completion_items_scaffold_scheme() -> String {
    json_string(&reference::completion_items(&reference_document()))
}

#[wasm_bindgen(js_name = completionItemsScaffoldSchemeForDocument)]
#[must_use]
pub fn completion_items_scaffold_scheme_for_document(text: &str, workspace_json: &str) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference::completion_items(&reference_index_for_document(
        text, &workspace,
    )))
}

#[wasm_bindgen(js_name = hoverScaffoldScheme)]
#[must_use]
pub fn hover_scaffold_scheme(symbol: &str) -> String {
    reference::hover_markdown(&reference_document(), symbol)
}

#[wasm_bindgen(js_name = hoverScaffoldSchemeForDocument)]
#[must_use]
pub fn hover_scaffold_scheme_for_document(
    text: &str,
    symbol: &str,
    workspace_json: &str,
) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    reference::hover_markdown(&reference_index_for_document(text, &workspace), symbol)
}

#[wasm_bindgen(js_name = signatureHelpScaffoldScheme)]
#[must_use]
pub fn signature_help_scaffold_scheme(symbol: &str) -> String {
    json_string(&reference::signature_help(&reference_document(), symbol))
}

#[wasm_bindgen(js_name = signatureHelpScaffoldSchemeForDocument)]
#[must_use]
pub fn signature_help_scaffold_scheme_for_document(
    text: &str,
    symbol: &str,
    workspace_json: &str,
) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference::signature_help(
        &reference_index_for_document(text, &workspace),
        symbol,
    ))
}

#[wasm_bindgen(js_name = referenceEntriesScaffoldScheme)]
#[must_use]
pub fn reference_entries_scaffold_scheme() -> String {
    json_string(&reference_document().entries)
}

#[wasm_bindgen(js_name = searchReferenceEntriesScaffoldScheme)]
#[must_use]
pub fn search_reference_entries_scaffold_scheme(query: &str, limit: usize) -> String {
    json_string(&reference::search_entries(
        &reference_document(),
        query,
        limit,
    ))
}

#[wasm_bindgen(js_name = suggestReferenceEntriesScaffoldScheme)]
#[must_use]
pub fn suggest_reference_entries_scaffold_scheme(query: &str, limit: usize) -> String {
    json_string(&reference::suggest_entries(
        &reference_document(),
        query,
        limit,
    ))
}

#[wasm_bindgen(js_name = referenceCapabilitiesScaffoldScheme)]
#[must_use]
pub fn reference_capabilities_scaffold_scheme() -> String {
    json_string(&reference_document().capabilities)
}

#[wasm_bindgen(js_name = referenceCatalogSchemaScaffoldScheme)]
#[must_use]
pub fn reference_catalog_schema_scaffold_scheme() -> String {
    json_string(&reference_document().catalog_schema)
}

#[wasm_bindgen(js_name = referenceEntriesScaffoldSchemeForWorkspace)]
#[must_use]
pub fn reference_entries_scaffold_scheme_for_workspace(workspace_json: &str) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference_index_for_workspace(&workspace).entries)
}

#[wasm_bindgen(js_name = searchReferenceEntriesScaffoldSchemeForWorkspace)]
#[must_use]
pub fn search_reference_entries_scaffold_scheme_for_workspace(
    query: &str,
    workspace_json: &str,
    limit: usize,
) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference::search_entries(
        &reference_index_for_workspace(&workspace),
        query,
        limit,
    ))
}

#[wasm_bindgen(js_name = suggestReferenceEntriesScaffoldSchemeForWorkspace)]
#[must_use]
pub fn suggest_reference_entries_scaffold_scheme_for_workspace(
    query: &str,
    workspace_json: &str,
    limit: usize,
) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference::suggest_entries(
        &reference_index_for_workspace(&workspace),
        query,
        limit,
    ))
}

#[wasm_bindgen(js_name = referenceEntriesScaffoldSchemeForDocument)]
#[must_use]
pub fn reference_entries_scaffold_scheme_for_document(
    text: &str,
    uri: &str,
    workspace_json: &str,
) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference_index_for_source(uri, text, &workspace).entries)
}

#[wasm_bindgen(js_name = symbolAtScaffoldScheme)]
#[must_use]
pub fn symbol_at_scaffold_scheme(text: &str, line: u32, character: u32) -> String {
    navigation::symbol_at_position(text, line, character).unwrap_or_default()
}

#[wasm_bindgen(js_name = formContextScaffoldScheme)]
#[must_use]
pub fn form_context_scaffold_scheme(text: &str, line: u32, character: u32) -> String {
    json_string(&editor_symbols::form_context_at_position(
        text, line, character,
    ))
}

#[wasm_bindgen(js_name = referenceLocationsScaffoldScheme)]
#[must_use]
pub fn reference_locations_scaffold_scheme(symbol: &str, workspace_json: &str) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference::reference_locations(&workspace, symbol))
}

#[wasm_bindgen(js_name = documentReferenceSymbolsScaffoldScheme)]
#[must_use]
pub fn document_reference_symbols_scaffold_scheme(text: &str) -> String {
    json_string(&reference::document_symbols(text))
}

#[wasm_bindgen(js_name = inlayHintsScaffoldScheme)]
#[must_use]
pub fn inlay_hints_scaffold_scheme(
    text: &str,
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
) -> String {
    json_string(&inlay::inlay_hints(
        &reference_document(),
        text,
        TextRange {
            start_line,
            start_character,
            end_line,
            end_character,
        },
    ))
}

#[wasm_bindgen(js_name = inlayHintsScaffoldSchemeForDocument)]
#[must_use]
pub fn inlay_hints_scaffold_scheme_for_document(
    text: &str,
    workspace_json: &str,
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&inlay::inlay_hints(
        &reference_index_for_document(text, &workspace),
        text,
        TextRange {
            start_line,
            start_character,
            end_line,
            end_character,
        },
    ))
}

#[wasm_bindgen(js_name = definitionScaffoldScheme)]
#[must_use]
pub fn definition_scaffold_scheme(
    text: &str,
    uri: &str,
    line: u32,
    character: u32,
    workspace_json: &str,
) -> String {
    let Some(symbol) = navigation::symbol_at_position(text, line, character) else {
        return "null".to_owned();
    };
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference::definition_location(
        &reference_index_for_source(uri, text, &workspace),
        uri,
        &symbol,
    ))
}

#[wasm_bindgen(js_name = workspaceSymbolsScaffoldScheme)]
#[must_use]
pub fn workspace_symbols_scaffold_scheme(query: &str, workspace_json: &str) -> String {
    let workspace = reference::workspace_documents(workspace_json);
    json_string(&reference::workspace_symbols(
        &reference_index_for_workspace(&workspace),
        query,
    ))
}

#[cfg(test)]
mod tests;
