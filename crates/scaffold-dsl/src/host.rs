use scheme_rs::{exceptions::Exception, registry::bridge, value::Value};

use scaffold_process as process;
use scaffold_scheme::value_to_string;

use super::literal::scheme_string;

const HOST_LIBRARY_TEMPLATE: &str = include_str!("std/host.scm");

#[derive(Clone, Debug)]
pub(super) struct Host {
    os: &'static str,
    arch: &'static str,
}

impl Host {
    pub(super) const fn current() -> Self {
        Self {
            os: host_os_symbol(),
            arch: host_arch_symbol(),
        }
    }

    fn platform(&self) -> String {
        format!("{}-{}", self.os, self.arch)
    }
}

pub(super) fn host_library_source(host: &Host) -> String {
    HOST_LIBRARY_TEMPLATE
        .replace("@HOST_OS@", host.os)
        .replace("@HOST_ARCH@", host.arch)
        .replace("@HOST_PLATFORM@", &scheme_string(&host.platform()))
}

#[bridge(name = "%command/available?", lib = "(scaffold host builtins)")]
pub fn command_available(name: &Value) -> Result<Vec<Value>, Exception> {
    let name = value_to_string(name)?;
    Ok(vec![Value::from(process::path_of(&name).is_some())])
}

#[bridge(name = "%command/path", lib = "(scaffold host builtins)")]
pub fn command_path(name: &Value) -> Result<Vec<Value>, Exception> {
    let name = value_to_string(name)?;
    Ok(vec![process::path_of(&name).map_or_else(
        || Value::from(false),
        |path| Value::from(path.to_string_lossy().into_owned()),
    )])
}

#[bridge(name = "%command/available", lib = "(scaffold host builtins)")]
pub fn available_commands() -> Result<Vec<Value>, Exception> {
    Ok(vec![Value::from(
        process::available_commands()
            .into_iter()
            .map(Value::from)
            .collect::<Vec<_>>(),
    )])
}

#[bridge(name = "%env/var", lib = "(scaffold host builtins)")]
pub fn env_var(name: &Value) -> Result<Vec<Value>, Exception> {
    let name = value_to_string(name)?;
    Ok(vec![
        std::env::var(name).map_or_else(|_| Value::from(false), Value::from),
    ])
}

const fn host_os_symbol() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

const fn host_arch_symbol() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "x86_64"
    }
}
