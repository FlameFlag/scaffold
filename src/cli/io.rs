use std::{
    io::{self, Write},
    path::Path,
};

use super::CliError;

pub(super) fn pretty_json(value: &serde_json::Value) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(value).map(|mut json| {
        json.push('\n');
        json
    })
}

pub(super) fn print_json_values(values: Vec<serde_json::Value>) -> Result<(), CliError> {
    values
        .into_iter()
        .try_for_each(|value| write_stdout(&pretty_json(&value)?))
}

pub(super) fn write_output_file(path: &Path, text: &str) -> Result<(), CliError> {
    ensure_output_parent(path)?;
    let mut output = tempfile::NamedTempFile::new_in(output_parent(path))?;
    output.write_all(text.as_bytes())?;
    output
        .persist(path)
        .map_err(|err| CliError::Io(err.error))?;
    Ok(())
}

fn output_parent(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn ensure_output_parent(path: &Path) -> Result<(), CliError> {
    std::fs::create_dir_all(output_parent(path))?;
    Ok(())
}

pub(super) fn write_stdout(text: &str) -> Result<(), CliError> {
    write_stream(std::io::stdout().lock(), text)
}

pub(super) fn write_stderr(text: &str) -> Result<(), CliError> {
    write_stream(std::io::stderr().lock(), text)
}

pub(super) fn write_stream(mut stream: impl Write, text: &str) -> Result<(), CliError> {
    match stream.write_all(text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(CliError::Io(err)),
    }
}
