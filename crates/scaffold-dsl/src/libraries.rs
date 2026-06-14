use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use scheme_rs::syntax::Syntax;

use super::stdlib::SchemeLibrary;
use super::{DslError, Result, bundled};

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
    let mut seen_dirs = HashSet::new();
    let mut seen_files = HashSet::new();
    for dir in extension_dirs {
        collect_scheme_files(dir, &mut paths, &mut seen_dirs, &mut seen_files)?;
    }
    paths.sort();

    let mut names = HashMap::<Vec<String>, String>::new();
    let mut libraries = Vec::new();
    for path in paths {
        let source = std::fs::read_to_string(&path)?;
        let name = library_name_from_source(&source, &path)?;
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

fn collect_scheme_files(
    dir: &Path,
    output: &mut Vec<PathBuf>,
    seen_dirs: &mut HashSet<PathBuf>,
    seen_files: &mut HashSet<PathBuf>,
) -> Result<()> {
    match canonical_path(dir) {
        Ok(canonical_dir) => {
            if !seen_dirs.insert(canonical_dir) {
                return Ok(());
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err.into()),
    }

    match std::fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries {
                let path = entry?.path();
                if path.is_dir() {
                    collect_scheme_files(&path, output, seen_dirs, seen_files)?;
                } else if path.extension().is_some_and(|extension| extension == "scm")
                    && path.file_name().is_none_or(|name| name != "test.scm")
                    && seen_files.insert(canonical_path(&path)?)
                {
                    output.push(path);
                }
            }
            Ok(())
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}

fn canonical_path(path: &Path) -> std::io::Result<PathBuf> {
    std::fs::canonicalize(path)
}

fn library_name_from_source(source: &str, path: &Path) -> Result<Vec<String>> {
    let path_text = path.to_string_lossy();
    let syntax = Syntax::from_str(source, Some(path_text.as_ref()))
        .map_err(|err| DslError::Eval(err.to_string()))?;
    let Some([form, end]) = syntax.as_list() else {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected a single Scheme library form".to_owned(),
        });
    };
    if !end.is_null() {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected a single Scheme library form".to_owned(),
        });
    }

    let Some([head, name, ..]) = form.as_list() else {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected a Scheme library form".to_owned(),
        });
    };
    if !head.as_ident().is_some_and(|ident| ident == "library") {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected a Scheme library form".to_owned(),
        });
    }

    library_name_from_syntax(name, path)
}

fn library_name_from_syntax(name: &Syntax, path: &Path) -> Result<Vec<String>> {
    let Some(items) = name.as_list() else {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected library name list".to_owned(),
        });
    };
    let Some((end, components)) = items.split_last() else {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected library name list".to_owned(),
        });
    };
    if !end.is_null() {
        return Err(DslError::Shape {
            path: path.display().to_string(),
            message: "expected proper library name list".to_owned(),
        });
    }
    components
        .iter()
        .map(|component| {
            component
                .as_ident()
                .map(|ident| ident.symbol().to_string())
                .ok_or_else(|| DslError::Shape {
                    path: path.display().to_string(),
                    message: "library name components must be identifiers".to_owned(),
                })
        })
        .collect()
}
