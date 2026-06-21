use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};

use scaffold_docs as docs;
use scaffold_docs::{DocEntry, DocIndex, search_doc_entries, suggest_doc_entries};

use crate::{internal_error, server::ScaffoldMcp};

use super::structured;

const DEFAULT_REFERENCE_SEARCH_LIMIT: usize = 20;
const MAX_REFERENCE_SEARCH_LIMIT: usize = 100;

pub(super) fn router() -> ToolRouter<ScaffoldMcp> {
    ScaffoldMcp::reference_tools()
}

#[tool_router(router = reference_tools)]
impl ScaffoldMcp {
    #[tool(
        description = "Return the full generated Scaffold Scheme reference; prefer search_reference for targeted lookups",
        annotations(read_only_hint = true)
    )]
    fn render_reference(
        &self,
        Parameters(args): Parameters<RenderReferenceRequest>,
    ) -> Result<CallToolResult, McpError> {
        let value = match reference_format(args.format.as_deref())? {
            ReferenceFormat::Markdown => json!({
                "format": "markdown",
                "content": docs::scaffold_reference_markdown(),
            }),
            ReferenceFormat::Json => json!({
                "format": "json",
                "content": docs::scaffold_reference_value().map_err(internal_error)?,
            }),
        };
        Ok(structured(value))
    }

    #[tool(
        description = "Search generated Scaffold Scheme reference entries; entries include raw markdown fields plus rendered_markdown/content_markdown for display",
        annotations(read_only_hint = true)
    )]
    fn search_reference(
        &self,
        Parameters(args): Parameters<SearchReferenceRequest>,
    ) -> Result<CallToolResult, McpError> {
        let query = args.query.trim();
        if query.is_empty() {
            return Err(McpError::invalid_params(
                "reference search query must not be empty",
                None,
            ));
        }
        let limit = reference_search_limit(args.limit)?;
        Ok(structured(reference_search_response(query, limit)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReferenceFormat {
    Markdown,
    Json,
}

fn reference_format(format: Option<&str>) -> Result<ReferenceFormat, McpError> {
    let Some(format) = format.map(str::trim).filter(|format| !format.is_empty()) else {
        return Ok(ReferenceFormat::Markdown);
    };
    match format.to_ascii_lowercase().as_str() {
        "markdown" | "md" => Ok(ReferenceFormat::Markdown),
        "json" => Ok(ReferenceFormat::Json),
        _ => Err(McpError::invalid_params(
            format!("unsupported reference format `{format}`"),
            None,
        )),
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct RenderReferenceRequest {
    #[schemars(description = "Reference format: markdown, md, or json")]
    format: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct SearchReferenceRequest {
    #[schemars(
        description = "Fuzzy query over names, groups, signatures, parameters, returns, docs, examples, see-also links, effects, required capabilities, lifecycle metadata, deprecation notes, source paths, and source locations"
    )]
    query: String,
    #[schemars(description = "Maximum number of entries to return; default 20, range 1-100")]
    limit: Option<usize>,
}

fn reference_search_limit(limit: Option<usize>) -> Result<usize, McpError> {
    let limit = limit.unwrap_or(DEFAULT_REFERENCE_SEARCH_LIMIT);
    if limit == 0 || limit > MAX_REFERENCE_SEARCH_LIMIT {
        return Err(McpError::invalid_params(
            format!("reference search limit must be between 1 and {MAX_REFERENCE_SEARCH_LIMIT}"),
            None,
        ));
    }
    Ok(limit)
}

fn reference_search_response(query: &str, limit: usize) -> Value {
    let index = DocIndex::scaffold();
    let matches = search_doc_entries(&index, query, limit);
    let suggestions = if matches.is_empty() {
        reference_entries_json(suggest_doc_entries(&index, query, 5))
    } else {
        Vec::new()
    };
    let entries = reference_entries_json(matches);
    let mut response = json!({
        "mode": "search",
        "query": query,
        "count": entries.len(),
        "limit": limit,
        "entries": entries,
    });
    if !suggestions.is_empty() {
        response["suggestions"] = json!(suggestions);
    }
    response
}

fn reference_entries_json<'a>(entries: impl IntoIterator<Item = &'a DocEntry>) -> Vec<Value> {
    entries.into_iter().map(reference_entry_json).collect()
}

fn reference_entry_json(entry: &DocEntry) -> Value {
    let mut value = docs::reference_entry_json(entry);
    value["content_markdown"] = value["rendered_markdown"].clone();
    value
}

#[cfg(test)]
mod tests {
    use scaffold_docs::DocIndex;
    use serde_json::json;

    use super::{
        ReferenceFormat, reference_format, reference_search_limit, reference_search_response,
        search_doc_entries,
    };

    #[test]
    fn reference_format_defaults_and_normalizes_input() {
        assert_eq!(
            reference_format(None).expect("format"),
            ReferenceFormat::Markdown
        );
        assert_eq!(
            reference_format(Some(" Markdown ")).expect("format"),
            ReferenceFormat::Markdown
        );
        assert_eq!(
            reference_format(Some("MD")).expect("format"),
            ReferenceFormat::Markdown
        );
        assert_eq!(
            reference_format(Some("JSON")).expect("format"),
            ReferenceFormat::Json
        );
        assert!(reference_format(Some("html")).is_err());
    }

    #[test]
    fn source_location_includes_recorded_line_number() {
        let index = DocIndex::scaffold();
        let entry = index.get("tool").expect("tool docs");
        let location = entry.display_source_location().expect("source location");

        assert!(location.starts_with("src/dsl/std/catalog/tool.scm:"));
    }

    #[test]
    fn reference_search_limit_rejects_out_of_range_values() {
        assert_eq!(reference_search_limit(None).expect("default limit"), 20);
        assert_eq!(reference_search_limit(Some(1)).expect("min limit"), 1);
        assert_eq!(reference_search_limit(Some(100)).expect("max limit"), 100);
        assert!(reference_search_limit(Some(0)).is_err());
        assert!(reference_search_limit(Some(101)).is_err());
    }

    #[test]
    fn reference_search_uses_shared_fuzzy_ranking() {
        let index = DocIndex::scaffold();
        let matches = search_doc_entries(&index, "ctlg tool", 20);

        assert!(matches.iter().any(|entry| entry.name == "catalog/tool"));
    }

    #[test]
    fn reference_search_response_matches_documented_examples() {
        let value = reference_search_response("ripgrep", 5);
        let entries = value["entries"].as_array().expect("entries");

        let catalog_tool = entries
            .iter()
            .find(|entry| entry["name"] == "catalog/tool")
            .expect("catalog/tool entry");
        assert_eq!(
            catalog_tool["raw_markdown"],
            "Prefer `tool` for ordinary catalog entries. Use `catalog/tool` when writing extension macros that need to splice fields directly into the raw catalog shape before Scaffold normalizes it."
        );
        assert_ne!(
            catalog_tool["raw_markdown"],
            catalog_tool["rendered_markdown"]
        );
        assert_eq!(catalog_tool["markdown"], catalog_tool["raw_markdown"]);
        assert_eq!(
            catalog_tool["content_markdown"],
            catalog_tool["rendered_markdown"]
        );
        assert!(value.get("suggestions").is_none());
    }

    #[test]
    fn reference_search_response_suggests_close_symbol_typos() {
        let value = reference_search_response("catlgtool", 20);

        assert_eq!(value["mode"], "search");
        assert_eq!(value["query"], "catlgtool");
        assert_eq!(value["count"], 0);
        assert_eq!(value["entries"].as_array().map(Vec::len), Some(0));
        assert_eq!(value["suggestions"][0]["name"], "catalog/tool");
        assert_eq!(
            value["suggestions"][0]["content_markdown"],
            value["suggestions"][0]["rendered_markdown"]
        );
    }

    #[test]
    fn reference_search_response_includes_count_limit_and_entry_metadata() {
        let value = reference_search_response("source/path", 1);

        assert_eq!(value["query"], "source/path");
        assert_eq!(value["limit"], 1);
        assert_eq!(value["count"], 1);
        assert_eq!(value["entries"][0]["name"], "source/path");
        assert_eq!(value["entries"][0]["kind"], "function");
        assert!(value["entries"][0]["markdown"].is_null());
        assert!(value["entries"][0]["raw_markdown"].is_null());
        assert_eq!(
            value["entries"][0]["content_markdown"],
            value["entries"][0]["rendered_markdown"]
        );
        assert!(
            value["entries"][0]["rendered_markdown"]
                .as_str()
                .is_some_and(|markdown| {
                    markdown.contains("```scheme\nsource/path\n```")
                        && markdown.contains("**Returns:**")
                })
        );
        assert_eq!(value["entries"][0]["effect"], "context-read-only");
        assert_eq!(
            value["entries"][0]["requires_capability"],
            json!(["scaffold.workspace"])
        );
        assert_eq!(
            value["entries"][0]["returns"],
            "A path string, or `#f` when no source path is available."
        );
        assert!(
            value["entries"][0]["source_location"]
                .as_str()
                .is_some_and(|source| source.starts_with("src/dsl/std/workspace.scm:"))
        );
        assert!(
            value["entries"][0]["range"]["length"]
                .as_u64()
                .is_some_and(|length| length > 0)
        );
    }

    #[test]
    fn reference_search_response_matches_source_locations() {
        let value = reference_search_response("src/dsl/std/catalog/tool.scm:16:1", 5);

        assert_eq!(value["mode"], "search");
        assert_eq!(value["query"], "src/dsl/std/catalog/tool.scm:16:1");
        assert!(
            value["entries"]
                .as_array()
                .is_some_and(|entries| entries.iter().any(|entry| entry["name"] == "tool"))
        );
    }
}
