use std::path::PathBuf;

use scaffold_docs::{self as docs, WorkspaceDocIndex};
use tower_lsp::lsp_types::InitializeParams;

pub(super) fn workspace_doc_index_from_roots(roots: &[PathBuf]) -> WorkspaceDocIndex {
    let mut index = WorkspaceDocIndex::empty();
    for root in roots {
        for path in scaffold_context::workspace_scheme_paths(root) {
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            index.push_source(docs::source_docs_with_definitions(
                &path.display().to_string(),
                &source,
            ));
        }
    }
    index
}

pub(super) fn workspace_roots(params: &InitializeParams) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(workspace_folders) = &params.workspace_folders {
        roots.extend(
            workspace_folders
                .iter()
                .filter_map(|folder| folder.uri.to_file_path().ok()),
        );
    }
    if roots.is_empty()
        && let Some(root_uri) = &params.root_uri
        && let Ok(path) = root_uri.to_file_path()
    {
        roots.push(path);
    }
    roots
}

#[cfg(test)]
fn workspace_doc_index(params: &InitializeParams) -> WorkspaceDocIndex {
    workspace_doc_index_from_roots(&workspace_roots(params))
}

#[cfg(test)]
mod tests {
    use super::*;
    use scaffold_docs::DocIndex;
    use tower_lsp::lsp_types::{Url, WorkspaceFolder};

    #[test]
    fn collects_workspace_scheme_files_without_build_outputs() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scaffold-dsl")
            .join("src");
        let files = scaffold_context::workspace_scheme_paths(&root);

        assert!(files.iter().any(|path| path.ends_with("std/config.scm")));
        assert!(!files.iter().any(|path| {
            path.components()
                .any(|component| component.as_os_str() == "target")
        }));
    }

    #[test]
    fn builds_workspace_doc_index_from_initialize_params() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scaffold-dsl")
            .join("src");
        let params = InitializeParams {
            workspace_folders: Some(vec![WorkspaceFolder {
                uri: Url::from_file_path(&root).expect("file url"),
                name: "src".to_owned(),
            }]),
            ..Default::default()
        };
        let index = workspace_doc_index(&params);

        assert!(index.all().get("acme-tool").is_some());
    }

    #[test]
    fn workspace_docs_are_scoped_to_imported_libraries() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("scaffold-dsl")
            .join("src");
        let workspace = workspace_doc_index_from_roots(&[root]);
        let config_source = include_str!("../../scaffold-dsl/src/std/config.scm");
        let mut index = DocIndex::with_language_keywords();

        workspace.extend_imported_docs(&mut index, config_source);
        index.extend_source("src/dsl/std/config.scm", config_source);

        let define = index.get("define").expect("define keyword docs");
        assert_eq!(define.source.as_deref(), Some("scheme keyword"));
        assert_eq!(
            define.summary.as_deref(),
            Some("Bind a Scheme value or procedure in the current scope.")
        );
    }
}
