use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::stdlib::SchemeLibrary;
use super::{DslError, Result, bundled};
use scaffold_scheme::{identifier_text, is_identifier, parse_source, proper_list};
use scheme_rs::syntax::Syntax;

pub(super) fn extension_dirs_for_root(root: &Path) -> Vec<PathBuf> {
    scaffold_context::extension_dirs_for_root(root)
}

pub(super) fn extension_dirs_for_catalog_path(catalog_path: &Path) -> Vec<PathBuf> {
    scaffold_context::extension_dirs_for_catalog_path(catalog_path)
}

pub(super) fn load_bundled_libraries() -> Result<Vec<SchemeLibrary>> {
    let mut names = HashSet::<Vec<String>>::new();
    let mut libraries = Vec::new();
    for source in bundled::BUNDLED_EXTENSION_SOURCES {
        let name = library_name_from_source(source.source, Path::new(source.path))?;
        if !names.insert(name.clone()) {
            return Err(DslError::Shape {
                path: source.path.to_owned(),
                message: format!("duplicate Scheme library ({})", name.join(" ")),
            });
        }
        libraries.push(SchemeLibrary {
            name,
            source: Cow::Borrowed(source.source),
        });
    }
    Ok(libraries)
}

pub(super) fn load_user_libraries(extension_dirs: &[PathBuf]) -> Result<Vec<SchemeLibrary>> {
    let mut paths = Vec::new();
    let mut seen_files = HashSet::new();
    for dir in extension_dirs {
        collect_scheme_files(dir, &mut paths, &mut seen_files);
    }
    paths.sort();

    let mut names = HashMap::<Vec<String>, String>::new();
    let mut libraries = Vec::new();
    for path in paths {
        let source = std::fs::read_to_string(&path)?;
        if !looks_like_library_source(&source) {
            continue;
        }
        let name = library_name_from_source(&source, &path)?;
        if name
            .first()
            .is_some_and(|component| component == "scaffold")
        {
            continue;
        }
        if let Some(previous_path) = names.insert(name.clone(), path.display().to_string()) {
            return Err(DslError::Shape {
                path: path.display().to_string(),
                message: format!(
                    "duplicate Scheme library ({}) already defined at {previous_path}",
                    name.join(" ")
                ),
            });
        }
        libraries.push(SchemeLibrary {
            name,
            source: Cow::Owned(source),
        });
    }
    Ok(libraries)
}

fn collect_scheme_files(dir: &Path, output: &mut Vec<PathBuf>, seen_files: &mut HashSet<PathBuf>) {
    for path in scaffold_context::scheme_paths(dir) {
        if path.file_name().is_none_or(|name| name != "test.scm")
            && seen_files.insert(canonical_path(&path).unwrap_or_else(|_| path.to_path_buf()))
        {
            output.push(path);
        }
    }
}

fn canonical_path(path: &Path) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(path)
}

fn looks_like_library_source(source: &str) -> bool {
    let mut rest = source;
    loop {
        rest = rest.trim_start_matches(char::is_whitespace);
        if let Some(after_comment) = rest.strip_prefix(';') {
            if let Some((_, after_line)) = after_comment.split_once('\n') {
                rest = after_line;
                continue;
            }
            return false;
        }
        return rest.starts_with("(library");
    }
}

fn library_name_from_source(source: &str, path: &Path) -> Result<Vec<String>> {
    let path_text = path.to_string_lossy();
    let syntax =
        parse_source(source, path_text.as_ref()).map_err(|err| DslError::Eval(err.to_string()))?;
    let Some([form]) = proper_list(&syntax) else {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected a single Scheme library form".to_owned(),
        });
    };

    let Some([head, name, ..]) = form.as_list() else {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected a Scheme library form".to_owned(),
        });
    };
    if !is_identifier(head, "library") {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected a Scheme library form".to_owned(),
        });
    }

    library_name_from_syntax(name, path)
}

fn library_name_from_syntax(name: &Syntax, path: &Path) -> Result<Vec<String>> {
    let Some(components) = proper_list(name) else {
        let message = if name.as_list().is_some() {
            "expected proper library name list"
        } else {
            "expected library name list"
        };
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: message.to_owned(),
        });
    };
    components
        .iter()
        .map(|component| {
            identifier_text(component).ok_or_else(|| DslError::Shape {
                path: path.display().to_string(),
                message: "library name components must be identifiers".to_owned(),
            })
        })
        .collect()
}
