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
        ResourceContents::text(
            serde_json::to_string_pretty(&server.project_paths_json()?).map_err(internal_error)?,
            PATHS_URI,
        )
        .with_mime_type("application/json")
    } else {
        return Err(McpError::resource_not_found(
            "unknown Scaffold MCP resource",
            Some(json!({ "uri": uri })),
        ));
    };
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rmcp::model::ResourceContents;

    use super::*;

    #[test]
    fn list_includes_reference_and_project_resources() {
        let resources = list().resources;
        let uris = resources
            .iter()
            .map(|resource| resource.raw.uri.as_str())
            .collect::<Vec<_>>();

        assert!(uris.contains(&REFERENCE_MARKDOWN_URI));
        assert!(uris.contains(&REFERENCE_JSON_URI));
        assert!(uris.contains(&CATALOG_URI));
        assert!(uris.contains(&PATHS_URI));
    }

    #[test]
    fn paths_resource_returns_pretty_json_with_mime_type() {
        let root = tempfile::tempdir().expect("root");
        let server = ScaffoldMcp::new(root.path().join("scaffold.scm"), None);

        let contents = read(&server, PATHS_URI).expect("paths resource");
        let ResourceContents::TextResourceContents {
            text, mime_type, ..
        } = contents
        else {
            panic!("expected text resource contents");
        };
        let value: serde_json::Value = serde_json::from_str(&text).expect("paths json");

        assert_eq!(mime_type.as_deref(), Some("application/json"));
        assert!(text.starts_with("{\n"));
        assert_eq!(value["catalog_exists"], false);
        assert_eq!(value["root_exists"], true);
    }

    #[test]
    fn unknown_resource_reports_requested_uri() {
        let server = ScaffoldMcp::new(PathBuf::from("/workspace/scaffold.scm"), None);

        let err = read(&server, "scaffold://missing").expect_err("unknown resource");
        let message = err.to_string();

        assert!(message.contains("unknown Scaffold MCP resource"));
        assert!(message.contains("scaffold://missing"));
    }
}
