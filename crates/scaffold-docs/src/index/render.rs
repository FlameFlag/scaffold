use std::fmt::Write as _;

use scaffold_editor::reference::{markdown_code_span, markdown_table, markdown_text};

use super::{DocEntry, join_text};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryDocumentation<'a> {
    pub signature: Option<&'a str>,
    pub summary: Option<&'a str>,
    pub deprecated: Option<&'a str>,
    pub markdown: Option<&'a str>,
    pub params: &'a [super::DocParam],
    pub returns: Option<&'a str>,
    pub example: Option<&'a str>,
    pub details: Vec<EntryDetail>,
    pub see: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryDetail {
    pub field: &'static str,
    pub value: String,
    pub markdown_value: String,
}

impl EntryDocumentation<'_> {
    #[must_use]
    pub fn has_body(&self) -> bool {
        self.signature.is_some()
            || self.summary.is_some()
            || self.deprecated.is_some()
            || self.markdown.is_some()
            || !self.params.is_empty()
            || self.returns.is_some()
            || self.example.is_some()
    }
}

#[must_use]
pub fn entry_documentation(entry: &DocEntry) -> EntryDocumentation<'_> {
    let signature = trimmed(entry.signature.as_deref());
    let summary = trimmed(entry.summary.as_deref());
    let deprecated = trimmed(entry.deprecated.as_deref());
    let markdown = trimmed(entry.markdown.as_deref()).filter(|markdown| {
        summary.is_none_or(|summary| {
            !scaffold_editor::reference::same_markdown_paragraph(summary, markdown)
        })
    });
    let returns = trimmed(entry.returns.as_deref());
    let example = trimmed(entry.example.as_deref());

    EntryDocumentation {
        signature,
        summary,
        deprecated,
        markdown,
        params: &entry.params,
        returns,
        example,
        details: entry_details(entry),
        see: &entry.see,
    }
}

#[must_use]
pub fn snippet_for_signature(signature: &str) -> Option<String> {
    scaffold_editor::reference::snippet_for_signature(signature)
}

#[must_use]
pub fn markdown_for_entry(entry: &DocEntry) -> String {
    scaffold_editor::reference::markdown_for_entry(entry)
}

#[must_use]
pub fn rendered_markdown_for_entry(entry: &DocEntry) -> String {
    let markdown = markdown_for_entry(entry);
    if markdown.trim().is_empty() {
        "No documentation provided.".to_owned()
    } else {
        markdown
    }
}

#[must_use]
pub fn entry_count_label(count: usize) -> String {
    count_label(count, "entry", "entries")
}

#[must_use]
pub fn group_count_label(count: usize) -> String {
    count_label(count, "group", "groups")
}

fn count_label(count: usize, singular: &str, plural: &str) -> String {
    let noun = if count == 1 { singular } else { plural };
    format!("{count} {noun}")
}

pub fn group_markdown_table<'a, I, C>(groups: I) -> String
where
    I: IntoIterator<Item = (&'a str, C)>,
    C: ToString,
{
    markdown_table(
        &["Group", "Entries"],
        groups
            .into_iter()
            .map(|(group, count)| vec![markdown_code_span(group), count.to_string()]),
    )
}

pub fn entry_summary_markdown_table<'a>(entries: impl IntoIterator<Item = &'a DocEntry>) -> String {
    markdown_table(
        &["Symbol", "Group", "Summary"],
        entries.into_iter().map(|entry| {
            vec![
                markdown_code_span(&entry.name),
                markdown_text(entry.group_name()),
                entry.summary.as_deref().unwrap_or("No summary.").to_owned(),
            ]
        }),
    )
}

#[must_use]
pub fn detailed_markdown_for_entry(entry: &DocEntry) -> String {
    let documentation = entry_documentation(entry);
    let mut output = String::new();
    let _ = writeln!(&mut output, "## {}\n", markdown_code_span(&entry.name));

    if let Some(signature) = documentation.signature {
        output.push_str("```scheme\n");
        output.push_str(signature);
        output.push_str("\n```\n");
    }

    if let Some(summary) = documentation.summary {
        push_markdown_section_break(&mut output);
        output.push_str(summary);
        output.push('\n');
    }

    if let Some(deprecated) = documentation.deprecated {
        push_markdown_section_break(&mut output);
        let _ = writeln!(&mut output, "**Deprecated:** {deprecated}");
    }

    if let Some(markdown) = documentation.markdown {
        push_markdown_section_break(&mut output);
        output.push_str(markdown);
        output.push('\n');
    }

    if !documentation.params.is_empty() {
        push_markdown_section_break(&mut output);
        output.push_str("### Parameters\n\n");
        output.push_str(&markdown_table(
            &["Parameter", "Summary"],
            documentation
                .params
                .iter()
                .map(|param| vec![markdown_code_span(&param.name), param.summary.clone()]),
        ));
    }

    if let Some(returns) = documentation.returns {
        push_markdown_section_break(&mut output);
        output.push_str("### Returns\n\n");
        output.push_str(returns);
        output.push('\n');
    }

    if let Some(example) = documentation.example {
        push_markdown_section_break(&mut output);
        output.push_str("### Example\n\n```scheme\n");
        output.push_str(example);
        output.push_str("\n```\n");
    }

    if !documentation.has_body() {
        push_markdown_section_break(&mut output);
        output.push_str("No documentation provided.\n");
    }

    if !documentation.details.is_empty() {
        push_markdown_section_break(&mut output);
        output.push_str("### Details\n\n");
        output.push_str(&markdown_table(
            &["Field", "Value"],
            documentation
                .details
                .iter()
                .map(|detail| vec![detail.field.to_owned(), detail.markdown_value.clone()]),
        ));
    }

    if !documentation.see.is_empty() {
        push_markdown_section_break(&mut output);
        output.push_str("### See also\n\n");
        output.push_str(&join_text(
            documentation.see.iter().map(markdown_code_span),
            ", ",
        ));
        output.push('\n');
    }

    output
}

#[must_use]
pub fn titled_markdown_for_entry(entry: &DocEntry) -> String {
    let mut output = format!("## {}\n\n", markdown_code_span(&entry.name));
    output.push_str(rendered_markdown_for_entry(entry).trim());
    output.push_str("\n\n");

    if let Some(location) = entry.display_source_location() {
        output.push_str(&markdown_table(
            &["Field", "Value"],
            vec![vec!["Source".to_owned(), markdown_code_span(location)]],
        ));
    }

    output
}

#[must_use]
pub fn source_markdown_for_entry(entry: &DocEntry) -> Option<String> {
    let location = entry.display_source_location()?;
    let mut output = format!("## {} source\n\n", markdown_code_span(&entry.name));
    output.push_str(&markdown_table(
        &["Field", "Value"],
        std::iter::once(vec!["Source".to_owned(), markdown_code_span(location)]).chain(
            entry
                .signature
                .as_deref()
                .map(str::trim)
                .filter(|signature| !signature.is_empty())
                .map(|signature| vec!["Signature".to_owned(), markdown_code_span(signature)]),
        ),
    ));
    Some(output)
}

fn entry_details(entry: &DocEntry) -> Vec<EntryDetail> {
    let capability_detail = (!entry.requires_capability.is_empty()).then(|| EntryDetail {
        field: "requires capability",
        value: entry.requires_capability.join(", "),
        markdown_value: entry
            .requires_capability
            .iter()
            .map(markdown_code_span)
            .collect::<Vec<_>>()
            .join(", "),
    });

    std::iter::once(EntryDetail::code("group", entry.group_name()))
        .chain(
            [
                entry
                    .display_source_location()
                    .map(|location| EntryDetail::code("source", location)),
                entry
                    .since
                    .as_deref()
                    .map(|since| EntryDetail::code("since", since)),
                entry
                    .stability
                    .as_deref()
                    .map(|stability| EntryDetail::code("status", stability)),
                entry
                    .effect
                    .as_deref()
                    .map(|effect| EntryDetail::code("effect", effect)),
                capability_detail,
            ]
            .into_iter()
            .flatten(),
        )
        .collect()
}

impl EntryDetail {
    fn code(field: &'static str, value: impl Into<String>) -> Self {
        let value = value.into();
        Self {
            field,
            markdown_value: markdown_code_span(&value),
            value,
        }
    }
}

fn trimmed(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

fn push_markdown_section_break(output: &mut String) {
    if output.trim().is_empty() {
        return;
    }
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output.push('\n');
}
