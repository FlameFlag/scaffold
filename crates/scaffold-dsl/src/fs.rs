use std::path::{Path, PathBuf};

use scaffold_scheme::value_to_string;
use scheme_rs::{exceptions::Exception, registry::bridge, value::Value};

#[bridge(name = "%path/exists?", lib = "(scaffold fs builtins)")]
pub fn path_exists(path: &Value) -> Result<Vec<Value>, Exception> {
    Ok(vec![Value::from(absolute_path(path)?.exists())])
}

#[bridge(name = "%file/exists?", lib = "(scaffold fs builtins)")]
pub fn file_exists(path: &Value) -> Result<Vec<Value>, Exception> {
    Ok(vec![Value::from(absolute_path(path)?.is_file())])
}

#[bridge(name = "%directory/exists?", lib = "(scaffold fs builtins)")]
pub fn directory_exists(path: &Value) -> Result<Vec<Value>, Exception> {
    Ok(vec![Value::from(absolute_path(path)?.is_dir())])
}

fn absolute_path(value: &Value) -> Result<PathBuf, Exception> {
    let path = value_to_path(value)?;
    if path.is_absolute() {
        Ok(path)
    } else {
        Err(Exception::error(format!(
            "filesystem predicates require an absolute path, got `{}`",
            path.to_string_lossy()
        )))
    }
}

fn value_to_path(value: &Value) -> Result<PathBuf, Exception> {
    value_to_string(value).map(|value| Path::new(&value).to_path_buf())
}
