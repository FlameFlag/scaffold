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
        let root = unique_test_dir("install-all-skips-unsupported-tools");
        std::fs::create_dir_all(&root).expect("root");
        let catalog_path = root.join("catalog.scm");
        write_catalog_fixture(
            &catalog_path,
            include_str!("../fixtures/install/all-with-unsupported.scm"),
        );
        let ctx = Context {
            catalog_path,
            root_dir: root.clone(),
            bin_dir: root.join("bin"),
            state_dir: root.join("state"),
        };

        install_catalog(&ctx, Policy::Missing, &[]).expect("install all");
        drop(std::fs::remove_dir_all(root));
    }

    #[test]
    fn explicit_install_rejects_unsupported_tools() {
        let root = unique_test_dir("explicit-install-rejects-unsupported-tools");
        std::fs::create_dir_all(&root).expect("root");
        let catalog_path = root.join("catalog.scm");
        write_catalog_fixture(
            &catalog_path,
            include_str!("../fixtures/install/unsupported-only.scm"),
        );
        let ctx = Context {
            catalog_path,
            root_dir: root.clone(),
            bin_dir: root.join("bin"),
            state_dir: root.join("state"),
        };

        assert!(matches!(
            install_catalog(&ctx, Policy::Missing, &["unsupported".to_owned()]),
            Err(InstallError::UnsupportedHost { tool }) if tool == "unsupported"
        ));
        drop(std::fs::remove_dir_all(root));
    }

    #[test]
    fn uninstall_removes_declared_paths() {
        let root = unique_test_dir("uninstall-removes-declared-paths");
        std::fs::create_dir_all(&root).expect("root");
        let trash = root.join("trash");
        std::fs::write(&trash, "remove me").expect("trash");
        let catalog_path = root.join("catalog.scm");
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
            root_dir: root.clone(),
            bin_dir: root.join("bin"),
            state_dir: root.join("state"),
        };

        uninstall_catalog(&ctx, &["demo".to_owned()], false).expect("uninstall");

        assert!(!trash.exists());
        drop(std::fs::remove_dir_all(root));
    }

    #[test]
    fn uninstall_dry_run_keeps_declared_paths() {
        let root = unique_test_dir("uninstall-dry-run-keeps-paths");
        std::fs::create_dir_all(&root).expect("root");
        let trash = root.join("trash");
        std::fs::write(&trash, "keep me").expect("trash");
        let catalog_path = root.join("catalog.scm");
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
            root_dir: root.clone(),
            bin_dir: root.join("bin"),
            state_dir: root.join("state"),
        };

        uninstall_catalog(&ctx, &["demo".to_owned()], true).expect("dry-run uninstall");

        assert!(trash.exists());
        drop(std::fs::remove_dir_all(root));
    }

    #[test]
    fn uninstall_rejects_unsafe_declared_paths() {
        let root = unique_test_dir("uninstall-rejects-unsafe-paths");
        std::fs::create_dir_all(&root).expect("root");
        let catalog_path = root.join("catalog.scm");
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
            root_dir: root.clone(),
            bin_dir: root.join("bin"),
            state_dir: root.join("state"),
        };

        assert!(matches!(
            uninstall_catalog(&ctx, &["demo".to_owned()], false),
            Err(InstallError::UnsafeUninstallPath { tool, path })
                if tool == "demo" && path == *"/"
        ));
        drop(std::fs::remove_dir_all(root));
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "scaffold-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
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
            .replace("{{ unsupported_host_os }}", unsupported_host_os_symbol());
        std::fs::write(path, source).expect("catalog");
    }
}
