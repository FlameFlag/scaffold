mod bindings;
mod error;
mod execute;
mod order;
mod presence;
mod uninstall;

use scaffold_catalog::Catalog;
use scaffold_context::Context;
use scaffold_platform::Host;

pub use error::InstallError;
pub use presence::tool_is_present;

use execute::install_tool;
use order::resolve_install_order;
use uninstall::uninstall_tool;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Policy {
    Missing,
    Force,
}

pub fn install_catalog(
    ctx: &Context,
    policy: Policy,
    names: &[String],
) -> Result<(), InstallError> {
    let catalog = Catalog::load(&ctx.catalog_path)?;
    let installing_all = names.is_empty();
    let host = Host::current();
    for tool in resolve_install_order(&catalog, names)? {
        if installing_all && !tool.supports_host(host) {
            println!("{}: unsupported, skipping", tool.name);
            continue;
        }
        install_tool(ctx, tool, policy)?;
    }
    Ok(())
}

pub fn uninstall_catalog(
    ctx: &Context,
    names: &[String],
    dry_run: bool,
) -> Result<(), InstallError> {
    let catalog = Catalog::load(&ctx.catalog_path)?;
    let mut tools = resolve_install_order(&catalog, names)?;
    tools.reverse();
    for tool in tools {
        uninstall_tool(ctx, tool, dry_run)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use scaffold_catalog::{Catalog, Tool};
    use scaffold_context::Context;
    use scaffold_platform::{Host, HostOs};

    use super::*;

    #[test]
    fn explicit_checks_can_define_presence_without_bins() {
        let current_exe = std::env::current_exe().expect("current test executable");
        let tool: Tool = serde_json::from_value(serde_json::json!({
            "name": "checked",
            "bins": [{ "name": "definitely-not-a-real-scaffold-test-bin" }],
            "checks": [{ "argv": [current_exe.to_string_lossy(), "--list"] }],
            "action": { "type": "required" }
        }))
        .expect("tool");
        let ctx = Context {
            catalog_path: PathBuf::from("catalog.scm"),
            root_dir: PathBuf::from("."),
            bin_dir: PathBuf::from("."),
            state_dir: PathBuf::from("."),
        };

        assert!(tool_is_present(&ctx, &tool));
    }

    #[test]
    fn install_order_expands_dependencies_and_honors_ordering_edges() {
        let catalog: Catalog = serde_json::from_value(serde_json::json!({
            "tools": [
                { "name": "dep", "action": { "type": "required" } },
                {
                    "name": "app",
                    "depends": ["dep"],
                    "action": { "type": "required" }
                },
                {
                    "name": "early",
                    "before": ["none"],
                    "action": { "type": "required" }
                },
                {
                    "name": "late",
                    "after": ["app"],
                    "action": { "type": "required" }
                }
            ]
        }))
        .expect("catalog");

        let order = resolve_install_order(&catalog, &["app".to_owned(), "early".to_owned()])
            .expect("install order")
            .into_iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(order, vec!["early", "dep", "app"]);

        let all_order = resolve_install_order(&catalog, &[])
            .expect("all install order")
            .into_iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(all_order, vec!["early", "dep", "app", "late"]);
    }

    #[test]
    fn install_order_rejects_cycles() {
        let catalog: Catalog = serde_json::from_value(serde_json::json!({
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
        .expect("catalog");

        assert!(matches!(
            resolve_install_order(&catalog, &[]),
            Err(InstallError::CyclicInstallOrder)
        ));
    }

    #[test]
    fn install_all_skips_unsupported_tools() {
        let root = test_root("install-all-skips-unsupported-tools");
        let catalog_path = root.path().join("catalog.scm");
        write_catalog_fixture(
            &catalog_path,
            include_str!("../fixtures/install/all-with-unsupported.scm"),
        );
        let ctx = Context {
            catalog_path,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        install_catalog(&ctx, Policy::Missing, &[]).expect("install all");
    }

    #[test]
    fn install_all_skips_package_tools_without_matching_installer() {
        let root = test_root("install-all-skips-package-tools-without-matching-installer");
        let catalog_path = root.path().join("catalog.scm");
        write_catalog_fixture(
            &catalog_path,
            include_str!("../fixtures/install/all-with-package-platform-unsupported.scm"),
        );
        let ctx = Context {
            catalog_path,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        install_catalog(&ctx, Policy::Missing, &[]).expect("install all");
    }

    #[test]
    fn explicit_install_rejects_unsupported_tools() {
        let root = test_root("explicit-install-rejects-unsupported-tools");
        let catalog_path = root.path().join("catalog.scm");
        write_catalog_fixture(
            &catalog_path,
            include_str!("../fixtures/install/unsupported-only.scm"),
        );
        let ctx = Context {
            catalog_path,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        assert!(matches!(
            install_catalog(&ctx, Policy::Missing, &["unsupported".to_owned()]),
            Err(InstallError::UnsupportedHost { tool }) if tool == "unsupported"
        ));
    }

    #[test]
    fn uninstall_removes_declared_paths() {
        let root = test_root("uninstall-removes-declared-paths");
        let trash = root.path().join("trash");
        std::fs::write(&trash, "remove me").expect("trash");
        let catalog_path = root.path().join("catalog.scm");
        std::fs::write(
            &catalog_path,
            r#"(import (rnrs) (scaffold catalog))

(catalog
  (tool "demo"
    (required)
    (field 'uninstall
      (uninstall
        (field 'paths (arr (uninstall/path "{{ root_dir }}/trash")))))))
"#,
        )
        .expect("catalog");
        let ctx = Context {
            catalog_path,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        uninstall_catalog(&ctx, &["demo".to_owned()], false).expect("uninstall");

        assert!(!trash.exists());
    }

    #[test]
    fn uninstall_dry_run_keeps_declared_paths() {
        let root = test_root("uninstall-dry-run-keeps-paths");
        let trash = root.path().join("trash");
        std::fs::write(&trash, "keep me").expect("trash");
        let catalog_path = root.path().join("catalog.scm");
        std::fs::write(
            &catalog_path,
            r#"(import (rnrs) (scaffold catalog))

(catalog
  (tool "demo"
    (required)
    (field 'uninstall
      (uninstall
        (field 'paths (arr (uninstall/path "{{ root_dir }}/trash")))))))
"#,
        )
        .expect("catalog");
        let ctx = Context {
            catalog_path,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        uninstall_catalog(&ctx, &["demo".to_owned()], true).expect("dry-run uninstall");

        assert!(trash.exists());
    }

    #[test]
    fn uninstall_rejects_unsafe_declared_paths() {
        let root = test_root("uninstall-rejects-unsafe-paths");
        let catalog_path = root.path().join("catalog.scm");
        std::fs::write(
            &catalog_path,
            r#"(import (rnrs) (scaffold catalog))

(catalog
  (tool "demo"
    (required)
    (field 'uninstall
      (uninstall
        (field 'paths (arr (uninstall/path "/")))))))
"#,
        )
        .expect("catalog");
        let ctx = Context {
            catalog_path,
            root_dir: root.path().to_path_buf(),
            bin_dir: root.path().join("bin"),
            state_dir: root.path().join("state"),
        };

        assert!(matches!(
            uninstall_catalog(&ctx, &["demo".to_owned()], false),
            Err(InstallError::UnsafeUninstallPath { tool, path })
                if tool == "demo" && path == *"/"
        ));
    }

    fn test_root(name: &str) -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix(&format!("scaffold-{name}-"))
            .tempdir()
            .expect("root")
    }

    const fn unsupported_host_os_symbol() -> &'static str {
        match Host::current().os {
            HostOs::Linux => "windows",
            HostOs::Macos | HostOs::Windows => "linux",
        }
    }

    fn escape_scheme_string(value: &str) -> String {
        value.replace('\\', "\\\\").replace('"', "\\\"")
    }

    fn write_catalog_fixture(path: &std::path::Path, fixture: &str) {
        let current_exe = std::env::current_exe().expect("current test executable");
        let source = fixture
            .replace(
                "{{ current_exe }}",
                &escape_scheme_string(&current_exe.to_string_lossy()),
            )
            .replace("{{ current_host_os }}", current_host_os_symbol())
            .replace("{{ unsupported_host_os }}", unsupported_host_os_symbol());
        std::fs::write(path, source).expect("catalog");
    }

    const fn current_host_os_symbol() -> &'static str {
        match Host::current().os {
            HostOs::Linux => "linux",
            HostOs::Macos => "macos",
            HostOs::Windows => "windows",
        }
    }
}
