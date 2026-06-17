use std::collections::BTreeMap;

use termimad::{
    CompoundStyle, MadSkin,
    crossterm::style::{Attribute, Color},
};

use scaffold_docs::{
    DocEntry, DocIndex, detailed_markdown_for_entry, entry_count_label,
    entry_summary_markdown_table, group_markdown_table, markdown_code_span, markdown_table,
    search_doc_entries, source_markdown_for_entry, suggest_doc_entries,
};

use crate::cli::docs::{get_doc_entry, get_doc_source_entries, search_doc_groups};

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
    let Some(entry) = get_doc_entry(docs, name) else {
        eprintln!("no documented symbol named `{name}`");
        if let Some(markdown) = doc_possible_matches_markdown(docs, name, ":doc") {
            print_repl_markdown(&markdown);
        }
        return;
    };
    print_repl_markdown(&doc_entry_markdown(entry));
}

fn print_doc_search(docs: &DocIndex, query: &str) {
    if query.is_empty() {
        eprintln!("usage: :search QUERY");
        return;
    }
    let matches = search_doc_entries(docs, query, 20);
    if matches.is_empty() {
        eprintln!("no reference entries matched `{query}`");
        if let Some(markdown) = doc_possible_matches_markdown(docs, query, ":doc") {
            print_repl_markdown(&markdown);
        }
        return;
    }
    print_repl_markdown(&doc_entries_markdown(
        &format!("Search results for {}", markdown_code_span(query)),
        &matches,
    ));
}

fn print_doc_groups(docs: &DocIndex) {
    print_repl_markdown(&doc_groups_markdown(docs));
}

fn print_doc_group(docs: &DocIndex, group: &str) {
    if group.is_empty() {
        eprintln!("usage: :group GROUP");
        return;
    }
    let mut entries = docs
        .visible_entries()
        .filter(|entry| entry.group_name().eq_ignore_ascii_case(group))
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    if entries.is_empty() {
        eprintln!("no documentation group named `{group}`");
        let suggestions = search_doc_groups(docs, group, 5);
        if !suggestions.is_empty() {
            print_repl_markdown(&doc_group_suggestions_markdown(
                "Did you mean",
                &suggestions,
            ));
        }
        return;
    }
    print_repl_markdown(&doc_entries_markdown(
        &doc_group_title(entries[0]),
        &entries,
    ));
}

fn print_doc_source(docs: &DocIndex, name: &str) {
    if name.is_empty() {
        eprintln!("usage: :source SYMBOL_OR_SOURCE");
        return;
    }
    if let Some(entry) = get_doc_entry(docs, name) {
        match source_markdown_for_entry(entry) {
            Some(markdown) => print_repl_markdown(&markdown),
            None => println!("no source recorded for `{name}`"),
        }
        return;
    }

    if let Some((source, entries)) = get_doc_source_entries(docs, name) {
        print_repl_markdown(&doc_entries_markdown(
            &format!("Docs from source {}", markdown_code_span(source)),
            &entries,
        ));
        return;
    }

    if name.ends_with(".scm") || name.contains(".scm:") {
        eprintln!("no documented source matched `{name}`");
    } else {
        eprintln!("no documented symbol named `{name}`");
        if let Some(markdown) = doc_possible_matches_markdown(docs, name, ":source") {
            print_repl_markdown(&markdown);
        }
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
    output.push_str(&markdown_table(
        &["Command", "Description"],
        REPL_COMMAND_SPECS
            .iter()
            .map(|command| {
                vec![
                    format!("`{}`", command.usage),
                    command.description.to_owned(),
                ]
            })
            .collect(),
    ));
    output.push_str("\nUse `Alt+Enter` for a newline. Use `:q`, `:quit`, or `(exit)` to leave.\n");
    output
}

fn doc_entry_markdown(entry: &DocEntry) -> String {
    detailed_markdown_for_entry(entry)
}

fn doc_groups_markdown(docs: &DocIndex) -> String {
    let mut groups = BTreeMap::<&str, usize>::new();
    for entry in docs.visible_entries() {
        *groups.entry(entry.group_name()).or_default() += 1;
    }
    let mut output = format!(
        "## Documentation groups\n\n{}.\n\n",
        entry_count_label(groups.values().sum::<usize>())
    );
    output.push_str(&group_markdown_table(
        groups
            .into_iter()
            .map(|(group, count)| (group, entry_count_label(count))),
    ));
    output
}

fn doc_group_suggestions_markdown(title: &str, groups: &[(&str, usize)]) -> String {
    let mut output = format!("## {title}\n\n");
    output.push_str(&group_markdown_table(
        groups
            .iter()
            .map(|(group, count)| (*group, entry_count_label(*count))),
    ));
    if let Some((group, _)) = groups.first() {
        output.push_str("\n## Try\n\n");
        output.push_str(&format!(
            "- {}\n",
            markdown_code_span(format!(":group {group}"))
        ));
    }
    output
}

fn doc_group_title(entry: &DocEntry) -> String {
    format!(
        "Documentation group {}",
        markdown_code_span(entry.group_name())
    )
}

fn doc_entries_markdown(title: &str, entries: &[&DocEntry]) -> String {
    let mut output = format!("## {title}\n\n{}.\n\n", entry_count_label(entries.len()));
    output.push_str(&entry_summary_markdown_table(entries.iter().copied()));
    output
}

fn doc_possible_matches_markdown(
    docs: &DocIndex,
    query: &str,
    try_command: &str,
) -> Option<String> {
    let mut matches = search_doc_entries(docs, query, 10);
    if matches.is_empty() {
        matches = suggest_doc_entries(docs, query, 5);
    }
    let first = matches.first()?;
    let mut output = doc_entries_markdown(
        &format!("Possible matches for {}", markdown_code_span(query)),
        &matches,
    );
    output.push_str("\n## Try\n\n");
    output.push_str(&format!(
        "- {}\n",
        markdown_code_span(format!("{try_command} {}", first.name))
    ));
    Some(output)
}

#[cfg(test)]
fn doc_entry_table_row(entry: &DocEntry) -> String {
    entry_summary_markdown_table([entry])
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
        description: "List doc groups, or search reference docs when QUERY is present.",
    },
    ReplCommandSpec {
        name: ":search",
        usage: ":search QUERY",
        description: "Search reference docs, examples, source paths or locations, and metadata.",
    },
    ReplCommandSpec {
        name: ":groups",
        usage: ":groups",
        description: "List documentation groups.",
    },
    ReplCommandSpec {
        name: ":group",
        usage: ":group GROUP",
        description: "List docs in one group.",
    },
    ReplCommandSpec {
        name: ":source",
        usage: ":source SYMBOL_OR_SOURCE",
        description: "Show where a documented symbol comes from, or list docs from a source file or location.",
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
