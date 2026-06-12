use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::json;

use scaffold_docs as docs;
use scaffold_docs::{DocEntry, DocIndex, markdown_for_entry};

use crate::{internal_error, server::ScaffoldMcp};

use super::structured;

pub(super) fn router() -> ToolRouter<ScaffoldMcp> {
    ScaffoldMcp::reference_tools()
}

#[tool_router(router = reference_tools)]
impl ScaffoldMcp {
    #[tool(
        description = "Return generated Scaffold Scheme reference documentation",
        annotations(read_only_hint = true)
    )]
    fn render_reference(
        &self,
        Parameters(args): Parameters<RenderReferenceRequest>,
    ) -> Result<CallToolResult, McpError> {
        let format = args.format.unwrap_or_else(|| "markdown".to_owned());
        let value = match format.as_str() {
            "markdown" | "md" => json!({
                "format": "markdown",
                "content": docs::scaffold_reference_markdown(),
            }),
            "json" => json!({
                "format": "json",
                "content": serde_json::from_str::<serde_json::Value>(
                    &docs::scaffold_reference_json().map_err(internal_error)?
                ).map_err(internal_error)?,
            }),
            other => {
                return Err(McpError::invalid_params(
                    format!("unsupported reference format `{other}`"),
                    None,
                ));
            }
        };
        Ok(structured(value))
    }

    #[tool(
        description = "Search generated Scaffold Scheme reference entries",
        annotations(read_only_hint = true)
    )]
    fn search_reference(
        &self,
        Parameters(args): Parameters<SearchReferenceRequest>,
    ) -> Result<CallToolResult, McpError> {
        let query = args.query.to_ascii_lowercase();
        let limit = args.limit.unwrap_or(20).clamp(1, 100);
        let entries = DocIndex::scaffold()
            .visible_entries()
            .filter(|entry| reference_entry_matches(entry, &query))
            .take(limit)
            .map(|entry| {
                json!({
                    "name": entry.name,
                    "signature": entry.signature,
                    "summary": entry.summary,
                    "group": entry.group.as_deref().unwrap_or("Language"),
                    "source": entry.source,
                    "markdown": markdown_for_entry(entry),
                })
            })
            .collect::<Vec<_>>();
        Ok(structured(json!({
            "query": args.query,
            "entries": entries,
        })))
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
        description = "Case-insensitive query for symbol names, groups, summaries, and docs"
    )]
    query: String,
    #[schemars(description = "Maximum number of entries to return; default 20, max 100")]
    limit: Option<usize>,
}

fn reference_entry_matches(entry: &DocEntry, query: &str) -> bool {
    entry.name.to_ascii_lowercase().contains(query)
        || entry
            .signature
            .as_ref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(query))
        || entry
            .summary
            .as_ref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(query))
        || entry
            .markdown
            .as_ref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(query))
        || entry
            .group
            .as_ref()
            .is_some_and(|value| value.to_ascii_lowercase().contains(query))
}
