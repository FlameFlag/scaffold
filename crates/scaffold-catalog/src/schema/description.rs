use serde_json::{Value, json};

#[must_use]
pub fn catalog_schema() -> Value {
    json!({
        "title": "Scaffold Catalog",
        "version": 1,
        "root": {
            "type": "object",
            "required": ["tools"],
            "fields": {
                "tools": { "type": "array", "items": "tool" }
            }
        },
        "objects": [
            object_schema("tool", TOOL_FIELDS, &["name", "action"]),
            object_schema("bin", BIN_FIELDS, &["name"]),
            object_schema("tool_path", TOOL_PATH_FIELDS, &["path"]),
            object_schema("check", CHECK_FIELDS, &["argv"]),
            object_schema("uninstall", UNINSTALL_FIELDS, &[]),
            object_schema("uninstall_command", UNINSTALL_COMMAND_FIELDS, &["argv"]),
            object_schema("predicate", PREDICATE_FIELDS, &[]),
            object_schema("tool_meta", TOOL_META_FIELDS, &[]),
            object_schema("package_action", PACKAGE_ACTION_FIELDS, &["type"]),
            object_schema("package_platform", PACKAGE_PLATFORM_FIELDS, &["when"]),
            object_schema("build_action", BUILD_ACTION_FIELDS, &["type", "path"]),
            object_schema("archive_action", ARCHIVE_ACTION_FIELDS, &["type", "path"]),
            object_schema("required_action", REQUIRED_ACTION_FIELDS, &["type"]),
        ],
        "enums": {
            "action_type": ["archive", "build", "package", "required"],
            "host_os": ["linux", "macos", "windows"],
            "host_arch": ["aarch64", "x86_64"],
            "phase": ["prerequisites", "packages", "builds"]
        },
        "relationships": [
            {
                "fields": ["depends", "before", "after"],
                "target": "tool.name",
                "cycle_checked": true,
                "special_before_values": ["first", "none"]
            }
        ]
    })
}

pub(super) const TOOL_FIELDS: &[&str] = &[
    "action",
    "after",
    "before",
    "bins",
    "checks",
    "depends",
    "meta",
    "name",
    "order",
    "paths",
    "passthru",
    "phase",
    "platforms",
    "requires",
    "uninstall",
    "verify_after_install",
];
pub(super) const BIN_FIELDS: &[&str] = &["name", "version_argv"];
pub(super) const TOOL_PATH_FIELDS: &[&str] = &["path", "when"];
pub(super) const CHECK_FIELDS: &[&str] = &["argv", "when"];
pub(super) const UNINSTALL_FIELDS: &[&str] = &["commands", "paths", "remove_bins", "remove_prefix"];
pub(super) const UNINSTALL_COMMAND_FIELDS: &[&str] = &["argv", "when"];
pub(super) const PREDICATE_FIELDS: &[&str] = &["arch", "os"];
pub(super) const TOOL_META_FIELDS: &[&str] = &[
    "description",
    "home_page",
    "license",
    "main_program",
    "maintainers",
    "source",
    "tags",
];
pub(super) const REQUIRED_ACTION_FIELDS: &[&str] = &["type"];
pub(super) const PACKAGE_ACTION_FIELDS: &[&str] =
    &["install_argv", "install_argvs", "name", "platforms", "type"];
pub(super) const PACKAGE_PLATFORM_FIELDS: &[&str] = &[
    "install_argv",
    "install_argvs",
    "name",
    "requires_commands",
    "when",
];
pub(super) const BUILD_ACTION_FIELDS: &[&str] = &["argv", "argvs", "path", "type"];
pub(super) const ARCHIVE_ACTION_FIELDS: &[&str] = &["path", "strip_components", "type"];

fn object_schema(name: &str, fields: &[&str], required: &[&str]) -> Value {
    json!({
        "name": name,
        "type": "object",
        "fields": fields,
        "required": required
    })
}
