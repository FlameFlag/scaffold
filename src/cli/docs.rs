use std::{collections::BTreeMap, path::Path};

use comfy_table::{Cell, Color};
use scaffold_docs::{
    DocEntry, DocIndex, DocParam, entry_count_label, entry_documentation,
    entry_summary_markdown_table, group_count_label, group_markdown_table, markdown_code_span,
    markdown_text, reference_entry_json, search_doc_entries, source_markdown_for_entry,
    suggest_doc_entries, titled_markdown_for_entry,
};
use serde_json::{Value, json};

use super::{
    CliError,
    args::{DocsArgs, DocsFormat},
    table::{header_cell, output_table},
};

const DOC_TABLE_WIDTH: u16 = 100;
const DEFAULT_SEARCH_LIMIT: usize = 20;
const MAX_SEARCH_LIMIT: usize = 100;
const DOCS_TRY_EXAMPLES: &[&str] = &[
    "scaffold docs tool",
    "scaffold docs --search \"ctlg tool\"",
    "scaffold docs --group Catalog",
    "scaffold docs --source tool",
    "scaffold docs --source src/dsl/std/catalog/tool.scm",
    "scaffold docs --all",
    "scaffold docs --output reference.md",
    "scaffold docs --output reference.json",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DocsBrowseFormat {
    Text,
    Markdown,
    Json,
}

pub(super) fn render_docs(args: &DocsArgs) -> Result<String, CliError> {
    let raw_query = args.query.join(" ");
    let query = raw_query.trim();
    if args.all || args.output.is_some() {
        if !args.query.is_empty()
            || args.search.is_some()
            || args.group.is_some()
            || args.source.is_some()
        {
            return Err(CliError::message(
                "--all and --output export the full reference and cannot be combined with a docs \
                 query, --search, --group, or --source",
            ));
        }
        if args.limit.is_some() {
            return Err(CliError::message(
                "--limit only applies to reference search; full reference exports ignore it",
            ));
        }
        return render_generated_reference(docs_export_format(args)?);
    }

    let docs = DocIndex::scaffold();
    if !args.query.is_empty() && query.is_empty() {
        return Err(CliError::message("docs query cannot be empty"));
    }
    ensure_single_browse_selector(args)?;
    ensure_limit_applies_to_search(args)?;
    let output_format = docs_browse_format(args);

    if let Some(search) = args.search.as_deref() {
        return render_doc_search(
            &docs,
            trim_docs_selector("--search", search)?,
            docs_search_limit(args)?,
            output_format,
        );
    }

    if let Some(name) = args.source.as_deref() {
        return render_doc_source(&docs, trim_docs_selector("--source", name)?, output_format);
    }

    if let Some(group) = args.group.as_deref() {
        return render_doc_group(&docs, trim_docs_selector("--group", group)?, output_format);
    }

    if query.is_empty() {
        return render_doc_overview(&docs, output_format);
    }

    if let Some(entry) = get_doc_entry(&docs, query) {
        return render_doc_entry(entry, query, output_format);
    }

    render_doc_search(&docs, query, docs_search_limit(args)?, output_format)
}

fn render_generated_reference(format: DocsFormat) -> Result<String, CliError> {
    match format {
        DocsFormat::Markdown => Ok(scaffold_docs::scaffold_reference_markdown()),
        DocsFormat::Json => Ok(scaffold_docs::scaffold_reference_json()?),
    }
}

fn docs_browse_format(args: &DocsArgs) -> DocsBrowseFormat {
    match args.format {
        Some(DocsFormat::Markdown) => DocsBrowseFormat::Markdown,
        Some(DocsFormat::Json) => DocsBrowseFormat::Json,
        None => DocsBrowseFormat::Text,
    }
}

fn ensure_single_browse_selector(args: &DocsArgs) -> Result<(), CliError> {
    let selected = [
        !args.query.is_empty(),
        args.search.is_some(),
        args.group.is_some(),
        args.source.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();

    if selected > 1 {
        return Err(CliError::message(
            "docs browse selectors cannot be combined; use one of a query, --search, --group, or \
             --source",
        ));
    }

    Ok(())
}

fn ensure_limit_applies_to_search(args: &DocsArgs) -> Result<(), CliError> {
    if args.limit.is_some() && (args.group.is_some() || args.source.is_some()) {
        return Err(CliError::message(
            "--limit only applies to reference search; it cannot be combined with --group or --source",
        ));
    }
    if args.limit.is_some() && args.search.is_none() && args.query.is_empty() {
        return Err(CliError::message(
            "--limit only applies to reference search; use a query or --search",
        ));
    }
    Ok(())
}

fn docs_search_limit(args: &DocsArgs) -> Result<usize, CliError> {
    let limit = args.limit.unwrap_or(DEFAULT_SEARCH_LIMIT);
    if limit == 0 || limit > MAX_SEARCH_LIMIT {
        return Err(CliError::message(format!(
            "--limit must be between 1 and {MAX_SEARCH_LIMIT}"
        )));
    }
    Ok(limit)
}

fn trim_docs_selector<'a>(selector: &str, value: &'a str) -> Result<&'a str, CliError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(CliError::message(format!("{selector} cannot be empty")));
    }
    Ok(value)
}

fn render_doc_overview(docs: &DocIndex, format: DocsBrowseFormat) -> Result<String, CliError> {
    let groups = doc_group_counts(docs);

    match format {
        DocsBrowseFormat::Json => {
            return Ok(to_pretty_json(&json!({
                "mode": "overview",
                "entry_count": groups.values().sum::<usize>(),
                "group_count": groups.len(),
                "groups": groups
                    .iter()
                    .map(|(group, count)| json!({
                        "name": group,
                        "entries": count,
                    }))
                    .collect::<Vec<_>>(),
            }))?);
        }
        DocsBrowseFormat::Markdown => {
            let mut output = String::from("# Scaffold Docs\n\n");
            output.push_str(&format!(
                "{} across {}.\n\n",
                entry_count_label(groups.values().sum()),
                group_count_label(groups.len())
            ));
            output.push_str(&group_markdown_table(
                groups.iter().map(|(group, count)| (*group, *count)),
            ));
            push_try_examples_markdown(&mut output);
            return Ok(output);
        }
        DocsBrowseFormat::Text => {}
    }

    let mut output = String::from("Scaffold Docs\n\n");
    output.push_str(&format!(
        "{} across {}.\n\n",
        entry_count_label(groups.values().sum()),
        group_count_label(groups.len())
    ));
    output.push_str(&render_group_table(&groups));
    push_try_examples_text(&mut output);
    Ok(output)
}

fn push_try_examples_markdown(output: &mut String) {
    output.push_str("\n## Try\n\n");
    for example in DOCS_TRY_EXAMPLES {
        output.push_str(&markdown_try_command(example));
    }
}

fn markdown_try_command(command: impl AsRef<str>) -> String {
    format!("- {}\n", markdown_code_span(command))
}

fn push_try_examples_text(output: &mut String) {
    output.push_str("\nTry:\n");
    for example in DOCS_TRY_EXAMPLES {
        output.push_str(&format!("  {example}\n"));
    }
}

fn docs_export_format(args: &DocsArgs) -> Result<DocsFormat, CliError> {
    if let Some(format) = args.format {
        return Ok(format);
    }

    let Some(output) = args.output.as_deref() else {
        return Ok(DocsFormat::Markdown);
    };

    docs_format_from_path(output).ok_or_else(|| {
        CliError::message(format!(
            "cannot infer docs format from output path `{}`; use .md, .markdown, .json, or pass \
             --format",
            output.display()
        ))
    })
}

fn docs_format_from_path(path: &Path) -> Option<DocsFormat> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("json") => Some(DocsFormat::Json),
        Some("md" | "markdown") => Some(DocsFormat::Markdown),
        _ => None,
    }
}

fn render_doc_group(
    docs: &DocIndex,
    group: &str,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    let mut entries = docs
        .visible_entries()
        .filter(|entry| entry.group_name().eq_ignore_ascii_case(group))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.name.cmp(&right.name));

    if entries.is_empty() {
        return render_missing_doc_group(docs, group, format);
    }

    if format == DocsBrowseFormat::Json {
        return Ok(entries_json_response("group", Some(group), None, &entries)?);
    }
    if format == DocsBrowseFormat::Markdown {
        return Ok(render_entry_markdown_table(
            &doc_group_markdown_title(entries[0].group_name()),
            &entries,
        ));
    }

    Ok(render_entry_table(
        &format!("{} docs", entries[0].group_name()),
        &entries,
    ))
}

fn render_missing_doc_group(
    docs: &DocIndex,
    group: &str,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    let suggestions = search_doc_groups(docs, group, 5);
    if format == DocsBrowseFormat::Json {
        return Ok(to_pretty_json(&json!({
            "mode": "group",
            "query": group,
            "count": 0,
            "entries": [],
            "suggestions": suggestions
                .iter()
                .map(|(group, count)| json!({
                    "name": group,
                    "entries": count,
                }))
                .collect::<Vec<_>>(),
        }))?);
    }
    if format == DocsBrowseFormat::Markdown {
        let mut output = format!(
            "No documentation group named {}.\n",
            markdown_code_span(group)
        );
        if !suggestions.is_empty() {
            let suggested_group = suggestions[0].0;
            output.push_str("\n## Did you mean\n\n");
            output.push_str(&group_markdown_table(suggestions));
            output.push_str("\n## Try\n\n");
            output.push_str(&markdown_try_command(format!(
                "scaffold docs --group {}",
                shell_arg(suggested_group)
            )));
        }
        return Ok(output);
    }

    if suggestions.is_empty() {
        return Ok(format!("No documentation group named `{group}`.\n"));
    }

    let mut output = format!("No documentation group named `{group}`.\n\nDid you mean:\n");
    let suggested_group = suggestions[0].0;
    let groups = suggestions.into_iter().collect::<BTreeMap<_, _>>();
    output.push_str(&render_group_table(&groups));
    output.push_str("\nTry:\n");
    output.push_str(&format!(
        "  scaffold docs --group {}\n",
        shell_arg(suggested_group)
    ));
    Ok(output)
}

fn render_doc_search(
    docs: &DocIndex,
    query: &str,
    limit: usize,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    let matches = search_doc_entries(docs, query, limit);
    let suggestions = if matches.is_empty() {
        suggest_doc_entries(docs, query, 5)
    } else {
        Vec::new()
    };
    if format == DocsBrowseFormat::Json {
        let mut response = entries_json_value("search", Some(query), Some(limit), &matches);
        if !suggestions.is_empty() {
            response["suggestions"] = json_entries(&suggestions);
        }
        return Ok(to_pretty_json(&response)?);
    }
    if format == DocsBrowseFormat::Markdown {
        if matches.is_empty() {
            return Ok(render_no_search_matches(query, &suggestions, true));
        }
        return Ok(render_entry_markdown_table(
            &format!("Search results for {}", markdown_code_span(query)),
            &matches,
        ));
    }

    if matches.is_empty() {
        return Ok(render_no_search_matches(query, &suggestions, false));
    }
    Ok(render_entry_table(
        &format!("Search results for `{query}`"),
        &matches,
    ))
}

fn render_doc_source(
    docs: &DocIndex,
    name: &str,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    if let Some(entry) = get_doc_entry(docs, name.trim()) {
        return render_doc_entry_source(entry, name, format);
    }

    let Some((source, entries)) = get_doc_source_entries(docs, name) else {
        return render_missing_doc_source(docs, name, format);
    };

    render_doc_source_entries(name, &source, &entries, format)
}

fn render_doc_entry_source(
    entry: &DocEntry,
    name: &str,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    if format == DocsBrowseFormat::Json {
        return Ok(to_pretty_json(&json!({
            "mode": "source",
            "query": name,
            "entry": reference_entry_json(entry),
        }))?);
    }
    if format == DocsBrowseFormat::Markdown {
        return Ok(source_markdown_for_entry(entry).unwrap_or_else(|| {
            format!(
                "## {} source\n\nNo source recorded.\n",
                markdown_code_span(&entry.name)
            )
        }));
    }

    match entry.display_source_location() {
        Some(location) => {
            let mut output = format!("Source for `{}`\n\n", entry.name);
            let mut details = vec![("source", location)];
            if let Some(signature) = entry.signature.as_deref() {
                details.push(("signature", signature.to_owned()));
            }
            output.push_str(&render_detail_table(&details));
            Ok(output)
        }
        None => Ok(format!("No source recorded for `{}`.\n", entry.name)),
    }
}

pub(crate) fn get_doc_source_entries<'a>(
    docs: &'a DocIndex,
    query: &str,
) -> Option<(String, Vec<&'a DocEntry>)> {
    let source = source_query_path(docs, query)?;
    let mut entries = docs
        .visible_entries()
        .filter(|entry| entry.source.as_deref() == Some(source.as_str()))
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return None;
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Some((source, entries))
}

fn source_query_path(docs: &DocIndex, query: &str) -> Option<String> {
    if docs
        .visible_entries()
        .any(|entry| entry.source.as_deref() == Some(query))
    {
        return Some(query.to_owned());
    }

    if let Some(source) = docs
        .visible_entries()
        .find(|entry| entry.display_source_location().as_deref() == Some(query))
        .and_then(|entry| entry.source.clone())
    {
        return Some(source);
    }

    let source = source_path_from_location_query(query)?;
    docs.visible_entries()
        .any(|entry| entry.source.as_deref() == Some(source))
        .then(|| source.to_owned())
}

fn source_path_from_location_query(query: &str) -> Option<&str> {
    let (source, line_or_column) = query.rsplit_once(':')?;
    if source.is_empty()
        || line_or_column.is_empty()
        || !line_or_column.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    if let Some((source, line)) = source.rsplit_once(':')
        && !source.is_empty()
        && !line.is_empty()
        && line.chars().all(|ch| ch.is_ascii_digit())
    {
        return Some(source);
    }

    Some(source)
}

fn render_doc_source_entries(
    query: &str,
    source: &str,
    entries: &[&DocEntry],
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    if format == DocsBrowseFormat::Json {
        return Ok(to_pretty_json(&json!({
            "mode": "source",
            "query": query,
            "source": source,
            "count": entries.len(),
            "entries": entries
                .iter()
                .map(|entry| reference_entry_json(entry))
                .collect::<Vec<_>>(),
        }))?);
    }
    if format == DocsBrowseFormat::Markdown {
        return Ok(render_entry_markdown_table(
            &format!("Docs from source {}", markdown_code_span(source)),
            entries,
        ));
    }

    Ok(render_entry_table(
        &format!("Docs from source `{source}`"),
        entries,
    ))
}

fn render_missing_doc_source(
    docs: &DocIndex,
    name: &str,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    let matches = search_doc_entries(docs, name, 10);
    let suggestions = if matches.is_empty() {
        suggest_doc_entries(docs, name, 5)
    } else {
        Vec::new()
    };
    if format == DocsBrowseFormat::Json {
        let mut response = json!({
            "mode": "source",
            "query": name,
            "missing_kind": missing_doc_source_kind(name),
            "entry": Value::Null,
            "count": matches.len(),
            "limit": 10,
            "matches": matches.iter()
                .map(|entry| reference_entry_json(entry))
                .collect::<Vec<_>>(),
        });
        if !suggestions.is_empty() {
            response["suggestions"] = json_entries(&suggestions);
        }
        return Ok(to_pretty_json(&response)?);
    }

    let mut output = render_missing_doc_source_message(name, format == DocsBrowseFormat::Markdown);
    let possible_matches = if matches.is_empty() {
        suggestions
    } else {
        matches
    };
    if possible_matches.is_empty() {
        return Ok(output);
    }

    output.push('\n');
    if format == DocsBrowseFormat::Markdown {
        output.push_str(&render_entry_markdown_table(
            &format!("Possible matches for {}", markdown_code_span(name)),
            &possible_matches,
        ));
        if let Some(entry) = possible_matches.first() {
            output.push_str("\n## Try\n\n");
            output.push_str(&markdown_try_command(format!(
                "scaffold docs --source {}",
                shell_arg(&entry.name)
            )));
        }
    } else {
        output.push_str(&render_entry_table(
            &format!("Possible matches for `{name}`"),
            &possible_matches,
        ));
        if let Some(entry) = possible_matches.first() {
            output.push_str("\nTry:\n");
            output.push_str(&format!(
                "  scaffold docs --source {}\n",
                shell_arg(&entry.name)
            ));
        }
    }
    Ok(output)
}

fn render_missing_doc_source_message(name: &str, markdown: bool) -> String {
    let name_code = if markdown {
        markdown_code_span(name)
    } else {
        format!("`{name}`")
    };

    if source_query_looks_like_path(name) {
        format!("No documented source matched {name_code}.\n")
    } else {
        format!("No documented symbol named {name_code}.\n")
    }
}

fn missing_doc_source_kind(query: &str) -> &'static str {
    if source_query_looks_like_path(query) {
        "source"
    } else {
        "symbol"
    }
}

fn source_query_looks_like_path(query: &str) -> bool {
    let source = source_path_from_location_query(query).unwrap_or(query);
    source.ends_with(".scm") || query.contains(".scm:")
}

pub(crate) fn get_doc_entry<'a>(docs: &'a DocIndex, name: &str) -> Option<&'a DocEntry> {
    docs.get(name).filter(|entry| !entry.hidden).or_else(|| {
        docs.visible_entries()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
    })
}

fn render_doc_entry(
    entry: &DocEntry,
    query: &str,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    if format == DocsBrowseFormat::Json {
        return Ok(to_pretty_json(&json!({
            "mode": "entry",
            "query": query,
            "entry": reference_entry_json(entry),
        }))?);
    }
    if format == DocsBrowseFormat::Markdown {
        return Ok(titled_markdown_for_entry(entry));
    }

    let documentation = entry_documentation(entry);
    let mut output = String::new();
    output.push_str(&format!("{}\n", entry.name));
    output.push_str(&format!("{}\n", "=".repeat(entry.name.len())));

    if let Some(signature) = documentation.signature {
        output.push_str(&format!("\n{signature}\n"));
    }

    if let Some(summary) = documentation.summary {
        push_section_break(&mut output);
        output.push_str(summary);
        output.push('\n');
    }

    if let Some(deprecated) = documentation.deprecated {
        push_section_break(&mut output);
        output.push_str(&format!("Deprecated: {deprecated}\n"));
    }

    if let Some(markdown) = documentation.markdown {
        push_section_break(&mut output);
        output.push_str(markdown);
        output.push('\n');
    }

    if !documentation.params.is_empty() {
        push_section_break(&mut output);
        output.push_str("Parameters\n");
        output.push_str(&render_param_table(documentation.params));
    }

    if let Some(returns) = documentation.returns {
        push_section_break(&mut output);
        output.push_str("Returns\n");
        output.push_str(returns);
        output.push('\n');
    }

    if let Some(example) = documentation.example {
        push_section_break(&mut output);
        output.push_str("Example\n");
        output.push_str(example);
        output.push('\n');
    }

    if !documentation.has_body() {
        push_section_break(&mut output);
        output.push_str("No documentation provided.\n");
    }

    push_section_break(&mut output);
    output.push_str("Details\n");
    let details = documentation
        .details
        .iter()
        .map(|detail| (detail.field, detail.value.clone()))
        .collect::<Vec<_>>();
    output.push_str(&render_detail_table(&details));

    if !documentation.see.is_empty() {
        push_section_break(&mut output);
        output.push_str("See also\n");
        output.push_str(&format!("  {}\n", documentation.see.join(", ")));
    }

    Ok(output)
}

fn render_group_table(groups: &BTreeMap<&str, usize>) -> String {
    let mut table = output_table(None);
    table.set_header(vec![header_cell("group"), header_cell("entries")]);

    for (group, count) in groups {
        table.add_row(vec![Cell::new(group), Cell::new(count)]);
    }

    format!("{}\n", table.trim_fmt())
}

pub(crate) fn doc_group_counts(docs: &DocIndex) -> BTreeMap<&str, usize> {
    let mut groups = BTreeMap::<&str, usize>::new();
    for entry in docs.visible_entries() {
        *groups.entry(entry.group_name()).or_default() += 1;
    }
    groups
}

pub(crate) fn search_doc_groups<'a>(
    docs: &'a DocIndex,
    query: &str,
    limit: usize,
) -> Vec<(&'a str, usize)> {
    let mut matches = doc_group_counts(docs)
        .into_iter()
        .filter_map(|(group, count)| {
            doc_group_match_score(group, query).map(|score| (group, count, score))
        })
        .collect::<Vec<_>>();
    matches.sort_by(
        |(left_group, left_count, left_score), (right_group, right_count, right_score)| {
            right_score
                .cmp(left_score)
                .then_with(|| right_count.cmp(left_count))
                .then_with(|| left_group.cmp(right_group))
        },
    );
    matches
        .into_iter()
        .take(limit)
        .map(|(group, count, _score)| (group, count))
        .collect()
}

fn doc_group_match_score(group: &str, query: &str) -> Option<usize> {
    let group = normalize_group_query(group);
    let query = normalize_group_query(query);
    if query.is_empty() {
        return None;
    }
    if group == query {
        return Some(10_000 + query.len());
    }
    if group.starts_with(&query) {
        return Some(9_000 + query.len());
    }
    if group.contains(&query) {
        return Some(8_000 + query.len());
    }
    (query.chars().count() >= 5 && is_group_subsequence(&query, &group))
        .then_some(6_000 + query.len())
}

fn normalize_group_query(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_group_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    needle
        .chars()
        .all(|needle_ch| chars.any(|ch| ch == needle_ch))
}

fn render_entry_table(title: &str, entries: &[&DocEntry]) -> String {
    let mut output = format!("{title}\n\n{}.\n\n", entry_count_label(entries.len()));
    let mut table = output_table(Some(DOC_TABLE_WIDTH));
    table.set_header(vec![
        header_cell("symbol"),
        header_cell("group"),
        header_cell("summary"),
    ]);

    for entry in entries {
        table.add_row(vec![
            Cell::new(&entry.name).fg(Color::Green),
            Cell::new(entry.group_name()),
            Cell::new(entry.summary.as_deref().unwrap_or("No summary.")),
        ]);
    }

    output.push_str(&format!("{}\n", table.trim_fmt()));
    output
}

fn render_entry_markdown_table(title: &str, entries: &[&DocEntry]) -> String {
    let mut output = format!("## {title}\n\n{}.\n\n", entry_count_label(entries.len()));
    output.push_str(&entry_summary_markdown_table(entries.iter().copied()));
    output
}

fn doc_group_markdown_title(group: &str) -> String {
    format!("{} docs", markdown_text(group))
}

fn render_no_search_matches(query: &str, suggestions: &[&DocEntry], markdown: bool) -> String {
    let query_code = if markdown {
        markdown_code_span(query)
    } else {
        format!("`{query}`")
    };
    let mut output = format!("No reference entries matched {query_code}.\n");
    if suggestions.is_empty() {
        return output;
    }

    output.push('\n');
    if markdown {
        output.push_str(&render_entry_markdown_table(
            &format!("Possible matches for {query_code}"),
            suggestions,
        ));
        if let Some(entry) = suggestions.first() {
            output.push_str("\n## Try\n\n");
            output.push_str(&markdown_try_command(format!(
                "scaffold docs {}",
                shell_arg(&entry.name)
            )));
        }
    } else {
        output.push_str(&render_entry_table(
            &format!("Possible matches for `{query}`"),
            suggestions,
        ));
        if let Some(entry) = suggestions.first() {
            output.push_str("\nTry:\n");
            output.push_str(&format!("  scaffold docs {}\n", shell_arg(&entry.name)));
        }
    }

    output
}

fn shell_arg(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '/' | '.' | ':' | '+'))
    {
        return value.to_owned();
    }

    format!("'{}'", value.replace('\'', "'\\''"))
}

fn render_param_table(params: &[DocParam]) -> String {
    let mut table = output_table(Some(DOC_TABLE_WIDTH));
    table.set_header(vec![header_cell("parameter"), header_cell("summary")]);

    for param in params {
        table.add_row(vec![
            Cell::new(&param.name).fg(Color::Green),
            Cell::new(&param.summary),
        ]);
    }

    format!("{}\n", table.trim_fmt())
}

fn render_detail_table(rows: &[(&str, String)]) -> String {
    let mut table = output_table(Some(DOC_TABLE_WIDTH));
    table.set_header(vec![header_cell("field"), header_cell("value")]);

    for (field, value) in rows {
        table.add_row(vec![Cell::new(field), Cell::new(value)]);
    }

    format!("{}\n", table.trim_fmt())
}

fn entries_json_response(
    mode: &str,
    query: Option<&str>,
    limit: Option<usize>,
    entries: &[&DocEntry],
) -> Result<String, serde_json::Error> {
    to_pretty_json(&entries_json_value(mode, query, limit, entries))
}

fn entries_json_value(
    mode: &str,
    query: Option<&str>,
    limit: Option<usize>,
    entries: &[&DocEntry],
) -> Value {
    let mut response = json!({
        "mode": mode,
        "count": entries.len(),
        "entries": json_entries(entries),
    });

    if let Some(query) = query {
        response["query"] = json!(query);
    }
    if let Some(limit) = limit {
        response["limit"] = json!(limit);
    }

    response
}

fn json_entries(entries: &[&DocEntry]) -> Value {
    json!(
        entries
            .iter()
            .map(|entry| reference_entry_json(entry))
            .collect::<Vec<_>>()
    )
}

fn to_pretty_json(value: &Value) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value).map(|json| format!("{json}\n"))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn docs_args() -> DocsArgs {
        DocsArgs {
            query: Vec::new(),
            all: false,
            search: None,
            group: None,
            source: None,
            limit: None,
            output: None,
            format: None,
        }
    }

    #[test]
    fn docs_default_renders_group_overview_not_full_reference() {
        let rendered = render_docs(&docs_args()).expect("render docs");

        assert!(rendered.starts_with("Scaffold Docs"));
        assert!(rendered.contains("scaffold docs tool"));
        assert!(rendered.contains("scaffold docs --search \"ctlg tool\""));
        assert!(rendered.contains("scaffold docs --source src/dsl/std/catalog/tool.scm"));
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
        assert!(rendered.contains("parameter"));
        assert!(rendered.contains("summary"));
        assert!(rendered.contains("field"));
        assert!(rendered.contains("value"));
        assert!(!rendered.contains("  group   Catalog"));
    }

    #[test]
    fn docs_query_renders_no_documentation_fallback_for_sparse_entry() {
        let mut args = docs_args();
        args.query = vec!["subject".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("subject\n======="));
        assert!(rendered.contains("No documentation provided."));
        assert!(rendered.contains("Details"));
        assert!(rendered.contains("src/dsl/std/core/doc.scm:"));
    }

    #[test]
    fn docs_query_exact_entry_is_case_insensitive() {
        let mut args = docs_args();
        args.query = vec!["TOOL/PATH".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("tool/path\n========="));
        assert!(!rendered.contains("Search results for"));
    }

    #[test]
    fn docs_query_does_not_expose_hidden_exact_entries() {
        let mut args = docs_args();
        args.query = vec!["action".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert!(!rendered.starts_with("action\n======"));
        assert!(!rendered.contains("(action ...)"));
        assert!(rendered.contains("Search results for `action`"));
        assert!(rendered.contains("package"));
        assert!(!rendered.contains("src/dsl/std/catalog/action.scm:11"));
    }

    #[test]
    fn docs_query_fuzzy_searches_when_exact_entry_is_missing() {
        let mut args = docs_args();
        args.query = vec!["ctlg".to_owned(), "tool".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.contains("Search results for `ctlg tool`"));
        assert!(rendered.contains("catalog/tool"));
        assert!(!rendered.contains("signature"));
        assert!(!rendered.contains("(catalog/tool name action field ...)"));
    }

    #[test]
    fn docs_search_flag_always_renders_search_results() {
        let mut args = docs_args();
        args.search = Some("tool".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.contains("Search results for `tool`"));
        assert!(rendered.contains("catalog/tool"));
        assert!(!rendered.starts_with("tool\n===="));
    }

    #[test]
    fn docs_query_rejects_unrelated_fuzzy_noise() {
        let mut args = docs_args();
        args.query = vec!["zzzzzzz".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No reference entries matched `zzzzzzz`.\n");

        args.query = vec!["no-such-query".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No reference entries matched `no-such-query`.\n");
    }

    #[test]
    fn docs_search_empty_results_suggest_close_symbol_typos() {
        let mut args = docs_args();
        args.search = Some("catalgo".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No reference entries matched `catalgo`."));
        assert!(rendered.contains("Possible matches for `catalgo`"));
        assert!(rendered.contains("catalog"));
        assert!(rendered.contains("scaffold docs catalog"));
    }

    #[test]
    fn docs_search_empty_markdown_results_suggest_close_symbol_typos() {
        let mut args = docs_args();
        args.search = Some("sourcpath".to_owned());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No reference entries matched `sourcpath`."));
        assert!(rendered.contains("## Possible matches for `sourcpath`"));
        assert!(rendered.contains("`source/path`"));
        assert!(rendered.contains("- `scaffold docs source/path`"));
        assert!(!rendered.contains("┌"));
    }

    #[test]
    fn docs_markdown_search_escapes_reflected_backtick_queries() {
        let mut args = docs_args();
        args.search = Some("zzzz`query".to_owned());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No reference entries matched `` zzzz`query ``.\n");
        assert!(!rendered.contains("`zzzz`query`"));
    }

    #[test]
    fn docs_query_rejects_empty_positional_query() {
        let mut args = docs_args();
        args.query = vec!["   ".to_owned()];

        let err = render_docs(&args).expect_err("empty query should fail");

        assert_eq!(err.to_string(), "docs query cannot be empty");
    }

    #[test]
    fn docs_browse_flags_reject_empty_values() {
        let mut args = docs_args();
        args.search = Some("   ".to_owned());

        let err = render_docs(&args).expect_err("empty search should fail");
        assert_eq!(err.to_string(), "--search cannot be empty");

        args = docs_args();
        args.group = Some("   ".to_owned());

        let err = render_docs(&args).expect_err("empty group should fail");
        assert_eq!(err.to_string(), "--group cannot be empty");

        args = docs_args();
        args.source = Some("   ".to_owned());

        let err = render_docs(&args).expect_err("empty source should fail");
        assert_eq!(err.to_string(), "--source cannot be empty");
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
    fn docs_group_title_uses_canonical_group_label() {
        let mut args = docs_args();
        args.group = Some("language".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("Language docs"));
        assert!(!rendered.starts_with("language docs"));
    }

    #[test]
    fn docs_markdown_group_titles_escape_inline_markup() {
        assert_eq!(
            doc_group_markdown_title("Bad [Group] | Plus+"),
            "Bad \\[Group\\] | Plus\\+ docs"
        );
    }

    #[test]
    fn docs_group_typo_suggests_matching_groups_without_full_overview() {
        let mut args = docs_args();
        args.group = Some("Catlog".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No documentation group named `Catlog`."));
        assert!(rendered.contains("Did you mean:"));
        assert!(rendered.contains("Catalog"));
        assert!(rendered.contains("scaffold docs --group Catalog"));
        assert!(!rendered.contains("Scaffold Docs"));
    }

    #[test]
    fn docs_group_suggestions_quote_shell_arguments_with_spaces() {
        let mut args = docs_args();
        args.group = Some("mactools".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No documentation group named `mactools`."));
        assert!(rendered.contains("macOS tools"));
        assert!(rendered.contains("scaffold docs --group 'macOS tools'"));
        assert!(!rendered.contains("scaffold docs --group macOS tools"));
    }

    #[test]
    fn docs_group_unrelated_short_query_does_not_suggest_noise() {
        let mut args = docs_args();
        args.group = Some("Nope".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No documentation group named `Nope`.\n");
        assert!(!rendered.contains("Download helpers"));
    }

    #[test]
    fn docs_markdown_group_errors_escape_reflected_backticks() {
        let mut args = docs_args();
        args.group = Some("zzzz`group".to_owned());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No documentation group named `` zzzz`group ``.\n");
        assert!(!rendered.contains("`zzzz`group`"));
    }

    #[test]
    fn docs_try_commands_shell_quote_only_when_needed() {
        assert_eq!(shell_arg("catalog/tool"), "catalog/tool");
        assert_eq!(shell_arg("macOS tools"), "'macOS tools'");
        assert_eq!(shell_arg("author's tool"), "'author'\\''s tool'");
    }

    #[test]
    fn docs_markdown_try_commands_use_safe_code_spans() {
        assert_eq!(
            markdown_try_command("scaffold docs bad`name"),
            "- `` scaffold docs bad`name ``\n"
        );
    }

    #[test]
    fn docs_markdown_code_spans_use_longer_delimiters_when_needed() {
        assert_eq!(markdown_code_span("catalog/tool"), "`catalog/tool`");
        assert_eq!(markdown_code_span("bad`query"), "`` bad`query ``");
        assert_eq!(markdown_code_span("bad``query"), "``` bad``query ```");
    }

    #[test]
    fn docs_source_shows_recorded_location() {
        let mut args = docs_args();
        args.source = Some("tool".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("Source for `tool`"));
        assert!(rendered.contains("field"));
        assert!(rendered.contains("value"));
        assert!(rendered.contains("source"));
        assert!(rendered.contains("src/dsl/std/catalog/tool.scm"));
        assert!(rendered.contains("signature"));
        assert!(rendered.contains("(tool name action field ...)"));
    }

    #[test]
    fn docs_query_text_includes_effect_and_capabilities() {
        let mut args = docs_args();
        args.query = vec!["source/path".to_owned()];

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.contains("effect"));
        assert!(rendered.contains("context-read-only"));
        assert!(rendered.contains("requires capability"));
        assert!(rendered.contains("scaffold.workspace"));
    }

    #[test]
    fn docs_source_lookup_is_case_insensitive() {
        let mut args = docs_args();
        args.source = Some("Tool".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.contains("src/dsl/std/catalog/tool.scm"));
        assert!(rendered.contains("(tool name action field ...)"));
        assert!(!rendered.contains("No docs for"));
    }

    #[test]
    fn docs_source_lists_entries_from_source_file() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/tool.scm".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("Docs from source `src/dsl/std/catalog/tool.scm`"));
        assert!(rendered.contains("tool"));
        assert!(rendered.contains("catalog/tool"));
        assert!(rendered.contains("Create a catalog tool object."));
        assert!(!rendered.contains("No documented symbol named"));
    }

    #[test]
    fn docs_source_accepts_recorded_source_location() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/tool.scm:16".to_owned());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("## Docs from source `src/dsl/std/catalog/tool.scm`\n\n"));
        assert!(rendered.contains("| `tool`"));
        assert!(rendered.contains("| `tool/append-bins`"));
        assert!(!rendered.contains("No documented symbol named"));
    }

    #[test]
    fn docs_source_accepts_any_line_in_known_source_file() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/tool.scm:999".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("Docs from source `src/dsl/std/catalog/tool.scm`"));
        assert!(rendered.contains("tool/append-bins"));
        assert!(!rendered.contains("No documented symbol named"));
    }

    #[test]
    fn docs_source_accepts_line_and_column_in_known_source_file() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/tool.scm:16:1".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("Docs from source `src/dsl/std/catalog/tool.scm`"));
        assert!(rendered.contains("tool/append-bins"));
        assert!(!rendered.contains("No documented symbol named"));
    }

    #[test]
    fn docs_source_missing_source_path_reports_source_not_symbol() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/missing.scm:1:1".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(
            rendered,
            "No documented source matched `src/dsl/std/catalog/missing.scm:1:1`.\n"
        );

        args.source = Some("catalog/missing".to_owned());
        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No documented symbol named `catalog/missing`."));
    }

    #[test]
    fn docs_markdown_source_missing_source_path_reports_source_not_symbol() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/tool.scm:not-a-line".to_owned());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(
            rendered,
            "No documented source matched `src/dsl/std/catalog/tool.scm:not-a-line`.\n"
        );
    }

    #[test]
    fn docs_source_typo_reports_missing_symbol_then_suggestions() {
        let mut args = docs_args();
        args.source = Some("Catlog".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No documented symbol named `Catlog`."));
        assert!(rendered.contains("Possible matches for `Catlog`"));
        assert!(rendered.contains("catalog/tool"));
        assert!(rendered.contains("Try:\n  scaffold docs --source catalog"));
        assert!(!rendered.contains("No docs for"));
    }

    #[test]
    fn docs_source_empty_results_suggest_close_symbol_typos() {
        let mut args = docs_args();
        args.source = Some("catlgtool".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No documented symbol named `catlgtool`."));
        assert!(rendered.contains("Possible matches for `catlgtool`"));
        assert!(rendered.contains("catalog/tool"));
        assert!(rendered.contains("Create a raw catalog tool list for macro-oriented helpers."));
        assert!(rendered.contains("Try:\n  scaffold docs --source catalog/tool"));
    }

    #[test]
    fn docs_source_unrelated_symbol_does_not_dump_noise() {
        let mut args = docs_args();
        args.source = Some("nope".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No documented symbol named `nope`.\n");
    }

    #[test]
    fn docs_source_does_not_expose_hidden_exact_entries() {
        let mut args = docs_args();
        args.source = Some("action".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No documented symbol named `action`."));
        assert!(!rendered.contains("(action ...)"));
        assert!(!rendered.contains("src/dsl/std/catalog/action.scm:11"));
    }

    #[test]
    fn docs_markdown_source_errors_escape_reflected_backticks() {
        let mut args = docs_args();
        args.source = Some("zzzz`source".to_owned());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert_eq!(rendered, "No documented symbol named `` zzzz`source ``.\n");
        assert!(!rendered.contains("`zzzz`source`"));
    }

    #[test]
    fn docs_markdown_source_empty_results_suggest_close_symbol_typos() {
        let mut args = docs_args();
        args.source = Some("catlgtool".to_owned());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("No documented symbol named `catlgtool`."));
        assert!(rendered.contains("## Possible matches for `catlgtool`"));
        assert!(rendered.contains("| `catalog/tool`"));
        assert!(rendered.contains("Create a raw catalog tool list for macro-oriented helpers."));
        assert!(rendered.contains("## Try\n\n- `scaffold docs --source catalog/tool`"));
    }

    #[test]
    fn docs_format_markdown_keeps_browse_overview() {
        let mut args = docs_args();
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("# Scaffold Docs"));
        assert!(rendered.contains("| Group"));
        assert!(rendered.contains("`Catalog`"));
        assert!(!rendered.contains("┌"));
        assert!(!rendered.starts_with("# Scaffold Scheme Reference"));
    }

    #[test]
    fn docs_format_markdown_renders_search_markdown() {
        let mut args = docs_args();
        args.search = Some("ctlg tool".to_owned());
        args.limit = Some(1);
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("## Search results for `ctlg tool`"));
        assert!(rendered.contains("| Symbol"));
        assert!(rendered.contains("`catalog/tool`"));
        assert!(!rendered.contains("┌"));
    }

    #[test]
    fn docs_search_matches_documented_examples() {
        let mut args = docs_args();
        args.search = Some("ripgrep".to_owned());
        args.limit = Some(5);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.contains("catalog/tool"));
        assert!(!rendered.contains("No reference entries matched"));
    }

    #[test]
    fn docs_search_matches_source_locations() {
        let mut args = docs_args();
        args.search = Some("src/dsl/std/catalog/tool.scm:16:1".to_owned());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("Search results for `src/dsl/std/catalog/tool.scm:16:1`"));
        assert!(rendered.contains("tool"));
        assert!(!rendered.contains("No reference entries matched"));
    }

    #[test]
    fn docs_format_markdown_renders_exact_entry_markdown_with_title() {
        let mut args = docs_args();
        args.query = vec!["tool".to_owned()];
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("## `tool`"));
        assert!(rendered.contains("```scheme\n(tool name action field ...)\n```"));
        assert!(rendered.contains("| Field  | Value"));
        assert!(rendered.contains("| Source | `src/dsl/std/catalog/tool.scm:"));
        assert!(!rendered.contains("┌"));
    }

    #[test]
    fn docs_format_markdown_renders_no_documentation_fallback() {
        let mut args = docs_args();
        args.query = vec!["subject".to_owned()];
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("## `subject`"));
        assert!(rendered.contains("No documentation provided."));
        assert!(rendered.contains("| Source | `src/dsl/std/core/doc.scm:"));
    }

    #[test]
    fn docs_format_markdown_renders_source_markdown() {
        let mut args = docs_args();
        args.source = Some("tool".to_owned());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("## `tool` source"));
        assert!(rendered.contains("| Field     | Value"));
        assert!(rendered.contains("| Source    | `src/dsl/std/catalog/tool.scm:"));
        assert!(rendered.contains("| Signature | `(tool name action field ...)`"));
    }

    #[test]
    fn docs_format_json_renders_overview_json() {
        let mut args = docs_args();
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("overview json");

        assert_eq!(value["mode"], "overview");
        assert!(value["entry_count"].as_u64().is_some_and(|count| count > 0));
        assert!(
            value["groups"]
                .as_array()
                .is_some_and(|groups| { groups.iter().any(|group| group["name"] == "Catalog") })
        );
    }

    #[test]
    fn docs_format_json_renders_exact_entry_json() {
        let mut args = docs_args();
        args.query = vec!["TOOL".to_owned()];
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("entry json");

        assert_eq!(value["mode"], "entry");
        assert_eq!(value["query"], "TOOL");
        assert_eq!(value["entry"]["name"], "tool");
        assert_eq!(value["entry"]["group"], "Catalog");
        assert_eq!(value["entry"]["kind"], "function");
        assert!(value["entry"]["markdown"].is_null());
        assert!(value["entry"]["raw_markdown"].is_null());
        assert!(
            value["entry"]["rendered_markdown"]
                .as_str()
                .is_some_and(|markdown| {
                    markdown.contains("```scheme\n(tool name action field ...)\n```")
                        && markdown.contains("**Parameters**")
                })
        );
        assert!(
            value["entry"]["range"]["length"]
                .as_u64()
                .is_some_and(|length| length > 0)
        );
        assert!(
            value["entry"]["params"]
                .as_array()
                .is_some_and(|params| { params.iter().any(|param| param["name"] == "name") })
        );
    }

    #[test]
    fn docs_format_json_renders_search_json() {
        let mut args = docs_args();
        args.search = Some("ctlg tool".to_owned());
        args.limit = Some(3);
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("search json");

        assert_eq!(value["mode"], "search");
        assert_eq!(value["query"], "ctlg tool");
        assert_eq!(value["limit"], 3);
        assert_eq!(value["count"], 3);
        assert_eq!(value["entries"][0]["name"], "catalog/tool");
        assert_eq!(
            value["entries"][0]["markdown"],
            "Prefer `tool` for ordinary catalog entries. Use `catalog/tool` when writing extension macros that need to splice fields directly into the raw catalog shape before Scaffold normalizes it."
        );
        assert_eq!(
            value["entries"][0]["raw_markdown"],
            value["entries"][0]["markdown"]
        );
        assert!(
            value["entries"][0]["rendered_markdown"]
                .as_str()
                .is_some_and(|markdown| {
                    markdown.contains("```scheme\n(catalog/tool name action field ...)\n```")
                        && markdown.contains("**Example**")
                })
        );
    }

    #[test]
    fn docs_format_json_search_suggestions_use_entries_field() {
        let mut args = docs_args();
        args.search = Some("catlgtool".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("search json");

        assert_eq!(value["mode"], "search");
        assert_eq!(value["query"], "catlgtool");
        assert_eq!(value["count"], 0);
        assert_eq!(value["entries"].as_array().map(Vec::len), Some(0));
        assert_eq!(value["suggestions"][0]["name"], "catalog/tool");
        assert!(value["suggestions"][0].get("group").is_some());
    }

    #[test]
    fn docs_format_json_renders_group_json() {
        let mut args = docs_args();
        args.group = Some("Catalog".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("group json");

        assert_eq!(value["mode"], "group");
        assert_eq!(value["query"], "Catalog");
        assert!(value["count"].as_u64().is_some_and(|count| count > 0));
        assert!(
            value["entries"]
                .as_array()
                .is_some_and(|entries| { entries.iter().any(|entry| entry["name"] == "tool") })
        );
    }

    #[test]
    fn docs_format_json_group_suggestions_use_group_name_field() {
        let mut args = docs_args();
        args.group = Some("Catlog".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("missing group json");

        assert_eq!(value["mode"], "group");
        assert_eq!(value["count"], 0);
        assert_eq!(value["suggestions"][0]["name"], "Catalog");
        assert!(value["suggestions"][0].get("group").is_none());
    }

    #[test]
    fn docs_format_json_group_unrelated_short_query_has_no_suggestions() {
        let mut args = docs_args();
        args.group = Some("Nope".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("missing group json");

        assert_eq!(value["mode"], "group");
        assert_eq!(value["count"], 0);
        assert_eq!(value["suggestions"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn docs_format_json_renders_source_json() {
        let mut args = docs_args();
        args.source = Some("source/path".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("source json");

        assert_eq!(value["mode"], "source");
        assert_eq!(value["query"], "source/path");
        assert_eq!(value["entry"]["name"], "source/path");
        assert_eq!(value["entry"]["effect"], "context-read-only");
        assert_eq!(
            value["entry"]["requires_capability"],
            json!(["scaffold.workspace"])
        );
        assert_eq!(
            value["entry"]["returns"],
            "A path string, or `#f` when no source path is available."
        );
        assert!(
            value["entry"]["source_location"]
                .as_str()
                .is_some_and(|source| source.contains("src/dsl/std/workspace.scm"))
        );
        assert!(
            value["entry"]["range"]["length"]
                .as_u64()
                .is_some_and(|length| length > 0)
        );
    }

    #[test]
    fn docs_format_json_renders_source_file_entries() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/tool.scm".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("source json");

        assert_eq!(value["mode"], "source");
        assert_eq!(value["query"], "src/dsl/std/catalog/tool.scm");
        assert_eq!(value["source"], "src/dsl/std/catalog/tool.scm");
        assert_eq!(
            value["count"].as_u64(),
            value["entries"]
                .as_array()
                .map(|entries| entries.len() as u64)
        );
        assert!(
            value["entries"]
                .as_array()
                .is_some_and(|entries| entries.iter().any(|entry| entry["name"] == "tool"))
        );
        assert!(value.get("entry").is_none());
    }

    #[test]
    fn docs_format_json_preserves_source_location_query() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/tool.scm:999:1".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("source json");

        assert_eq!(value["mode"], "source");
        assert_eq!(value["query"], "src/dsl/std/catalog/tool.scm:999:1");
        assert_eq!(value["source"], "src/dsl/std/catalog/tool.scm");
        assert!(
            value["entries"]
                .as_array()
                .is_some_and(|entries| entries.iter().any(|entry| entry["name"] == "tool"))
        );
    }

    #[test]
    fn docs_format_json_missing_source_includes_match_count_and_limit() {
        let mut args = docs_args();
        args.source = Some("Catlog".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("missing source json");

        assert_eq!(value["mode"], "source");
        assert_eq!(value["missing_kind"], "symbol");
        assert!(value["entry"].is_null());
        assert_eq!(value["limit"], 10);
        assert_eq!(
            value["count"].as_u64(),
            value["matches"]
                .as_array()
                .map(|matches| matches.len() as u64)
        );
    }

    #[test]
    fn docs_format_json_missing_source_path_reports_source_kind() {
        let mut args = docs_args();
        args.source = Some("src/dsl/std/catalog/missing.scm:1:1".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("missing source json");

        assert_eq!(value["mode"], "source");
        assert_eq!(value["query"], "src/dsl/std/catalog/missing.scm:1:1");
        assert_eq!(value["missing_kind"], "source");
        assert!(value["entry"].is_null());
        assert_eq!(value["count"], 0);
        assert_eq!(value["matches"].as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn docs_format_json_missing_source_suggests_close_symbol_typos() {
        let mut args = docs_args();
        args.source = Some("catlgtool".to_owned());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("missing source json");

        assert_eq!(value["mode"], "source");
        assert!(value["entry"].is_null());
        assert_eq!(value["count"], 0);
        assert_eq!(value["matches"].as_array().map(Vec::len), Some(0));
        assert_eq!(value["suggestions"][0]["name"], "catalog/tool");
        assert_eq!(
            value["suggestions"][0]["summary"],
            "Create a raw catalog tool list for macro-oriented helpers."
        );
    }

    #[test]
    fn docs_all_exports_markdown_by_default() {
        let mut args = docs_args();
        args.all = true;

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("# Scaffold Scheme Reference"));
        assert!(rendered.contains("## Contents"));
        assert!(!rendered.starts_with("{\n"));
    }

    #[test]
    fn docs_all_respects_json_format() {
        let mut args = docs_args();
        args.all = true;
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");
        let value: Value = serde_json::from_str(&rendered).expect("reference json");

        assert_eq!(value["title"], "Scaffold Scheme Reference");
        assert!(
            value["entries"]
                .as_array()
                .is_some_and(|entries| { entries.iter().any(|entry| entry["name"] == "tool") })
        );
        assert!(!rendered.starts_with("# Scaffold Scheme Reference"));
    }

    #[test]
    fn docs_output_json_extension_exports_json() {
        let mut args = docs_args();
        args.output = Some("reference.json".into());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("{\n"));
        assert!(rendered.contains("\"title\": \"Scaffold Scheme Reference\""));
        assert!(!rendered.starts_with("# Scaffold Scheme Reference"));
    }

    #[test]
    fn docs_output_markdown_extension_exports_markdown() {
        let mut args = docs_args();
        args.output = Some("reference.md".into());

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("# Scaffold Scheme Reference"));
    }

    #[test]
    fn docs_explicit_format_overrides_output_extension() {
        let mut args = docs_args();
        args.output = Some("reference.json".into());
        args.format = Some(DocsFormat::Markdown);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("# Scaffold Scheme Reference"));

        args.output = Some("reference.md".into());
        args.format = Some(DocsFormat::Json);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("{\n"));
        assert!(rendered.contains("\"title\": \"Scaffold Scheme Reference\""));
    }

    #[test]
    fn docs_unknown_output_extension_requires_explicit_format() {
        let mut args = docs_args();
        args.output = Some("reference.out".into());

        let err = render_docs(&args).expect_err("unknown extension should fail");

        assert!(err.to_string().contains("cannot infer docs format"));
        assert!(err.to_string().contains("--format"));
    }

    #[test]
    fn docs_output_without_extension_requires_explicit_format() {
        let mut args = docs_args();
        args.output = Some("reference".into());

        let err = render_docs(&args).expect_err("missing extension should fail");

        assert!(err.to_string().contains("cannot infer docs format"));
        assert!(err.to_string().contains(".json"));
    }

    #[test]
    fn docs_export_options_reject_browse_selectors() {
        let mut args = docs_args();
        args.output = Some("reference.md".into());
        args.query = vec!["   ".to_owned()];

        let err = render_docs(&args).expect_err("export with query should fail");

        assert!(err.to_string().contains("cannot be combined"));
    }

    #[test]
    fn docs_browse_selectors_reject_mixed_modes() {
        let mut args = docs_args();
        args.query = vec!["tool".to_owned()];
        args.group = Some("Catalog".to_owned());

        let err = render_docs(&args).expect_err("mixed selectors should fail");

        assert!(err.to_string().contains("cannot be combined"));
    }

    #[test]
    fn docs_limit_is_rejected_when_it_would_be_ignored() {
        let mut args = docs_args();
        args.limit = Some(5);

        let err = render_docs(&args).expect_err("overview limit should fail");
        assert_eq!(
            err.to_string(),
            "--limit only applies to reference search; use a query or --search"
        );

        args = docs_args();
        args.group = Some("Catalog".to_owned());
        args.limit = Some(5);

        let err = render_docs(&args).expect_err("group limit should fail");
        assert_eq!(
            err.to_string(),
            "--limit only applies to reference search; it cannot be combined with --group or --source"
        );

        args = docs_args();
        args.output = Some("reference.md".into());
        args.limit = Some(5);

        let err = render_docs(&args).expect_err("export limit should fail");
        assert_eq!(
            err.to_string(),
            "--limit only applies to reference search; full reference exports ignore it"
        );
    }

    #[test]
    fn docs_search_limit_rejects_out_of_range_values() {
        let mut args = docs_args();
        args.search = Some("tool".to_owned());
        args.limit = Some(0);

        let err = render_docs(&args).expect_err("zero search limit should fail");
        assert_eq!(
            err.to_string(),
            format!("--limit must be between 1 and {MAX_SEARCH_LIMIT}")
        );

        args.limit = Some(MAX_SEARCH_LIMIT + 1);

        let err = render_docs(&args).expect_err("oversized search limit should fail");
        assert_eq!(
            err.to_string(),
            format!("--limit must be between 1 and {MAX_SEARCH_LIMIT}")
        );
    }

    #[test]
    fn docs_search_accepts_max_limit() {
        let mut args = docs_args();
        args.search = Some("tool".to_owned());
        args.limit = Some(MAX_SEARCH_LIMIT);

        let rendered = render_docs(&args).expect("render docs");

        assert!(rendered.starts_with("Search results for `tool`"));
    }
}
