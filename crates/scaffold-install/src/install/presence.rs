use std::path::Path;

use scaffold_catalog::Tool;
use scaffold_context::Context;
use scaffold_platform::Host;
use scaffold_process as process;
use scaffold_template as template;

use super::InstallError;
use super::bindings::ToolBindings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolPresenceStatus {
    Present,
    Missing,
    Unsupported,
}

impl ToolPresenceStatus {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Missing => "missing",
            Self::Unsupported => "unsupported",
        }
    }
}

#[must_use]
pub fn tool_presence_status(ctx: &Context, tool: &Tool, host: Host) -> ToolPresenceStatus {
    if !tool.supports_host(host) {
        return ToolPresenceStatus::Unsupported;
    }
    if tool_is_present_on_host(ctx, tool, host) {
        ToolPresenceStatus::Present
    } else {
        ToolPresenceStatus::Missing
    }
}

#[must_use]
pub fn tool_is_present(ctx: &Context, tool: &Tool) -> bool {
    tool_is_present_on_host(ctx, tool, Host::current())
}

fn tool_is_present_on_host(ctx: &Context, tool: &Tool, host: Host) -> bool {
    if !required_paths_present(ctx, tool, host) || !explicit_checks_present(ctx, tool, host) {
        return false;
    }
    if tool_has_checks_for_host(tool, host) {
        return true;
    }
    tool.bins
        .iter()
        .all(|bin| process::path_of(&bin.name).is_some())
}

fn tool_has_checks_for_host(tool: &Tool, host: Host) -> bool {
    tool.checks_for_host(host).next().is_some()
}

fn explicit_checks_present(ctx: &Context, tool: &Tool, host: Host) -> bool {
    let tool_bindings = ToolBindings::for_tool(ctx, tool, host);
    let bindings = tool_bindings.as_map();

    tool.checks_for_host(host).all(|argv| {
        let argv = template::render_slice(argv, &bindings);
        process::capture(&argv).is_ok_and(|output| output.status.success())
    })
}

fn required_paths_present(ctx: &Context, tool: &Tool, host: Host) -> bool {
    let tool_bindings = ToolBindings::for_context(ctx);
    let bindings = tool_bindings.as_map();

    tool.required_paths_for_host(host).all(|path| {
        let rendered = template::render(path, &bindings);
        Path::new(&rendered).exists()
    })
}

pub(super) fn tool_is_present_after_install(
    ctx: &Context,
    tool: &Tool,
) -> Result<bool, InstallError> {
    Ok(tool_is_present(ctx, tool))
}
