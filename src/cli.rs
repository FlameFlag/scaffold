mod args;
mod docs;
mod repl;

use std::path::PathBuf;

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use comfy_table::{
    Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL_CONDENSED,
};

use scaffold_catalog::{Action, Catalog, CatalogError, Phase, Tool};
use scaffold_context::{self as context, Context};
use scaffold_dsl as dsl;
use scaffold_fmt::FormatMode;
use scaffold_install::{self as install, Policy};
use scaffold_platform::Host;

use args::{Cli, Command};

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CliError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Context(#[from] scaffold_context::ContextError),
    #[error(transparent)]
    Catalog(#[from] scaffold_catalog::CatalogError),
    #[error(transparent)]
    Install(#[from] scaffold_install::InstallError),
    #[error(transparent)]
    Format(#[from] scaffold_fmt::FormatError),
    #[error(transparent)]
    Reedline(#[from] reedline::ReedlineError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Dsl(#[from] dsl::DslError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Source(Box<scaffold_diagnostic::SourceDiagnostic>),
    #[error("language server failed: {0}")]
    Lsp(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("MCP server failed: {0}")]
    Mcp(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    Message(String),
}

impl CliError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl From<scaffold_diagnostic::SourceDiagnostic> for CliError {
    fn from(value: scaffold_diagnostic::SourceDiagnostic) -> Self {
        Self::Source(Box::new(value))
    }
}

pub fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Lsp => scaffold_lsp::run().map_err(CliError::Lsp),
        Command::Mcp => scaffold_mcp::run(cli.catalog).map_err(CliError::Mcp),
        Command::Completions(args) => {
            let mut command = Cli::command();
            let shell: Shell = args.shell.into();
            generate(shell, &mut command, "scaffold", &mut std::io::stdout());
            Ok(())
        }
        Command::Docs(args) => {
            let rendered = docs::render_docs(&args)?;
            if let Some(path) = args.output {
                ensure_output_parent(&path)?;
                std::fs::write(path, rendered)?;
            } else {
                print!("{rendered}");
            }
            Ok(())
        }
        command => run_with_context(command, cli.catalog),
    }
}

fn run_with_context(command: Command, catalog: Option<PathBuf>) -> Result<(), CliError> {
    let catalog_path = catalog.unwrap_or_else(context::default_catalog_path);
    let ctx = Context::new(catalog_path)?;

    match command {
        Command::Analyze(args) => {
            let files = if args.files.is_empty() {
                ctx.source_paths()
            } else {
                args.files
            };
            if files.is_empty() {
                return Err(CliError::message("no Scheme files found to analyze"));
            }
            let diagnostics = scaffold_analyzer::analyze_paths(&files)?;
            let has_errors = diagnostics
                .iter()
                .any(scaffold_diagnostic::SourceDiagnostic::is_error);
            for diagnostic in diagnostics {
                eprintln!("{:?}", miette::Report::new(diagnostic));
            }
            if has_errors {
                return Err(CliError::message("analysis found errors"));
            }
        }
        Command::Docs(args) => {
            let rendered = docs::render_docs(&args)?;
            if let Some(path) = args.output {
                ensure_output_parent(&path)?;
                std::fs::write(path, rendered)?;
            } else {
                print!("{rendered}");
            }
        }
        Command::Fmt(args) => {
            let files = if args.files.is_empty() {
                ctx.source_paths()
            } else {
                args.files
            };
            if files.is_empty() {
                return Err(CliError::message("no Scheme files found to format"));
            }

            let mut changed = Vec::new();
            for path in files {
                let mode = if args.check {
                    FormatMode::Check
                } else {
                    FormatMode::Write
                };
                if scaffold_fmt::format_path(&path, mode).map_err(|err| match err {
                    scaffold_fmt::FormatError::Io(err) => CliError::Io(err),
                    err => {
                        let source = std::fs::read_to_string(&path).unwrap_or_default();
                        scaffold_diagnostic::SourceDiagnostic::syntax(
                            path.display().to_string(),
                            source,
                            err.primary_offset(),
                            1,
                            err.to_string(),
                        )
                        .into()
                    }
                })? {
                    changed.push(path);
                }
            }
            if args.check && !changed.is_empty() {
                for path in &changed {
                    eprintln!("would reformat {}", path.display());
                }
                return Err(CliError::message(format!(
                    "{} files would be reformatted",
                    changed.len()
                )));
            }
        }
        Command::Eval(args) => {
            let session = dsl::session_with_catalog_path(&ctx.catalog_path, true)?;
            print_json_values(session.eval_json(&args.expression, Some("<eval>"))?)?;
        }
        Command::Install(args) => {
            install::install_catalog(
                &ctx,
                if args.force {
                    Policy::Force
                } else {
                    Policy::Missing
                },
                &args.tools,
            )
            .map_err(|err| contextualize_install_error(&ctx, err))?;
        }
        Command::Uninstall(args) => {
            install::uninstall_catalog(&ctx, &args.tools, args.dry_run)
                .map_err(|err| contextualize_install_error(&ctx, err))?;
        }
        Command::Lsp => {
            scaffold_lsp::run().map_err(CliError::Lsp)?;
        }
        Command::Mcp => {
            scaffold_mcp::run(Some(ctx.catalog_path)).map_err(CliError::Mcp)?;
        }
        Command::Repl => {
            repl::run_repl(&ctx)?;
        }
        Command::List => {
            let catalog = load_catalog(&ctx)?;
            let host = Host::current();
            print!("{}", render_catalog_list(&catalog.tools, host));
        }
        Command::Check => {
            let catalog = load_catalog(&ctx)?;
            let host = Host::current();
            let (rows, missing) = catalog_check_rows(&ctx, &catalog.tools, host);
            print!("{}", render_catalog_check(&rows));
            if missing != 0 {
                return Err(CliError::message(format!("{missing} tools missing")));
            }
        }
        Command::Test(args) => {
            let files = if args.files.is_empty() {
                ctx.test_paths()
            } else {
                args.files
            };
            if files.is_empty() {
                return Err(CliError::message("no test files found"));
            }
            for path in files {
                let _values = dsl::values_from_path_with_catalog_path(&path, &ctx.catalog_path)?;
                println!("ok\t{}", path.display());
            }
        }
        Command::Paths => {
            print!("{}", render_paths(&path_rows(&ctx)));
        }
        Command::Completions(args) => {
            let mut command = Cli::command();
            let shell: Shell = args.shell.into();
            generate(shell, &mut command, "scaffold", &mut std::io::stdout());
        }
    }

    Ok(())
}

fn render_catalog_list(tools: &[Tool], host: Host) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            header_cell("tool"),
            header_cell("host"),
            header_cell("action"),
            header_cell("phase"),
            header_cell("bins"),
            header_cell("description"),
        ]);

    for tool in tools {
        table.add_row(vec![
            Cell::new(&tool.name),
            host_status_cell(tool.supports_host(host)),
            Cell::new(action_label(&tool.action)),
            Cell::new(phase_label(tool.phase())),
            Cell::new(bin_summary(tool)),
            Cell::new(tool.meta.description.as_deref().unwrap_or_default()),
        ]);
    }

    format!("{}\n", table.trim_fmt())
}

#[derive(Debug)]
struct CatalogCheckRow {
    name: String,
    status: CheckStatus,
    version: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckStatus {
    Present,
    Missing,
    Unsupported,
}

fn catalog_check_rows(ctx: &Context, tools: &[Tool], host: Host) -> (Vec<CatalogCheckRow>, usize) {
    let mut missing = 0;
    let rows = tools
        .iter()
        .map(|tool| {
            let status = if !tool.supports_host(host) {
                CheckStatus::Unsupported
            } else if install::tool_is_present(ctx, tool) {
                CheckStatus::Present
            } else {
                missing += 1;
                CheckStatus::Missing
            };
            CatalogCheckRow {
                name: tool.name.clone(),
                status,
                version: tool.version_summary(),
            }
        })
        .collect();
    (rows, missing)
}

fn render_catalog_check(rows: &[CatalogCheckRow]) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            header_cell("tool"),
            header_cell("status"),
            header_cell("version"),
        ]);

    for row in rows {
        table.add_row(vec![
            Cell::new(&row.name),
            check_status_cell(row.status),
            Cell::new(&row.version),
        ]);
    }

    format!("{}\n", table.trim_fmt())
}

#[derive(Debug)]
struct PathRow {
    kind: &'static str,
    path: String,
    resolved: String,
}

fn path_rows(ctx: &Context) -> Vec<PathRow> {
    let mut rows = vec![
        PathRow::new("catalog", ctx.catalog_path.display().to_string()),
        PathRow::new("root", ctx.root_dir.display().to_string()),
        PathRow::new("bin", ctx.bin_dir.display().to_string()),
        PathRow::new("state", ctx.state_dir.display().to_string()),
    ];
    rows.extend(ctx.extension_dirs().into_iter().map(|dir| {
        let resolved = std::fs::canonicalize(&dir)
            .ok()
            .filter(|canonical| canonical != &dir)
            .map(|canonical| canonical.display().to_string())
            .unwrap_or_default();
        PathRow {
            kind: "extension",
            path: dir.display().to_string(),
            resolved,
        }
    }));
    rows
}

impl PathRow {
    fn new(kind: &'static str, path: String) -> Self {
        Self {
            kind,
            path,
            resolved: String::new(),
        }
    }
}

fn render_paths(rows: &[PathRow]) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            header_cell("kind"),
            header_cell("path"),
            header_cell("resolved"),
        ]);

    for row in rows {
        table.add_row(vec![
            Cell::new(row.kind),
            Cell::new(&row.path),
            Cell::new(&row.resolved),
        ]);
    }

    format!("{}\n", table.trim_fmt())
}

fn header_cell(label: &str) -> Cell {
    Cell::new(label).add_attribute(Attribute::Bold)
}

fn host_status_cell(supported: bool) -> Cell {
    if supported {
        Cell::new("supported").fg(Color::Green)
    } else {
        Cell::new("unsupported").fg(Color::Yellow)
    }
}

fn check_status_cell(status: CheckStatus) -> Cell {
    match status {
        CheckStatus::Present => Cell::new("present").fg(Color::Green),
        CheckStatus::Missing => Cell::new("missing").fg(Color::Red),
        CheckStatus::Unsupported => Cell::new("unsupported").fg(Color::Yellow),
    }
}

const fn action_label(action: &Action) -> &'static str {
    match action {
        Action::Required => "required",
        Action::Package(_) => "package",
        Action::Build(_) => "build",
        Action::Archive(_) => "archive",
    }
}

const fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Prerequisites => "prerequisites",
        Phase::Packages => "packages",
        Phase::Builds => "builds",
    }
}

fn bin_summary(tool: &Tool) -> String {
    tool.bins
        .iter()
        .map(|bin| bin.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn print_json_values(values: Vec<serde_json::Value>) -> Result<(), CliError> {
    for value in values {
        println!("{}", serde_json::to_string_pretty(&value)?);
    }
    Ok(())
}

fn ensure_output_parent(path: &std::path::Path) -> Result<(), CliError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn load_catalog(ctx: &Context) -> Result<Catalog, CliError> {
    Catalog::load(&ctx.catalog_path).map_err(|err| contextualize_catalog_error(ctx, err))
}

fn contextualize_install_error(ctx: &Context, err: install::InstallError) -> CliError {
    match err {
        install::InstallError::Catalog(err) => contextualize_catalog_error(ctx, err),
        err => CliError::Install(err),
    }
}

fn contextualize_catalog_error(ctx: &Context, err: CatalogError) -> CliError {
    match err {
        CatalogError::Dsl(dsl::DslError::Io(source)) => CliError::message(format!(
            "failed while loading catalog {}: {source}",
            ctx.catalog_path.display()
        )),
        err => CliError::Catalog(err),
    }
}

#[cfg(test)]
mod tests {
    use scaffold_catalog::Catalog;
    use scaffold_platform::{Host, HostArch, HostOs};
    use serde_json::json;

    use super::{
        CatalogCheckRow, CheckStatus, PathRow, catalog_check_rows, contextualize_catalog_error,
        render_catalog_check, render_catalog_list, render_paths,
    };

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
                    "checks": [{ "argv": [current_exe.to_string_lossy(), "--list"] }],
                    "action": { "type": "required" }
                },
                {
                    "name": "definitely-not-a-real-scaffold-test-bin",
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
        assert_eq!(rows[0].status, CheckStatus::Present);
        assert_eq!(rows[1].status, CheckStatus::Missing);
        assert_eq!(rows[2].status, CheckStatus::Unsupported);
    }

    #[test]
    fn check_renderer_uses_table_layout() {
        let rendered = render_catalog_check(&[
            CatalogCheckRow {
                name: "demo".to_owned(),
                status: CheckStatus::Present,
                version: "demo 1.0.0".to_owned(),
            },
            CatalogCheckRow {
                name: "missing".to_owned(),
                status: CheckStatus::Missing,
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
    fn paths_renderer_uses_table_layout() {
        let rendered = render_paths(&[
            PathRow::new("catalog", "/workspace/scaffold.scm".to_owned()),
            PathRow {
                kind: "extension",
                path: "/workspace/link".to_owned(),
                resolved: "/workspace/actual".to_owned(),
            },
        ]);

        assert!(rendered.contains("kind"));
        assert!(rendered.contains("path"));
        assert!(rendered.contains("resolved"));
        assert!(rendered.contains("/workspace/scaffold.scm"));
        assert!(rendered.contains("/workspace/actual"));
        assert!(!rendered.contains("catalog\t/workspace/scaffold.scm"));
    }

    #[test]
    fn catalog_io_error_names_catalog_path() {
        let ctx = scaffold_context::Context {
            catalog_path: std::path::PathBuf::from("/definitely/missing/scaffold.scm"),
            root_dir: std::path::PathBuf::from("/definitely/missing"),
            bin_dir: std::path::PathBuf::from("."),
            state_dir: std::path::PathBuf::from("."),
        };
        let err = scaffold_catalog::CatalogError::Dsl(scaffold_dsl::DslError::Io(
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        ));

        let message = contextualize_catalog_error(&ctx, err).to_string();

        assert!(message.contains("/definitely/missing/scaffold.scm"));
        assert!(message.contains("failed while loading catalog"));
    }
}
