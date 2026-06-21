use std::fmt::Write as _;

use scaffold_docs::{DocIndex, entry_count_label, group_count_label, group_markdown_table};
use serde_json::json;

use crate::cli::CliError;

use super::{
    DocsBrowseFormat,
    group::{doc_group_counts, render_group_table},
    json::{group_counts_json, to_pretty_json},
    text::markdown_try_command,
};

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

pub(super) fn render_doc_overview(
    docs: &DocIndex,
    format: DocsBrowseFormat,
) -> Result<String, CliError> {
    let groups = doc_group_counts(docs);

    match format {
        DocsBrowseFormat::Json => {
            return Ok(to_pretty_json(&json!({
                "mode": "overview",
                "entry_count": groups.values().sum::<usize>(),
                "group_count": groups.len(),
                "groups": group_counts_json(groups.iter().map(|(group, count)| (*group, *count))),
            }))?);
        }
        DocsBrowseFormat::Markdown => {
            let mut output = String::from("# Scaffold Docs\n\n");
            let _ = writeln!(
                &mut output,
                "{} across {}.\n",
                entry_count_label(groups.values().sum()),
                group_count_label(groups.len())
            );
            output.push_str(&group_markdown_table(
                groups.iter().map(|(group, count)| (*group, *count)),
            ));
            push_try_examples_markdown(&mut output);
            return Ok(output);
        }
        DocsBrowseFormat::Text => {}
    }

    let mut output = String::from("Scaffold Docs\n\n");
    let _ = writeln!(
        &mut output,
        "{} across {}.\n",
        entry_count_label(groups.values().sum()),
        group_count_label(groups.len())
    );
    output.push_str(&render_group_table(
        groups.iter().map(|(group, count)| (*group, *count)),
    ));
    push_try_examples_text(&mut output);
    Ok(output)
}

fn push_try_examples_markdown(output: &mut String) {
    output.push_str("\n## Try\n\n");
    for example in DOCS_TRY_EXAMPLES {
        output.push_str(&markdown_try_command(example));
    }
}

fn push_try_examples_text(output: &mut String) {
    output.push_str("\nTry:\n");
    for example in DOCS_TRY_EXAMPLES {
        let _ = writeln!(output, "  {example}");
    }
}
