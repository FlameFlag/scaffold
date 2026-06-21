use std::fmt::Write as _;

use scaffold_docs::{
    DocEntry, DocIndex, markdown_code_span, search_doc_entries, suggest_doc_entries,
};

use crate::cli::CliError;

use super::{
    DocsBrowseFormat,
    entries::{render_entry_markdown_table, render_entry_table},
    json::{entries_json_value, json_entries, to_pretty_json},
    text::{markdown_try_command, shell_arg},
};

pub(super) fn render_doc_search(
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
            response["suggestions"] = json_entries(suggestions.iter().copied());
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

pub(super) fn render_no_search_matches(
    query: &str,
    suggestions: &[&DocEntry],
    markdown: bool,
) -> String {
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
            let _ = writeln!(&mut output, "  scaffold docs {}", shell_arg(&entry.name));
        }
    }

    output
}
