use rmcp::{
    ErrorData as McpError,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::{Value, json};

use scaffold_catalog::Catalog;
use scaffold_context::Context;
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
            None => {
                self.require_catalog("evaluate the active catalog")?;
                self.catalog_path().to_path_buf()
            }
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
        self.require_catalog("list catalog tools")?;
        let ctx = self.context()?;
        let catalog = Catalog::load_with_mode(&ctx.catalog_path, ctx.catalog_mode.as_deref())
            .map_err(internal_error)?;
        let host = Host::current();
        Ok(structured(json!({
            "catalog": ctx.catalog_path.display().to_string(),
            "host": host,
            "tools": catalog_tool_list_json(&catalog, host),
        })))
    }

    #[tool(
        description = "Check whether active catalog tools are present on this host",
        annotations(read_only_hint = true)
    )]
    fn check_catalog_tools(&self) -> Result<CallToolResult, McpError> {
        self.require_catalog("check catalog tools")?;
        let ctx = self.context()?;
        let catalog = Catalog::load_with_mode(&ctx.catalog_path, ctx.catalog_mode.as_deref())
            .map_err(internal_error)?;
        let host = Host::current();
        Ok(structured(catalog_check_json(&ctx, &catalog, host)))
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

fn catalog_tool_list_json(catalog: &Catalog, host: Host) -> Vec<Value> {
    catalog
        .tools
        .iter()
        .map(|tool| {
            json!({
                "name": tool.name,
                "supports_host": tool.supports_host(host),
                "action": tool.action.label(),
                "phase": tool.phase().label(),
                "bins": tool.bin_names().collect::<Vec<_>>(),
                "meta": &tool.meta,
            })
        })
        .collect()
}

fn catalog_check_json(ctx: &Context, catalog: &Catalog, host: Host) -> Value {
    let tools = catalog
        .tools
        .iter()
        .map(|tool| {
            let presence = install::tool_presence_summary(ctx, tool, host);
            json!({
                "name": tool.name,
                "status": presence.status.label(),
                "version": presence.version,
            })
        })
        .collect::<Vec<_>>();
    let missing = tools
        .iter()
        .filter(|tool| tool["status"] == install::ToolPresenceStatus::Missing.label())
        .count();
    json!({
        "ok": missing == 0,
        "host": host,
        "missing": missing,
        "tools": tools,
    })
}

#[cfg(test)]
mod tests {
    use scaffold_catalog::Catalog;
    use scaffold_context::Context;
    use scaffold_platform::{Host, HostArch, HostOs};

    use super::{catalog_check_json, catalog_tool_list_json};

    #[test]
    fn tool_bin_names_preserve_catalog_bin_names() {
        let catalog = Catalog::from_value(serde_json::json!({
            "tools": [{
                "name": "demo",
                "bins": [{ "name": "demo" }, { "name": "democtl" }],
                "action": { "type": "required" }
            }]
        }))
        .expect("catalog");

        assert_eq!(
            catalog.tools[0].bin_names().collect::<Vec<_>>(),
            vec!["demo", "democtl"]
        );
    }

    #[test]
    fn catalog_tool_list_json_stays_static_without_version_probing() {
        let current_exe = std::env::current_exe().expect("current test executable");
        let catalog = Catalog::from_value(serde_json::json!({
            "tools": [{
                "name": "demo",
                "bins": [{ "name": current_exe.to_string_lossy(), "version_argv": [current_exe.to_string_lossy(), "--list"] }],
                "meta": {
                    "description": "Demo tool.",
                    "tags": ["fixture"]
                },
                "action": { "type": "required" }
            }]
        }))
        .expect("catalog");

        let tools = catalog_tool_list_json(
            &catalog,
            Host {
                os: HostOs::Linux,
                arch: HostArch::X86_64,
            },
        );

        assert_eq!(tools[0]["name"], "demo");
        assert_eq!(tools[0]["supports_host"], true);
        assert_eq!(tools[0]["action"], "required");
        assert_eq!(tools[0]["phase"], "prerequisites");
        assert_eq!(
            tools[0]["bins"],
            serde_json::json!([current_exe.to_string_lossy()])
        );
        assert_eq!(tools[0]["meta"]["description"], "Demo tool.");
        assert!(tools[0].get("version").is_none());
    }

    #[test]
    fn catalog_check_json_reports_ok_host_and_missing_count() {
        let current_exe = std::env::current_exe().expect("current test executable");
        let catalog = Catalog::from_value(serde_json::json!({
            "tools": [
                {
                    "name": "present",
                    "bins": [{ "name": current_exe.to_string_lossy(), "version_argv": [current_exe.to_string_lossy(), "--list"] }],
                    "checks": [{ "argv": [current_exe.to_string_lossy(), "--list"] }],
                    "action": { "type": "required" }
                },
                {
                    "name": "missing",
                    "bins": [{ "name": current_exe.to_string_lossy(), "version_argv": [current_exe.to_string_lossy(), "--list"] }],
                    "paths": [{ "path": "/definitely/not/a/real/scaffold/test/path" }],
                    "action": { "type": "required" }
                },
                {
                    "name": "unsupported",
                    "bins": [{ "name": current_exe.to_string_lossy(), "version_argv": [current_exe.to_string_lossy(), "--list"] }],
                    "platforms": ["windows"],
                    "action": { "type": "required" }
                }
            ]
        }))
        .expect("catalog");
        let ctx = Context {
            catalog_path: "catalog.scm".into(),
            catalog_mode: None,
            root_dir: ".".into(),
            bin_dir: ".".into(),
            state_dir: ".".into(),
        };
        let value = catalog_check_json(
            &ctx,
            &catalog,
            Host {
                os: HostOs::Linux,
                arch: HostArch::X86_64,
            },
        );

        assert_eq!(value["ok"], false);
        assert_eq!(value["missing"], 1);
        assert_eq!(value["host"]["os"], "linux");
        assert_eq!(value["tools"][0]["status"], "present");
        assert!(
            value["tools"][0]["version"]
                .as_str()
                .is_some_and(|version| !version.is_empty())
        );
        assert_eq!(value["tools"][1]["status"], "missing");
        assert_eq!(value["tools"][1]["version"], "");
        assert_eq!(value["tools"][2]["status"], "unsupported");
        assert_eq!(value["tools"][2]["version"], "");
    }
}
