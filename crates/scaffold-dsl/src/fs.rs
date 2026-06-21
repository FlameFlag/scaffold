use std::path::Path;

use scaffold_scheme::value_to_string;
use scheme_rs::{exceptions::Exception, registry::bridge, value::Value};

#[bridge(name = "%path/exists?", lib = "(scaffold fs builtins)")]
pub fn path_exists(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = value_to_string(path)?;
    Ok(vec![Value::from(Path::new(&path).exists())])
}

#[bridge(name = "%file/exists?", lib = "(scaffold fs builtins)")]
pub fn file_exists(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = value_to_string(path)?;
    Ok(vec![Value::from(Path::new(&path).is_file())])
}

#[bridge(name = "%directory/exists?", lib = "(scaffold fs builtins)")]
pub fn directory_exists(path: &Value) -> Result<Vec<Value>, Exception> {
    let path = value_to_string(path)?;
    Ok(vec![Value::from(Path::new(&path).is_dir())])
}
