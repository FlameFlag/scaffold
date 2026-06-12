use std::path::Path;

use scaffold_archive as archive;
use scaffold_catalog::{Action, Tool};
use scaffold_context::Context;
use scaffold_platform::Host;
use scaffold_process as process;
use scaffold_template as template;

use super::bindings::ToolBindings;
use super::presence::{tool_is_present, tool_is_present_after_install};
use super::{InstallError, Policy};

pub(super) fn install_tool(ctx: &Context, tool: &Tool, policy: Policy) -> Result<(), InstallError> {
    let host = Host::current();
    if !tool.supports_host(host) {
        return Err(InstallError::UnsupportedHost {
            tool: tool.name.clone(),
        });
    }

    if matches!(tool.action, Action::Required) {
        if tool_is_present(ctx, tool) {
            println!("{}: present", tool.name);
            return Ok(());
        }
        return Err(InstallError::MissingRequired {
            tool: tool.name.clone(),
        });
    }

    if policy == Policy::Missing && tool_is_present(ctx, tool) {
        println!("{}: present, skipping", tool.name);
        return Ok(());
    }

    match &tool.action {
        Action::Required => unreachable!(),
        Action::Package(action) => {
            let package = action
                .for_host(host)
                .ok_or_else(|| InstallError::NoInstaller {
                    tool: tool.name.clone(),
                })?;
            let tool_bindings = ToolBindings::for_package(ctx, tool, package.name);
            let bindings = tool_bindings.as_map();
            for install_argv in package.install_argvs {
                let argv = template::render_slice(install_argv, &bindings);
                println!("{}: {}", tool.name, argv.join(" "));
                process::run(&argv)?;
            }
            Ok::<(), InstallError>(())
        }
        Action::Build(action) => {
            let command_argvs = action.command_argvs();
            if command_argvs.is_empty() {
                return Err(InstallError::NoBuildCommand {
                    tool: tool.name.clone(),
                });
            }
            let source_dir = ctx.root_dir.join(&action.path);
            let prefix = ctx.install_prefix(&tool.name);
            let tool_bindings = ToolBindings::for_build(ctx, tool, &source_dir);
            let bindings = tool_bindings.as_map();
            std::fs::create_dir_all(&prefix)?;
            std::fs::create_dir_all(prefix.join("bin"))?;
            for command_argv in command_argvs {
                let argv = template::render_slice(command_argv, &bindings);
                println!("{}: {}", tool.name, argv.join(" "));
                process::run_in(Some(&source_dir), &argv)?;
            }
            link_binaries(ctx, tool, &prefix)?;
            Ok(())
        }
        Action::Archive(action) => {
            let prefix = ctx.install_prefix(&tool.name);
            let archive_path = ctx.root_dir.join(&action.path);
            println!("{}: extract {}", tool.name, archive_path.display());
            std::fs::create_dir_all(&prefix)?;
            archive::extract_archive(&archive_path, &prefix, action.strip_components)?;
            link_binaries(ctx, tool, &prefix)?;
            Ok(())
        }
    }?;

    if tool.verify_after_install && !tool_is_present_after_install(ctx, tool)? {
        return Err(InstallError::VerificationFailed {
            tool: tool.name.clone(),
        });
    }

    Ok(())
}

fn link_binaries(ctx: &Context, tool: &Tool, prefix: &Path) -> Result<(), InstallError> {
    std::fs::create_dir_all(&ctx.bin_dir)?;
    for bin in &tool.bins {
        let source = prefix.join("bin").join(process::executable_name(&bin.name));
        if source.is_file() {
            let target = ctx.bin_dir.join(process::executable_name(&bin.name));
            replace_link_or_copy(&source, &target)?;
        }
    }
    Ok(())
}

fn replace_link_or_copy(source: &Path, target: &Path) -> std::io::Result<()> {
    remove_file_if_exists(target)?;
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(source, target)
    }
    #[cfg(not(unix))]
    {
        std::fs::copy(source, target).map(drop)
    }
}

fn remove_file_if_exists(path: &Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}
