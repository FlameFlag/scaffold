use std::fmt::Write as _;

use comfy_table::{Cell, Color};
use scaffold_docs::{
    DocEntry, DocIndex, DocParam, entry_documentation, reference_entry_json,
    titled_markdown_for_entry,
};
use serde_json::json;

use crate::cli::{CliError, table::render_output_table};

use super::{DOC_TABLE_WIDTH, DocsBrowseFormat, json::to_pretty_json};

pub(crate) fn get_doc_entry<'a>(docs: &'a DocIndex, name: &str) -> Option<&'a DocEntry> {
    docs.get(name).filter(|entry| !entry.hidden).or_else(|| {
        docs.visible_entries()
            .find(|entry| entry.name.eq_ignore_ascii_case(name))
    })
}

pub(super) fn render_doc_entry(
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
    let _ = writeln!(&mut output, "{}", entry.name);
    let _ = writeln!(&mut output, "{}", "=".repeat(entry.name.len()));

    if let Some(signature) = documentation.signature {
        let _ = writeln!(&mut output, "\n{signature}");
    }

    if let Some(summary) = documentation.summary {
        push_section_break(&mut output);
        output.push_str(summary);
        output.push('\n');
    }

    if let Some(deprecated) = documentation.deprecated {
        push_section_break(&mut output);
        let _ = writeln!(&mut output, "Deprecated: {deprecated}");
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
    output.push_str(&render_detail_table(
        documentation
            .details
            .iter()
            .map(|detail| (detail.field, detail.value.clone())),
    ));

    if !documentation.see.is_empty() {
        push_section_break(&mut output);
        output.push_str("See also\n");
        let _ = writeln!(&mut output, "  {}", documentation.see.join(", "));
    }

    Ok(output)
}

pub(super) fn render_detail_table<'a>(rows: impl IntoIterator<Item = (&'a str, String)>) -> String {
    render_output_table(
        Some(DOC_TABLE_WIDTH),
        &["field", "value"],
        rows.into_iter()
            .map(|(field, value)| vec![Cell::new(field), Cell::new(value)]),
    )
}

fn render_param_table(params: &[DocParam]) -> String {
    render_output_table(
        Some(DOC_TABLE_WIDTH),
        &["parameter", "summary"],
        params.iter().map(|param| {
            vec![
                Cell::new(&param.name).fg(Color::Green),
                Cell::new(&param.summary),
            ]
        }),
    )
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
