use crate::editor_reference::{
    ReferenceEntry as EditorReferenceEntry, ReferenceItem as EditorReferenceItem,
    ReferenceParam as EditorReferenceParam,
};

const REFERENCE_JSON: &str = include_str!("reference.min.json");

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct ReferenceDocument {
    #[serde(default)]
    pub(crate) capabilities: Vec<ReferenceCapability>,
    #[serde(default)]
    pub(crate) catalog_schema: serde_json::Value,
    pub(crate) entries: Vec<ReferenceEntry>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct ReferenceCapability {
    #[serde(default)]
    pub(crate) library_name: Vec<String>,
    pub(crate) library: String,
    #[serde(default)]
    pub(crate) bridge_library_name: Vec<String>,
    pub(crate) bridge_library: String,
    pub(crate) effect: String,
    pub(crate) modes: std::collections::BTreeMap<String, String>,
    pub(crate) docs_source: String,
    pub(crate) notes: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct ReferenceEntry {
    pub(crate) name: String,
    pub(crate) kind: ReferenceKind,
    pub(crate) signature: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) markdown: Option<String>,
    pub(crate) example: Option<String>,
    pub(crate) params: Vec<ReferenceParam>,
    pub(crate) returns: Option<String>,
    pub(crate) group: String,
    pub(crate) see: Vec<String>,
    pub(crate) stability: Option<String>,
    pub(crate) since: Option<String>,
    pub(crate) deprecated: Option<String>,
    pub(crate) source: Option<String>,
    #[serde(default)]
    pub(crate) range: Option<crate::editor_symbols::SymbolRange>,
    #[serde(default)]
    pub(crate) hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReferenceKind {
    Function,
    Keyword,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(crate) struct ReferenceParam {
    pub(crate) name: String,
    pub(crate) summary: String,
}

impl From<scaffold_docs::DocEntry> for ReferenceEntry {
    fn from(entry: scaffold_docs::DocEntry) -> Self {
        Self {
            name: entry.name,
            kind: ReferenceKind::from(entry.kind),
            signature: entry.signature,
            summary: entry.summary,
            markdown: entry.markdown,
            example: entry.example,
            params: entry
                .params
                .into_iter()
                .map(|param| ReferenceParam {
                    name: param.name,
                    summary: param.summary,
                })
                .collect(),
            returns: entry.returns,
            group: entry.group.unwrap_or_default(),
            see: entry.see,
            stability: entry.stability,
            since: entry.since,
            deprecated: entry.deprecated,
            source: entry.source,
            range: entry.range.map(symbol_range),
            hidden: entry.hidden,
        }
    }
}

impl From<scaffold_docs::DocKind> for ReferenceKind {
    fn from(kind: scaffold_docs::DocKind) -> Self {
        match kind {
            scaffold_docs::DocKind::Function => Self::Function,
            scaffold_docs::DocKind::Keyword => Self::Keyword,
        }
    }
}

impl EditorReferenceEntry for ReferenceEntry {
    fn signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }

    fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    fn group(&self) -> Option<&str> {
        (!self.group.is_empty()).then_some(self.group.as_str())
    }

    fn since(&self) -> Option<&str> {
        self.since.as_deref()
    }

    fn stability(&self) -> Option<&str> {
        self.stability.as_deref()
    }

    fn params(&self) -> Vec<EditorReferenceParam<'_>> {
        self.params
            .iter()
            .map(|param| EditorReferenceParam {
                name: &param.name,
                summary: &param.summary,
            })
            .collect()
    }

    fn returns(&self) -> Option<&str> {
        self.returns.as_deref()
    }

    fn markdown(&self) -> Option<&str> {
        self.markdown.as_deref()
    }

    fn example(&self) -> Option<&str> {
        self.example.as_deref()
    }

    fn deprecated(&self) -> Option<&str> {
        self.deprecated.as_deref()
    }

    fn see(&self) -> Vec<&str> {
        self.see.iter().map(String::as_str).collect()
    }
}

impl EditorReferenceItem for ReferenceEntry {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> crate::editor_reference::ReferenceKind {
        match self.kind {
            ReferenceKind::Function => crate::editor_reference::ReferenceKind::Function,
            ReferenceKind::Keyword => crate::editor_reference::ReferenceKind::Keyword,
        }
    }

    fn hidden(&self) -> bool {
        self.hidden
    }

    fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    fn range(&self) -> Option<crate::editor_symbols::SymbolRange> {
        self.range.clone()
    }
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct WorkspaceDocument {
    pub(crate) uri: String,
    pub(crate) text: String,
}

pub(crate) fn reference_document() -> ReferenceDocument {
    serde_json::from_str(REFERENCE_JSON).expect("embedded Scaffold reference JSON is valid")
}

pub(crate) fn workspace_documents(workspace_json: &str) -> Vec<WorkspaceDocument> {
    serde_json::from_str(workspace_json).unwrap_or_default()
}

pub(crate) fn workspace_doc_index(
    workspace: &[WorkspaceDocument],
) -> scaffold_docs::WorkspaceDocIndex {
    let mut index = scaffold_docs::WorkspaceDocIndex::empty();
    for document in workspace {
        index.push_source(scaffold_docs::source_docs_with_definitions(
            &document.uri,
            &document.text,
        ));
    }
    index
}

pub(crate) fn reference_index_for_document(
    text: &str,
    workspace: &[WorkspaceDocument],
) -> ReferenceDocument {
    reference_index_for_source("<open document>", text, workspace)
}

pub(crate) fn reference_index_for_source(
    source_name: &str,
    text: &str,
    workspace: &[WorkspaceDocument],
) -> ReferenceDocument {
    let workspace_docs = workspace_doc_index(workspace);
    let mut index = ReferenceIndex::scaffold();
    let mut imported_docs = scaffold_docs::DocIndex::empty();
    workspace_docs.extend_imported_docs(&mut imported_docs, text);
    index.extend_doc_entries(imported_docs.entries().cloned());
    index.extend_doc_entries(source_docs(source_name, text).entries);
    index.into_document()
}

pub(crate) fn reference_index_for_workspace(workspace: &[WorkspaceDocument]) -> ReferenceDocument {
    let workspace_docs = workspace_doc_index(workspace);
    let mut index = ReferenceIndex::scaffold();
    index.extend_doc_entries(workspace_docs.all().entries().cloned());
    index.into_document()
}

pub(crate) fn completion_items(
    reference: &ReferenceDocument,
) -> Vec<crate::editor_reference::CompletionItem> {
    crate::editor_reference::completion_items(reference.entries.iter())
}

pub(crate) fn hover_markdown(reference: &ReferenceDocument, symbol: &str) -> String {
    crate::editor_reference::hover_markdown(reference.entries.iter(), symbol).unwrap_or_default()
}

pub(crate) fn signature_help(
    reference: &ReferenceDocument,
    symbol: &str,
) -> Option<crate::editor_reference::SignatureHelp> {
    crate::editor_reference::signature_help(reference.entries.iter(), symbol)
}

pub(crate) fn definition_location(
    reference: &ReferenceDocument,
    uri: &str,
    symbol: &str,
) -> Option<crate::editor_reference::DefinitionLocation> {
    let entry = reference
        .entries
        .iter()
        .find(|entry| entry.name == symbol)?;
    crate::editor_reference::definition_location(entry, uri)
}

pub(crate) fn reference_locations(
    workspace: &[WorkspaceDocument],
    symbol: &str,
) -> Vec<crate::editor_symbols::SymbolLocation> {
    crate::editor_symbols::reference_locations(
        workspace
            .iter()
            .map(|document| (document.uri.as_str(), document.text.as_str())),
        symbol,
    )
}

pub(crate) fn workspace_symbols(
    reference: &ReferenceDocument,
    query: &str,
) -> Vec<crate::editor_reference::WorkspaceSymbol> {
    crate::editor_reference::workspace_symbols(reference.entries.iter(), query)
}

pub(crate) fn document_symbols(text: &str) -> Vec<crate::editor_reference::DocumentSymbol> {
    let entries = source_docs("<open document>", text)
        .entries
        .into_iter()
        .map(ReferenceEntry::from)
        .collect::<Vec<_>>();
    crate::editor_reference::document_symbols(entries.iter())
}

#[derive(Debug, Clone)]
struct ReferenceIndex {
    capabilities: Vec<ReferenceCapability>,
    catalog_schema: serde_json::Value,
    entries: Vec<ReferenceEntry>,
}

impl ReferenceIndex {
    fn scaffold() -> Self {
        let document = reference_document();
        Self {
            capabilities: document.capabilities,
            catalog_schema: document.catalog_schema,
            entries: document.entries,
        }
    }

    fn extend_entries(&mut self, entries: impl IntoIterator<Item = ReferenceEntry>) {
        for entry in entries {
            self.insert(entry);
        }
    }

    fn extend_doc_entries(&mut self, entries: impl IntoIterator<Item = scaffold_docs::DocEntry>) {
        self.extend_entries(entries.into_iter().map(ReferenceEntry::from));
    }

    fn insert(&mut self, entry: ReferenceEntry) {
        if self
            .entries
            .iter()
            .any(|existing| existing.name == entry.name && is_language_keyword(existing))
        {
            return;
        }
        if let Some(index) = self
            .entries
            .iter()
            .position(|existing| existing.name == entry.name)
        {
            self.entries[index] = entry;
        } else {
            self.entries.push(entry);
        }
    }

    fn into_document(self) -> ReferenceDocument {
        ReferenceDocument {
            capabilities: self.capabilities,
            catalog_schema: self.catalog_schema,
            entries: self.entries,
        }
    }
}

fn source_docs(source_name: &str, source: &str) -> scaffold_docs::SourceDocs {
    scaffold_docs::source_docs_with_definitions(source_name, source)
}

const fn symbol_range(range: scaffold_docs::SourceRange) -> crate::editor_symbols::SymbolRange {
    crate::editor_symbols::SymbolRange {
        line: range.start.line,
        start: range.start.character,
        length: range.end.character.saturating_sub(range.start.character),
    }
}

fn is_language_keyword(entry: &ReferenceEntry) -> bool {
    entry.kind == ReferenceKind::Keyword
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_docs_override_non_keyword_reference_entries() {
        let index =
            reference_index_for_document("(doc 'tool (summary \"Project-local tool docs.\"))", &[]);

        assert_eq!(
            index
                .entries
                .iter()
                .find(|entry| entry.name == "tool")
                .and_then(|entry| entry.summary.as_deref()),
            Some("Project-local tool docs.")
        );
    }

    #[test]
    fn document_docs_do_not_override_language_keywords() {
        let index =
            reference_index_for_document("(doc 'define (summary \"Local define docs.\"))", &[]);

        assert_eq!(
            index
                .entries
                .iter()
                .find(|entry| entry.name == "define")
                .and_then(|entry| entry.source.as_deref()),
            Some("scheme keyword")
        );
    }

    #[test]
    fn workspace_docs_are_scoped_to_imported_libraries() {
        let workspace = vec![
            WorkspaceDocument {
                uri: "file:///workspace/acme.scm".to_owned(),
                text: include_str!("fixtures/reference-workspace-acme.scm").to_owned(),
            },
            WorkspaceDocument {
                uri: "file:///workspace/other.scm".to_owned(),
                text: include_str!("fixtures/reference-workspace-other.scm").to_owned(),
            },
        ];
        let index =
            reference_index_for_document("(import (acme tools))\n(acme-tool \"demo\")", &workspace);

        assert!(index.entries.iter().any(|entry| entry.name == "acme-tool"));
        assert!(!index.entries.iter().any(|entry| entry.name == "other-tool"));
    }

    #[test]
    fn workspace_imports_include_undocumented_definitions() {
        let workspace = vec![WorkspaceDocument {
            uri: "file:///workspace/acme.scm".to_owned(),
            text: include_str!("fixtures/reference-workspace-undocumented.scm").to_owned(),
        }];
        let index = reference_index_for_document(
            "(import (acme tools))\n(acme-helper \"demo\")",
            &workspace,
        );
        let helper = index
            .entries
            .iter()
            .find(|entry| entry.name == "acme-helper")
            .expect("imported helper definition");

        assert_eq!(helper.source.as_deref(), Some("file:///workspace/acme.scm"));
        assert_eq!(helper.range.as_ref().expect("definition range").line, 2);
        assert!(helper.summary.is_none());
    }

    #[test]
    fn signature_help_uses_signature_parameters_and_param_docs() {
        let index =
            reference_index_for_document(include_str!("fixtures/reference-signature-doc.scm"), &[]);
        let help = signature_help(&index, "demo").expect("signature help");

        assert_eq!(help.label, "(demo name [mode] rest ...)");
        assert!(help.documentation.contains("Demo helper."));
        assert_eq!(
            help.parameters
                .iter()
                .map(|param| (param.label.as_str(), param.documentation.as_deref()))
                .collect::<Vec<_>>(),
            vec![
                ("name", Some("Name docs.")),
                ("[mode]", None),
                ("rest", None),
                ("...", None),
            ],
        );
    }
}
