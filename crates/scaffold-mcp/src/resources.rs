use rmcp::{
    ErrorData as McpError,
    model::{AnnotateAble, ListResourcesResult, RawResource, ResourceContents},
};
use serde_json::json;

use scaffold_docs as docs;

use crate::internal_error;

use super::server::ScaffoldMcp;

const REFERENCE_MARKDOWN_URI: &str = "scaffold://reference/markdown";
const REFERENCE_JSON_URI: &str = "scaffold://reference/json";
const CATALOG_URI: &str = "scaffold://catalog";
const PATHS_URI: &str = "scaffold://paths";

pub(super) fn list() -> ListResourcesResult {
    ListResourcesResult::with_all_items(vec![
        RawResource::new(
            REFERENCE_MARKDOWN_URI,
            "Scaffold Scheme Reference (Markdown)",
        )
        .with_description("Generated Markdown reference for Scaffold Scheme.")
        .with_mime_type("text/markdown")
        .no_annotation(),
        RawResource::new(REFERENCE_JSON_URI, "Scaffold Scheme Reference (JSON)")
            .with_description("Generated structured JSON reference for Scaffold Scheme.")
            .with_mime_type("application/json")
            .no_annotation(),
        RawResource::new(CATALOG_URI, "Active Scaffold Catalog")
            .with_description("The Scheme source for the active catalog file.")
            .with_mime_type("text/x-scheme")
            .no_annotation(),
        RawResource::new(PATHS_URI, "Scaffold Paths")
            .with_description("Resolved catalog, root, bin, and state paths.")
            .with_mime_type("application/json")
            .no_annotation(),
    ])
}

pub(super) fn read(server: &ScaffoldMcp, uri: &str) -> Result<ResourceContents, McpError> {
    let contents = if uri == REFERENCE_MARKDOWN_URI {
        ResourceContents::text(docs::scaffold_reference_markdown(), REFERENCE_MARKDOWN_URI)
            .with_mime_type("text/markdown")
    } else if uri == REFERENCE_JSON_URI {
        ResourceContents::text(
            docs::scaffold_reference_json().map_err(internal_error)?,
            REFERENCE_JSON_URI,
        )
        .with_mime_type("application/json")
    } else if uri == CATALOG_URI {
        let source = std::fs::read_to_string(server.catalog_path()).map_err(internal_error)?;
        ResourceContents::text(source, CATALOG_URI).with_mime_type("text/x-scheme")
    } else if uri == PATHS_URI {
        ResourceContents::text(server.project_paths_json()?.to_string(), PATHS_URI)
            .with_mime_type("application/json")
    } else {
        return Err(McpError::resource_not_found(
            "unknown Scaffold MCP resource",
            Some(json!({ "uri": uri })),
        ));
    };
    Ok(contents)
}
