use std::path::Path;

use scaffold_catalog::Tool;
use scaffold_context::Context;
use scaffold_platform::Host;
use scaffold_process as process;
use scaffold_template as template;

use super::InstallError;
use super::bindings::ToolBindings;

#[must_use]
pub fn tool_is_present(ctx: &Context, tool: &Tool) -> bool {
    if !required_paths_present(ctx, tool) || !explicit_checks_present(ctx, tool) {
        return false;
    }
    if tool_has_checks_for_host(tool) {
        return true;
    }
    tool.bins
        .iter()
        .all(|bin| process::path_of(&bin.name).is_some())
}

fn tool_has_checks_for_host(tool: &Tool) -> bool {
    tool.checks_for_host(Host::current()).next().is_some()
}

fn explicit_checks_present(ctx: &Context, tool: &Tool) -> bool {
    let host = Host::current();
    let tool_bindings = ToolBindings::for_tool(ctx, tool, host);
    let bindings = tool_bindings.as_map();

    tool.checks_for_host(host).all(|argv| {
        let argv = template::render_slice(argv, &bindings);
        process::capture(&argv).is_ok_and(|output| output.status.success())
    })
}

fn required_paths_present(ctx: &Context, tool: &Tool) -> bool {
    let tool_bindings = ToolBindings::for_context(ctx);
    let bindings = tool_bindings.as_map();

    tool.required_paths_for_host(Host::current()).all(|path| {
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
