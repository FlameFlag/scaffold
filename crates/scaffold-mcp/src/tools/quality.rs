use std::path::PathBuf;

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
use scaffold_context::Context;
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
        let (ctx, files) = self.quality_paths(
            args.paths,
            QualityPathDefault::Tests {
                action: "run tests",
            },
        )?;
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
                Err(err) => results.push(test_failure_json(path.display().to_string(), err)),
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
        let (_ctx, files) = self.quality_paths(
            args.paths,
            QualityPathDefault::Sources {
                action: "analyze files",
            },
        )?;
        if files.is_empty() {
            return Ok(structured(json!({
                "ok": false,
                "message": "no Scheme files found to analyze",
                "files": [],
                "diagnostics": [],
            })));
        }

        let diagnostics = analyzer::analyze_paths(&files).map_err(internal_error)?;
        let has_errors = diagnostics.iter().any(SourceDiagnostic::is_error);
        let diagnostics = diagnostics
            .into_iter()
            .map(source_diagnostic_json)
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
        let (_ctx, files) = self.quality_paths(
            args.paths,
            QualityPathDefault::Sources {
                action: "check formatting",
            },
        )?;
        let checked = files
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>();
        let mut changed = Vec::new();
        for path in files {
            if fmt::format_path(&path, fmt::FormatMode::Check).map_err(internal_error)? {
                changed.push(path.display().to_string());
            }
        }
        Ok(structured(json!({
            "ok": changed.is_empty(),
            "files": checked,
            "changed": changed,
        })))
    }
}

enum QualityPathDefault {
    Tests { action: &'static str },
    Sources { action: &'static str },
}

impl QualityPathDefault {
    const fn action(&self) -> &'static str {
        match self {
            Self::Tests { action } | Self::Sources { action } => action,
        }
    }

    fn paths(&self, ctx: &Context) -> Vec<PathBuf> {
        match self {
            Self::Tests { .. } => ctx.test_paths(),
            Self::Sources { .. } => ctx.source_paths(),
        }
    }
}

impl ScaffoldMcp {
    fn quality_paths(
        &self,
        paths: Option<Vec<String>>,
        default: QualityPathDefault,
    ) -> Result<(Context, Vec<PathBuf>), McpError> {
        let ctx = self.context()?;
        let files = match paths {
            Some(paths) if !paths.is_empty() => paths
                .into_iter()
                .map(|path| resolve_quality_path(&ctx, &path))
                .collect(),
            _ => {
                self.require_catalog(default.action())?;
                default.paths(&ctx)
            }
        };
        Ok((ctx, files))
    }
}

fn resolve_quality_path(ctx: &Context, path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        ctx.root_dir.join(path)
    }
}

fn source_diagnostic_json(diagnostic: SourceDiagnostic) -> serde_json::Value {
    let error = diagnostic.is_error();
    let code = diagnostic.code_str();
    let severity = diagnostic.severity_label();
    let help = diagnostic.help_text().to_owned();
    let message = diagnostic.to_string();
    json!({
        "error": error,
        "code": code,
        "severity": severity,
        "message": message,
        "help": help,
        "report": format!("{:?}", Report::new(diagnostic)),
    })
}

fn test_failure_json(path: String, err: dsl::DslError) -> serde_json::Value {
    match err {
        dsl::DslError::Diagnostic(diagnostic) => {
            let error = diagnostic.to_string();
            let code = diagnostic.code_str();
            let severity = diagnostic.severity_label();
            let help = diagnostic.help_text().to_owned();
            json!({
                "path": path,
                "ok": false,
                "error": error,
                "code": code,
                "severity": severity,
                "help": help,
                "report": format!("{:?}", Report::new(dsl::DslError::Diagnostic(diagnostic))),
            })
        }
        err => json!({
            "path": path,
            "ok": false,
            "error": err.to_string(),
            "report": format!("{:?}", Report::new(err)),
        }),
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use scaffold_diagnostic::SourceDiagnostic;
    use scaffold_dsl as dsl;

    use crate::server::ScaffoldMcp;

    use super::{QualityPathDefault, source_diagnostic_json, test_failure_json};

    #[test]
    fn quality_paths_resolve_explicit_paths_without_catalog_requirement() {
        let root = std::env::temp_dir().join(format!(
            "scaffold-mcp-quality-explicit-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).expect("root");
        let server = ScaffoldMcp::new(root.join("missing-scaffold.scm"));

        let (_ctx, files) = server
            .quality_paths(
                Some(vec![
                    "relative.scm".to_owned(),
                    root.join("absolute.scm").display().to_string(),
                ]),
                QualityPathDefault::Sources {
                    action: "analyze files",
                },
            )
            .expect("explicit paths");

        assert_eq!(files[0], root.join("relative.scm"));
        assert_eq!(files[1], root.join("absolute.scm"));
    }

    #[test]
    fn quality_paths_require_catalog_for_discovered_defaults() {
        let server = ScaffoldMcp::new(PathBuf::from("/workspace/scaffold.scm"));

        let err = match server.quality_paths(
            None,
            QualityPathDefault::Sources {
                action: "analyze files",
            },
        ) {
            Ok(_) => panic!("missing catalog should fail"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("analyze files"));
    }

    #[test]
    fn quality_paths_discover_test_files_from_catalog_root() {
        let root =
            std::env::temp_dir().join(format!("scaffold-mcp-quality-tests-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(&root).expect("root");
        let catalog = root.join("scaffold.scm");
        let test = root.join("test.scm");
        std::fs::write(&catalog, "(import (rnrs))\n").expect("catalog");
        std::fs::write(&test, "(import (rnrs))\n").expect("test");
        let server = ScaffoldMcp::new(catalog);

        let (_ctx, files) = server
            .quality_paths(
                None,
                QualityPathDefault::Tests {
                    action: "run tests",
                },
            )
            .expect("test paths");

        assert_eq!(files, vec![test]);
    }

    #[test]
    fn source_diagnostic_json_includes_structured_fields_and_report() {
        let value = source_diagnostic_json(SourceDiagnostic::syntax(
            "test.scm",
            "(bad",
            0,
            1,
            "bad syntax",
        ));

        assert_eq!(value["error"], true);
        assert_eq!(value["code"], "scaffold::dsl::syntax");
        assert_eq!(value["severity"], "error");
        assert_eq!(value["message"], "bad syntax");
        assert!(
            value["help"]
                .as_str()
                .is_some_and(|help| help.contains("fix the Scheme syntax"))
        );
        assert!(
            value["report"]
                .as_str()
                .is_some_and(|report| report.contains("bad syntax"))
        );
    }

    #[test]
    fn test_failure_json_includes_diagnostic_fields_when_available() {
        let value = test_failure_json(
            "test.scm".to_owned(),
            SourceDiagnostic::syntax("test.scm", "(bad", 0, 1, "bad syntax").into(),
        );

        assert_eq!(value["path"], "test.scm");
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"], "bad syntax");
        assert_eq!(value["code"], "scaffold::dsl::syntax");
        assert_eq!(value["severity"], "error");
        assert!(
            value["help"]
                .as_str()
                .is_some_and(|help| help.contains("fix the Scheme syntax"))
        );
        assert!(
            value["report"]
                .as_str()
                .is_some_and(|report| report.contains("bad syntax"))
        );
    }

    #[test]
    fn test_failure_json_keeps_plain_errors_simple() {
        let value = test_failure_json(
            "test.scm".to_owned(),
            dsl::DslError::Eval("boom".to_owned()),
        );

        assert_eq!(value["path"], "test.scm");
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"], "Scheme evaluation failed: boom");
        assert!(value.get("code").is_none());
        assert!(value.get("severity").is_none());
        assert!(value.get("help").is_none());
    }
}
