use std::path::{Component, MAIN_SEPARATOR, Path, PathBuf};

use scheme_rs::{exceptions::Exception, registry::bridge, strings::WideString, value::Value};

#[bridge(name = "%path/join", lib = "(scaffold path builtins)")]
pub fn path_join(first: &Value, rest: &[Value]) -> Result<Vec<Value>, Exception> {
    let mut path = PathBuf::from(value_to_string(first)?);
    for part in rest {
        path.push(value_to_string(part)?);
    }
    Ok(vec![Value::from(path_to_string(path))])
}

#[bridge(name = "%path/normalize", lib = "(scaffold path builtins)")]
pub fn path_normalize(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = PathBuf::from(value_to_string(path)?);
    Ok(vec![Value::from(path_to_string(normalize_lexically(
        &path,
    )))])
}

#[bridge(name = "%path/parent", lib = "(scaffold path builtins)")]
pub fn path_parent(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = PathBuf::from(value_to_string(path)?);
    Ok(vec![optional_path(path.parent())])
}

#[bridge(name = "%path/file-name", lib = "(scaffold path builtins)")]
pub fn path_file_name(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = PathBuf::from(value_to_string(path)?);
    Ok(vec![path.file_name().map_or_else(
        || Value::from(false),
        |name| Value::from(name.to_string_lossy().into_owned()),
    )])
}

#[bridge(name = "%path/extension", lib = "(scaffold path builtins)")]
pub fn path_extension(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = PathBuf::from(value_to_string(path)?);
    Ok(vec![path.extension().map_or_else(
        || Value::from(false),
        |extension| Value::from(extension.to_string_lossy().into_owned()),
    )])
}

#[bridge(name = "%path/absolute?", lib = "(scaffold path builtins)")]
pub fn path_absolute(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = PathBuf::from(value_to_string(path)?);
    Ok(vec![Value::from(path.is_absolute())])
}

#[bridge(name = "%path/relative?", lib = "(scaffold path builtins)")]
pub fn path_relative(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = PathBuf::from(value_to_string(path)?);
    Ok(vec![Value::from(path.is_relative())])
}

#[bridge(name = "%path/separator", lib = "(scaffold path builtins)")]
pub fn path_separator() -> Result<Vec<Value>, Exception> {
    Ok(vec![Value::from(MAIN_SEPARATOR.to_string())])
}

fn value_to_string(value: &Value) -> Result<String, Exception> {
    let value: WideString = value.clone().try_into()?;
    Ok(value.into())
}

fn optional_path(path: Option<&Path>) -> Value {
    path.map_or_else(
        || Value::from(false),
        |path| Value::from(path.to_string_lossy().into_owned()),
    )
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().into_owned()
}

pub(super) fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    normalized.push(component.as_os_str());
                }
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}
