mod entries;
mod entry;
mod group;
mod json;
mod overview;
mod search;
mod source;
mod text;

use std::path::Path;

use clap::ValueEnum;
use scaffold_docs::DocIndex;

use super::{
    CliError,
    args::{DocsArgs, DocsFormat},
};

#[cfg(test)]
use scaffold_docs::markdown_code_span;
#[cfg(test)]
use serde_json::{Value, json};

pub(crate) use entries::render_entry_markdown_table;
pub(crate) use entry::get_doc_entry;
use entry::render_doc_entry;
#[cfg(test)]
use group::doc_group_markdown_title;
use group::render_doc_group;
pub(crate) use group::search_doc_groups;
pub(crate) use group::{doc_group_counts, doc_group_entries};
use overview::render_doc_overview;
use search::render_doc_search;
pub(crate) use source::get_doc_source_entries;
use source::render_doc_source;
#[cfg(test)]
use text::{markdown_try_command, shell_arg};

pub(super) const DOC_TABLE_WIDTH: u16 = 100;
const DEFAULT_SEARCH_LIMIT: usize = 20;
pub(super) const MAX_SEARCH_LIMIT: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DocsBrowseFormat {
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
                "--all and --output export the full reference and cannot be combined with a docs                  query, --search, --group, or --source",
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
            "docs browse selectors cannot be combined; use one of a query, --search, --group, or              --source",
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

fn docs_export_format(args: &DocsArgs) -> Result<DocsFormat, CliError> {
    if let Some(format) = args.format {
        return Ok(format);
    }

    let Some(output) = args.output.as_deref() else {
        return Ok(DocsFormat::Markdown);
    };

    docs_format_from_path(output).ok_or_else(|| {
        CliError::message(format!(
            "cannot infer docs format from output path `{}`; use .md, .markdown, .json, or pass              --format",
            output.display()
        ))
    })
}

fn docs_format_from_path(path: &Path) -> Option<DocsFormat> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .and_then(|extension| DocsFormat::from_str(extension, true).ok())
}

#[cfg(test)]
mod tests;
