use clap::{CommandFactory, Parser};
use scaffold_catalog::Catalog;
use scaffold_platform::{Host, HostArch, HostOs};
use serde_json::json;

use scaffold_install::{self as install, ToolPresenceStatus};

use super::{
    catalog_output::{
        CatalogCheckRow, catalog_check_rows, render_catalog_check, render_catalog_list,
    },
    catalog_selection::selected_catalog,
    fmt_output::render_format_check_failures,
    install_output::{render_install_events, uninstall_targets},
    io::{write_output_file, write_stream},
    path_output::{PathRow, PathStatus, path_rows, render_paths},
    test_output::{TestRow, render_test_results},
    workspace::{
        contextualize_catalog_error, require_catalog_for_discovery, require_catalog_for_workspace,
    },
};
use crate::cli::args::{CatalogArgs, Cli};

#[test]
fn help_describes_catalog_auto_discovery() {
    let mut help = Vec::new();
    crate::cli::args::Cli::command()
        .write_long_help(&mut help)
        .expect("help");
    let help = String::from_utf8(help).expect("utf8 help");

    assert!(help.contains("--catalog <FILE>"));
    assert!(help.contains("auto-discovery"));
    assert!(help.contains("SCAFFOLD_CATALOG"));
}

#[test]
fn command_help_does_not_show_dead_catalog_option() {
    for command in ["docs", "completions"] {
        let mut cli = crate::cli::args::Cli::command();
        let subcommand = cli.find_subcommand_mut(command).expect("subcommand");
        let mut help = Vec::new();
        subcommand.write_long_help(&mut help).expect("help");
        let help = String::from_utf8(help).expect("utf8 help");

        assert!(!help.contains("--catalog <FILE>"));
        assert!(!help.contains("SCAFFOLD_CATALOG"));
    }
}

#[test]
fn docs_help_describes_browse_and_export_formats() {
    let mut cli = crate::cli::args::Cli::command();
    let subcommand = cli.find_subcommand_mut("docs").expect("docs subcommand");
    let mut help = Vec::new();
    subcommand.write_long_help(&mut help).expect("help");
    let help = String::from_utf8(help).expect("utf8 help");

    assert!(help.contains("--format <FORMAT>"));
    assert!(help.contains("Render browse output or full-reference exports"));
    assert!(help.contains("use .md, .markdown, .json, or pass --format"));
    assert!(help.contains("Browse targeted Scaffold Scheme reference docs"));
    assert!(help.contains("Use --all or --output only when you need the complete"));
    assert!(help.contains("Examples:"));
    assert!(help.contains("scaffold docs --search \"ctlg tool\""));
    assert!(help.contains("scaffold docs --source src/dsl/std/catalog/tool.scm"));
    assert!(help.contains("scaffold docs --all"));
    assert!(help.contains("scaffold docs --output reference.json"));
    assert!(!help.contains("combine with --all or --output"));
}

#[test]
fn catalog_command_help_shows_catalog_option() {
    for command in [
        "analyze",
        "eval",
        "fmt",
        "install",
        "uninstall",
        "mcp",
        "repl",
        "list",
        "check",
        "test",
        "paths",
    ] {
        let mut cli = crate::cli::args::Cli::command();
        let subcommand = cli.find_subcommand_mut(command).expect("subcommand");
        let mut help = Vec::new();
        subcommand.write_long_help(&mut help).expect("help");
        let help = String::from_utf8(help).expect("utf8 help");

        assert!(help.contains("--catalog <FILE>"), "{command} help");
        assert!(!help.contains("SCAFFOLD_CATALOG"), "{command} help");
    }
}

#[test]
fn top_level_catalog_applies_to_catalog_commands() {
    let cli = Cli::parse_from(["scaffold", "--catalog", "/tmp/top-catalog", "list"]);
    let catalog = selected_catalog(&cli.command, cli.catalog).expect("catalog");

    assert_eq!(catalog, Some(std::path::PathBuf::from("/tmp/top-catalog")));
}

#[test]
fn command_catalog_overrides_top_level_catalog() {
    let cli = Cli::parse_from([
        "scaffold",
        "--catalog",
        "/tmp/top-catalog",
        "list",
        "--catalog",
        "/tmp/sub-catalog",
    ]);
    let catalog = selected_catalog(&cli.command, cli.catalog).expect("catalog");

    assert_eq!(catalog, Some(std::path::PathBuf::from("/tmp/sub-catalog")));
}

#[test]
fn top_level_catalog_is_rejected_for_builtin_docs() {
    let cli = Cli::parse_from(["scaffold", "--catalog", "/tmp/top-catalog", "docs"]);
    let message = selected_catalog(&cli.command, cli.catalog)
        .expect_err("docs should reject explicit catalog")
        .to_string();

    assert!(message.contains("docs"));
    assert!(message.contains("--catalog"));
}

#[test]
fn broken_pipe_output_is_not_reported_as_cli_failure() {
    struct BrokenPipeWriter;

    impl std::io::Write for BrokenPipeWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    write_stream(BrokenPipeWriter, "large docs output").expect("broken pipe is normal");
}

#[test]
fn output_file_writer_creates_parent_and_replaces_existing_file() {
    let root = tempfile::tempdir().expect("root");
    let output = root.path().join("nested").join("reference.md");

    write_output_file(&output, "first\n").expect("write first output");
    assert_eq!(
        std::fs::read_to_string(&output).expect("read first output"),
        "first\n"
    );

    write_output_file(&output, "second\n").expect("replace output");
    assert_eq!(
        std::fs::read_to_string(&output).expect("read replaced output"),
        "second\n"
    );

    let parent = output.parent().expect("output parent");
    let leftover_temp_files = std::fs::read_dir(parent)
        .expect("read output parent")
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(".reference.md.")
        })
        .count();
    assert_eq!(leftover_temp_files, 0);
}

#[test]
fn list_renderer_includes_catalog_metadata_and_host_status() {
    let catalog = Catalog::from_value(json!({
        "tools": [
            {
                "name": "demo",
                "bins": [{ "name": "demo-bin" }],
                "meta": { "description": "Demo tool." },
                "action": { "type": "required" }
            },
            {
                "name": "windows-only",
                "platforms": ["windows"],
                "action": { "type": "required" }
            }
        ]
    }))
    .expect("catalog");
    let rendered = render_catalog_list(
        &catalog.tools,
        Host {
            os: HostOs::Linux,
            arch: HostArch::X86_64,
        },
    );

    assert!(rendered.contains("tool"));
    assert!(rendered.contains("host"));
    assert!(rendered.contains("demo"));
    assert!(rendered.contains("demo-bin"));
    assert!(rendered.contains("Demo tool."));
    assert!(rendered.contains("supported"));
    assert!(rendered.contains("unsupported"));
}

#[test]
fn check_rows_mark_present_missing_and_unsupported_tools() {
    let current_exe = std::env::current_exe().expect("current test executable");
    let catalog = Catalog::from_value(json!({
        "tools": [
            {
                "name": "checked",
                "bins": [{ "name": current_exe.to_string_lossy(), "version_argv": [current_exe.to_string_lossy(), "--list"] }],
                "checks": [{ "argv": [current_exe.to_string_lossy(), "--list"] }],
                "action": { "type": "required" }
            },
            {
                "name": "definitely-not-a-real-scaffold-test-bin",
                "bins": [{ "name": current_exe.to_string_lossy(), "version_argv": [current_exe.to_string_lossy(), "--list"] }],
                "paths": [{ "path": "/definitely/not/a/real/scaffold/test/path" }],
                "action": { "type": "required" }
            },
            {
                "name": "windows-only",
                "platforms": ["windows"],
                "action": { "type": "required" }
            }
        ]
    }))
    .expect("catalog");
    let ctx = scaffold_context::Context {
        catalog_path: std::path::PathBuf::from("catalog.scm"),
        root_dir: std::path::PathBuf::from("."),
        bin_dir: std::path::PathBuf::from("."),
        state_dir: std::path::PathBuf::from("."),
    };

    let (rows, missing) = catalog_check_rows(
        &ctx,
        &catalog.tools,
        Host {
            os: HostOs::Linux,
            arch: HostArch::X86_64,
        },
    );

    assert_eq!(missing, 1);
    assert_eq!(rows[0].status, ToolPresenceStatus::Present);
    assert_eq!(rows[1].status, ToolPresenceStatus::Missing);
    assert_eq!(rows[2].status, ToolPresenceStatus::Unsupported);
    assert!(!rows[0].version.is_empty());
    assert_eq!(rows[1].version, "");
    assert_eq!(rows[2].version, "");
}

#[test]
fn check_renderer_uses_table_layout() {
    let rendered = render_catalog_check(&[
        CatalogCheckRow {
            name: "demo".to_owned(),
            status: ToolPresenceStatus::Present,
            version: "demo 1.0.0".to_owned(),
        },
        CatalogCheckRow {
            name: "missing".to_owned(),
            status: ToolPresenceStatus::Missing,
            version: String::new(),
        },
    ]);

    assert!(rendered.contains("tool"));
    assert!(rendered.contains("status"));
    assert!(rendered.contains("version"));
    assert!(rendered.contains("demo 1.0.0"));
    assert!(!rendered.contains("demo\tpresent"));
}

#[test]
fn test_renderer_uses_table_layout() {
    let rendered = render_test_results(&[TestRow {
        path: "extensions/acme/test.scm".to_owned(),
    }]);

    assert!(rendered.contains("test"));
    assert!(rendered.contains("status"));
    assert!(rendered.contains("extensions/acme/test.scm"));
    assert!(rendered.contains("ok"));
    assert!(!rendered.contains("ok\textensions/acme/test.scm"));
}

#[test]
fn format_check_renderer_uses_table_layout() {
    let rendered = render_format_check_failures(&[
        std::path::PathBuf::from("extensions/acme/init.scm"),
        std::path::PathBuf::from("extensions/acme/test.scm"),
    ]);

    assert!(rendered.contains("file"));
    assert!(rendered.contains("status"));
    assert!(rendered.contains("extensions/acme/init.scm"));
    assert!(rendered.contains("would reformat"));
    assert!(!rendered.contains("would reformat extensions/acme/init.scm"));
}

#[test]
fn install_event_renderer_uses_table_layout() {
    let rendered = render_install_events(&[
        install::InstallEvent {
            tool: "demo".to_owned(),
            action: install::InstallEventKind::Run,
            detail: "echo install".to_owned(),
        },
        install::InstallEvent {
            tool: "old-demo".to_owned(),
            action: install::InstallEventKind::Remove,
            detail: "/workspace/bin/demo".to_owned(),
        },
    ]);

    assert!(rendered.contains("tool"));
    assert!(rendered.contains("action"));
    assert!(rendered.contains("detail"));
    assert!(rendered.contains("demo"));
    assert!(rendered.contains("run"));
    assert!(rendered.contains("/workspace/bin/demo"));
    assert!(!rendered.contains("demo: echo install"));
}

#[test]
fn paths_renderer_uses_table_layout() {
    let rendered = render_paths(&[
        PathRow {
            kind: "catalog",
            path: "/workspace/scaffold.scm".to_owned(),
            status: PathStatus::Missing,
            resolved: String::new(),
        },
        PathRow {
            kind: "extension",
            path: "/workspace/link".to_owned(),
            status: PathStatus::Exists,
            resolved: "/workspace/actual".to_owned(),
        },
    ]);

    assert!(rendered.contains("kind"));
    assert!(rendered.contains("path"));
    assert!(rendered.contains("status"));
    assert!(rendered.contains("resolved"));
    assert!(rendered.contains("/workspace/scaffold.scm"));
    assert!(rendered.contains("missing"));
    assert!(rendered.contains("/workspace/actual"));
    assert!(!rendered.contains("catalog\t/workspace/scaffold.scm"));
}

#[test]
fn path_rows_can_include_discovered_scheme_sources() {
    let root = tempfile::tempdir().expect("root");
    let entries = root.path().join("extensions").join("entries");
    std::fs::create_dir_all(&entries).expect("entries");
    std::fs::write(root.path().join("scaffold.scm"), "(import (rnrs))\n").expect("catalog");
    std::fs::write(
        entries.join("demo.scm"),
        "(library (entries demo) (export demo) (import (rnrs)) (define demo 'ok))\n",
    )
    .expect("source");
    std::fs::write(entries.join("test.scm"), "(import (rnrs))\n").expect("test");
    let ctx = scaffold_context::Context {
        catalog_path: root.path().join("scaffold.scm"),
        root_dir: root.path().to_path_buf(),
        bin_dir: root.path().join("bin"),
        state_dir: root.path().join("state"),
    };

    let rows = path_rows(&ctx, true);

    assert!(
        rows.iter().any(|row| {
            row.kind == "source" && row.path.ends_with("extensions/entries/demo.scm")
        })
    );
    assert!(
        rows.iter()
            .any(|row| { row.kind == "test" && row.path.ends_with("extensions/entries/test.scm") })
    );
}

#[test]
fn default_discovery_error_names_missing_catalog() {
    let ctx = scaffold_context::Context {
        catalog_path: std::path::PathBuf::from("/workspace/scaffold.scm"),
        root_dir: std::path::PathBuf::from("/workspace"),
        bin_dir: std::path::PathBuf::from("."),
        state_dir: std::path::PathBuf::from("."),
    };

    let message = require_catalog_for_discovery(&ctx, "analyze")
        .expect_err("missing catalog should fail")
        .to_string();

    assert!(message.contains("no catalog found at /workspace/scaffold.scm"));
    assert!(message.contains("pass files explicitly"));
    assert!(message.contains("--catalog"));
}

#[test]
fn catalog_anchor_error_can_describe_eval_and_repl() {
    let ctx = scaffold_context::Context {
        catalog_path: std::path::PathBuf::from("/workspace/scaffold.scm"),
        root_dir: std::path::PathBuf::from("/workspace"),
        bin_dir: std::path::PathBuf::from("."),
        state_dir: std::path::PathBuf::from("."),
    };

    let eval_message = require_catalog_for_workspace(&ctx, "evaluate expressions")
        .expect_err("missing catalog should fail")
        .to_string();
    let repl_message = require_catalog_for_workspace(&ctx, "start the REPL")
        .expect_err("missing catalog should fail")
        .to_string();

    assert!(eval_message.contains("to evaluate expressions"));
    assert!(!eval_message.contains("pass files explicitly"));
    assert!(repl_message.contains("to start the REPL"));
    assert!(!repl_message.contains("pass files explicitly"));
}

#[test]
fn uninstall_requires_explicit_targets_or_all() {
    let args = crate::cli::args::UninstallArgs {
        catalog: CatalogArgs { catalog: None },
        tools: Vec::new(),
        all: false,
        dry_run: false,
    };

    let message = uninstall_targets(&args)
        .expect_err("empty uninstall target should fail")
        .to_string();

    assert!(message.contains("no tools selected"));
    assert!(message.contains("--all"));
}

#[test]
fn uninstall_all_uses_empty_target_list_for_existing_install_api() {
    let args = crate::cli::args::UninstallArgs {
        catalog: CatalogArgs { catalog: None },
        tools: Vec::new(),
        all: true,
        dry_run: true,
    };

    assert!(uninstall_targets(&args).expect("all targets").is_empty());
}

#[test]
fn catalog_io_error_names_catalog_path() {
    let ctx = scaffold_context::Context {
        catalog_path: std::path::PathBuf::from("/definitely/missing/scaffold.scm"),
        root_dir: std::path::PathBuf::from("/definitely/missing"),
        bin_dir: std::path::PathBuf::from("."),
        state_dir: std::path::PathBuf::from("."),
    };
    let err = scaffold_catalog::CatalogError::Dsl(scaffold_dsl::DslError::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "missing",
    )));

    let message = contextualize_catalog_error(&ctx, err).to_string();

    assert!(message.contains("/definitely/missing/scaffold.scm"));
    assert!(message.contains("failed while loading catalog"));
}
