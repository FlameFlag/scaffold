use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::json;

use scaffold_dsl as dsl;

use crate::{internal_error, server::ScaffoldMcp};

use super::structured;

pub(super) fn router() -> ToolRouter<ScaffoldMcp> {
    ScaffoldMcp::evaluation_tools()
}

#[tool_router(router = evaluation_tools)]
impl ScaffoldMcp {
    #[tool(
        description = "Evaluate a Scaffold Scheme expression with the default Scaffold imports",
        annotations(read_only_hint = true)
    )]
    fn eval_expression(
        &self,
        Parameters(args): Parameters<EvalExpressionRequest>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = self.context()?;
        let session =
            dsl::session_with_extension_root(&ctx.root_dir, true).map_err(internal_error)?;
        let values = session
            .eval_json(&args.expression, Some("<mcp-eval>"))
            .map_err(internal_error)?;
        Ok(structured(json!({ "values": values })))
    }

    #[tool(
        description = "Evaluate a Scaffold Scheme file and return its JSON values",
        annotations(read_only_hint = true)
    )]
    fn eval_file(
        &self,
        Parameters(args): Parameters<EvalFileRequest>,
    ) -> Result<CallToolResult, McpError> {
        let path = self.resolve_path(&args.path)?;
        let values = dsl::values_from_path(&path).map_err(internal_error)?;
        Ok(structured(json!({
            "path": path.display().to_string(),
            "values": values,
        })))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EvalExpressionRequest {
    #[schemars(description = "Scaffold Scheme expression or top-level forms to evaluate")]
    expression: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct EvalFileRequest {
    #[schemars(description = "Scheme file path, absolute or relative to the active catalog root")]
    path: String,
}
