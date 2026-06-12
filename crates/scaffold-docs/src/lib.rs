#[cfg(feature = "reference")]
use serde::Serialize;
#[cfg(feature = "reference")]
use std::collections::BTreeMap;

mod index;

pub use index::{
    DocEntry, DocIndex, DocKind, DocParam, SourceDocs, SourcePosition, SourceRange,
    WorkspaceDocIndex, markdown_for_entry, snippet_for_signature, source_docs,
    source_docs_with_definitions,
};

#[cfg(feature = "reference")]
#[must_use]
pub fn scaffold_reference_markdown() -> String {
    render_reference_markdown(&DocIndex::scaffold())
}

#[cfg(feature = "reference")]
pub fn scaffold_reference_json() -> serde_json::Result<String> {
    serde_json::to_string_pretty(&ReferenceDocument::from_index(&DocIndex::scaffold()))
}

#[cfg(feature = "reference")]
#[must_use]
pub fn render_reference_markdown(index: &DocIndex) -> String {
    let mut output = String::from("# Scaffold Scheme Reference\n\n");
    output.push_str(
        "Generated from parsable Doc v2 forms such as `(doc ...)`, `(doc-next ...)`, `(extern-doc ...)`, `(moduledoc ...)`, and `(typedoc ...)`.\n",
    );
    output.push_str("\n## Capability Contracts\n\n");
    output.push_str("| Library | Effect | Catalog | Test | Editor | WASM |\n");
    output.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for capability in scaffold_dsl::rust_backed_capabilities() {
        output.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            capability.library,
            capability.effect,
            capability_mode(capability, "catalog"),
            capability_mode(capability, "test"),
            capability_mode(capability, "editor"),
            capability_mode(capability, "wasm"),
        ));
    }
    output.push_str("\n## Catalog Schema\n\n");
    output.push_str(
        "The generated JSON reference includes `catalog_schema` with field, enum, and relationship metadata used by Scaffold catalog validation.\n",
    );

    let mut groups = BTreeMap::<String, Vec<&DocEntry>>::new();
    for entry in index.visible_entries() {
        groups
            .entry(entry.group.clone().unwrap_or_else(|| "Language".to_owned()))
            .or_default()
            .push(entry);
    }

    for entries in groups.values_mut() {
        entries.sort_by(|left, right| left.name.cmp(&right.name));
    }

    output.push_str("\n## Contents\n\n");
    for (group, entries) in &groups {
        output.push_str(&format!(
            "- [{}](#{}) ({})\n",
            group,
            anchor(group),
            entries.len()
        ));
    }

    for (group, entries) in groups {
        output.push_str(&format!("\n## {group}\n\n"));
        for entry in entries {
            output.push_str(&format!("### `{}`\n\n", entry.name));
            let markdown = markdown_for_entry(entry);
            if markdown.trim().is_empty() {
                output.push_str("No documentation provided.\n\n");
            } else {
                output.push_str(markdown.trim());
                output.push_str("\n\n");
            }
            if let Some(source) = &entry.source {
                output.push_str(&format!("Source: `{source}`\n\n"));
            }
        }
    }

    output
}

#[cfg(feature = "reference")]
#[derive(Debug, Serialize)]
struct ReferenceDocument {
    title: String,
    capabilities: Vec<ReferenceCapability>,
    catalog_schema: serde_json::Value,
    entries: Vec<ReferenceEntry>,
}

#[cfg(feature = "reference")]
impl ReferenceDocument {
    fn from_index(index: &DocIndex) -> Self {
        let mut entries = index
            .visible_entries()
            .map(ReferenceEntry::from_doc_entry)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            left.group
                .cmp(&right.group)
                .then_with(|| left.name.cmp(&right.name))
        });
        Self {
            title: "Scaffold Scheme Reference".to_owned(),
            capabilities: scaffold_dsl::rust_backed_capabilities()
                .iter()
                .map(ReferenceCapability::from_descriptor)
                .collect(),
            catalog_schema: scaffold_catalog::catalog_schema(),
            entries,
        }
    }
}

#[cfg(feature = "reference")]
#[derive(Debug, Serialize)]
struct ReferenceCapability {
    library_name: Vec<String>,
    library: String,
    bridge_library_name: Vec<String>,
    bridge_library: String,
    effect: String,
    modes: BTreeMap<String, String>,
    docs_source: String,
    notes: String,
}

#[cfg(feature = "reference")]
impl ReferenceCapability {
    fn from_descriptor(descriptor: &scaffold_dsl::CapabilityDescriptor) -> Self {
        Self {
            library_name: descriptor
                .library_name
                .iter()
                .map(|component| (*component).to_owned())
                .collect(),
            library: descriptor.library.to_owned(),
            bridge_library_name: descriptor
                .bridge_library_name
                .iter()
                .map(|component| (*component).to_owned())
                .collect(),
            bridge_library: descriptor.bridge_library.to_owned(),
            effect: descriptor.effect.to_owned(),
            modes: descriptor
                .modes
                .iter()
                .map(|mode| (mode.name.to_owned(), mode.availability.to_owned()))
                .collect(),
            docs_source: descriptor.docs_source.to_owned(),
            notes: descriptor.notes.to_owned(),
        }
    }
}

#[cfg(feature = "reference")]
#[derive(Debug, Serialize)]
struct ReferenceEntry {
    name: String,
    kind: &'static str,
    signature: Option<String>,
    summary: Option<String>,
    markdown: Option<String>,
    example: Option<String>,
    params: Vec<ReferenceParam>,
    returns: Option<String>,
    group: String,
    see: Vec<String>,
    effect: Option<String>,
    requires_capability: Vec<String>,
    stability: Option<String>,
    since: Option<String>,
    deprecated: Option<String>,
    source: Option<String>,
    range: Option<ReferenceRange>,
}

#[cfg(feature = "reference")]
impl ReferenceEntry {
    fn from_doc_entry(entry: &DocEntry) -> Self {
        Self {
            name: entry.name.clone(),
            kind: doc_kind_name(entry.kind),
            signature: entry.signature.clone(),
            summary: entry.summary.clone(),
            markdown: entry.markdown.clone(),
            example: entry.example.clone(),
            params: entry
                .params
                .iter()
                .map(|param| ReferenceParam {
                    name: param.name.clone(),
                    summary: param.summary.clone(),
                })
                .collect(),
            returns: entry.returns.clone(),
            group: entry.group.clone().unwrap_or_else(|| "Language".to_owned()),
            see: entry.see.clone(),
            effect: entry.effect.clone(),
            requires_capability: entry.requires_capability.clone(),
            stability: entry.stability.clone(),
            since: entry.since.clone(),
            deprecated: entry.deprecated.clone(),
            source: entry.source.clone(),
            range: entry.range.map(ReferenceRange::from_source_range),
        }
    }
}

#[cfg(feature = "reference")]
#[derive(Debug, Serialize)]
struct ReferenceRange {
    line: u32,
    start: u32,
    length: u32,
}

#[cfg(feature = "reference")]
impl ReferenceRange {
    const fn from_source_range(range: SourceRange) -> Self {
        Self {
            line: range.start.line,
            start: range.start.character,
            length: range.end.character.saturating_sub(range.start.character),
        }
    }
}

#[cfg(feature = "reference")]
const fn doc_kind_name(kind: DocKind) -> &'static str {
    match kind {
        DocKind::Function => "function",
        DocKind::Keyword => "keyword",
    }
}

#[cfg(feature = "reference")]
#[derive(Debug, Serialize)]
struct ReferenceParam {
    name: String,
    summary: String,
}

#[cfg(feature = "reference")]
fn capability_mode(capability: &scaffold_dsl::CapabilityDescriptor, mode: &str) -> &'static str {
    capability
        .modes
        .iter()
        .find(|item| item.name == mode)
        .map_or("unavailable", |item| item.availability)
}

#[cfg(feature = "reference")]
fn anchor(text: &str) -> String {
    let mut output = String::new();
    let mut previous_dash = false;
    for ch in text.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            output.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            output.push('-');
            previous_dash = true;
        }
    }
    output.trim_matches('-').to_owned()
}

#[cfg(all(test, feature = "reference"))]
mod tests {
    use super::*;

    #[test]
    fn renders_grouped_reference_markdown() {
        let markdown = scaffold_reference_markdown();

        assert!(markdown.starts_with("# Scaffold Scheme Reference"));
        assert!(markdown.contains("## Capability Contracts"));
        assert!(markdown.contains("## Catalog Schema"));
        assert!(markdown.contains("| `(scaffold fs)` | host-read-only |"));
        assert!(markdown.contains("## Catalog"));
        assert!(markdown.contains("## Filesystem"));
        assert!(markdown.contains("## Paths"));
        assert!(markdown.contains("## Workspace"));
        assert!(markdown.contains("### `tool`"));
        assert!(markdown.contains("### `path/exists?`"));
        assert!(markdown.contains("### `path/join`"));
        assert!(markdown.contains("### `workspace/path`"));
        assert!(markdown.contains("Parameters:"));
        assert!(!markdown.contains("### `doc-field`"));
    }

    #[test]
    fn renders_structured_reference_json() {
        let json = scaffold_reference_json().expect("reference json");

        assert!(json.contains("\"title\": \"Scaffold Scheme Reference\""));
        assert!(json.contains("\"capabilities\""));
        assert!(json.contains("\"catalog_schema\""));
        assert!(json.contains("\"relationships\""));
        assert!(json.contains("\"library\": \"(scaffold fs)\""));
        assert!(json.contains("\"effect\": \"host-read-only\""));
        assert!(json.contains("\"name\": \"tool\""));
        assert!(json.contains("\"kind\": \"function\""));
        assert!(json.contains("\"name\": \"path/exists?\""));
        assert!(json.contains("\"name\": \"path/join\""));
        assert!(json.contains("\"name\": \"command/path\""));
        assert!(json.contains("\"name\": \"workspace/path\""));
        assert!(json.contains("\"kind\": \"keyword\""));
        assert!(json.contains("\"group\": \"Catalog\""));
        assert!(json.contains("\"group\": \"Filesystem\""));
        assert!(json.contains("\"group\": \"Paths\""));
        assert!(json.contains("\"group\": \"Workspace\""));
        assert!(!json.contains("\"name\": \"doc-field\""));
    }

    #[test]
    fn writes_reference_json_when_requested() {
        let Ok(path) = std::env::var("SCAFFOLD_WRITE_REFERENCE_JSON") else {
            return;
        };
        std::fs::write(path, scaffold_reference_json().expect("reference json"))
            .expect("write reference json");
    }
}
