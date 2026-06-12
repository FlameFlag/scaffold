use std::collections::BTreeMap;

use nucleo_matcher::{
    Config as FuzzyConfig, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};
use scheme_rs::syntax::parse::{LexerError, ParseSyntaxError};
use termimad::{
    CompoundStyle, MadSkin,
    crossterm::style::{Attribute, Color},
};

use scaffold_docs::{DocEntry, DocIndex};

pub(super) enum ReplControl {
    Continue,
    Break,
}

pub(super) fn handle_repl_command(line: &str, docs: &DocIndex) -> ReplControl {
    let (command, rest) = split_repl_command(line);
    match command {
        ":help" | ",help" => print_repl_help(),
        ":quit" | ":q" => return ReplControl::Break,
        ":docs" => {
            if rest.is_empty() {
                print_doc_groups(docs);
            } else {
                print_doc_search(docs, rest);
            }
        }
        ":doc" => print_doc_entry(docs, rest),
        ":search" => print_doc_search(docs, rest),
        ":groups" => print_doc_groups(docs),
        ":group" => print_doc_group(docs, rest),
        ":source" => print_doc_source(docs, rest),
        "" => {}
        other => {
            eprintln!("unknown REPL command `{other}`");
            eprintln!("try `:help`");
        }
    }
    ReplControl::Continue
}

pub(super) fn split_repl_command(line: &str) -> (&str, &str) {
    let trimmed = line.trim();
    match trimmed.find(char::is_whitespace) {
        Some(index) => (&trimmed[..index], trimmed[index..].trim()),
        None => (trimmed, ""),
    }
}

fn print_repl_help() {
    print_repl_markdown(&repl_help_markdown());
}

fn print_doc_entry(docs: &DocIndex, name: &str) {
    if name.is_empty() {
        eprintln!("usage: :doc NAME");
        return;
    }
    let Some(entry) = docs.get(name) else {
        eprintln!("no docs for `{name}`");
        print_doc_search(docs, name);
        return;
    };
    print_repl_markdown(&doc_entry_markdown(entry));
}

fn print_doc_search(docs: &DocIndex, query: &str) {
    if query.is_empty() {
        eprintln!("usage: :search QUERY");
        return;
    }
    let matches = doc_search_results(docs, query, 20);
    if matches.is_empty() {
        eprintln!("no docs matched `{query}`");
        return;
    }
    print_repl_markdown(&doc_entries_markdown(
        &format!("Search results for `{query}`"),
        &matches,
    ));
}

fn print_doc_groups(docs: &DocIndex) {
    print_repl_markdown(&doc_groups_markdown(docs));
}

fn print_doc_group(docs: &DocIndex, group: &str) {
    if group.is_empty() {
        eprintln!("usage: :group NAME");
        return;
    }
    let mut entries = docs
        .visible_entries()
        .filter(|entry| doc_entry_group(entry).eq_ignore_ascii_case(group))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    if entries.is_empty() {
        eprintln!("no documentation group named `{group}`");
        return;
    }
    print_repl_markdown(&doc_entries_markdown(
        &format!(
            "Documentation group `{}`",
            entries[0].group.as_deref().unwrap_or(group)
        ),
        &entries,
    ));
}

fn print_doc_source(docs: &DocIndex, name: &str) {
    if name.is_empty() {
        eprintln!("usage: :source NAME");
        return;
    }
    let Some(entry) = docs.get(name) else {
        eprintln!("no docs for `{name}`");
        return;
    };
    match &entry.source {
        Some(source) => print_repl_markdown(&format!("**Source:** `{source}`")),
        None => println!("no source recorded for `{name}`"),
    }
}

fn print_repl_markdown(markdown: &str) {
    println!("{}", repl_doc_skin().term_text(markdown.trim()));
}

fn repl_doc_skin() -> MadSkin {
    let mut skin = MadSkin::default_dark();
    skin.set_headers_fg(Color::Cyan);
    skin.bold.set_fg(Color::Yellow);
    skin.italic.set_fg(Color::Magenta);
    skin.inline_code = CompoundStyle::with_fg(Color::Green);
    skin.inline_code.add_attr(Attribute::Bold);
    skin.code_block.left_margin = 2;
    skin
}

fn repl_help_markdown() -> String {
    let mut output = String::from("## REPL commands\n\n");
    for command in REPL_COMMAND_SPECS {
        output.push_str(&format!(
            "- `{}` - {}\n",
            command.usage, command.description
        ));
    }
    output.push_str("\nUse `Alt+Enter` for a newline. Use `:q`, `:quit`, or `(exit)` to leave.\n");
    output
}

fn doc_entry_markdown(entry: &DocEntry) -> String {
    let mut output = String::new();
    output.push_str(&format!("## `{}`\n\n", entry.name));

    if let Some(signature) = entry
        .signature
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        output.push_str("```scheme\n");
        output.push_str(signature);
        output.push_str("\n```\n");
    }

    let summary = entry
        .summary
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(summary) = summary {
        push_markdown_section_break(&mut output);
        output.push_str(summary);
        output.push('\n');
    }

    if let Some(deprecated) = entry
        .deprecated
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        push_markdown_section_break(&mut output);
        output.push_str(&format!("**Deprecated:** {deprecated}\n"));
    }

    let markdown = entry.markdown.as_deref().map(str::trim).filter(|s| {
        !s.is_empty() && summary.is_none_or(|summary| !same_markdown_paragraph(summary, s))
    });
    if let Some(markdown) = markdown {
        push_markdown_section_break(&mut output);
        output.push_str(markdown);
        output.push('\n');
    }

    if !entry.params.is_empty() {
        push_markdown_section_break(&mut output);
        output.push_str("### Parameters\n\n");
        for param in &entry.params {
            output.push_str(&format!("- `{}` - {}\n", param.name, param.summary));
        }
    }

    if let Some(returns) = entry
        .returns
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        push_markdown_section_break(&mut output);
        output.push_str("### Returns\n\n");
        output.push_str(returns);
        output.push('\n');
    }

    if let Some(example) = entry
        .example
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        push_markdown_section_break(&mut output);
        output.push_str("### Example\n\n```scheme\n");
        output.push_str(example);
        output.push_str("\n```\n");
    }

    let mut details = Vec::new();
    details.push(format!("Group: `{}`", doc_entry_group(entry)));
    if let Some(source) = &entry.source {
        details.push(format!("Source: `{source}`"));
    }
    if let Some(since) = &entry.since {
        details.push(format!("Since: `{since}`"));
    }
    if let Some(stability) = &entry.stability {
        details.push(format!("Stability: `{stability}`"));
    }
    if !details.is_empty() {
        push_markdown_section_break(&mut output);
        output.push_str("### Details\n\n");
        for detail in details {
            output.push_str(&format!("- {detail}\n"));
        }
    }

    if !entry.see.is_empty() {
        push_markdown_section_break(&mut output);
        output.push_str("### See also\n\n");
        output.push_str(
            &entry
                .see
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(", "),
        );
        output.push('\n');
    }

    if output.trim() == format!("## `{}`", entry.name) {
        push_markdown_section_break(&mut output);
        output.push_str("No documentation provided.\n");
    }
    output
}

fn doc_groups_markdown(docs: &DocIndex) -> String {
    let mut groups = BTreeMap::<&str, usize>::new();
    for entry in docs.visible_entries() {
        *groups.entry(doc_entry_group(entry)).or_default() += 1;
    }
    let mut output = format!(
        "## Documentation groups\n\n{}.\n\n",
        entry_count_label(groups.values().sum::<usize>())
    );
    for (group, count) in groups {
        output.push_str(&format!("- `{group}` - {}\n", entry_count_label(count)));
    }
    output
}

fn doc_entries_markdown(title: &str, entries: &[&DocEntry]) -> String {
    let mut output = format!("## {title}\n\n{}.\n\n", entry_count_label(entries.len()));
    for entry in entries {
        let signature = entry.signature.as_deref().unwrap_or(&entry.name);
        let summary = entry.summary.as_deref().unwrap_or("No summary.");
        output.push_str(&format!(
            "- `{}`  \n  `{}`  \n  {} - {}\n",
            entry.name,
            signature,
            doc_entry_group(entry),
            summary
        ));
    }
    output
}

#[cfg(test)]
fn doc_entry_summary_line(entry: &DocEntry) -> String {
    let summary = entry.summary.as_deref().unwrap_or("No summary.");
    let signature = entry.signature.as_deref().unwrap_or(&entry.name);
    format!("  {signature} - {summary}")
}

pub(super) fn doc_entry_group(entry: &DocEntry) -> &str {
    entry.group.as_deref().unwrap_or("Language")
}

fn entry_count_label(count: usize) -> String {
    match count {
        1 => "1 entry".to_owned(),
        count => format!("{count} entries"),
    }
}

fn push_markdown_section_break(output: &mut String) {
    if !output.is_empty() && !output.ends_with("\n\n") {
        if output.ends_with('\n') {
            output.push('\n');
        } else {
            output.push_str("\n\n");
        }
    }
}

fn same_markdown_paragraph(left: &str, right: &str) -> bool {
    normalize_markdown_paragraph(left) == normalize_markdown_paragraph(right)
}

fn normalize_markdown_paragraph(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
fn doc_entry_matches(entry: &DocEntry, query: &str) -> bool {
    let query = query.to_ascii_lowercase();
    entry.name.to_ascii_lowercase().contains(&query)
        || entry
            .signature
            .as_ref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(&query))
        || entry
            .summary
            .as_ref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(&query))
        || entry
            .markdown
            .as_ref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(&query))
}

fn doc_search_results<'a>(docs: &'a DocIndex, query: &str, limit: usize) -> Vec<&'a DocEntry> {
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);
    let mut matcher = Matcher::new(FuzzyConfig::DEFAULT);
    let mut matches = docs
        .visible_entries()
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

fn doc_entry_search_text(entry: &DocEntry) -> String {
    let mut parts = vec![
        entry.name.as_str(),
        doc_entry_group(entry),
        entry.signature.as_deref().unwrap_or_default(),
        entry.summary.as_deref().unwrap_or_default(),
        entry.markdown.as_deref().unwrap_or_default(),
        entry.returns.as_deref().unwrap_or_default(),
        entry.source.as_deref().unwrap_or_default(),
    ];
    parts.extend(entry.params.iter().map(|param| param.name.as_str()));
    parts.extend(entry.params.iter().map(|param| param.summary.as_str()));
    parts.extend(entry.see.iter().map(String::as_str));
    parts.join(" ")
}

pub(super) const fn parse_error_is_incomplete(error: &ParseSyntaxError) -> bool {
    matches!(
        error,
        ParseSyntaxError::UnexpectedEof
            | ParseSyntaxError::ExpectedClosingParen { .. }
            | ParseSyntaxError::UnclosedParen { .. }
            | ParseSyntaxError::Lex(LexerError::UnexpectedEof)
    )
}

pub(super) struct ReplCommandSpec {
    pub(super) name: &'static str,
    pub(super) usage: &'static str,
    pub(super) description: &'static str,
}

pub(super) const REPL_COMMAND_SPECS: &[ReplCommandSpec] = &[
    ReplCommandSpec {
        name: ":help",
        usage: ":help",
        description: "Show REPL commands.",
    },
    ReplCommandSpec {
        name: ":doc",
        usage: ":doc NAME",
        description: "Show reference docs for one symbol.",
    },
    ReplCommandSpec {
        name: ":docs",
        usage: ":docs [QUERY]",
        description: "List doc groups, or search docs when QUERY is present.",
    },
    ReplCommandSpec {
        name: ":search",
        usage: ":search QUERY",
        description: "Search names, signatures, summaries, and long docs.",
    },
    ReplCommandSpec {
        name: ":groups",
        usage: ":groups",
        description: "List documentation groups.",
    },
    ReplCommandSpec {
        name: ":group",
        usage: ":group NAME",
        description: "List docs in one group.",
    },
    ReplCommandSpec {
        name: ":source",
        usage: ":source NAME",
        description: "Show where a documented symbol comes from.",
    },
    ReplCommandSpec {
        name: ":quit",
        usage: ":quit",
        description: "Exit the REPL.",
    },
    ReplCommandSpec {
        name: ":q",
        usage: ":q",
        description: "Exit the REPL.",
    },
];

#[cfg(test)]
mod tests;
