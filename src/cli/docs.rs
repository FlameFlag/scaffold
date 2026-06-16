use std::collections::BTreeMap;

use comfy_table::{
    Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL_CONDENSED,
};
use nucleo_matcher::{
    Config as FuzzyConfig, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};

use scaffold_docs::{DocEntry, DocIndex};

use super::{
    CliError,
    args::{DocsArgs, DocsFormat},
};

pub(super) fn render_docs(args: &DocsArgs) -> Result<String, CliError> {
    let query = args.query.join(" ");
    if args.all || args.format.is_some() || args.output.is_some() {
        if !query.trim().is_empty() || args.group.is_some() || args.source.is_some() {
            return Err(CliError::message(
                "--all, --format, and --output export the full reference and cannot be combined \
                 with a docs query, --group, or --source",
            ));
        }
        return render_generated_reference(args.format.unwrap_or(DocsFormat::Markdown));
    }

    let docs = DocIndex::scaffold();

    if let Some(name) = args.source.as_deref() {
        return Ok(render_doc_source(&docs, name));
    }

    if let Some(group) = args.group.as_deref() {
        return Ok(render_doc_group(&docs, group));
    }

    if query.trim().is_empty() {
        return Ok(render_doc_overview(&docs));
    }

    if let Some(entry) = docs.get(query.trim()) {
        return Ok(render_doc_entry(entry));
    }

    Ok(render_doc_search(&docs, query.trim(), args.limit))
}

fn render_generated_reference(format: DocsFormat) -> Result<String, CliError> {
    match format {
        DocsFormat::Markdown => Ok(scaffold_docs::scaffold_reference_markdown()),
        DocsFormat::Json => Ok(scaffold_docs::scaffold_reference_json()?),
    }
}

fn render_doc_overview(docs: &DocIndex) -> String {
    let mut groups = BTreeMap::<&str, usize>::new();
    for entry in docs.visible_entries() {
        *groups.entry(doc_entry_group(entry)).or_default() += 1;
    }

    let mut output = String::from("Scaffold Docs\n\n");
    output.push_str(&format!(
        "{} across {}.\n\n",
        entry_count_label(groups.values().sum()),
        group_count_label(groups.len())
    ));
    output.push_str(&render_group_table(&groups));
    output.push_str("\nTry:\n");
    output.push_str("  scaffold docs tool\n");
    output.push_str("  scaffold docs \"ctlg tool\"\n");
    output.push_str("  scaffold docs --group Catalog\n");
    output.push_str("  scaffold docs --source tool\n");
    output.push_str("  scaffold docs --all > reference.md\n");
    output.push_str("  scaffold docs --format json --output reference.json\n");
    output
}

fn render_doc_group(docs: &DocIndex, group: &str) -> String {
    let mut entries = docs
        .visible_entries()
        .filter(|entry| doc_entry_group(entry).eq_ignore_ascii_case(group))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.name.cmp(&right.name));

    if entries.is_empty() {
        return format!(
            "No documentation group named `{group}`.\n\n{}",
            render_doc_overview(docs)
        );
    }

    render_entry_table(
        &format!(
            "{} docs",
            entries[0].group.as_deref().unwrap_or_else(|| group.trim())
        ),
        &entries,
    )
}

fn render_doc_search(docs: &DocIndex, query: &str, limit: usize) -> String {
    let matches = doc_search_results(docs, query, limit);
    if matches.is_empty() {
        return format!("No docs matched `{query}`.\n");
    }
    render_entry_table(&format!("Search results for `{query}`"), &matches)
}

fn render_doc_source(docs: &DocIndex, name: &str) -> String {
    let Some(entry) = docs.get(name.trim()) else {
        return format!(
            "No docs for `{name}`.\n\n{}",
            render_doc_search(docs, name, 10)
        );
    };

    match &entry.source {
        Some(source) => {
            let mut output = format!("{}\n", source_location(source, entry));
            if let Some(signature) = entry.signature.as_deref() {
                output.push_str(&format!("{signature}\n"));
            }
            output
        }
        None => format!("No source recorded for `{}`.\n", entry.name),
    }
}

fn render_doc_entry(entry: &DocEntry) -> String {
    let mut output = String::new();
    output.push_str(&format!("{}\n", entry.name));
    output.push_str(&format!("{}\n", "=".repeat(entry.name.len())));

    if let Some(signature) = entry
        .signature
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        output.push_str(&format!("\n{signature}\n"));
    }

    let summary = entry
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if let Some(summary) = summary {
        push_section_break(&mut output);
        output.push_str(summary);
        output.push('\n');
    }

    if let Some(deprecated) = entry
        .deprecated
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        push_section_break(&mut output);
        output.push_str(&format!("Deprecated: {deprecated}\n"));
    }

    let markdown = entry.markdown.as_deref().map(str::trim).filter(|value| {
        !value.is_empty() && summary.is_none_or(|summary| !same_markdown_paragraph(summary, value))
    });
    if let Some(markdown) = markdown {
        push_section_break(&mut output);
        output.push_str(markdown);
        output.push('\n');
    }

    if !entry.params.is_empty() {
        push_section_break(&mut output);
        output.push_str("Parameters\n");
        for param in &entry.params {
            output.push_str(&format!("  {}  {}\n", param.name, param.summary));
        }
    }

    if let Some(returns) = entry
        .returns
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        push_section_break(&mut output);
        output.push_str("Returns\n");
        output.push_str(returns);
        output.push('\n');
    }

    if let Some(example) = entry
        .example
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        push_section_break(&mut output);
        output.push_str("Example\n");
        output.push_str(example);
        output.push('\n');
    }

    push_section_break(&mut output);
    output.push_str("Details\n");
    output.push_str(&format!("  group   {}\n", doc_entry_group(entry)));
    if let Some(source) = &entry.source {
        output.push_str(&format!("  source  {}\n", source_location(source, entry)));
    }
    if let Some(since) = &entry.since {
        output.push_str(&format!("  since   {since}\n"));
    }
    if let Some(stability) = &entry.stability {
        output.push_str(&format!("  status  {stability}\n"));
    }

    if !entry.see.is_empty() {
        push_section_break(&mut output);
        output.push_str("See also\n");
        output.push_str(&format!("  {}\n", entry.see.join(", ")));
    }

    output
}

fn render_group_table(groups: &BTreeMap<&str, usize>) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![header_cell("group"), header_cell("entries")]);

    for (group, count) in groups {
        table.add_row(vec![Cell::new(group), Cell::new(count)]);
    }

    format!("{}\n", table.trim_fmt())
}

fn render_entry_table(title: &str, entries: &[&DocEntry]) -> String {
    let mut output = format!("{title}\n\n{}.\n\n", entry_count_label(entries.len()));
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            header_cell("symbol"),
            header_cell("group"),
            header_cell("signature"),
            header_cell("summary"),
        ]);

    for entry in entries {
        table.add_row(vec![
            Cell::new(&entry.name).fg(Color::Green),
            Cell::new(doc_entry_group(entry)),
            Cell::new(entry.signature.as_deref().unwrap_or(&entry.name)),
            Cell::new(entry.summary.as_deref().unwrap_or("No summary.")),
        ]);
    }

    output.push_str(&format!("{}\n", table.trim_fmt()));
    output
}

fn header_cell(label: &str) -> Cell {
    Cell::new(label).add_attribute(Attribute::Bold)
}

pub(crate) fn doc_search_results<'a>(
    docs: &'a DocIndex,
    query: &str,
    limit: usize,
) -> Vec<&'a DocEntry> {
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut matcher = Matcher::new(FuzzyConfig::DEFAULT);
    let mut matches = docs
        .visible_entries()
        .filter(|entry| doc_entry_is_plausible_fuzzy_match(entry, query))
        .filter_map(|entry| {
            doc_entry_search_score(entry, &pattern, &mut matcher).map(|score| (entry, score))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|(left_entry, left_score), (right_entry, right_score)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_entry.name.cmp(&right_entry.name))
    });
    matches
        .into_iter()
        .take(limit)
        .map(|(entry, _score)| entry)
        .collect()
}

fn doc_entry_search_score(
    entry: &DocEntry,
    pattern: &Pattern,
    matcher: &mut Matcher,
) -> Option<u32> {
    let mut buf = Vec::new();
    let searchable = doc_entry_search_text(entry);
    let mut score = pattern.score(Utf32Str::new(&searchable, &mut buf), matcher)?;

    if let Some(name_score) = pattern.score(Utf32Str::new(&entry.name, &mut buf), matcher) {
        score += name_score * 8;
    }
    if let Some(signature) = &entry.signature
        && let Some(signature_score) = pattern.score(Utf32Str::new(signature, &mut buf), matcher)
    {
        score += signature_score * 4;
    }
    if let Some(summary) = &entry.summary
        && let Some(summary_score) = pattern.score(Utf32Str::new(summary, &mut buf), matcher)
    {
        score += summary_score * 2;
    }

    Some(score)
}

fn doc_entry_is_plausible_fuzzy_match(entry: &DocEntry, query: &str) -> bool {
    let tokens = query
        .split_whitespace()
        .map(normalize_search_token)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return false;
    }

    let symbolic_fields = doc_entry_symbolic_search_fields(entry)
        .into_iter()
        .map(normalize_search_token)
        .collect::<Vec<_>>();
    let prose_fields = doc_entry_prose_search_fields(entry)
        .into_iter()
        .map(normalize_search_token)
        .collect::<Vec<_>>();

    tokens.iter().all(|token| {
        symbolic_fields
            .iter()
            .any(|field| is_subsequence(token, field))
            || prose_fields.iter().any(|field| field.contains(token))
    })
}

fn doc_entry_search_text(entry: &DocEntry) -> String {
    let mut fields = doc_entry_symbolic_search_fields(entry);
    fields.extend(doc_entry_prose_search_fields(entry));
    fields.join(" ")
}

fn doc_entry_symbolic_search_fields(entry: &DocEntry) -> Vec<&str> {
    let mut parts = vec![
        entry.name.as_str(),
        doc_entry_group(entry),
        entry.signature.as_deref().unwrap_or_default(),
        entry.source.as_deref().unwrap_or_default(),
    ];
    parts.extend(entry.params.iter().map(|param| param.name.as_str()));
    parts.extend(entry.see.iter().map(String::as_str));
    parts
}

fn doc_entry_prose_search_fields(entry: &DocEntry) -> Vec<&str> {
    let mut parts = vec![
        entry.summary.as_deref().unwrap_or_default(),
        entry.markdown.as_deref().unwrap_or_default(),
        entry.returns.as_deref().unwrap_or_default(),
        entry.deprecated.as_deref().unwrap_or_default(),
    ];
    parts.extend(entry.params.iter().map(|param| param.summary.as_str()));
    parts
}

fn normalize_search_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    needle
        .chars()
        .all(|needle_ch| chars.any(|ch| ch == needle_ch))
}

pub(crate) fn doc_entry_group(entry: &DocEntry) -> &str {
    entry.group.as_deref().unwrap_or("Language")
}

pub(crate) fn source_location(source: &str, entry: &DocEntry) -> String {
    match entry.range {
        Some(range) => format!("{source}:{}", range.start.line + 1),
        None => source.to_owned(),
    }
}

pub(crate) fn entry_count_label(count: usize) -> String {
    match count {
        1 => "1 entry".to_owned(),
        count => format!("{count} entries"),
    }
}

fn group_count_label(count: usize) -> String {
    match count {
        1 => "1 group".to_owned(),
        count => format!("{count} groups"),
    }
}

fn push_section_break(output: &mut String) {
    if !output.is_empty() && !output.ends_with("\n\n") {
        if output.ends_with('\n') {
            output.push('\n');
        } else {
            output.push_str("\n\n");
        }
    }
}

pub(crate) fn same_markdown_paragraph(left: &str, right: &str) -> bool {
    normalize_markdown_paragraph(left) == normalize_markdown_paragraph(right)
}

fn normalize_markdown_paragraph(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn docs_args() -> DocsArgs {
        DocsArgs {
            query: Vec::new(),
            all: false,
            group: None,
            source: None,
            limit: 20,
            output: None,
            format: None,
        }
    }

    #[test]
    fn docs_default_renders_group_overview_not_full_reference() {
        let rendered = render_docs(&docs_args()).expect("render docs");

        assert!(rendered.starts_with("Scaffold Docs"));
        assert!(rendered.contains("scaffold docs tool"));
        assert!(rendered.contains("Catalog"));
        assert!(!rendered.contains("# Scaffold Scheme Reference"));
    }

    #[test]
    fn docs_query_renders_exact_entry() {
        let mut args = docs_args();
        args.query = vec!["tool".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("tool\n===="));
        assert!(rendered.contains("(tool name action field ...)"));
        assert!(rendered.contains("Create a catalog tool object."));
    }

    #[test]
    fn docs_query_fuzzy_searches_when_exact_entry_is_missing() {
        let mut args = docs_args();
        args.query = vec!["ctlg".to_owned(), "tool".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.contains("Search results for `ctlg tool`"));
        assert!(rendered.contains("catalog/tool"));
    }

    #[test]
    fn docs_query_rejects_unrelated_fuzzy_noise() {
        let mut args = docs_args();
        args.query = vec!["zzzzzzz".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No docs matched `zzzzzzz`.\n");

        args.query = vec!["no-such-query".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No docs matched `no-such-query`.\n");
    }

    #[test]
    fn docs_group_lists_entries_in_one_group() {
        let mut args = docs_args();
        args.group = Some("Catalog".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("Catalog docs"));
        assert!(rendered.contains("tool"));
        assert!(!rendered.contains("Scaffold Docs"));
    }

    #[test]
    fn docs_source_shows_recorded_location() {
        let mut args = docs_args();
        args.source = Some("tool".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.contains("src/dsl/std/catalog/tool.scm"));
        assert!(rendered.contains("(tool name action field ...)"));
    }

    #[test]
    fn docs_format_keeps_generated_reference_output() {
        let mut args = docs_args();
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("# Scaffold Scheme Reference"));
    }

    #[test]
    fn docs_export_options_reject_browse_selectors() {
        let mut args = docs_args();
        args.output = Some("reference.md".into());
        args.query = vec!["tool".to_owned()];

        let err = render_docs(&args).expect_err("export with query should fail");

        assert!(err.to_string().contains("cannot be combined"));
    }
}
