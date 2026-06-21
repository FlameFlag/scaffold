#[cfg(feature = "reference")]
use serde::Serialize;
#[cfg(feature = "reference")]
use std::collections::BTreeMap;
#[cfg(feature = "reference")]
use std::fmt::Write as _;

mod index;

pub use index::{
    DocEntry, DocIndex, DocKind, DocParam, EntryDetail, EntryDocumentation, SourceDocs,
    SourcePosition, SourceRange, WorkspaceDocIndex, detailed_markdown_for_entry, entry_count_label,
    entry_documentation, entry_summary_markdown_table, group_count_label, group_markdown_table,
    markdown_for_entry, normalize_reference_query_token, rendered_markdown_for_entry,
    search_doc_entries, search_reference_entries, snippet_for_signature, source_docs,
    source_docs_with_definitions, source_markdown_for_entry, source_path_from_location_query,
    suggest_doc_entries, suggest_reference_entries, titled_markdown_for_entry,
};
#[cfg(feature = "reference")]
pub use scaffold_editor::reference::{
    CompletionItem, completion_items, format_markdown_tables, markdown_code_span, markdown_table,
    markdown_text, same_markdown_paragraph,
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
pub fn scaffold_reference_value() -> serde_json::Result<serde_json::Value> {
    serde_json::to_value(ReferenceDocument::from_index(&DocIndex::scaffold()))
}

#[cfg(feature = "reference")]
#[must_use]
pub fn reference_entry_json(entry: &DocEntry) -> serde_json::Value {
    serde_json::to_value(ReferenceEntry::from_doc_entry(entry))
        .expect("reference entries serialize to JSON values")
}

#[cfg(feature = "reference")]
#[must_use]
pub fn render_reference_markdown(index: &DocIndex) -> String {
    let mut output = String::from("# Scaffold Scheme Reference\n\n");
    output.push_str(
        "Generated reference for Scaffold Scheme symbols, catalog helpers, and standard extension libraries.\n",
    );

    let groups = index.visible_entries().fold(
        BTreeMap::<String, Vec<&DocEntry>>::new(),
        |mut groups, entry| {
            groups
                .entry(entry.group_name().to_owned())
                .or_default()
                .push(entry);
            groups
        },
    );

    output.push_str("\n## Contents\n\n");
    output.push_str(&markdown_table(
        &["Group", "Entries"],
        groups.iter().map(|(group, entries)| {
            vec![
                format!("[{}](#{})", markdown_text(group), anchor(group)),
                entries.len().to_string(),
            ]
        }),
    ));
    output.push_str("\n## Capability Contracts\n\n");
    output.push_str("Rust-backed libraries expose different capability levels depending on where the DSL is running.\n\n");
    output.push_str(&markdown_table(
        &["Library", "Effect", "Catalog", "Test", "Editor", "WASM"],
        scaffold_dsl::rust_backed_capabilities()
            .iter()
            .map(|capability| {
                vec![
                    markdown_code_span(capability.library),
                    capability.effect.to_owned(),
                    capability_mode(capability, "catalog").to_owned(),
                    capability_mode(capability, "test").to_owned(),
                    capability_mode(capability, "editor").to_owned(),
                    capability_mode(capability, "wasm").to_owned(),
                ]
            }),
    ));

    output.push_str("\n## Catalog Schema\n\n");
    output.push_str(
        "The JSON reference export includes `catalog_schema` with field, enum, and relationship metadata used by Scaffold catalog validation.\n",
    );

    for (group, entries) in &groups {
        let _ = writeln!(
            &mut output,
            "\n## <a id=\"{}\"></a>{}\n",
            anchor(group),
            markdown_text(group)
        );
        for entry in entries {
            let _ = writeln!(&mut output, "### {}\n", markdown_code_span(&entry.name));
            let markdown = markdown_for_entry(entry);
            if markdown.trim().is_empty() {
                output.push_str("No documentation provided.\n\n");
            } else {
                output.push_str(markdown.trim());
                output.push_str("\n\n");
            }
            if let Some(location) = entry.display_source_location() {
                output.push_str(&markdown_table(
                    &["Field", "Value"],
                    vec![vec!["Source".to_owned(), markdown_code_span(location)]],
                ));
                output.push('\n');
            }
        }
    }

    format_markdown_tables(output)
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
            library_name: owned_components(descriptor.library_name),
            library: descriptor.library.to_owned(),
            bridge_library_name: owned_components(descriptor.bridge_library_name),
            bridge_library: descriptor.bridge_library.to_owned(),
            effect: descriptor.effect.to_owned(),
            modes: capability_modes(descriptor),
            docs_source: descriptor.docs_source.to_owned(),
            notes: descriptor.notes.to_owned(),
        }
    }
}

#[cfg(feature = "reference")]
fn owned_components(components: &[&str]) -> Vec<String> {
    components.iter().copied().map(str::to_owned).collect()
}

#[cfg(feature = "reference")]
fn capability_modes(descriptor: &scaffold_dsl::CapabilityDescriptor) -> BTreeMap<String, String> {
    descriptor
        .modes
        .iter()
        .map(|mode| (mode.name.to_owned(), mode.availability.to_owned()))
        .collect()
}

#[cfg(feature = "reference")]
#[derive(Debug, Serialize)]
struct ReferenceEntry {
    name: String,
    kind: &'static str,
    signature: Option<String>,
    summary: Option<String>,
    markdown: Option<String>,
    raw_markdown: Option<String>,
    rendered_markdown: String,
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
    source_location: Option<String>,
    range: Option<ReferenceRange>,
    hidden: bool,
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
            raw_markdown: entry.markdown.clone(),
            rendered_markdown: rendered_markdown_for_entry(entry),
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
            group: entry.group_name().to_owned(),
            see: entry.see.clone(),
            effect: entry.effect.clone(),
            requires_capability: entry.requires_capability.clone(),
            stability: entry.stability.clone(),
            since: entry.since.clone(),
            deprecated: entry.deprecated.clone(),
            source: entry.source.clone(),
            source_location: entry.display_source_location(),
            range: entry.range.map(SourceRange::symbol_range).map(Into::into),
            hidden: entry.hidden,
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
impl From<scaffold_editor::symbols::SymbolRange> for ReferenceRange {
    fn from(range: scaffold_editor::symbols::SymbolRange) -> Self {
        Self {
            line: range.line,
            start: range.start,
            length: range.length,
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
        assert!(markdown.contains("## Contents"));
        assert!(markdown.contains("| Group                                   | Entries |"));
        assert!(markdown.contains("| [Catalog](#catalog)                     | 41      |"));
        assert!(markdown.contains("## Capability Contracts"));
        assert!(markdown.contains("## Catalog Schema"));
        assert!(
            markdown
                .contains("| `(scaffold fs)`        | host-read-only    | available | available |")
        );
        assert!(markdown.contains("## <a id=\"catalog\"></a>Catalog"));
        assert!(markdown.contains("## <a id=\"filesystem\"></a>Filesystem"));
        assert!(markdown.contains("## <a id=\"paths\"></a>Paths"));
        assert!(markdown.contains("## <a id=\"workspace\"></a>Workspace"));
        assert!(markdown.contains("### `tool`"));
        assert!(markdown.contains("### `path/exists?`"));
        assert!(markdown.contains("### `path/join`"));
        assert!(markdown.contains("### `workspace/path`"));
        assert!(markdown.contains("**Parameters**"));
        assert!(markdown.contains("| Source | `src/dsl/std/catalog/tool.scm:"));
        assert!(!markdown.contains("\nSource: `"));
        assert!(!markdown.contains("### `doc-field`"));
    }

    #[test]
    fn reference_markdown_escapes_group_link_and_heading_text() {
        let mut index = DocIndex::empty();
        let entry = DocEntry {
            name: "group/entry".to_owned(),
            signature: None,
            summary: None,
            markdown: None,
            example: None,
            params: Vec::new(),
            returns: None,
            group: Some("Bad [Group] | Plus+".to_owned()),
            see: Vec::new(),
            effect: None,
            requires_capability: Vec::new(),
            stability: None,
            since: None,
            deprecated: None,
            hidden: false,
            source: None,
            range: None,
            kind: DocKind::Function,
        };
        index.insert(entry);

        let markdown = render_reference_markdown(&index);

        assert!(markdown.contains("| [Bad \\[Group\\] \\| Plus\\+](#bad-group-plus) | 1       |"));
        assert!(markdown.contains("## <a id=\"bad-group-plus\"></a>Bad \\[Group\\] | Plus\\+"));
        assert!(!markdown.contains("[Bad [Group] | Plus+]"));
        assert!(!markdown.contains("## Bad [Group] | Plus+"));
    }

    #[test]
    fn markdown_table_formats_and_escapes_cells() {
        let table = markdown_table(
            &["Name", "Summary"],
            vec![vec![
                "`pipe`".to_owned(),
                "keeps | escaped\nand compact".to_owned(),
            ]],
        );

        assert!(table.contains("| Name   | Summary"));
        assert!(table.contains("keeps \\| escaped<br>and compact"));
    }

    #[test]
    fn renders_structured_reference_json() {
        let json = scaffold_reference_json().expect("reference json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("reference value");
        let tool = value["entries"]
            .as_array()
            .expect("entries")
            .iter()
            .find(|entry| entry["name"] == "tool")
            .expect("tool entry");

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
        assert_eq!(
            tool["source_location"].as_str(),
            Some("src/dsl/std/catalog/tool.scm:16")
        );
        assert_eq!(tool["hidden"].as_bool(), Some(false));
        assert!(tool["markdown"].is_null());
        assert!(tool["raw_markdown"].is_null());
        assert!(tool["rendered_markdown"].as_str().is_some_and(|markdown| {
            markdown.contains("```scheme\n(tool name action field ...)\n```")
                && markdown.contains("**Parameters**")
        }));
        let subject = value["entries"]
            .as_array()
            .expect("entries")
            .iter()
            .find(|entry| entry["name"] == "subject")
            .expect("subject entry");
        assert_eq!(
            subject["rendered_markdown"].as_str(),
            Some("No documentation provided.")
        );
        assert!(!json.contains("\"name\": \"doc-field\""));
    }

    #[test]
    fn renders_reference_entry_json_value() {
        let index = DocIndex::scaffold();
        let entry = index.get("tool").expect("tool entry");
        let value = reference_entry_json(entry);

        assert_eq!(value["name"], "tool");
        assert_eq!(value["kind"], "function");
        assert_eq!(value["group"], "Catalog");
        assert!(value["markdown"].is_null());
        assert!(value["raw_markdown"].is_null());
        assert!(value["rendered_markdown"].as_str().is_some_and(|markdown| {
            markdown.contains("```scheme\n(tool name action field ...)\n```")
                && markdown.contains("**Parameters**")
        }));
        assert_eq!(
            value["source_location"].as_str(),
            Some("src/dsl/std/catalog/tool.scm:16")
        );
        assert_eq!(value["hidden"].as_bool(), Some(false));
        assert!(
            value["range"]["length"]
                .as_u64()
                .is_some_and(|length| length > 0)
        );
        assert!(
            value["params"]
                .as_array()
                .is_some_and(|params| params.iter().any(|param| param["name"] == "name"))
        );
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
