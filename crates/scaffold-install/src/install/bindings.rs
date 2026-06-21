use std::collections::HashMap;
use std::path::Path;

use scaffold_catalog::{Action, Tool};
use scaffold_context::Context;
use scaffold_platform::Host;

#[derive(Debug)]
pub(super) struct ToolBindings {
    home: String,
    root_dir: String,
    bin_dir: String,
    state_dir: String,
    prefix: Option<String>,
    source_dir: Option<String>,
    tool: Option<String>,
    package: Option<String>,
}

impl ToolBindings {
    pub(super) fn for_context(ctx: &Context) -> Self {
        Self {
            home: std::env::var("HOME").unwrap_or_default(),
            root_dir: ctx.root_dir.to_string_lossy().into_owned(),
            bin_dir: ctx.bin_dir.to_string_lossy().into_owned(),
            state_dir: ctx.state_dir.to_string_lossy().into_owned(),
            prefix: None,
            source_dir: None,
            tool: None,
            package: None,
        }
    }

    pub(super) fn for_tool(ctx: &Context, tool: &Tool, host: Host) -> Self {
        let package = match &tool.action {
            Action::Package(action) => action.for_host(host).map(|package| package.name),
            Action::Required | Action::Build(_) | Action::Archive(_) => None,
        };
        let mut bindings = Self::for_context(ctx);
        bindings.prefix = Some(
            ctx.install_prefix(&tool.name)
                .to_string_lossy()
                .into_owned(),
        );
        bindings.tool = Some(tool.name.clone());
        bindings.package = Some(package.unwrap_or(tool.name.as_str()).to_owned());
        bindings
    }

    pub(super) fn for_package(ctx: &Context, tool: &Tool, package: &str) -> Self {
        let mut bindings = Self::for_context(ctx);
        bindings.prefix = Some(
            ctx.install_prefix(&tool.name)
                .to_string_lossy()
                .into_owned(),
        );
        bindings.tool = Some(tool.name.clone());
        bindings.package = Some(package.to_owned());
        bindings
    }

    pub(super) fn for_build(ctx: &Context, tool: &Tool, source_dir: &Path) -> Self {
        let mut bindings = Self::for_tool(ctx, tool, Host::current());
        bindings.source_dir = Some(source_dir.to_string_lossy().into_owned());
        bindings
    }

    pub(super) fn as_map(&self) -> HashMap<&str, &str> {
        let mut bindings = HashMap::from([
            ("home", self.home.as_str()),
            ("root_dir", self.root_dir.as_str()),
            ("bin_dir", self.bin_dir.as_str()),
            ("state_dir", self.state_dir.as_str()),
        ]);
        bindings.extend(
            [
                self.prefix.as_deref().map(|value| ("prefix", value)),
                self.source_dir
                    .as_deref()
                    .map(|value| ("source_dir", value)),
                self.tool.as_deref().map(|value| ("tool", value)),
                self.package.as_deref().map(|value| ("package", value)),
            ]
            .into_iter()
            .flatten(),
        );
        bindings
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use scaffold_catalog::Tool;

    use super::*;

    #[test]
    fn package_bindings_include_install_context_paths() {
        let root = PathBuf::from("/tmp/scaffold-bindings-test");
        let ctx = Context {
            catalog_path: root.join("catalog.scm"),
            root_dir: root.join("repo"),
            bin_dir: root.join("bin"),
            state_dir: root.join("state"),
        };
        let tool: Tool = serde_json::from_value(serde_json::json!({
            "name": "demo",
            "action": {
                "type": "package",
                "name": "demo-package",
                "install_argv": ["install", "{{ package }}", "{{ bin_dir }}"]
            }
        }))
        .expect("tool");

        let bindings = ToolBindings::for_package(&ctx, &tool, "demo-package");
        let map = bindings.as_map();

        assert_eq!(map["tool"], "demo");
        assert_eq!(map["package"], "demo-package");
        assert_eq!(map["root_dir"], "/tmp/scaffold-bindings-test/repo");
        assert_eq!(map["bin_dir"], "/tmp/scaffold-bindings-test/bin");
        assert_eq!(map["state_dir"], "/tmp/scaffold-bindings-test/state");
        assert_eq!(
            map["prefix"],
            "/tmp/scaffold-bindings-test/state/tools/demo/latest"
        );
    }
}
