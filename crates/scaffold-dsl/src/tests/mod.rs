use std::path::{Path, PathBuf};

use super::*;

mod helpers;

use helpers::{assert_platform_extension_files, extension_path, fixture_path, single_value};

#[test]
fn reads_basic_scheme_data() {
    let value = single_value(include_str!("../fixtures/eval/basic-values.scm"));

    assert_eq!(value["name"], "demo");
    assert_eq!(value["argv"], serde_json::json!(["cargo", "test"]));
    assert_eq!(value["nested_value"]["kind"], "package");
}

#[test]
fn rejects_imports_outside_scaffold_policy() {
    let error = values_from_str(include_str!("../fixtures/eval/disallowed-import.scm"))
        .expect_err("non-allowed import should fail");

    assert!(error.to_string().contains("evaluation failed"));
}

#[test]
fn rejects_editor_and_wasm_evaluation_modes() {
    for mode in [eval::DslEvalMode::Editor, eval::DslEvalMode::Wasm] {
        let result =
            DslSession::with_context(&[], false, eval::DslEvalContext::default().with_mode(mode));
        let Err(error) = result else {
            panic!("reference-only modes should not evaluate");
        };

        assert!(error.to_string().contains("not available"));
    }
}

#[test]
fn reports_parse_errors_as_diagnostics() {
    let error = values_from_str(include_str!("../fixtures/eval/invalid-syntax.fixture"))
        .expect_err("invalid syntax should fail");

    assert!(matches!(error, DslError::Diagnostic(_)));
    assert!(error.to_string().contains("Scheme syntax failed"));
}

#[test]
fn supports_top_level_helper_definitions() {
    let value = single_value(include_str!("../fixtures/eval/top-level-helpers.scm"));

    assert_eq!(value["name"], "demo");
}

#[test]
fn supports_local_syntax_rules_macros() {
    let value = single_value(include_str!("../fixtures/eval/local-syntax-rules.scm"));

    assert_eq!(value["name"], "demo");
    assert_eq!(
        value["argv"],
        serde_json::json!(["cargo", "test", "--quiet"])
    );
}

#[test]
fn supports_top_level_syntax_rules_macros() {
    let value = single_value(include_str!("../fixtures/eval/top-level-syntax-rules.scm"));

    assert_eq!(value["name"], "demo");
    assert_eq!(
        value["argv"],
        serde_json::json!(["cargo", "test", "--quiet"])
    );
}

#[test]
fn supports_multiple_output_values() {
    let values =
        values_from_str(include_str!("../fixtures/eval/multiple-values.scm")).expect("values");

    assert_eq!(values.len(), 2);
    assert_eq!(values[0]["name"], "one");
    assert_eq!(values[1]["name"], "two");
}

#[test]
fn supports_scaffold_config_library() {
    let value = single_value(include_str!("../fixtures/eval/config-import.scm"));

    assert_eq!(value["name"], "demo");
    assert_eq!(value["argv"], serde_json::json!(["cargo", "test"]));
}

#[test]
fn rejects_internal_core_imports_from_catalog_files() {
    let error = values_from_str("(import (rnrs) (scaffold core object))")
        .expect_err("internal stdlib module import should fail");

    assert!(error.to_string().contains("evaluation failed"));
}

#[test]
fn config_helpers_support_object_transformations() {
    let value = single_value(include_str!("../fixtures/eval/object-transformations.scm"));

    assert_eq!(value["name"], "demo-override");
    assert_eq!(value["phase"], "missing");
    assert_eq!(value["bins"], serde_json::json!(["demo", "democtl"]));
    assert_eq!(value["platforms"], serde_json::json!(["linux"]));
}

#[test]
fn supports_generated_scaffold_host_library() {
    let value = single_value(include_str!("../fixtures/eval/generated-host.scm"));

    assert!(matches!(
        value["os"].as_str(),
        Some("macos" | "linux" | "windows")
    ));
    assert!(matches!(value["arch"].as_str(), Some("aarch64" | "x86_64")));
    assert!(
        value["platform"]
            .as_str()
            .is_some_and(|platform| platform.contains('-'))
    );
    assert!(value["commands"].is_array());
    assert_eq!(
        value["has_shell"],
        scaffold_process::path_of("sh").is_some()
    );
    assert_eq!(
        value["has_shell_path"],
        scaffold_process::path_of("sh").is_some()
    );
    if scaffold_process::path_of("sh").is_some() {
        let shell_path = scaffold_process::path_of("sh")
            .expect("shell path")
            .to_string_lossy()
            .into_owned();
        assert_eq!(value["shell_path"], shell_path);
        assert!(
            value["commands"]
                .as_array()
                .is_some_and(|commands| commands.iter().any(|command| command == "sh"))
        );
    } else {
        assert_eq!(value["shell_path"], false);
    }
    assert_eq!(value["missing_command_path"], false);
    assert_eq!(value["missing_command_path?"], false);
    assert_eq!(value["matches_os"], true);
    assert_eq!(value["matches_platform"], true);
    assert_eq!(value["matches_predicate"], true);
    assert_eq!(value["matches_wrong_os"], false);
    assert_eq!(value["matches_invalid"], false);
    assert_eq!(value["has_path_env"], std::env::var("PATH").is_ok());
    assert_eq!(value["missing_env"], false);
}

#[test]
fn supports_scaffold_path_library() {
    let value = single_value(include_str!("../fixtures/eval/path-builtins.scm"));
    let joined = PathBuf::from("vendor")
        .join("rg")
        .to_string_lossy()
        .into_owned();
    let parent = PathBuf::from("vendor")
        .join("rg")
        .to_string_lossy()
        .into_owned();

    assert_eq!(value["separator"], std::path::MAIN_SEPARATOR.to_string());
    assert_eq!(value["joined"], joined);
    assert_eq!(value["normalized"], joined);
    assert_eq!(value["parent"], parent);
    assert_eq!(value["file_name"], "Cargo.toml");
    assert_eq!(value["extension"], "gz");
    assert_eq!(
        value["absolute?"],
        Path::new(&std::path::MAIN_SEPARATOR.to_string()).is_absolute()
    );
    assert_eq!(value["relative?"], true);
}

#[test]
fn supports_scaffold_workspace_library_for_file_evaluation() {
    let path = fixture_path("eval/workspace-builtins.scm");
    let value = values_from_path(&path)
        .expect("scheme values")
        .into_iter()
        .next()
        .expect("single value");
    let root = path.parent().expect("fixture parent");
    let workspace_file = root
        .join("nested")
        .join("file.txt")
        .to_string_lossy()
        .into_owned();

    assert_eq!(value["workspace_root"], root.to_string_lossy().into_owned());
    assert_eq!(value["source_path"], path.to_string_lossy().into_owned());
    assert_eq!(value["source_dir"], root.to_string_lossy().into_owned());
    assert_eq!(value["workspace_file"], workspace_file);
}

#[test]
fn supports_scaffold_fs_library_for_absolute_workspace_paths() {
    let path = fixture_path("eval/fs-builtins.scm");
    let value = values_from_path(&path)
        .expect("scheme values")
        .into_iter()
        .next()
        .expect("single value");

    assert_eq!(value["source_exists"], true);
    assert_eq!(value["source_file"], true);
    assert_eq!(value["source_dir"], false);
    assert_eq!(value["root_exists"], true);
    assert_eq!(value["root_dir"], true);
    assert_eq!(value["missing_exists"], false);
    assert_eq!(value["missing_file"], false);
}

#[test]
fn evaluates_scaffold_catalog_helpers() {
    let _values = values_from_str(include_str!("../fixtures/extensions/composition.scm"))
        .expect("scheme test");
}

#[test]
fn evaluates_catalog_domain_helpers() {
    let path = fixture_path("eval/workspace-builtins.scm");
    let value = values_from_path(&path)
        .expect("scheme values")
        .into_iter()
        .find(|value| value["name"] == "helper-demo")
        .expect("helper tool");
    let source_path = path
        .parent()
        .expect("fixture parent")
        .join("vendor")
        .join("demo")
        .to_string_lossy()
        .into_owned();

    assert_eq!(value["action"]["type"], "build");
    assert_eq!(value["action"]["path"], source_path);
    assert_eq!(value["platforms"], serde_json::json!(["linux", "macos"]));
    assert_eq!(
        value["uninstall"]["paths"][0]["path"],
        "{{ home }}/.helper-demo"
    );
}

#[test]
fn supports_focused_catalog_module_imports() {
    let _values = values_from_str(include_str!("../fixtures/catalog/focused-imports.scm"))
        .expect("scheme test");
}

#[cfg(unix)]
#[test]
fn catalog_loading_dedupes_symlinked_extension_libraries() {
    let root = tempfile::Builder::new()
        .prefix("scaffold-dsl-catalog-loading-dedupes-symlinked-extension-libraries-")
        .tempdir()
        .expect("root");
    let entries = root.path().join("scaffold").join("entries");
    let scaffold_dot_extensions = root.path().join(".scaffold").join("extensions");
    std::fs::create_dir_all(&entries).expect("entries");
    std::fs::create_dir_all(&scaffold_dot_extensions).expect("dot extensions");
    std::fs::write(
        root.path().join("scaffold.scm"),
        r#"(import (rnrs) (scaffold catalog) (entries demo))

(catalog demo)
"#,
    )
    .expect("catalog");
    std::fs::write(
        entries.join("demo.scm"),
        r#"(library
  (entries demo)
  (export demo)
  (import (rnrs) (scaffold catalog))

  (define demo (tool "demo" (required))))
"#,
    )
    .expect("entry");
    std::os::unix::fs::symlink(
        "../../scaffold/entries",
        scaffold_dot_extensions.join("entries"),
    )
    .expect("symlink");

    let value = catalog_value_from_path(root.path().join("scaffold.scm")).expect("catalog");

    assert_eq!(value["tools"][0]["name"], "demo");
}

#[test]
fn supports_scoped_r6rs_import_sets_for_scaffold_libraries() {
    let value = catalog_value_from_str(
        r#"(import
  (rnrs)
  (only (scaffold catalog root) catalog)
  (only (scaffold catalog action) required)
  (only (scaffold catalog tool) tool bin override)
  (only (scaffold config vector) arr)
  (only (scaffold config object) object field object/ref))

(define base
  (tool "demo" (required) (field 'bins (arr (bin "demo")))))

(catalog
  (override
    base
    (lambda (old)
      (object
        (field 'name (string-append (object/ref old 'name) "-scoped"))
        (field 'bins (arr (bin (string-append (object/ref old 'name) "-scoped"))))))))"#,
    )
    .expect("catalog");

    assert_eq!(value["tools"][0]["name"], "demo-scoped");
    assert_eq!(value["tools"][0]["bins"][0]["name"], "demo-scoped");
}

#[test]
fn supports_r6rs_rename_prefix_and_records_in_catalog_dsl() {
    let value = catalog_value_from_str(
        r#"(import
  (rnrs)
  (rename (scaffold catalog root) (catalog make-catalog))
  (rename (scaffold catalog action) (required action-required))
  (prefix (scaffold catalog tool) tool:)
  (only (scaffold config vector) arr)
  (only (scaffold config object) field))

(define-record-type tool-spec
  (fields name bin))

(define spec (make-tool-spec "demo" "democtl"))

(make-catalog
  (tool:tool
    (tool-spec-name spec)
    (action-required)
    (field 'bins (arr (tool:bin (tool-spec-bin spec))))))"#,
    )
    .expect("catalog");

    assert_eq!(value["tools"][0]["name"], "demo");
    assert_eq!(value["tools"][0]["bins"][0]["name"], "democtl");
}

#[test]
fn bundled_scheme_extensions_emit_presence_checks() {
    let _values = values_from_str(include_str!("../fixtures/extensions/presence-checks.scm"))
        .expect("scheme test");
}

#[test]
fn bundled_scheme_extensions_emit_uninstall_commands() {
    let _values = values_from_str(include_str!("../fixtures/extensions/package-lifecycle.scm"))
        .expect("scheme test");
}

#[test]
fn bundled_scheme_extensions_cover_windows_and_build_tools() {
    let _values = values_from_str(include_str!("../fixtures/extensions/build-and-winget.scm"))
        .expect("scheme test");
}

#[test]
fn bundled_scheme_extensions_can_import_each_other() {
    let _values = values_from_str(include_str!(
        "../fixtures/extensions/import-dependencies.scm"
    ))
    .expect("scheme test");
}

#[test]
fn bundled_scheme_extensions_cover_macos_tools() {
    let _values = values_from_str(include_str!("../fixtures/extensions/platform-macos.scm"))
        .expect("scheme test");
}

#[test]
fn bundled_scheme_extensions_cover_windows_tools() {
    let _values = values_from_str(include_str!("../fixtures/extensions/platform-windows.scm"))
        .expect("scheme test");
}

#[test]
fn bundled_platform_extensions_stay_within_scaffold_scope() {
    assert_platform_extension_files("macos", &["base.scm", "mod.scm"]);
    assert_platform_extension_files(
        "windows",
        &["base.scm", "mod.scm", "registry.scm", "shell.scm"],
    );
}

#[test]
fn distrobox_extension_wraps_package_install_argvs_in_scheme() {
    let _values = values_from_str(include_str!(
        "../fixtures/extensions/distrobox-transform.scm"
    ))
    .expect("scheme test");
}

#[test]
fn supports_catalog_macros() {
    let value = catalog_value_from_str(include_str!("../fixtures/catalog/macro-tools.scm"))
        .expect("catalog");

    assert_eq!(value["tools"][0]["name"], "library-macro");
    assert_eq!(value["tools"][1]["name"], "local-macro");
    assert_eq!(
        value["tools"][1]["bins"][0]["version_argv"],
        serde_json::json!(["local-macro", "--version"])
    );
}

#[test]
fn supports_catalog_metadata_helpers() {
    let value = catalog_value_from_str(
        r#"(import (rnrs) (scaffold catalog))

(catalog
  (tool "demo"
    (required)
    (meta
      (home-page "https://example.test/demo")
      (description "Demo tool.")
      (license "MIT")
      (maintainers "flame")
      (tags "cli" "demo")
      (main-program "demo")
      (source "https://example.test/demo.git"))
    (passthru (field 'updater "manual"))))
"#,
    )
    .expect("catalog");

    assert_eq!(
        value["tools"][0]["meta"]["home_page"],
        "https://example.test/demo"
    );
    assert_eq!(value["tools"][0]["meta"]["main_program"], "demo");
    assert_eq!(value["tools"][0]["meta"]["maintainers"][0], "flame");
    assert_eq!(value["tools"][0]["passthru"]["updater"], "manual");
}

#[test]
fn supports_tool_override_helpers() {
    let value = catalog_value_from_str(
        r#"(import (rnrs) (scaffold catalog))

(define base-tool
  (tool
    "demo"
    (required)
    (field 'bins (arr (bin "demo")))))

(catalog
  (tool/override
    base-tool
    (lambda (old)
      (object
        (field 'name (string-append (object/ref old 'name) "-nightly"))
        (field
          'bins
          (arr (bin (string-append (object/ref old 'name) "-nightly"))))))))
"#,
    )
    .expect("catalog");

    assert_eq!(value["tools"][0]["name"], "demo-nightly");
    assert_eq!(value["tools"][0]["bins"][0]["name"], "demo-nightly");
}

#[test]
fn supports_raw_catalog_shape() {
    let value = catalog_value_from_str(include_str!("../fixtures/catalog/raw-required-tool.scm"))
        .expect("catalog");

    assert_eq!(value["tools"][0]["name"], "demo");
    assert_eq!(value["tools"][0]["action"]["type"], "required");
}

#[test]
fn ignores_top_level_doc_forms_as_catalog_output() {
    let value = catalog_value_from_str(include_str!("../fixtures/catalog/ignores-doc-forms.scm"))
        .expect("catalog");

    assert_eq!(value["tools"].as_array().map(Vec::len), Some(1));
    assert_eq!(value["tools"][0]["name"], "demo");
}

#[test]
fn loads_catalog_local_scheme_extension_libraries() {
    let _values = values_from_path(fixture_path("local/library/test.scm")).expect("scheme test");
}

#[test]
fn local_scheme_extension_libraries_can_import_each_other() {
    let _values =
        values_from_path(fixture_path("local/dependencies/test.scm")).expect("scheme test");
}

#[test]
fn catalog_stem_directory_is_a_local_library_root() {
    let value =
        catalog_value_from_path(fixture_path("local/catalog-stem/scaffold.scm")).expect("catalog");

    assert_eq!(value["tools"][0]["name"], "demo");
}

#[test]
fn bundled_extension_tests_live_with_extensions() {
    for path in [
        "distro/nix/tests/base/test.scm",
        "distro/nix/tests/profile/test.scm",
    ] {
        let _values = values_from_path_with_extension_root(
            extension_path(path),
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src"),
        )
        .unwrap_or_else(|err| panic!("{path} failed: {err}"));
    }
}
