use std::collections::BTreeMap;
use std::fmt::Write as _;

use comfy_table::Cell;
use nucleo_matcher::{
    Config as FuzzyConfig, Matcher, Utf32Str,
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
};
use scaffold_docs::{
    DocEntry, DocIndex, group_markdown_table, markdown_code_span, markdown_text,
    normalize_reference_query_token,
};
use serde_json::json;

use crate::cli::{CliError, table::render_output_table};

use super::{
    DocsBrowseFormat,
    entries::{render_entry_markdown_table, render_entry_table},
    json::{entries_json_response, group_counts_json, to_pretty_json},
    text::{markdown_try_command, shell_arg},
};

pub(super) fn render_doc_group(
    docs: &DocIndex,
    group: &str,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    let entries = doc_group_entries(docs, group);

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

pub(crate) fn doc_group_entries<'a>(docs: &'a DocIndex, group: &str) -> Vec<&'a DocEntry> {
    docs.visible_entries()
        .filter(|entry| entry.group_name().eq_ignore_ascii_case(group))
        .collect()
}

pub(super) fn render_group_table<'a>(groups: impl IntoIterator<Item = (&'a str, usize)>) -> String {
    render_output_table(
        None,
        &["group", "entries"],
        groups
            .into_iter()
            .map(|(group, count)| vec![Cell::new(group), Cell::new(count)]),
    )
}

pub(crate) fn doc_group_counts(docs: &DocIndex) -> BTreeMap<&str, usize> {
    docs.visible_entries()
        .fold(BTreeMap::<&str, usize>::new(), |mut groups, entry| {
            *groups.entry(entry.group_name()).or_default() += 1;
            groups
        })
}

pub(crate) fn search_doc_groups<'a>(
    docs: &'a DocIndex,
    query: &str,
    limit: usize,
) -> Vec<(&'a str, usize)> {
    let query = normalize_reference_query_token(query);
    if query.is_empty() {
        return Vec::new();
    }
    let pattern = Pattern::new(
        &query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let mut matcher = Matcher::new(FuzzyConfig::DEFAULT);
    let mut buf = Vec::new();
    let mut matches = doc_group_counts(docs)
        .into_iter()
        .filter_map(|(group, count)| {
            doc_group_match_score(group, &query, &pattern, &mut matcher, &mut buf)
                .map(|score| (group, count, score))
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

pub(super) fn doc_group_markdown_title(group: &str) -> String {
    format!("{} docs", markdown_text(group))
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
            "suggestions": group_counts_json(suggestions.iter().copied()),
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
    output.push_str(&render_group_table(suggestions.iter().copied()));
    output.push_str("\nTry:\n");
    let _ = writeln!(
        &mut output,
        "  scaffold docs --group {}",
        shell_arg(suggested_group)
    );
    Ok(output)
}

fn doc_group_match_score(
    group: &str,
    query: &str,
    pattern: &Pattern,
    matcher: &mut Matcher,
    buf: &mut Vec<char>,
) -> Option<usize> {
    let group = normalize_reference_query_token(group);
    if group == query {
        return Some(10_000 + query.len());
    }
    if group.starts_with(query) {
        return Some(9_000 + query.len());
    }
    if group.contains(query) {
        return Some(8_000 + query.len());
    }
    if query.chars().count() < 5 {
        return None;
    }
    pattern
        .score(Utf32Str::new(&group, buf), matcher)
        .map(|score| score as usize)
}
