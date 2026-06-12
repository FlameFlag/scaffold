use crate::symbols::SymbolRange;

pub struct ReferenceParam<'a> {
    pub name: &'a str,
    pub summary: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReferenceKind {
    Function,
    Keyword,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: ReferenceKind,
    pub detail: Option<String>,
    pub documentation: String,
    pub insert_text: String,
    pub insert_text_is_snippet: bool,
    pub sort_text: String,
    pub deprecated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SignatureParameter {
    pub label: String,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SignatureHelp {
    pub label: String,
    pub documentation: String,
    pub parameters: Vec<SignatureParameter>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DefinitionLocation {
    pub uri: String,
    pub line: u32,
    pub start: u32,
    pub length: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind: ReferenceKind,
    pub group: Option<String>,
    pub deprecated: bool,
    pub uri: String,
    pub line: u32,
    pub start: u32,
    pub length: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DocumentSymbol {
    pub name: String,
    pub detail: Option<String>,
    pub line: u32,
    pub start: u32,
    pub length: u32,
    pub kind: ReferenceKind,
}

pub trait ReferenceEntry {
    fn signature(&self) -> Option<&str>;
    fn summary(&self) -> Option<&str>;
    fn group(&self) -> Option<&str>;
    fn since(&self) -> Option<&str>;
    fn stability(&self) -> Option<&str>;
    fn params(&self) -> Vec<ReferenceParam<'_>>;
    fn returns(&self) -> Option<&str>;
    fn markdown(&self) -> Option<&str>;
    fn example(&self) -> Option<&str>;
    fn deprecated(&self) -> Option<&str>;
    fn see(&self) -> Vec<&str>;
}

pub trait ReferenceItem: ReferenceEntry {
    fn name(&self) -> &str;
    fn kind(&self) -> ReferenceKind;
    fn hidden(&self) -> bool;
    fn source(&self) -> Option<&str>;
    fn range(&self) -> Option<SymbolRange>;
}

pub fn completion_items<'a, E>(entries: impl IntoIterator<Item = &'a E>) -> Vec<CompletionItem>
where
    E: ReferenceItem + 'a,
{
    let mut items = entries
        .into_iter()
        .filter(|entry| !entry.hidden())
        .map(|entry| {
            let snippet = entry.signature().and_then(snippet_for_signature);
            CompletionItem {
                label: entry.name().to_owned(),
                kind: entry.kind(),
                detail: entry
                    .signature()
                    .map(str::to_owned)
                    .or_else(|| entry.summary().map(str::to_owned)),
                documentation: markdown_for_entry(entry),
                insert_text: snippet.clone().unwrap_or_else(|| entry.name().to_owned()),
                insert_text_is_snippet: snippet.is_some(),
                sort_text: completion_sort_text(entry, entry.name()),
                deprecated: entry.deprecated().is_some(),
            }
        })
        .collect::<Vec<_>>();
    items.sort_by(|left, right| left.sort_text.cmp(&right.sort_text));
    items
}

pub fn hover_markdown<'a, E>(
    entries: impl IntoIterator<Item = &'a E>,
    symbol: &str,
) -> Option<String>
where
    E: ReferenceItem + 'a,
{
    let markdown = markdown_for_entry(entries.into_iter().find(|entry| entry.name() == symbol)?);
    (!markdown.trim().is_empty()).then_some(markdown)
}

pub fn signature_help<'a, E>(
    entries: impl IntoIterator<Item = &'a E>,
    symbol: &str,
) -> Option<SignatureHelp>
where
    E: ReferenceItem + 'a,
{
    let entry = entries.into_iter().find(|entry| entry.name() == symbol)?;
    let signature = entry.signature()?;
    Some(SignatureHelp {
        label: signature.to_owned(),
        documentation: markdown_for_entry(entry),
        parameters: signature_parameters(entry),
    })
}

pub fn definition_location(
    entry: &impl ReferenceItem,
    fallback_uri: &str,
) -> Option<DefinitionLocation> {
    let range = entry.range()?;
    Some(DefinitionLocation {
        uri: entry
            .source()
            .map_or_else(|| fallback_uri.to_owned(), str::to_owned),
        line: range.line,
        start: range.start,
        length: range.length,
    })
}

pub fn workspace_symbols<'a, E>(
    entries: impl IntoIterator<Item = &'a E>,
    query: &str,
) -> Vec<WorkspaceSymbol>
where
    E: ReferenceItem + 'a,
{
    let query = query.trim().to_lowercase();
    let mut symbols = entries
        .into_iter()
        .filter(|entry| !entry.hidden())
        .filter(|entry| query.is_empty() || entry_matches_query(*entry, entry.name(), &query))
        .filter_map(|entry| {
            let range = entry.range()?;
            let uri = entry.source()?.to_owned();
            Some(WorkspaceSymbol {
                name: entry.name().to_owned(),
                kind: entry.kind(),
                group: entry.group().map(str::to_owned),
                deprecated: entry.deprecated().is_some(),
                uri,
                line: range.line,
                start: range.start,
                length: range.length,
            })
        })
        .collect::<Vec<_>>();
    symbols.sort_by(|left, right| {
        left.group
            .cmp(&right.group)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.uri.cmp(&right.uri))
            .then_with(|| left.line.cmp(&right.line))
            .then_with(|| left.start.cmp(&right.start))
            .then_with(|| left.length.cmp(&right.length))
    });
    symbols
}

pub fn document_symbols<'a, E>(entries: impl IntoIterator<Item = &'a E>) -> Vec<DocumentSymbol>
where
    E: ReferenceItem + 'a,
{
    entries
        .into_iter()
        .filter(|entry| !entry.hidden())
        .filter_map(|entry| {
            let range = entry.range()?;
            Some(DocumentSymbol {
                name: entry.name().to_owned(),
                detail: entry.signature().map(str::to_owned),
                line: range.line,
                start: range.start,
                length: range.length,
                kind: entry.kind(),
            })
        })
        .collect()
}

pub fn markdown_for_entry(entry: &impl ReferenceEntry) -> String {
    let mut output = String::new();
    if let Some(signature) = entry.signature() {
        output.push_str("```scheme\n");
        output.push_str(signature);
        output.push_str("\n```\n\n");
    }
    if let Some(summary) = entry.summary() {
        output.push_str(summary);
    }
    let mut metadata = Vec::new();
    if let Some(group) = entry.group() {
        metadata.push(format!("Group: {group}"));
    }
    if let Some(since) = entry.since() {
        metadata.push(format!("Since: {since}"));
    }
    if let Some(stability) = entry.stability() {
        metadata.push(format!("Stability: {stability}"));
    }
    if !metadata.is_empty() {
        push_section_break(&mut output);
        output.push_str(&metadata.join("  \n"));
    }
    let params = entry.params();
    if !params.is_empty() {
        push_section_break(&mut output);
        output.push_str("Parameters:\n");
        for param in params {
            output.push_str(&format!("- `{}`: {}\n", param.name, param.summary));
        }
    }
    if let Some(returns) = entry.returns() {
        push_section_break(&mut output);
        output.push_str("Returns: ");
        output.push_str(returns);
    }
    if let Some(markdown) = entry.markdown() {
        push_section_break(&mut output);
        output.push_str(markdown);
    }
    if let Some(example) = entry.example() {
        push_section_break(&mut output);
        output.push_str("Example:\n\n```scheme\n");
        output.push_str(example);
        output.push_str("\n```");
    }
    if let Some(deprecated) = entry.deprecated() {
        push_section_break(&mut output);
        output.push_str("Deprecated: ");
        output.push_str(deprecated);
    }
    let see = entry.see();
    if !see.is_empty() {
        push_section_break(&mut output);
        output.push_str("See also: ");
        output.push_str(
            &see.iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    output
}

pub fn completion_sort_text(entry: &impl ReferenceEntry, name: &str) -> String {
    match entry.group() {
        Some(group) => format!("{group}:{name}"),
        None => name.to_owned(),
    }
}

pub fn entry_matches_query(entry: &impl ReferenceEntry, name: &str, query: &str) -> bool {
    entry_field_matches(name, query)
        || entry
            .group()
            .is_some_and(|group| entry_field_matches(group, query))
        || entry
            .signature()
            .is_some_and(|signature| entry_field_matches(signature, query))
        || entry
            .summary()
            .is_some_and(|summary| entry_field_matches(summary, query))
}

pub fn signature_parameters(entry: &impl ReferenceEntry) -> Vec<SignatureParameter> {
    let Some(signature) = entry.signature() else {
        return Vec::new();
    };
    let params = entry.params();
    signature_parameter_names(signature)
        .skip(1)
        .map(|name| SignatureParameter {
            label: name.to_owned(),
            documentation: params
                .iter()
                .find(|param| param.name == clean_signature_parameter(name))
                .map(|param| param.summary.to_owned()),
        })
        .collect()
}

#[must_use]
pub fn snippet_for_signature(signature: &str) -> Option<String> {
    let signature = signature.trim();
    let inner = signature.strip_prefix('(')?.strip_suffix(')')?;
    let mut parts = inner.split_whitespace();
    let name = parts.next()?;
    let mut snippet = format!("({name}");
    let mut index = 1;
    for part in parts {
        if part == "..." {
            snippet.push_str(" ...");
        } else if part.starts_with('[') && part.ends_with(']') {
            let label = part.trim_matches(&['[', ']'][..]);
            snippet.push_str(&format!(" ${{{index}:{label}}}"));
            index += 1;
        } else {
            snippet.push_str(&format!(" ${{{index}:{part}}}"));
            index += 1;
        }
    }
    snippet.push(')');
    Some(snippet)
}

fn entry_field_matches(field: &str, query: &str) -> bool {
    field.to_lowercase().contains(query)
}

fn clean_signature_parameter(name: &str) -> &str {
    name.trim_matches(&['[', ']'][..])
}

fn signature_parameter_names(signature: &str) -> impl Iterator<Item = &str> {
    signature
        .trim_start_matches('(')
        .trim_end_matches(')')
        .split_whitespace()
}

fn push_section_break(output: &mut String) {
    if !output.is_empty() && !output.ends_with("\n\n") {
        output.push_str("\n\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Entry<'a> {
        signature: Option<&'a str>,
        summary: Option<&'a str>,
        group: Option<&'a str>,
        params: Vec<ReferenceParam<'a>>,
    }

    impl ReferenceEntry for Entry<'_> {
        fn signature(&self) -> Option<&str> {
            self.signature
        }

        fn summary(&self) -> Option<&str> {
            self.summary
        }

        fn group(&self) -> Option<&str> {
            self.group
        }

        fn since(&self) -> Option<&str> {
            Some("1.0")
        }

        fn stability(&self) -> Option<&str> {
            None
        }

        fn params(&self) -> Vec<ReferenceParam<'_>> {
            self.params
                .iter()
                .map(|param| ReferenceParam {
                    name: param.name,
                    summary: param.summary,
                })
                .collect()
        }

        fn returns(&self) -> Option<&str> {
            Some("A value.")
        }

        fn markdown(&self) -> Option<&str> {
            Some("Extra details.")
        }

        fn example(&self) -> Option<&str> {
            Some("(demo \"name\")")
        }

        fn deprecated(&self) -> Option<&str> {
            None
        }

        fn see(&self) -> Vec<&str> {
            vec!["tool"]
        }
    }

    #[test]
    fn renders_markdown_reference_entry() {
        let markdown = markdown_for_entry(&Entry {
            signature: Some("(demo name)"),
            summary: Some("Create a demo."),
            group: Some("Demo"),
            params: vec![ReferenceParam {
                name: "name",
                summary: "Demo name.",
            }],
        });

        assert!(markdown.contains("```scheme\n(demo name)\n```"));
        assert!(markdown.contains("Group: Demo"));
        assert!(markdown.contains("- `name`: Demo name."));
        assert!(markdown.contains("See also: `tool`"));
    }

    #[test]
    fn creates_snippets_from_signatures() {
        assert_eq!(
            snippet_for_signature("(tool name [action] field ...)").as_deref(),
            Some("(tool ${1:name} ${2:action} ${3:field} ...)")
        );
    }

    #[test]
    fn derives_signature_parameters_with_docs() {
        let params = signature_parameters(&Entry {
            signature: Some("(demo name [mode])"),
            summary: None,
            group: None,
            params: vec![ReferenceParam {
                name: "name",
                summary: "Demo name.",
            }],
        });

        assert_eq!(params[0].label, "name");
        assert_eq!(params[0].documentation.as_deref(), Some("Demo name."));
        assert_eq!(params[1].label, "[mode]");
        assert!(params[1].documentation.is_none());
    }

    #[test]
    fn matches_reference_entries_by_query_fields() {
        let entry = Entry {
            signature: Some("(demo name)"),
            summary: Some("Create a demo."),
            group: Some("Fixtures"),
            params: Vec::new(),
        };

        assert!(entry_matches_query(&entry, "demo", "fix"));
        assert!(entry_matches_query(&entry, "demo", "create"));
        assert!(entry_matches_query(&entry, "demo", "name"));
        assert_eq!(completion_sort_text(&entry, "demo"), "Fixtures:demo");
    }
}
