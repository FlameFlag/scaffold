use std::path::Path;

use scaffold_template as template;

const WORKSPACE_LIBRARY_TEMPLATE: &str = include_str!("std/workspace.scm");

pub(super) fn workspace_library_source(
    workspace_root: Option<&Path>,
    source_path: Option<&Path>,
) -> String {
    let workspace_root_value = path_template_value(workspace_root);
    let source_path_value = path_template_value(source_path);
    let bindings = std::collections::HashMap::from([
        (
            "workspace_root_present",
            template_bool(workspace_root.is_some()),
        ),
        ("workspace_root", workspace_root_value.as_str()),
        ("source_path_present", template_bool(source_path.is_some())),
        ("source_path", source_path_value.as_str()),
    ]);
    template::render(WORKSPACE_LIBRARY_TEMPLATE, &bindings)
}

fn template_bool(value: bool) -> &'static str {
    if value { "#t" } else { "#f" }
}

fn path_template_value(path: Option<&Path>) -> String {
    path.map(|path| path.to_string_lossy().into_owned())
        .map(|path| scaffold_scheme::escape_string_literal_body(&path))
        .unwrap_or_default()
}
