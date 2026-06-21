use std::path::{MAIN_SEPARATOR, Path, PathBuf};

use clean_path::Clean;
use scaffold_scheme::value_to_string;
use scheme_rs::{exceptions::Exception, registry::bridge, value::Value};

#[bridge(name = "%path/join", lib = "(scaffold path builtins)")]
pub fn path_join(first: &Value, part: &Value) -> Result<Vec<Value>, Exception> {
    let mut path = PathBuf::from(value_to_string(first)?);
    path.push(value_to_string(part)?);
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

#[bridge(name = "%path/separator", lib = "(scaffold path builtins)")]
pub fn path_separator() -> Result<Vec<Value>, Exception> {
    Ok(vec![Value::from(MAIN_SEPARATOR.to_string())])
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
    path.clean()
}
