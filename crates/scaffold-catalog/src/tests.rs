use super::Catalog;
use miette::Diagnostic as _;
use scaffold_platform::{Host, HostOs};

#[test]
fn action_and_phase_labels_are_stable_lowercase_values() {
    let catalog = Catalog::from_value(serde_json::json!({
        "tools": [
            {
                "name": "required-demo",
                "action": { "type": "required" }
            },
            {
                "name": "package-demo",
                "action": {
                    "type": "package",
                    "install_argv": ["pkg", "install", "package-demo"]
                }
            },
            {
                "name": "build-demo",
                "action": {
                    "type": "build",
                    "path": "vendor/build-demo",
                    "argv": ["make"]
                }
            },
            {
                "name": "archive-demo",
                "action": {
                    "type": "archive",
                    "path": "archive.zip"
                }
            }
        ]
    }))
    .expect("catalog");

    assert_eq!(catalog.tools[0].action.label(), "required");
    assert_eq!(catalog.tools[0].phase().label(), "prerequisites");
    assert_eq!(catalog.tools[1].action.label(), "package");
    assert_eq!(catalog.tools[1].phase().label(), "packages");
    assert_eq!(catalog.tools[2].action.label(), "build");
    assert_eq!(catalog.tools[2].phase().label(), "builds");
    assert_eq!(catalog.tools[3].action.label(), "archive");
    assert_eq!(catalog.tools[3].phase().label(), "builds");
}

#[test]
fn loads_extension_composition_fixture() {
    let value = scaffold_dsl::catalog_value_from_str(include_str!(
        "../../scaffold-dsl/src/fixtures/extensions/composed-catalog.scm"
    ))
    .expect("catalog value");
    let catalog = Catalog::from_value(value).expect("catalog");

    assert_eq!(catalog.tools.len(), 5);
    assert_eq!(catalog.tools[0].name, "native");
    assert_eq!(catalog.tools[4].name, "ripgrep");
}

#[test]
fn loads_required_macro_fixture() {
    let value = scaffold_dsl::catalog_value_from_str(include_str!(
        "../../scaffold-dsl/src/fixtures/catalog/macro-tools.scm"
    ))
    .expect("catalog value");
    let catalog = Catalog::from_value(value).expect("catalog");

    assert_eq!(catalog.tools.len(), 2);
    assert_eq!(catalog.tools[0].name, "library-macro");
    assert_eq!(catalog.tools[0].paths[0].path, "/tmp/scaffold");
    assert_eq!(catalog.tools[1].bins[0].name, "local-macro");
}

#[test]
fn build_actions_support_multiple_commands() {
    let value = serde_json::json!({
        "tools": [{
            "name": "cmake-demo",
            "action": {
                "type": "build",
                "path": "vendor/cmake-demo",
                "argvs": [
                    ["cmake", "-S", "{{ source_dir }}", "-B", "build"],
                    ["cmake", "--build", "build"]
                ]
            }
        }]
    });
    let catalog = Catalog::from_value(value).expect("catalog");

    let super::Action::Build(action) = &catalog.tools[0].action else {
        panic!("build action");
    };
    assert_eq!(action.command_argvs().len(), 2);
    assert_eq!(action.command_argvs()[0][0], "cmake");
}

#[test]
fn package_platforms_support_multiple_install_commands() {
    let host = Host::current();
    let host_os = match host.os {
        HostOs::Linux => "linux",
        HostOs::Macos => "macos",
        HostOs::Windows => "windows",
    };
    let value = serde_json::json!({
        "tools": [{
            "name": "multi-platform",
            "action": {
                "type": "package",
                "name": "fallback-package",
                "platforms": [{
                    "when": host_os,
                    "name": "platform-package",
                    "install_argvs": [
                        ["pkg", "prepare", "{{ package }}"],
                        ["pkg", "install", "{{ package }}"]
                    ]
                }]
            }
        }]
    });
    let catalog = Catalog::from_value(value).expect("catalog");

    let super::Action::Package(action) = &catalog.tools[0].action else {
        panic!("package action");
    };
    let package = action.for_host(host).expect("platform package");
    assert_eq!(package.name, "platform-package");
    assert_eq!(package.install_argvs.len(), 2);
    assert_eq!(
        package.install_argvs[0],
        ["pkg", "prepare", "{{ package }}"]
    );
    assert_eq!(
        package.install_argvs[1],
        ["pkg", "install", "{{ package }}"]
    );
}

#[test]
fn package_platforms_infer_tool_platforms() {
    let value = serde_json::json!({
        "tools": [{
            "name": "native",
            "action": {
                "type": "package",
                "platforms": [
                    {
                        "when": "linux",
                        "install_argv": ["apt-get", "install", "{{ package }}"]
                    },
                    {
                        "when": "macos-aarch64",
                        "install_argv": ["brew", "install", "{{ package }}"]
                    },
                    {
                        "when": "macos-x86_64",
                        "install_argv": ["brew", "install", "{{ package }}"]
                    },
                    {
                        "when": "windows",
                        "install_argv": ["winget", "install", "{{ package }}"]
                    }
                ]
            }
        }]
    });
    let catalog = Catalog::from_value(value).expect("catalog");

    assert_eq!(
        catalog.tools[0].platforms,
        vec![HostOs::Linux, HostOs::Macos, HostOs::Windows]
    );
}

#[test]
fn package_platform_support_requires_matching_installer() {
    let host = Host::current();
    let host_os = match host.os {
        HostOs::Linux => "linux",
        HostOs::Macos => "macos",
        HostOs::Windows => "windows",
    };
    let value = serde_json::json!({
        "tools": [{
            "name": "native",
            "action": {
                "type": "package",
                "platforms": [{
                    "when": host_os,
                    "requires_commands": ["definitely-not-a-real-scaffold-installer"],
                    "install_argv": ["definitely-not-a-real-scaffold-installer", "install"]
                }]
            }
        }]
    });
    let catalog = Catalog::from_value(value).expect("catalog");

    assert!(!catalog.tools[0].supports_host(host));
}

#[test]
fn archive_actions_load_from_catalog_dsl() {
    let value =
        scaffold_dsl::catalog_value_from_str(include_str!("fixtures/catalog/archive-action.scm"))
            .expect("catalog value");
    let catalog = Catalog::from_value(value).expect("catalog");

    let super::Action::Archive(action) = &catalog.tools[0].action else {
        panic!("archive action");
    };
    assert_eq!(action.path, "archives/demo.tar.gz");
    assert_eq!(action.strip_components, 1);
}

#[test]
fn dmg_actions_load_from_catalog_dsl() {
    let value =
        scaffold_dsl::catalog_value_from_str(include_str!("fixtures/catalog/dmg-action.scm"))
            .expect("catalog value");
    let catalog = Catalog::from_value(value).expect("catalog");

    let super::Action::Archive(action) = &catalog.tools[0].action else {
        panic!("archive action");
    };
    assert_eq!(action.path, "archives/demo.dmg");
    assert_eq!(action.strip_components, 0);
}

#[test]
fn metadata_and_passthru_load_from_catalog_dsl() {
    let value = scaffold_dsl::catalog_value_from_str(include_str!(
        "fixtures/catalog/metadata-and-passthru.scm"
    ))
    .expect("catalog value");
    let catalog = Catalog::from_value(value).expect("catalog");

    let tool = &catalog.tools[0];
    assert_eq!(
        tool.meta.home_page.as_deref(),
        Some("https://example.test/demo")
    );
    assert_eq!(tool.meta.description.as_deref(), Some("Demo tool."));
    assert_eq!(tool.meta.license.as_deref(), Some("MIT"));
    assert_eq!(tool.meta.maintainers, ["flame", "team"]);
    assert_eq!(tool.meta.tags, ["cli"]);
    assert_eq!(tool.meta.main_program.as_deref(), Some("demo"));
    assert_eq!(
        tool.meta.source.as_deref(),
        Some("https://example.test/demo.git")
    );
    assert_eq!(tool.passthru["updater"], "manual");
}

#[test]
fn rejects_unknown_metadata_fields() {
    let error = Catalog::from_value(serde_json::json!({
        "tools": [{
            "name": "demo",
            "action": { "type": "required" },
            "meta": { "surprise": true }
        }]
    }))
    .expect_err("catalog should be invalid")
    .to_string();

    assert!(error.contains("$.tools[0].meta.surprise"));
    assert!(error.contains("not a recognized catalog field"));
}

#[test]
fn rejects_unknown_tool_fields_with_catalog_path() {
    let error = Catalog::from_value(serde_json::json!({
        "tools": [{
            "name": "demo",
            "action": { "type": "required" },
            "surprise": true
        }]
    }))
    .expect_err("catalog should be invalid")
    .to_string();

    assert!(error.contains("$.tools[0].surprise"));
    assert!(error.contains("not a recognized catalog field"));
}

#[test]
fn rejects_missing_dependency_before_install_planning() {
    let error = Catalog::from_value(serde_json::json!({
        "tools": [{
            "name": "demo",
            "depends": ["missing"],
            "action": { "type": "required" }
        }]
    }))
    .expect_err("catalog should be invalid")
    .to_string();

    assert!(error.contains("$.tools[0].depends[0]"));
    assert!(error.contains("unknown tool"));
}

#[test]
fn rejects_install_order_cycles_before_install_planning() {
    let error = Catalog::from_value(serde_json::json!({
        "tools": [
            {
                "name": "one",
                "after": ["two"],
                "action": { "type": "required" }
            },
            {
                "name": "two",
                "after": ["one"],
                "action": { "type": "required" }
            }
        ]
    }))
    .expect_err("catalog should be invalid")
    .to_string();

    assert!(error.contains("dependency cycle"));
}

#[test]
fn exposes_catalog_schema_metadata() {
    let schema = super::catalog_schema();

    assert_eq!(schema["title"], "Scaffold Catalog");
    assert!(
        schema["objects"]
            .as_array()
            .is_some_and(|objects| objects.iter().any(|object| object["name"] == "tool"))
    );
    assert_eq!(schema["relationships"][0]["cycle_checked"], true);
}

#[test]
fn load_reports_catalog_validation_with_source_span() {
    let (_root, path) = temp_path("source-aware-catalog.scm");
    std::fs::write(
        &path,
        include_str!("fixtures/catalog/source-aware-catalog.scm"),
    )
    .expect("write catalog");

    let error = Catalog::load(&path).expect_err("catalog should be invalid");
    let labeled_source = catalog_error_labeled_source(&path, error);

    assert_eq!(labeled_source, "(field 'surprise #t)");
}

#[test]
fn load_reports_catalog_validation_with_nested_span_in_catalog_form() {
    let (_root, path) = temp_path("source-aware-catalog-form.scm");
    std::fs::write(
        &path,
        include_str!("fixtures/catalog/source-aware-catalog-form.scm"),
    )
    .expect("write catalog");

    let error = Catalog::load(&path).expect_err("catalog should be invalid");
    let labeled_source = catalog_error_labeled_source(&path, error);

    assert_eq!(labeled_source, "(field 'surprise #t)");
}

fn catalog_error_labeled_source(path: &std::path::Path, error: super::CatalogError) -> String {
    let super::CatalogError::Diagnostic(diagnostic) = error else {
        panic!("expected source diagnostic");
    };
    let label = diagnostic
        .labels()
        .expect("diagnostic labels")
        .next()
        .expect("primary label");
    let source = std::fs::read_to_string(path).expect("read catalog");
    let labeled_source = &source[label.offset()..label.offset() + label.len()];

    assert_eq!(
        diagnostic.code().expect("diagnostic code").to_string(),
        "scaffold::catalog::validation"
    );
    assert_eq!(
        label.label(),
        Some("invalid catalog data was produced here")
    );
    labeled_source.to_owned()
}

fn temp_path(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join(name);
    (dir, path)
}
