use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::json;

use scaffold_catalog::Catalog;
use scaffold_install as install;
use scaffold_platform::Host;

use crate::{internal_error, server::ScaffoldMcp};

use super::structured;

pub(super) fn router() -> ToolRouter<ScaffoldMcp> {
    ScaffoldMcp::catalog_tools()
}

#[tool_router(router = catalog_tools)]
impl ScaffoldMcp {
    #[tool(
        description = "Evaluate a Scaffold catalog file and return the catalog JSON shape",
        annotations(read_only_hint = true)
    )]
    fn eval_catalog(
        &self,
        Parameters(args): Parameters<EvalCatalogRequest>,
    ) -> Result<CallToolResult, McpError> {
        let path = match args.path {
            Some(path) => self.resolve_path(&path)?,
            None => self.catalog_path().to_path_buf(),
        };
        let catalog = scaffold_dsl::catalog_value_from_path(&path).map_err(internal_error)?;
        Ok(structured(json!({
            "path": path.display().to_string(),
            "catalog": catalog,
        })))
    }

    #[tool(
        description = "List tools from the active Scaffold catalog with host support information",
        annotations(read_only_hint = true)
    )]
    fn list_catalog_tools(&self) -> Result<CallToolResult, McpError> {
        let ctx = self.context()?;
        let catalog = Catalog::load(&ctx.catalog_path).map_err(internal_error)?;
        let host = Host::current();
        let tools = catalog
            .tools
            .iter()
            .map(|tool| {
                json!({
                    "name": tool.name,
                    "supports_host": tool.supports_host(host),
                    "phase": format!("{:?}", tool.phase()),
                    "version": tool.version_summary(),
                    "meta": {
                        "home_page": &tool.meta.home_page,
                        "description": &tool.meta.description,
                        "license": &tool.meta.license,
                        "maintainers": &tool.meta.maintainers,
                        "tags": &tool.meta.tags,
                        "main_program": &tool.meta.main_program,
                        "source": &tool.meta.source,
                    },
                })
            })
            .collect::<Vec<_>>();
        Ok(structured(json!({
            "catalog": ctx.catalog_path.display().to_string(),
            "tools": tools,
        })))
    }

    #[tool(
        description = "Check whether active catalog tools are present on this host",
        annotations(read_only_hint = true)
    )]
    fn check_catalog_tools(&self) -> Result<CallToolResult, McpError> {
        let ctx = self.context()?;
        let catalog = Catalog::load(&ctx.catalog_path).map_err(internal_error)?;
        let host = Host::current();
        let mut missing = 0usize;
        let tools = catalog
            .tools
            .iter()
            .map(|tool| {
                let status = if !tool.supports_host(host) {
                    "unsupported"
                } else if install::tool_is_present(&ctx, tool) {
                    "present"
                } else {
                    missing += 1;
                    "missing"
                };
                json!({
                    "name": tool.name,
                    "status": status,
                    "version": tool.version_summary(),
                })
            })
            .collect::<Vec<_>>();
        Ok(structured(json!({
            "missing": missing,
            "tools": tools,
        })))
    }

    #[tool(
        description = "Return the active Scaffold catalog, root, bin, and state paths",
        annotations(read_only_hint = true)
    )]
    fn project_paths(&self) -> Result<CallToolResult, McpError> {
        Ok(structured(self.project_paths_json()?))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EvalCatalogRequest {
    #[schemars(description = "Optional catalog file path; defaults to the active catalog")]
    path: Option<String>,
}
