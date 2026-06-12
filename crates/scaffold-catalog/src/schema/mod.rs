use std::collections::BTreeMap;

use serde_json::Value;

mod description;
mod relationships;

pub use description::catalog_schema;

use description::{
    ARCHIVE_ACTION_FIELDS, BIN_FIELDS, BUILD_ACTION_FIELDS, CHECK_FIELDS, PACKAGE_ACTION_FIELDS,
    PACKAGE_PLATFORM_FIELDS, PREDICATE_FIELDS, REQUIRED_ACTION_FIELDS, TOOL_FIELDS,
    TOOL_META_FIELDS, TOOL_PATH_FIELDS, UNINSTALL_COMMAND_FIELDS, UNINSTALL_FIELDS,
};
use relationships::{validate_install_order, validate_tool_references};

pub(crate) fn validate_catalog_value(value: &Value) -> Result<(), String> {
    let root = expect_object(value, "$")?;
    unknown_fields(root, "$", &["tools"])?;
    let tools = expect_array(required(root, "tools", "$")?, "$.tools")?;
    if tools.is_empty() {
        return Err("$.tools must contain at least one tool".to_owned());
    }

    let mut names = BTreeMap::<String, usize>::new();
    for (index, tool) in tools.iter().enumerate() {
        let path = format!("$.tools[{index}]");
        validate_tool(tool, &path)?;
        let name = required_string(expect_object(tool, &path)?, "name", &path)?;
        if let Some(previous) = names.insert(name.to_owned(), index) {
            return Err(format!(
                "{path}.name duplicates tool name {name:?} already defined at $.tools[{previous}]"
            ));
        }
    }

    validate_tool_references(tools, &names)?;
    validate_install_order(tools, &names)?;
    Ok(())
}

fn validate_tool(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    unknown_fields(object, path, TOOL_FIELDS)?;
    let name = required_string(object, "name", path)?;
    if name.trim().is_empty() {
        return Err(format!("{path}.name must not be empty"));
    }
    validate_action(required(object, "action", path)?, &format!("{path}.action"))?;
    validate_array_of_objects(object, "bins", path, validate_bin)?;
    validate_array_of_objects(object, "paths", path, validate_tool_path)?;
    validate_array_of_objects(object, "checks", path, validate_check)?;
    validate_optional_string_array(object, "platforms", path)?;
    validate_optional_predicate_array(object, "requires", path)?;
    validate_optional_string_array(object, "depends", path)?;
    validate_optional_string_array(object, "before", path)?;
    validate_optional_string_array(object, "after", path)?;
    validate_optional_i64(object, "order", path)?;
    validate_optional_phase(object, "phase", path)?;
    validate_optional_bool(object, "verify_after_install", path)?;
    if let Some(meta) = object.get("meta") {
        validate_tool_meta(meta, &format!("{path}.meta"))?;
    }
    if let Some(passthru) = object.get("passthru") {
        let _object = expect_object(passthru, &format!("{path}.passthru"))?;
    }
    if let Some(uninstall) = object.get("uninstall") {
        validate_uninstall(uninstall, &format!("{path}.uninstall"))?;
    }
    Ok(())
}

fn validate_tool_meta(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    unknown_fields(object, path, TOOL_META_FIELDS)?;
    validate_optional_string(object, "home_page", path)?;
    validate_optional_string(object, "description", path)?;
    validate_optional_string(object, "license", path)?;
    validate_optional_string(object, "main_program", path)?;
    validate_optional_string(object, "source", path)?;
    validate_optional_string_array(object, "maintainers", path)?;
    validate_optional_string_array(object, "tags", path)?;
    Ok(())
}

fn validate_action(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    let action_type = required_string(object, "type", path)?;
    match action_type {
        "required" => unknown_fields(object, path, REQUIRED_ACTION_FIELDS),
        "package" => validate_package_action(object, path),
        "build" => validate_build_action(object, path),
        "archive" => validate_archive_action(object, path),
        other => Err(format!("{path}.type has unknown action type {other:?}")),
    }
}

fn validate_package_action(
    object: &serde_json::Map<String, Value>,
    path: &str,
) -> Result<(), String> {
    unknown_fields(object, path, PACKAGE_ACTION_FIELDS)?;
    validate_optional_string(object, "name", path)?;
    validate_optional_argv(object, "install_argv", path)?;
    validate_optional_argvs(object, "install_argvs", path)?;
    validate_array_of_objects(object, "platforms", path, validate_package_platform)?;
    Ok(())
}

fn validate_package_platform(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    unknown_fields(object, path, PACKAGE_PLATFORM_FIELDS)?;
    validate_predicate(required(object, "when", path)?, &format!("{path}.when"))?;
    validate_optional_string_array(object, "requires_commands", path)?;
    validate_optional_string(object, "name", path)?;
    validate_optional_argv(object, "install_argv", path)?;
    validate_optional_argvs(object, "install_argvs", path)?;
    if !object.contains_key("install_argv") && !object.contains_key("install_argvs") {
        return Err(format!("{path} must contain install_argv or install_argvs"));
    }
    Ok(())
}

fn validate_build_action(
    object: &serde_json::Map<String, Value>,
    path: &str,
) -> Result<(), String> {
    unknown_fields(object, path, BUILD_ACTION_FIELDS)?;
    let _path = required_string(object, "path", path)?;
    validate_optional_argv(object, "argv", path)?;
    validate_optional_argvs(object, "argvs", path)?;
    Ok(())
}

fn validate_archive_action(
    object: &serde_json::Map<String, Value>,
    path: &str,
) -> Result<(), String> {
    unknown_fields(object, path, ARCHIVE_ACTION_FIELDS)?;
    let _path = required_string(object, "path", path)?;
    validate_optional_usize(object, "strip_components", path)?;
    Ok(())
}

fn validate_bin(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    unknown_fields(object, path, BIN_FIELDS)?;
    let _name = required_string(object, "name", path)?;
    validate_optional_string_array(object, "version_argv", path)?;
    Ok(())
}

fn validate_tool_path(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    unknown_fields(object, path, TOOL_PATH_FIELDS)?;
    let _path = required_string(object, "path", path)?;
    if let Some(when) = object.get("when") {
        validate_predicate(when, &format!("{path}.when"))?;
    }
    Ok(())
}

fn validate_check(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    unknown_fields(object, path, CHECK_FIELDS)?;
    validate_argv(required(object, "argv", path)?, &format!("{path}.argv"))?;
    if let Some(when) = object.get("when") {
        validate_predicate(when, &format!("{path}.when"))?;
    }
    Ok(())
}

fn validate_uninstall(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    unknown_fields(object, path, UNINSTALL_FIELDS)?;
    validate_array_of_objects(object, "commands", path, validate_uninstall_command)?;
    validate_array_of_objects(object, "paths", path, validate_tool_path)?;
    validate_optional_bool(object, "remove_bins", path)?;
    validate_optional_bool(object, "remove_prefix", path)?;
    Ok(())
}

fn validate_uninstall_command(value: &Value, path: &str) -> Result<(), String> {
    let object = expect_object(value, path)?;
    unknown_fields(object, path, UNINSTALL_COMMAND_FIELDS)?;
    validate_argv(required(object, "argv", path)?, &format!("{path}.argv"))?;
    if let Some(when) = object.get("when") {
        validate_predicate(when, &format!("{path}.when"))?;
    }
    Ok(())
}

fn validate_predicate(value: &Value, path: &str) -> Result<(), String> {
    if let Some(value) = value.as_str() {
        return validate_short_predicate(value, path);
    }
    let object = expect_object(value, path)?;
    unknown_fields(object, path, PREDICATE_FIELDS)?;
    if let Some(os) = object.get("os") {
        validate_one_of(os, &format!("{path}.os"), &["linux", "macos", "windows"])?;
    }
    if let Some(arch) = object.get("arch") {
        validate_one_of(arch, &format!("{path}.arch"), &["aarch64", "x86_64"])?;
    }
    Ok(())
}

fn validate_short_predicate(value: &str, path: &str) -> Result<(), String> {
    let (os, arch) = value
        .split_once('-')
        .map_or((value, None), |(os, arch)| (os, Some(arch)));
    match os {
        "linux" | "macos" | "darwin" | "windows" | "win32" => {}
        _ => return Err(format!("{path} has unknown host OS predicate {os:?}")),
    }
    match arch {
        None | Some("aarch64" | "arm64" | "x86_64" | "amd64" | "x64") => Ok(()),
        Some(arch) => Err(format!(
            "{path} has unknown host architecture predicate {arch:?}"
        )),
    }
}

fn validate_array_of_objects(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
    validate: fn(&Value, &str) -> Result<(), String>,
) -> Result<(), String> {
    let Some(values) = object.get(field) else {
        return Ok(());
    };
    let values = expect_array(values, &format!("{path}.{field}"))?;
    for (index, value) in values.iter().enumerate() {
        validate(value, &format!("{path}.{field}[{index}]"))?;
    }
    Ok(())
}

fn validate_optional_predicate_array(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    let Some(values) = object.get(field) else {
        return Ok(());
    };
    let values = expect_array(values, &format!("{path}.{field}"))?;
    for (index, value) in values.iter().enumerate() {
        validate_predicate(value, &format!("{path}.{field}[{index}]"))?;
    }
    Ok(())
}

fn validate_optional_string_array(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    let Some(values) = object.get(field) else {
        return Ok(());
    };
    let values = expect_array(values, &format!("{path}.{field}"))?;
    for (index, value) in values.iter().enumerate() {
        let _item = expect_string(value, &format!("{path}.{field}[{index}]"))?;
    }
    Ok(())
}

fn validate_optional_argv(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    if let Some(value) = object.get(field) {
        validate_argv(value, &format!("{path}.{field}"))?;
    }
    Ok(())
}

fn validate_optional_argvs(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    let Some(value) = object.get(field) else {
        return Ok(());
    };
    let values = expect_array(value, &format!("{path}.{field}"))?;
    for (index, value) in values.iter().enumerate() {
        validate_argv(value, &format!("{path}.{field}[{index}]"))?;
    }
    Ok(())
}

fn validate_argv(value: &Value, path: &str) -> Result<(), String> {
    let values = expect_array(value, path)?;
    if values.is_empty() {
        return Err(format!("{path} must contain at least one command argument"));
    }
    for (index, value) in values.iter().enumerate() {
        let _arg = expect_string(value, &format!("{path}[{index}]"))?;
    }
    Ok(())
}

fn validate_optional_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    if let Some(value) = object.get(field) {
        let _value = expect_string(value, &format!("{path}.{field}"))?;
    }
    Ok(())
}

fn validate_optional_bool(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    if let Some(value) = object.get(field)
        && !value.is_boolean()
    {
        return Err(format!("{path}.{field} must be a boolean"));
    }
    Ok(())
}

fn validate_optional_i64(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    if let Some(value) = object.get(field)
        && value.as_i64().is_none()
    {
        return Err(format!("{path}.{field} must be an integer"));
    }
    Ok(())
}

fn validate_optional_usize(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    if let Some(value) = object.get(field)
        && value.as_u64().is_none()
    {
        return Err(format!("{path}.{field} must be a non-negative integer"));
    }
    Ok(())
}

fn validate_optional_phase(
    object: &serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<(), String> {
    if let Some(value) = object.get(field) {
        validate_one_of(
            value,
            &format!("{path}.{field}"),
            &["prerequisites", "packages", "builds"],
        )?;
    }
    Ok(())
}

fn validate_one_of(value: &Value, path: &str, allowed: &[&str]) -> Result<(), String> {
    let value = expect_string(value, path)?;
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!("{path} must be one of {}", allowed.join(", ")))
    }
}

fn required<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<&'a Value, String> {
    object
        .get(field)
        .ok_or_else(|| format!("{path}.{field} is required"))
}

fn required_string<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
    path: &str,
) -> Result<&'a str, String> {
    expect_string(required(object, field, path)?, &format!("{path}.{field}"))
}

fn expect_object<'a>(
    value: &'a Value,
    path: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    value
        .as_object()
        .ok_or_else(|| format!("{path} must be an object"))
}

fn expect_array<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{path} must be an array"))
}

fn expect_string<'a>(value: &'a Value, path: &str) -> Result<&'a str, String> {
    value
        .as_str()
        .ok_or_else(|| format!("{path} must be a string"))
}

fn unknown_fields(
    object: &serde_json::Map<String, Value>,
    path: &str,
    allowed: &[&str],
) -> Result<(), String> {
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(format!("{path}.{key} is not a recognized catalog field"));
        }
    }
    Ok(())
}
