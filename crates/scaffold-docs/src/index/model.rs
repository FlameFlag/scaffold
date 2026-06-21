use std::collections::{BTreeMap, BTreeSet};

use scaffold_editor::reference::{
    ReferenceEntry, ReferenceItem, ReferenceKind, ReferenceParam as EditorReferenceParam,
};

use super::keywords::LANGUAGE_KEYWORDS;
use super::source::{source_docs, source_docs_with_definitions};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocParam {
    pub name: String,
    pub summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKind {
    Function,
    Keyword,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

impl SourceRange {
    #[must_use]
    pub const fn symbol_range(self) -> scaffold_editor::symbols::SymbolRange {
        scaffold_editor::symbols::SymbolRange {
            line: self.start.line,
            start: self.start.character,
            length: self.end.character.saturating_sub(self.start.character),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocEntry {
    pub name: String,
    pub signature: Option<String>,
    pub summary: Option<String>,
    pub markdown: Option<String>,
    pub example: Option<String>,
    pub params: Vec<DocParam>,
    pub returns: Option<String>,
    pub group: Option<String>,
    pub see: Vec<String>,
    pub effect: Option<String>,
    pub requires_capability: Vec<String>,
    pub stability: Option<String>,
    pub since: Option<String>,
    pub deprecated: Option<String>,
    pub hidden: bool,
    pub source: Option<String>,
    pub range: Option<SourceRange>,
    pub kind: DocKind,
}

impl DocEntry {
    pub(super) fn new(name: impl Into<String>, kind: DocKind) -> Self {
        Self {
            name: name.into(),
            signature: None,
            summary: None,
            markdown: None,
            example: None,
            params: Vec::new(),
            returns: None,
            group: None,
            see: Vec::new(),
            effect: None,
            requires_capability: Vec::new(),
            stability: None,
            since: None,
            deprecated: None,
            hidden: false,
            source: None,
            range: None,
            kind,
        }
    }

    #[must_use]
    pub fn group_name(&self) -> &str {
        self.group.as_deref().unwrap_or("Language")
    }

    #[must_use]
    pub fn display_source_location(&self) -> Option<String> {
        let source = self.source.as_deref()?;
        Some(match self.range {
            Some(range) => format!("{source}:{}", range.start.line + 1),
            None => source.to_owned(),
        })
    }
}

impl ReferenceEntry for DocEntry {
    fn signature(&self) -> Option<&str> {
        self.signature.as_deref()
    }

    fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    fn since(&self) -> Option<&str> {
        self.since.as_deref()
    }

    fn stability(&self) -> Option<&str> {
        self.stability.as_deref()
    }

    fn effect(&self) -> Option<&str> {
        self.effect.as_deref()
    }

    fn requires_capability(&self) -> impl Iterator<Item = &str> {
        self.requires_capability.iter().map(String::as_str)
    }

    fn params(&self) -> impl Iterator<Item = EditorReferenceParam<'_>> {
        self.params.iter().map(|param| EditorReferenceParam {
            name: &param.name,
            summary: &param.summary,
        })
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

    fn see(&self) -> impl Iterator<Item = &str> {
        self.see.iter().map(String::as_str)
    }
}

impl ReferenceItem for DocEntry {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> ReferenceKind {
        match self.kind {
            DocKind::Function => ReferenceKind::Function,
            DocKind::Keyword => ReferenceKind::Keyword,
        }
    }

    fn hidden(&self) -> bool {
        self.hidden
    }

    fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    fn source_location(&self) -> Option<String> {
        self.display_source_location()
    }

    fn range(&self) -> Option<scaffold_editor::symbols::SymbolRange> {
        self.range.map(SourceRange::symbol_range)
    }
}

#[derive(Debug, Clone)]
pub struct DocIndex {
    entries: BTreeMap<String, DocEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDocs {
    pub library_name: Option<Vec<String>>,
    pub imports: Vec<Vec<String>>,
    pub entries: Vec<DocEntry>,
}

#[derive(Debug, Clone)]
pub struct WorkspaceDocIndex {
    all: DocIndex,
    sources: Vec<WorkspaceDocSource>,
}

#[derive(Debug, Clone)]
struct WorkspaceDocSource {
    library_name: Option<Vec<String>>,
    imports: Vec<Vec<String>>,
    docs: Vec<DocEntry>,
}

impl DocIndex {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    #[cfg(feature = "reference")]
    #[must_use]
    pub fn scaffold() -> Self {
        let mut index = Self::with_language_keywords();
        for source in scaffold_dsl::documentation_sources() {
            index.extend_source(source.path, source.source);
        }
        index
    }

    #[must_use]
    pub fn with_language_keywords() -> Self {
        Self {
            entries: LANGUAGE_KEYWORDS
                .iter()
                .map(|(name, signature, summary, snippet_source)| {
                    let mut entry = DocEntry::new(*name, DocKind::Keyword);
                    entry.signature = Some((*signature).to_owned());
                    entry.summary = Some((*summary).to_owned());
                    entry.markdown = Some((*summary).to_owned());
                    entry.source = Some((*snippet_source).to_owned());
                    ((*name).to_owned(), entry)
                })
                .collect(),
        }
    }

    pub fn extend_source(&mut self, source_name: &str, source: &str) {
        self.extend_entries(source_docs(source_name, source).entries);
    }

    pub fn extend_editor_source(&mut self, source_name: &str, source: &str) {
        self.extend_entries(source_docs_with_definitions(source_name, source).entries);
    }

    pub fn extend_entries(&mut self, entries: impl IntoIterator<Item = DocEntry>) {
        for entry in entries {
            self.insert(entry);
        }
    }

    pub fn extend_index(&mut self, other: &Self) {
        for entry in other.entries() {
            self.insert(entry.clone());
        }
    }

    #[must_use]
    pub fn merged_with_document(&self, source_name: &str, source: &str) -> Self {
        let mut index = self.clone();
        index.extend_source(source_name, source);
        index
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<&DocEntry> {
        self.entries.get(name)
    }

    pub fn entries(&self) -> impl Iterator<Item = &DocEntry> {
        self.entries.values()
    }

    pub fn visible_entries(&self) -> impl Iterator<Item = &DocEntry> {
        self.entries.values().filter(|entry| !entry.hidden)
    }

    pub fn entries_in_source<'index, 'source>(
        &'index self,
        source_name: &'source str,
    ) -> impl Iterator<Item = &'index DocEntry> + 'source
    where
        'index: 'source,
    {
        self.entries
            .values()
            .filter(move |entry| entry.source.as_deref() == Some(source_name))
    }

    pub fn insert(&mut self, entry: DocEntry) {
        if self
            .entries
            .get(&entry.name)
            .is_some_and(|existing| existing.kind == DocKind::Keyword)
        {
            return;
        }
        let _previous = self.entries.insert(entry.name.clone(), entry);
    }
}

impl WorkspaceDocIndex {
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            all: DocIndex::empty(),
            sources: Vec::new(),
        }
    }

    pub fn push_source(&mut self, source_docs: SourceDocs) {
        self.all.extend_entries(source_docs.entries.clone());
        self.sources.push(WorkspaceDocSource {
            library_name: source_docs.library_name,
            imports: source_docs.imports,
            docs: source_docs.entries,
        });
    }

    #[must_use]
    pub const fn all(&self) -> &DocIndex {
        &self.all
    }

    pub fn extend_imported_docs(&self, index: &mut DocIndex, document_text: &str) {
        let document_docs = source_docs("<open document>", document_text);
        let mut pending = document_docs.imports;
        let mut seen = BTreeSet::new();

        while let Some(name) = pending.pop() {
            if !seen.insert(name.clone()) {
                continue;
            }
            for source in self
                .sources
                .iter()
                .filter(|source| source.library_name.as_ref() == Some(&name))
            {
                index.extend_entries(source.docs.clone());
                pending.extend(source.imports.clone());
            }
        }
    }
}
