use std::path::{Path, PathBuf};

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::router::{prompt::PromptRouter, tool::ToolRouter},
    model::{
        GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
        ListResourcesResult, PaginatedRequestParams, ReadResourceRequestParams, ReadResourceResult,
        ServerCapabilities, ServerInfo,
    },
    prompt_handler,
    service::RequestContext,
    tool_handler,
};
use serde_json::json;

use scaffold_context::Context;

use super::{internal_error, prompts, resources, tools};

#[derive(Debug, Clone)]
pub(super) struct ScaffoldMcp {
    catalog_path: PathBuf,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

impl ScaffoldMcp {
    pub(super) fn new(catalog_path: PathBuf) -> Self {
        Self {
            catalog_path,
            tool_router: tools::router(),
            prompt_router: prompts::router(),
        }
    }

    pub(super) fn context(&self) -> Result<Context, McpError> {
        Context::new(self.catalog_path.clone()).map_err(internal_error)
    }

    pub(super) fn require_catalog(&self, action: &str) -> Result<(), McpError> {
        if self.catalog_path.is_file() {
            return Ok(());
        }

        Err(internal_error(format!(
            "no catalog found at {}; start scaffold mcp with --catalog to {action}",
            self.catalog_path.display()
        )))
    }

    pub(super) fn resolve_path(&self, path: &str) -> Result<PathBuf, McpError> {
        let ctx = self.context()?;
        let path = PathBuf::from(path);
        Ok(if path.is_absolute() {
            path
        } else {
            ctx.root_dir.join(path)
        })
    }

    pub(super) fn catalog_path(&self) -> &Path {
        &self.catalog_path
    }

    pub(super) fn project_paths_json(&self) -> Result<serde_json::Value, McpError> {
        let ctx = self.context()?;
        Ok(json!({
            "catalog": ctx.catalog_path.display().to_string(),
            "catalog_exists": ctx.catalog_path.exists(),
            "root": ctx.root_dir.display().to_string(),
            "root_exists": ctx.root_dir.exists(),
            "bin": ctx.bin_dir.display().to_string(),
            "bin_exists": ctx.bin_dir.exists(),
            "state": ctx.state_dir.display().to_string(),
            "state_exists": ctx.state_dir.exists(),
        }))
    }
}

#[prompt_handler(router = self.prompt_router)]
#[tool_handler(router = self.tool_router)]
impl ServerHandler for ScaffoldMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
        .with_server_info(
            Implementation::new(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                .with_title("Scaffold MCP")
                .with_description("MCP access to Scaffold Scheme evaluation, catalog inspection, tests, analysis, formatting, and reference docs."),
        )
        .with_instructions(
            "Use read-only tools first: project_paths, search_reference for targeted reference lookups, render_reference only when the full reference export is needed, eval_catalog, analyze, and run_tests. The server intentionally does not expose install or uninstall operations.",
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(resources::list())
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        Ok(ReadResourceResult::new(vec![resources::read(
            self,
            &request.uri,
        )?]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_catalog_reports_missing_active_catalog() {
        let server = ScaffoldMcp::new(PathBuf::from("/workspace/scaffold.scm"));

        let err = server
            .require_catalog("evaluate expressions")
            .expect_err("missing catalog should fail");
        let message = err.to_string();

        assert!(message.contains("no catalog found at /workspace/scaffold.scm"));
        assert!(message.contains("--catalog"));
        assert!(message.contains("evaluate expressions"));
    }

    #[test]
    fn server_instructions_prefer_reference_search_over_full_render() {
        let server = ScaffoldMcp::new(PathBuf::from("/workspace/scaffold.scm"));
        let info = server.get_info();
        let instructions = info.instructions.expect("instructions");

        assert!(instructions.contains("search_reference for targeted reference lookups"));
        assert!(
            instructions.contains("render_reference only when the full reference export is needed")
        );
    }

    #[test]
    fn project_paths_include_existence_status() {
        let root = std::env::temp_dir().join(format!(
            "scaffold-mcp-paths-{}-missing-catalog",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("root");
        let catalog_path = root.join("scaffold.scm");
        let server = ScaffoldMcp::new(catalog_path);

        let paths = server.project_paths_json().expect("paths");

        assert_eq!(paths["catalog_exists"], false);
        assert_eq!(paths["root_exists"], true);
        assert!(paths.get("bin_exists").is_some());
        assert!(paths.get("state_exists").is_some());
    }
}
