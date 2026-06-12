use std::path::{Path, PathBuf};

use scaffold_catalog::{Action, Tool};
use scaffold_context::Context;
use scaffold_platform::Host;
use scaffold_process as process;
use scaffold_template as template;

use super::InstallError;
use super::bindings::ToolBindings;

pub(super) fn uninstall_tool(
    ctx: &Context,
    tool: &Tool,
    dry_run: bool,
) -> Result<(), InstallError> {
    let host = Host::current();
    if !tool.supports_host(host) {
        println!("{}: unsupported on this host, skipping", tool.name);
        return Ok(());
    }

    let tool_bindings = ToolBindings::for_tool(ctx, tool, host);
    let bindings = tool_bindings.as_map();

    for argv in tool.uninstall.commands_for_host(host) {
        let argv = template::render_slice(argv, &bindings);
        println!("{}: {}", tool.name, argv.join(" "));
        if !dry_run {
            process::run(&argv)?;
        }
    }

    if tool
        .uninstall
        .remove_bins
        .unwrap_or(matches!(tool.action, Action::Build(_) | Action::Archive(_)))
    {
        for bin in &tool.bins {
            let path = ctx.bin_dir.join(process::executable_name(&bin.name));
            println!("{}: remove {}", tool.name, path.display());
            if !dry_run {
                remove_path_if_exists(tool, &path)?;
            }
        }
    }

    for path in tool.uninstall.paths_for_host(host) {
        let rendered = template::render(path, &bindings);
        let path = PathBuf::from(rendered);
        println!("{}: remove {}", tool.name, path.display());
        if !dry_run {
            remove_path_if_exists(tool, &path)?;
        }
    }

    let prefix = ctx.install_prefix(&tool.name);
    if tool
        .uninstall
        .remove_prefix
        .unwrap_or(matches!(tool.action, Action::Build(_) | Action::Archive(_)))
    {
        println!("{}: remove {}", tool.name, prefix.display());
        if !dry_run {
            remove_path_if_exists(tool, &prefix)?;
        }
    }

    Ok(())
}

fn remove_path_if_exists(tool: &Tool, path: &Path) -> Result<(), InstallError> {
    if unsafe_uninstall_path(path) {
        return Err(InstallError::UnsafeUninstallPath {
            tool: tool.name.clone(),
            path: path.to_path_buf(),
        });
    }
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        std::fs::remove_dir_all(path)?;
    } else {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

fn unsafe_uninstall_path(path: &Path) -> bool {
    path.as_os_str().is_empty() || path.parent().is_none()
}
