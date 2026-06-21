use comfy_table::{Cell, Color};
use scaffold_docs::{DocEntry, entry_count_label, entry_summary_markdown_table};

use crate::cli::table::render_output_table;

use super::DOC_TABLE_WIDTH;

pub(super) fn render_entry_table(title: &str, entries: &[&DocEntry]) -> String {
    let mut output = format!("{title}\n\n{}.\n\n", entry_count_label(entries.len()));
    output.push_str(&render_output_table(
        Some(DOC_TABLE_WIDTH),
        &["symbol", "group", "summary"],
        entries.iter().map(|entry| {
            vec![
                Cell::new(&entry.name).fg(Color::Green),
                Cell::new(entry.group_name()),
                Cell::new(entry.summary.as_deref().unwrap_or("No summary.")),
            ]
        }),
    ));
    output
}

pub(crate) fn render_entry_markdown_table(title: &str, entries: &[&DocEntry]) -> String {
    let mut output = format!("## {title}\n\n{}.\n\n", entry_count_label(entries.len()));
    output.push_str(&entry_summary_markdown_table(entries.iter().copied()));
    output
}
