use std::fmt::Write as _;

use crate::symbols::SymbolRange;
use crate::text::{clean_signature_parameter, signature_parameter_names};
use markdown_table_formatter::format_tables;

#[derive(Clone, Copy)]
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
    fn effect(&self) -> Option<&str> {
        None
    }
    fn requires_capability(&self) -> impl Iterator<Item = &str> {
        std::iter::empty()
    }
    fn params(&self) -> impl Iterator<Item = ReferenceParam<'_>>;
    fn returns(&self) -> Option<&str>;
    fn markdown(&self) -> Option<&str>;
    fn example(&self) -> Option<&str>;
    fn deprecated(&self) -> Option<&str>;
    fn see(&self) -> impl Iterator<Item = &str>;
}

pub trait ReferenceItem: ReferenceEntry {
    fn name(&self) -> &str;
    fn kind(&self) -> ReferenceKind;
    fn hidden(&self) -> bool;
    fn source(&self) -> Option<&str>;
    fn source_location(&self) -> Option<String> {
        None
    }
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
    let markdown = markdown_for_entry(
        entries
            .into_iter()
            .find(|entry| !entry.hidden() && entry.name() == symbol)?,
    );
    (!markdown.trim().is_empty()).then_some(markdown)
}

pub fn signature_help<'a, E>(
    entries: impl IntoIterator<Item = &'a E>,
    symbol: &str,
) -> Option<SignatureHelp>
where
    E: ReferenceItem + 'a,
{
    let entry = entries
        .into_iter()
        .find(|entry| !entry.hidden() && entry.name() == symbol)?;
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
    if entry.hidden() {
        return None;
    }
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
        .filter(|entry| query.is_empty() || reference_item_matches_query(*entry, &query))
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
    let summary = entry
        .summary()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(summary) = summary {
        output.push_str(summary);
    }
    let mut capabilities = entry.requires_capability().peekable();
    let capability_metadata = capabilities.peek().is_some().then(|| {
        vec![
            "Requires capability".to_owned(),
            join_text(capabilities.map(markdown_code_span), ", "),
        ]
    });
    let metadata = [
        entry
            .group()
            .map(|group| vec!["Group".to_owned(), group.to_owned()]),
        entry
            .since()
            .map(|since| vec!["Since".to_owned(), since.to_owned()]),
        entry
            .stability()
            .map(|stability| vec!["Stability".to_owned(), stability.to_owned()]),
        entry
            .effect()
            .map(|effect| vec!["Effect".to_owned(), effect.to_owned()]),
        capability_metadata,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    if !metadata.is_empty() {
        push_section_break(&mut output);
        output.push_str(&markdown_table(&["Field", "Value"], metadata));
    }
    let mut params = entry.params().peekable();
    if params.peek().is_some() {
        push_section_break(&mut output);
        output.push_str("**Parameters**\n\n");
        output.push_str(&markdown_table(
            &["Parameter", "Description"],
            params.map(|param| vec![markdown_code_span(param.name), param.summary.to_owned()]),
        ));
    }
    if let Some(returns) = entry.returns() {
        push_section_break(&mut output);
        output.push_str("**Returns:** ");
        output.push_str(returns);
    }
    let markdown = entry.markdown().map(str::trim).filter(|value| {
        !value.is_empty() && summary.is_none_or(|summary| !same_markdown_paragraph(summary, value))
    });
    if let Some(markdown) = markdown {
        push_section_break(&mut output);
        output.push_str(markdown);
    }
    if let Some(example) = entry.example() {
        push_section_break(&mut output);
        output.push_str("**Example**\n\n```scheme\n");
        output.push_str(example);
        output.push_str("\n```");
    }
    if let Some(deprecated) = entry.deprecated() {
        push_section_break(&mut output);
        output.push_str("**Deprecated:** ");
        output.push_str(deprecated);
    }
    let mut see = entry.see().peekable();
    if see.peek().is_some() {
        push_section_break(&mut output);
        output.push_str("**See also:** ");
        output.push_str(&join_text(see.map(markdown_code_span), ", "));
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
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }
    let query = query.as_str();

    entry_field_matches(name, query)
        || entry
            .group()
            .is_some_and(|group| entry_field_matches(group, query))
        || entry
            .since()
            .is_some_and(|since| entry_field_matches(since, query))
        || entry
            .stability()
            .is_some_and(|stability| entry_field_matches(stability, query))
        || entry
            .effect()
            .is_some_and(|effect| entry_field_matches(effect, query))
        || entry
            .requires_capability()
            .any(|capability| entry_field_matches(capability, query))
        || entry
            .signature()
            .is_some_and(|signature| entry_field_matches(signature, query))
        || entry
            .summary()
            .is_some_and(|summary| entry_field_matches(summary, query))
        || entry.params().any(|param| {
            entry_field_matches(param.name, query) || entry_field_matches(param.summary, query)
        })
        || entry
            .returns()
            .is_some_and(|returns| entry_field_matches(returns, query))
        || entry
            .markdown()
            .is_some_and(|markdown| entry_field_matches(markdown, query))
        || entry
            .example()
            .is_some_and(|example| entry_field_matches(example, query))
        || entry
            .deprecated()
            .is_some_and(|deprecated| entry_field_matches(deprecated, query))
        || entry.see().any(|name| entry_field_matches(name, query))
}

fn reference_item_matches_query(entry: &impl ReferenceItem, query: &str) -> bool {
    entry_matches_query(entry, entry.name(), query)
        || entry
            .source()
            .is_some_and(|source| entry_field_matches(source, query))
}

pub fn signature_parameters(entry: &impl ReferenceEntry) -> Vec<SignatureParameter> {
    let Some(signature) = entry.signature() else {
        return Vec::new();
    };
    let params = entry.params().collect::<Vec<_>>();
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
            let _ = write!(&mut snippet, " ${{{index}:{label}}}");
            index += 1;
        } else {
            let _ = write!(&mut snippet, " ${{{index}:{part}}}");
            index += 1;
        }
    }
    snippet.push(')');
    Some(snippet)
}

fn entry_field_matches(field: &str, query: &str) -> bool {
    field.to_lowercase().contains(query)
}

#[must_use]
pub fn markdown_code_span(value: impl AsRef<str>) -> String {
    let value = value.as_ref();
    let delimiter = "`".repeat(max_consecutive_backticks(value) + 1);
    if value.contains('`') || value.starts_with(' ') || value.ends_with(' ') {
        format!("{delimiter} {value} {delimiter}")
    } else {
        format!("{delimiter}{value}{delimiter}")
    }
}

fn max_consecutive_backticks(value: &str) -> usize {
    value.split(|ch| ch != '`').map(str::len).max().unwrap_or(0)
}

#[must_use]
pub fn markdown_text(value: impl AsRef<str>) -> String {
    let normalized = value
        .as_ref()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\n', " ");
    let mut output = String::with_capacity(normalized.len());
    for ch in normalized.chars() {
        if matches!(
            ch,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '<'
                | '>'
                | '('
                | ')'
                | '#'
                | '+'
                | '-'
                | '.'
                | '!'
        ) {
            output.push('\\');
        }
        output.push(ch);
    }
    output
}

fn push_section_break(output: &mut String) {
    if !output.is_empty() {
        while output.ends_with('\n') {
            output.pop();
        }
        output.push_str("\n\n");
    }
}

#[must_use]
pub fn same_markdown_paragraph(left: &str, right: &str) -> bool {
    normalize_markdown_paragraph(left) == normalize_markdown_paragraph(right)
}

fn normalize_markdown_paragraph(value: &str) -> String {
    join_text(value.split_whitespace(), " ")
}

#[must_use]
pub fn markdown_table(headers: &[&str], rows: impl IntoIterator<Item = Vec<String>>) -> String {
    assert!(
        !headers.is_empty(),
        "markdown tables must have at least one column"
    );

    let mut output = markdown_table_row(headers.iter().copied());
    output.push_str(&markdown_table_row((0..headers.len()).map(|_| "---")));

    for row in rows {
        assert_eq!(
            row.len(),
            headers.len(),
            "markdown table row width must match header width"
        );
        output.push_str(&markdown_table_row(row));
    }
    format_tables(output)
}

#[must_use]
pub fn format_markdown_tables(markdown: String) -> String {
    format_tables(markdown)
}

fn markdown_table_row(cells: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    format!(
        "| {} |\n",
        join_text(cells.into_iter().map(markdown_table_cell), " | ")
    )
}

fn join_text(items: impl IntoIterator<Item = impl AsRef<str>>, separator: &str) -> String {
    let mut output = String::new();
    let mut first = true;
    for item in items {
        if first {
            first = false;
        } else {
            output.push_str(separator);
        }
        output.push_str(item.as_ref());
    }
    output
}

fn markdown_table_cell(value: impl AsRef<str>) -> String {
    value
        .as_ref()
        .trim()
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('|', "\\|")
        .replace('\n', "<br>")
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

        fn effect(&self) -> Option<&str> {
            Some("pure")
        }

        fn requires_capability(&self) -> impl Iterator<Item = &str> {
            std::iter::once("scaffold.demo")
        }

        fn params(&self) -> impl Iterator<Item = ReferenceParam<'_>> {
            self.params.iter().map(|param| ReferenceParam {
                name: param.name,
                summary: param.summary,
            })
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

        fn see(&self) -> impl Iterator<Item = &str> {
            std::iter::once("tool")
        }
    }

    struct Item<'a> {
        name: &'a str,
        source: Option<&'a str>,
        range: Option<SymbolRange>,
        hidden: bool,
        entry: Entry<'a>,
    }

    impl ReferenceEntry for Item<'_> {
        fn signature(&self) -> Option<&str> {
            self.entry.signature()
        }

        fn summary(&self) -> Option<&str> {
            self.entry.summary()
        }

        fn group(&self) -> Option<&str> {
            self.entry.group()
        }

        fn since(&self) -> Option<&str> {
            self.entry.since()
        }

        fn stability(&self) -> Option<&str> {
            self.entry.stability()
        }

        fn effect(&self) -> Option<&str> {
            self.entry.effect()
        }

        fn requires_capability(&self) -> impl Iterator<Item = &str> {
            self.entry.requires_capability()
        }

        fn params(&self) -> impl Iterator<Item = ReferenceParam<'_>> {
            self.entry.params()
        }

        fn returns(&self) -> Option<&str> {
            self.entry.returns()
        }

        fn markdown(&self) -> Option<&str> {
            self.entry.markdown()
        }

        fn example(&self) -> Option<&str> {
            self.entry.example()
        }

        fn deprecated(&self) -> Option<&str> {
            self.entry.deprecated()
        }

        fn see(&self) -> impl Iterator<Item = &str> {
            self.entry.see()
        }
    }

    impl ReferenceItem for Item<'_> {
        fn name(&self) -> &str {
            self.name
        }

        fn kind(&self) -> ReferenceKind {
            ReferenceKind::Function
        }

        fn hidden(&self) -> bool {
            self.hidden
        }

        fn source(&self) -> Option<&str> {
            self.source
        }

        fn range(&self) -> Option<SymbolRange> {
            self.range.clone()
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
                summary: "Demo | name.\nSecond line.",
            }],
        });

        assert!(markdown.contains("```scheme\n(demo name)\n```"));
        assert!(markdown.contains("| Field"));
        assert!(markdown.contains("| Value"));
        assert!(markdown.contains("Group"));
        assert!(markdown.contains("Demo"));
        assert!(markdown.contains("Effect"));
        assert!(markdown.contains("pure"));
        assert!(markdown.contains("Requires capability"));
        assert!(markdown.contains("`scaffold.demo`"));
        assert!(markdown.contains("**Parameters**"));
        assert!(!markdown.contains("\n\n\n**Parameters**"));
        assert!(markdown.contains("| Parameter | Description"));
        assert!(markdown.contains("| `name`    | Demo \\| name.<br>Second line. |"));
        assert!(!markdown.contains("- `name`:"));
        assert!(markdown.contains("**Returns:** A value."));
        assert!(!markdown.contains("Returns: A value."));
        assert!(markdown.contains("**Example**"));
        assert!(markdown.contains("```scheme\n(demo \"name\")\n```"));
        assert!(markdown.contains("**See also:** `tool`"));
        assert!(!markdown.contains("See also: `tool`"));
    }

    #[test]
    fn omits_markdown_when_it_repeats_the_summary() {
        let markdown = markdown_for_entry(&Entry {
            signature: Some("(demo name)"),
            summary: Some("Extra details."),
            group: Some("Demo"),
            params: Vec::new(),
        });

        assert_eq!(markdown.matches("Extra details.").count(), 1);
    }

    #[test]
    fn markdown_table_escapes_cells_and_formats_columns() {
        let markdown = markdown_table(
            &["Name", "Description"],
            vec![vec![
                "tool|path".to_owned(),
                "first line\r\nsecond line".to_owned(),
            ]],
        );

        assert!(markdown.contains("| Name       | Description"));
        assert!(markdown.contains("| tool\\|path | first line<br>second line |"));
    }

    #[test]
    fn markdown_code_spans_use_safe_delimiters() {
        assert_eq!(markdown_code_span("catalog/tool"), "`catalog/tool`");
        assert_eq!(markdown_code_span("bad`query"), "`` bad`query ``");
        assert_eq!(markdown_code_span("bad``query"), "``` bad``query ```");
        assert_eq!(markdown_code_span(" padded "), "`  padded  `");
    }

    #[test]
    fn markdown_text_escapes_inline_markup() {
        assert_eq!(
            markdown_text("Bad [Group] | Plus+"),
            "Bad \\[Group\\] | Plus\\+"
        );
        assert_eq!(markdown_text("line\r\nbreak"), "line break");
    }

    #[test]
    #[should_panic(expected = "markdown table row width must match header width")]
    fn markdown_table_rejects_mismatched_row_widths() {
        drop(markdown_table(
            &["Name", "Description"],
            vec![vec!["tool".to_owned()]],
        ));
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
            params: vec![ReferenceParam {
                name: "name",
                summary: "Demo name.",
            }],
        };

        assert!(entry_matches_query(&entry, "demo", "fix"));
        assert!(entry_matches_query(&entry, "demo", "1.0"));
        assert!(entry_matches_query(&entry, "demo", "pure"));
        assert!(entry_matches_query(&entry, "demo", "scaffold.demo"));
        assert!(entry_matches_query(&entry, "demo", "create"));
        assert!(entry_matches_query(&entry, "demo", "value"));
        assert!(entry_matches_query(&entry, "demo", "extra details"));
        assert!(entry_matches_query(&entry, "demo", "tool"));
        assert!(entry_matches_query(&entry, "demo", "name"));
        assert_eq!(completion_sort_text(&entry, "demo"), "Fixtures:demo");
    }

    #[test]
    fn workspace_symbols_match_metadata_and_source_paths() {
        let item = Item {
            name: "demo",
            source: Some("src/dsl/std/workspace.scm"),
            range: Some(SymbolRange {
                line: 2,
                start: 4,
                length: 6,
            }),
            hidden: false,
            entry: Entry {
                signature: Some("(demo name)"),
                summary: Some("Create a demo."),
                group: Some("Fixtures"),
                params: Vec::new(),
            },
        };

        assert_eq!(workspace_symbols([&item], "scaffold.demo")[0].name, "demo");
        assert_eq!(workspace_symbols([&item], "workspace.scm")[0].name, "demo");
    }

    #[test]
    fn hidden_items_are_omitted_from_public_reference_helpers() {
        let hidden = Item {
            name: "internal-demo",
            source: Some("src/internal.scm"),
            range: Some(SymbolRange {
                line: 2,
                start: 4,
                length: 13,
            }),
            hidden: true,
            entry: Entry {
                signature: Some("(internal-demo name)"),
                summary: Some("Internal helper."),
                group: Some("Internal"),
                params: Vec::new(),
            },
        };

        assert!(hover_markdown([&hidden], "internal-demo").is_none());
        assert!(signature_help([&hidden], "internal-demo").is_none());
        assert!(definition_location(&hidden, "file:///fallback.scm").is_none());
        assert!(workspace_symbols([&hidden], "internal").is_empty());
        assert!(document_symbols([&hidden]).is_empty());
        assert!(completion_items([&hidden]).is_empty());
    }
}
