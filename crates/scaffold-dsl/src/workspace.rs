use std::path::Path;

use super::literal::scheme_path;

const WORKSPACE_LIBRARY_TEMPLATE: &str = include_str!("std/workspace.scm");

pub(super) fn workspace_library_source(
    workspace_root: Option<&Path>,
    source_path: Option<&Path>,
) -> String {
    WORKSPACE_LIBRARY_TEMPLATE
        .replace("@WORKSPACE_ROOT@", &scheme_path(workspace_root))
        .replace("@SOURCE_PATH@", &scheme_path(source_path))
        .replace(
            "@SOURCE_DIR@",
            &scheme_path(source_path.and_then(Path::parent)),
        )
}
