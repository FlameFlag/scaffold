use scheme_rs::{exceptions::Exception, registry::bridge, value::Value};

use scaffold_process as process;
use scaffold_scheme::value_to_string;
use scaffold_template as template;

const HOST_LIBRARY_TEMPLATE: &str = include_str!("std/host.scm");

pub(super) use scaffold_platform::Host;

pub(super) fn host_library_source(host: &Host) -> String {
    let bindings = std::collections::HashMap::from([
        ("host_os", host.os.label()),
        ("host_arch", host.arch.label()),
    ]);
    template::render(HOST_LIBRARY_TEMPLATE, &bindings)
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
