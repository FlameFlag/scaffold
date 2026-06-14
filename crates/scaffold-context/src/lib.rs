use std::collections::HashSet;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("could not determine home directory")]
    MissingHome,
}

pub struct Context {
    pub catalog_path: PathBuf,
    pub root_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub state_dir: PathBuf,
}

impl Context {
    pub fn new(catalog_path: PathBuf) -> Result<Self, ContextError> {
        let home = home_dir().ok_or(ContextError::MissingHome)?;
        let root_dir = catalog_path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        Ok(Self {
            catalog_path,
            root_dir,
            bin_dir: home.join(".local").join("bin"),
            state_dir: home.join(".local").join("share").join("scaffold"),
        })
    }

    #[must_use]
    pub fn install_prefix(&self, tool: &str) -> PathBuf {
        self.state_dir.join("tools").join(tool).join("latest")
    }

    #[must_use]
    pub fn source_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let mut seen_dirs = HashSet::new();
        let mut seen_files = HashSet::new();
        if self.catalog_path.is_file() {
            push_unique_path(&self.catalog_path, &mut paths, &mut seen_files);
        }
        for dir in self.extension_dirs() {
            collect_scheme_paths(
                &dir,
                &mut paths,
                SchemePathFilter::All,
                &mut seen_dirs,
                &mut seen_files,
            );
        }
        paths.sort();
        paths
    }

    #[must_use]
    pub fn test_paths(&self) -> Vec<PathBuf> {
        let mut seen_dirs = HashSet::new();
        let mut seen_files = HashSet::new();
        let mut paths = ["test.scm", ".scaffold/test.scm"]
            .into_iter()
            .map(|name| self.root_dir.join(name))
            .filter(|path| path.is_file())
            .fold(Vec::new(), |mut paths, path| {
                push_unique_path(&path, &mut paths, &mut seen_files);
                paths
            });
        for dir in self.extension_dirs() {
            collect_scheme_paths(
                &dir,
                &mut paths,
                SchemePathFilter::NamedTest,
                &mut seen_dirs,
                &mut seen_files,
            );
        }
        paths.sort();
        paths
    }

    #[must_use]
    pub fn extension_dirs(&self) -> Vec<PathBuf> {
        extension_dirs_for_catalog_path(&self.catalog_path)
    }
}

#[must_use]
pub fn extension_dirs_for_catalog_path(catalog_path: &Path) -> Vec<PathBuf> {
    let root = catalog_path
        .parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let mut dirs = Vec::new();
    if let Some(stem) = catalog_path.file_stem().and_then(|stem| stem.to_str())
        && !stem.is_empty()
    {
        dirs.push(root.join(stem));
    }
    for dir in extension_dirs_for_root(&root) {
        if !dirs.contains(&dir) {
            dirs.push(dir);
        }
    }
    dirs
}

#[must_use]
pub fn extension_dirs_for_root(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("extensions"),
        root.join(".scaffold").join("extensions"),
    ]
}

#[must_use]
pub fn default_catalog_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for name in ["scaffold.scm", "catalog.scm"] {
        let path = cwd.join(name);
        if path.is_file() {
            return path;
        }
    }
    cwd.join("scaffold.scm")
}

#[must_use]
pub fn workspace_scheme_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen_dirs = HashSet::new();
    let mut seen_files = HashSet::new();
    collect_workspace_scheme_paths(root, &mut paths, &mut seen_dirs, &mut seen_files);
    paths.sort();
    paths
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

#[derive(Clone, Copy)]
enum SchemePathFilter {
    All,
    NamedTest,
}

fn collect_scheme_paths(
    dir: &Path,
    output: &mut Vec<PathBuf>,
    filter: SchemePathFilter,
    seen_dirs: &mut HashSet<PathBuf>,
    seen_files: &mut HashSet<PathBuf>,
) {
    let Ok(canonical_dir) = std::fs::canonicalize(dir) else {
        return;
    };
    if !seen_dirs.insert(canonical_dir) {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            collect_scheme_paths(&path, output, filter, seen_dirs, seen_files);
        } else if matches_scheme_filter(&path, filter) {
            push_unique_path(&path, output, seen_files);
        }
    }
}

fn matches_scheme_filter(path: &Path, filter: SchemePathFilter) -> bool {
    if path.extension().is_none_or(|extension| extension != "scm") {
        return false;
    }
    match filter {
        SchemePathFilter::All => true,
        SchemePathFilter::NamedTest => path.file_name().is_some_and(|name| name == "test.scm"),
    }
}

fn collect_workspace_scheme_paths(
    dir: &Path,
    output: &mut Vec<PathBuf>,
    seen_dirs: &mut HashSet<PathBuf>,
    seen_files: &mut HashSet<PathBuf>,
) {
    let Ok(canonical_dir) = std::fs::canonicalize(dir) else {
        return;
    };
    if !seen_dirs.insert(canonical_dir) {
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if path.is_dir() {
            if matches!(name, ".git" | "target" | "node_modules" | "out") {
                continue;
            }
            collect_workspace_scheme_paths(&path, output, seen_dirs, seen_files);
        } else if path.extension().is_some_and(|extension| extension == "scm") {
            push_unique_path(&path, output, seen_files);
        }
    }
}

fn push_unique_path(path: &Path, output: &mut Vec<PathBuf>, seen_files: &mut HashSet<PathBuf>) {
    let canonical_path = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if seen_files.insert(canonical_path) {
        output.push(path.to_path_buf());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_default_test_paths_next_to_catalog() {
        let ctx = Context {
            catalog_path: fixture_path("local/library/catalog.scm"),
            root_dir: fixture_path("local/library"),
            bin_dir: PathBuf::from("."),
            state_dir: PathBuf::from("."),
        };

        assert_eq!(
            ctx.test_paths(),
            vec![
                fixture_path("local/library/extensions/acme/test.scm"),
                fixture_path("local/library/test.scm"),
            ]
        );
    }

    #[test]
    fn catalog_stem_is_extension_root() {
        assert_eq!(
            extension_dirs_for_catalog_path(Path::new("/workspace/scaffold.scm")),
            vec![
                PathBuf::from("/workspace/scaffold"),
                PathBuf::from("/workspace/extensions"),
                PathBuf::from("/workspace/.scaffold/extensions"),
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn source_paths_dedupe_symlinked_extension_trees() {
        let root = unique_test_dir("source-paths-dedupe-symlinked-extension-trees");
        let entries = root.join("scaffold").join("entries");
        let scaffold_dot_extensions = root.join(".scaffold").join("extensions");
        std::fs::create_dir_all(&entries).expect("entries");
        std::fs::create_dir_all(&scaffold_dot_extensions).expect("dot extensions");
        std::fs::write(root.join("scaffold.scm"), "(import (rnrs))\n").expect("catalog");
        std::fs::write(
            entries.join("demo.scm"),
            "(library (entries demo) (export demo) (import (rnrs)) (define demo 'ok))\n",
        )
        .expect("entry");
        std::os::unix::fs::symlink(
            "../../scaffold/entries",
            scaffold_dot_extensions.join("entries"),
        )
        .expect("symlink");

        let ctx = Context {
            catalog_path: root.join("scaffold.scm"),
            root_dir: root.clone(),
            bin_dir: root.join("bin"),
            state_dir: root.join("state"),
        };

        let source_paths = ctx.source_paths();
        let demo_paths = source_paths
            .iter()
            .filter(|path| path.file_name().is_some_and(|name| name == "demo.scm"))
            .count();

        assert_eq!(demo_paths, 1);
        drop(std::fs::remove_dir_all(root));
    }

    #[test]
    fn collects_workspace_scheme_paths_without_build_outputs() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scaffold-dsl")
            .join("src");
        let files = workspace_scheme_paths(&root);

        assert!(files.iter().any(|path| path.ends_with("std/config.scm")));
        assert!(!files.iter().any(|path| {
            path.components()
                .any(|component| component.as_os_str() == "target")
        }));
    }

    fn fixture_path(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scaffold-dsl")
            .join("src")
            .join("fixtures")
            .join(path)
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "scaffold-context-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }
}
