use miette::Report;
use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::json;

use scaffold_analyzer as analyzer;
use scaffold_diagnostic::SourceDiagnostic;
use scaffold_dsl as dsl;
use scaffold_fmt as fmt;

use crate::{internal_error, server::ScaffoldMcp};

use super::structured;

pub(super) fn router() -> ToolRouter<ScaffoldMcp> {
    ScaffoldMcp::quality_tools()
}

#[tool_router(router = quality_tools)]
impl ScaffoldMcp {
    #[tool(
        description = "Run Scaffold Scheme tests; defaults to project test.scm files",
        annotations(read_only_hint = true)
    )]
    fn run_tests(
        &self,
        Parameters(args): Parameters<TestRequest>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = self.context()?;
        let files = match args.paths {
            Some(paths) if !paths.is_empty() => paths
                .into_iter()
                .map(|path| self.resolve_path(&path))
                .collect::<Result<Vec<_>, _>>()?,
            _ => ctx.test_paths(),
        };
        if files.is_empty() {
            return Ok(structured(json!({
                "ok": false,
                "message": "no test files found",
                "files": [],
            })));
        }

        let mut results = Vec::new();
        for path in files {
            match dsl::values_from_path_with_catalog_path(&path, &ctx.catalog_path) {
                Ok(_) => results.push(json!({
                    "path": path.display().to_string(),
                    "ok": true,
                })),
                Err(err) => results.push(json!({
                    "path": path.display().to_string(),
                    "ok": false,
                    "error": err.to_string(),
                    "report": format!("{:?}", Report::new(err)),
                })),
            }
        }
        let ok = results
            .iter()
            .all(|result| result["ok"].as_bool().unwrap_or(false));
        Ok(structured(json!({ "ok": ok, "results": results })))
    }

    #[tool(
        description = "Analyze Scaffold Scheme files for static diagnostics",
        annotations(read_only_hint = true)
    )]
    fn analyze(
        &self,
        Parameters(args): Parameters<AnalyzeRequest>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = self.context()?;
        let files = match args.paths {
            Some(paths) if !paths.is_empty() => paths
                .into_iter()
                .map(|path| self.resolve_path(&path))
                .collect::<Result<Vec<_>, _>>()?,
            _ => ctx.source_paths(),
        };
        if files.is_empty() {
            return Ok(structured(json!({
                "ok": false,
                "message": "no Scheme files found to analyze",
                "diagnostics": [],
            })));
        }

        let diagnostics = analyzer::analyze_paths(&files).map_err(internal_error)?;
        let has_errors = diagnostics.iter().any(SourceDiagnostic::is_error);
        let diagnostics = diagnostics
            .into_iter()
            .map(|diagnostic| {
                json!({
                    "error": diagnostic.is_error(),
                    "message": diagnostic.to_string(),
                    "report": format!("{:?}", Report::new(diagnostic)),
                })
            })
            .collect::<Vec<_>>();
        Ok(structured(json!({
            "ok": !has_errors,
            "files": files.iter().map(|path| path.display().to_string()).collect::<Vec<_>>(),
            "diagnostics": diagnostics,
        })))
    }

    #[tool(
        description = "Format Scaffold Scheme source text without writing to disk",
        annotations(read_only_hint = true)
    )]
    fn format_source(
        &self,
        Parameters(args): Parameters<FormatSourceRequest>,
    ) -> Result<CallToolResult, McpError> {
        let formatted = fmt::format_text(&args.source).map_err(internal_error)?;
        Ok(structured(json!({
            "changed": formatted != args.source,
            "formatted": formatted,
        })))
    }

    #[tool(
        description = "Check which Scaffold Scheme files would be reformatted without writing to disk",
        annotations(read_only_hint = true)
    )]
    fn format_check(
        &self,
        Parameters(args): Parameters<FormatCheckRequest>,
    ) -> Result<CallToolResult, McpError> {
        let ctx = self.context()?;
        let files = match args.paths {
            Some(paths) if !paths.is_empty() => paths
                .into_iter()
                .map(|path| self.resolve_path(&path))
                .collect::<Result<Vec<_>, _>>()?,
            _ => ctx.source_paths(),
        };
        let mut changed = Vec::new();
        for path in files {
            if fmt::format_path(&path, fmt::FormatMode::Check).map_err(internal_error)? {
                changed.push(path.display().to_string());
            }
        }
        Ok(structured(json!({
            "ok": changed.is_empty(),
            "changed": changed,
        })))
    }
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct TestRequest {
    #[schemars(description = "Optional test file paths; defaults to discovered test.scm files")]
    paths: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct AnalyzeRequest {
    #[schemars(
        description = "Optional Scheme file paths; defaults to the active catalog and extensions"
    )]
    paths: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FormatSourceRequest {
    #[schemars(description = "Scaffold Scheme source text to format")]
    source: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct FormatCheckRequest {
    #[schemars(
        description = "Optional Scheme file paths; defaults to the active catalog and extensions"
    )]
    paths: Option<Vec<String>>,
}
