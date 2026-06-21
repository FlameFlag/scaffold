use std::fmt::Write as _;

use scaffold_docs::{
    DocEntry, DocIndex, markdown_code_span, reference_entry_json, search_doc_entries,
    source_markdown_for_entry, source_path_from_location_query, suggest_doc_entries,
};
use serde_json::{Value, json};

use crate::cli::CliError;

use super::{
    DocsBrowseFormat,
    entries::{render_entry_markdown_table, render_entry_table},
    entry::{get_doc_entry, render_detail_table},
    json::{json_entries, to_pretty_json},
    text::{markdown_try_command, shell_arg},
};

pub(super) fn render_doc_source(
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

pub(crate) fn get_doc_source_entries<'a>(
    docs: &'a DocIndex,
    query: &str,
) -> Option<(String, Vec<&'a DocEntry>)> {
    let source = source_query_path(docs, query)?;
    let entries = docs
        .entries_in_source(&source)
        .filter(|entry| !entry.hidden)
        .collect::<Vec<_>>();
    if entries.is_empty() {
        return None;
    }
    Some((source, entries))
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
            output.push_str(&render_detail_table(
                std::iter::once(("source", location)).chain(
                    entry
                        .signature
                        .as_deref()
                        .map(|signature| ("signature", signature.to_owned())),
                ),
            ));
            Ok(output)
        }
        None => Ok(format!("No source recorded for `{}`.\n", entry.name)),
    }
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
            "entries": json_entries(entries.iter().copied()),
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
            "matches": json_entries(matches.iter().copied()),
        });
        if !suggestions.is_empty() {
            response["suggestions"] = json_entries(suggestions.iter().copied());
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
            let _ = writeln!(
                &mut output,
                "  scaffold docs --source {}",
                shell_arg(&entry.name)
            );
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
