use std::collections::HashSet;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("could not determine home directory")]
    MissingHome,
}

pub struct Context {
    pub catalog_path: PathBuf,
    pub catalog_mode: Option<String>,
    pub root_dir: PathBuf,
    pub bin_dir: PathBuf,
    pub state_dir: PathBuf,
}

impl Context {
    pub fn new(catalog_path: PathBuf) -> Result<Self, ContextError> {
        Self::new_with_catalog_mode(catalog_path, None)
    }

    pub fn new_with_catalog_mode(
        catalog_path: PathBuf,
        catalog_mode: Option<String>,
    ) -> Result<Self, ContextError> {
        let home = home::home_dir().ok_or(ContextError::MissingHome)?;
        let root_dir = catalog_parent_dir(&catalog_path).to_path_buf();
        Ok(Self {
            catalog_path,
            catalog_mode,
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
        if !self.catalog_path.is_file() {
            return Vec::new();
        }

        sorted_scheme_paths(
            &self.root_dir,
            SchemePathFilter::All,
            Some(&self.catalog_path),
        )
    }

    #[must_use]
    pub fn test_paths(&self) -> Vec<PathBuf> {
        if !self.catalog_path.is_file() {
            return Vec::new();
        }

        sorted_scheme_paths(&self.root_dir, SchemePathFilter::NamedTest, None)
    }

    #[must_use]
    pub fn extension_dirs(&self) -> Vec<PathBuf> {
        extension_dirs_for_catalog_path(&self.catalog_path)
    }

    #[must_use]
    pub fn resolve_workspace_path(&self, path: &str) -> PathBuf {
        let path = PathBuf::from(path);
        if path.is_absolute() {
            path
        } else {
            self.root_dir.join(path)
        }
    }
}

#[must_use]
pub fn extension_dirs_for_catalog_path(catalog_path: &Path) -> Vec<PathBuf> {
    let root = catalog_parent_dir(catalog_path).to_path_buf();
    extension_dirs_for_root(&root)
}

#[must_use]
pub fn extension_dirs_for_root(root: &Path) -> Vec<PathBuf> {
    vec![root.to_path_buf()]
}

#[must_use]
pub fn catalog_parent_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
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
    scheme_paths(root)
}

#[must_use]
pub fn scheme_paths(root: &Path) -> Vec<PathBuf> {
    sorted_scheme_paths(root, SchemePathFilter::All, None)
}

fn sorted_scheme_paths(root: &Path, filter: SchemePathFilter, seed: Option<&Path>) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen_files = HashSet::new();
    if let Some(path) = seed {
        push_unique_path(path, &mut paths, &mut seen_files);
    }
    collect_scheme_paths(root, &mut paths, filter, &mut seen_files);
    paths.sort();
    paths
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
    seen_files: &mut HashSet<PathBuf>,
) {
    for entry in scheme_walk(dir) {
        let path = entry.path();
        if entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
            && matches_scheme_filter(path, filter)
        {
            push_unique_path(path, output, seen_files);
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

fn scheme_walk(dir: &Path) -> impl Iterator<Item = ignore::DirEntry> {
    let mut builder = WalkBuilder::new(dir);
    builder
        .follow_links(true)
        .hidden(false)
        .require_git(false)
        .types(scheme_types());
    builder.build().filter_map(Result::ok)
}

fn scheme_types() -> ignore::types::Types {
    let mut builder = ignore::types::TypesBuilder::new();
    builder.add("scheme", "*.scm").expect("valid scheme glob");
    builder.select("scheme");
    builder.build().expect("valid scheme type selection")
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
            catalog_mode: None,
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
    fn catalog_root_is_extension_root() {
        assert_eq!(
            extension_dirs_for_catalog_path(Path::new("/workspace/scaffold.scm")),
            vec![PathBuf::from("/workspace")]
        );
    }

    #[test]
    fn bare_catalog_filename_uses_current_directory_as_extension_root() {
        let ctx = Context {
            catalog_path: PathBuf::from("scaffold.scm"),
            catalog_mode: None,
            root_dir: catalog_parent_dir(Path::new("scaffold.scm")).to_path_buf(),
            bin_dir: PathBuf::from("."),
            state_dir: PathBuf::from("."),
        };

        assert_eq!(ctx.root_dir, PathBuf::from("."));
        assert_eq!(
            extension_dirs_for_catalog_path(Path::new("scaffold.scm")),
            vec![PathBuf::from(".")]
        );
    }

    #[test]
    fn non_default_catalogs_use_workspace_extension_root() {
        assert_eq!(
            extension_dirs_for_catalog_path(Path::new("/workspace/scaffold-userland.scm")),
            vec![PathBuf::from("/workspace")]
        );
    }

    #[cfg(unix)]
    #[test]
    fn source_paths_dedupe_symlinked_extension_trees() {
        let root = tempfile::tempdir().expect("root");
        let entries = root.path().join("scaffold").join("entries");
        let scaffold_dot_extensions = root.path().join(".scaffold").join("extensions");
        std::fs::create_dir_all(&entries).expect("entries");
        std::fs::create_dir_all(&scaffold_dot_extensions).expect("dot extensions");
        std::fs::write(root.path().join("scaffold.scm"), "(import (rnrs))\n").expect("catalog");
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
            catalog_path: root.path().join("scaffold.scm"),
            catalog_mode: None,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        let source_paths = ctx.source_paths();
        let demo_paths = source_paths
            .iter()
            .filter(|path| path.file_name().is_some_and(|name| name == "demo.scm"))
            .count();

        assert_eq!(demo_paths, 1);
    }

    #[test]
    fn scheme_paths_respect_gitignore_without_git_repo() {
        let root = tempfile::tempdir().expect("root");
        let ignored = root.path().join("ignored");
        let included = root.path().join("included");
        std::fs::create_dir_all(&ignored).expect("ignored dir");
        std::fs::create_dir_all(&included).expect("included dir");
        std::fs::write(root.path().join(".gitignore"), "ignored/\n").expect("gitignore");
        std::fs::write(ignored.join("hidden.scm"), "(import (rnrs))\n").expect("ignored source");
        std::fs::write(included.join("visible.scm"), "(import (rnrs))\n").expect("visible source");

        let files = workspace_scheme_paths(root.path());

        assert!(
            files
                .iter()
                .any(|path| path.ends_with("included/visible.scm"))
        );
        assert!(
            !files
                .iter()
                .any(|path| path.ends_with("ignored/hidden.scm"))
        );
    }

    #[test]
    fn scheme_paths_include_hidden_scaffold_dirs() {
        let root = tempfile::tempdir().expect("root");
        let scaffold_dir = root.path().join(".scaffold");
        std::fs::create_dir_all(&scaffold_dir).expect("scaffold dir");
        std::fs::write(root.path().join("scaffold.scm"), "(import (rnrs))\n").expect("catalog");
        std::fs::write(scaffold_dir.join("test.scm"), "(import (rnrs))\n").expect("test source");

        let ctx = Context {
            catalog_path: root.path().join("scaffold.scm"),
            catalog_mode: None,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        assert_eq!(ctx.test_paths(), vec![scaffold_dir.join("test.scm")]);
    }

    #[test]
    fn collects_workspace_scheme_paths_without_ignored_outputs() {
        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join(".gitignore"), "target/\n").expect("gitignore");
        std::fs::create_dir_all(root.path().join("std")).expect("std dir");
        std::fs::create_dir_all(root.path().join("target")).expect("target dir");
        std::fs::write(
            root.path().join("std").join("config.scm"),
            "(import (rnrs))\n",
        )
        .expect("source");
        std::fs::write(
            root.path().join("target").join("generated.scm"),
            "(import (rnrs))\n",
        )
        .expect("ignored source");

        let files = workspace_scheme_paths(root.path());

        assert!(files.iter().any(|path| path.ends_with("std/config.scm")));
        assert!(
            !files
                .iter()
                .any(|path| path.ends_with("target/generated.scm"))
        );
    }

    #[test]
    fn missing_catalog_does_not_discover_workspace_sources() {
        let root = tempfile::tempdir().expect("root");
        std::fs::write(root.path().join("test.scm"), "(import (rnrs))\n").expect("test source");
        std::fs::write(root.path().join("library.scm"), "(import (rnrs))\n").expect("source");
        let ctx = Context {
            catalog_path: root.path().join("scaffold.scm"),
            catalog_mode: None,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        assert!(ctx.source_paths().is_empty());
        assert!(ctx.test_paths().is_empty());
    }

    fn fixture_path(path: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scaffold-dsl")
            .join("src")
            .join("fixtures")
            .join(path)
    }
}
