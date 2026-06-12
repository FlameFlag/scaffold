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
            "root": ctx.root_dir.display().to_string(),
            "bin": ctx.bin_dir.display().to_string(),
            "state": ctx.state_dir.display().to_string(),
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
            "Use read-only tools first: project_paths, render_reference, search_reference, eval_catalog, analyze, and run_tests. The server intentionally does not expose install or uninstall operations.",
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
