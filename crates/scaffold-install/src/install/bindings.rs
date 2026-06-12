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

    pub(super) fn for_package(tool: &Tool, package: &str) -> Self {
        Self {
            home: std::env::var("HOME").unwrap_or_default(),
            root_dir: String::new(),
            bin_dir: String::new(),
            state_dir: String::new(),
            prefix: None,
            source_dir: None,
            tool: Some(tool.name.clone()),
            package: Some(package.to_owned()),
        }
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
        if let Some(prefix) = &self.prefix {
            let _previous = bindings.insert("prefix", prefix.as_str());
        }
        if let Some(source_dir) = &self.source_dir {
            let _previous = bindings.insert("source_dir", source_dir.as_str());
        }
        if let Some(tool) = &self.tool {
            let _previous = bindings.insert("tool", tool.as_str());
        }
        if let Some(package) = &self.package {
            let _previous = bindings.insert("package", package.as_str());
        }
        bindings
    }
}
