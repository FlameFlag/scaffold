use std::path::Path;

pub(super) fn scheme_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub(super) fn scheme_maybe_string(value: Option<&str>) -> String {
    value.map_or_else(|| "#f".to_owned(), scheme_string)
}

pub(super) fn scheme_path(path: Option<&Path>) -> String {
    let text = path.map(|path| path.to_string_lossy().into_owned());
    scheme_maybe_string(text.as_deref())
}
