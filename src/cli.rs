mod args;
mod docs;
mod repl;
mod table;

use std::{
    ffi::OsString,
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};
use comfy_table::{Cell, Color};

use scaffold_catalog::{Catalog, CatalogError, Tool};
use scaffold_context::{self as context, Context};
use scaffold_dsl as dsl;
use scaffold_fmt::FormatMode;
use scaffold_install::{self as install, Policy, ToolPresenceStatus};
use scaffold_platform::Host;

use args::{Cli, Command, UninstallArgs};
use table::{DEFAULT_TABLE_WIDTH, header_cell, output_table};

enum ContextCommand {
    Analyze(args::AnalyzeArgs),
    Eval(args::EvalArgs),
    Fmt(args::FmtArgs),
    Install(args::InstallArgs),
    Uninstall(args::UninstallArgs),
    Repl,
    List,
    Check,
    Test(args::TestArgs),
    Paths(args::PathsArgs),
}

struct CommandInfo {
    name: &'static str,
    catalog: CatalogSupport,
}

enum CatalogSupport {
    Accepted(Option<PathBuf>),
    Rejected,
}

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
    let catalog = selected_catalog(&cli.command, cli.catalog)?;

    match cli.command {
        Command::Lsp => scaffold_lsp::run().map_err(CliError::Lsp),
        Command::Mcp(_) => scaffold_mcp::run(catalog).map_err(CliError::Mcp),
        Command::Completions(args) => {
            let mut command = Cli::command();
            let shell: Shell = args.shell.into();
            generate(shell, &mut command, "scaffold", &mut std::io::stdout());
            Ok(())
        }
        Command::Docs(args) => {
            let rendered = docs::render_docs(&args)?;
            if let Some(path) = args.output {
                write_output_file(&path, &rendered)?;
            } else {
                write_stdout(&rendered)?;
            }
            Ok(())
        }
        Command::Analyze(args) => run_with_context(ContextCommand::Analyze(args), catalog),
        Command::Eval(args) => run_with_context(ContextCommand::Eval(args), catalog),
        Command::Fmt(args) => run_with_context(ContextCommand::Fmt(args), catalog),
        Command::Install(args) => run_with_context(ContextCommand::Install(args), catalog),
        Command::Uninstall(args) => run_with_context(ContextCommand::Uninstall(args), catalog),
        Command::Repl(_) => run_with_context(ContextCommand::Repl, catalog),
        Command::List(_) => run_with_context(ContextCommand::List, catalog),
        Command::Check(_) => run_with_context(ContextCommand::Check, catalog),
        Command::Test(args) => run_with_context(ContextCommand::Test(args), catalog),
        Command::Paths(args) => run_with_context(ContextCommand::Paths(args), catalog),
    }
}

fn selected_catalog(
    command: &Command,
    top_level_catalog: Option<PathBuf>,
) -> Result<Option<PathBuf>, CliError> {
    let info = command_info(command);
    match info.catalog {
        CatalogSupport::Accepted(catalog) => Ok(catalog.or(top_level_catalog).or_else(env_catalog)),
        CatalogSupport::Rejected if top_level_catalog.is_some() => Err(CliError::message(format!(
            "`{}` does not use --catalog",
            info.name
        ))),
        CatalogSupport::Rejected => Ok(None),
    }
}

fn env_catalog() -> Option<PathBuf> {
    std::env::var_os("SCAFFOLD_CATALOG")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn command_info(command: &Command) -> CommandInfo {
    match command {
        Command::Analyze(args) => CommandInfo {
            name: "analyze",
            catalog: CatalogSupport::Accepted(args.catalog.catalog.clone()),
        },
        Command::Docs(_) => CommandInfo {
            name: "docs",
            catalog: CatalogSupport::Rejected,
        },
        Command::Eval(args) => CommandInfo {
            name: "eval",
            catalog: CatalogSupport::Accepted(args.catalog.catalog.clone()),
        },
        Command::Fmt(args) => CommandInfo {
            name: "fmt",
            catalog: CatalogSupport::Accepted(args.catalog.catalog.clone()),
        },
        Command::Install(args) => CommandInfo {
            name: "install",
            catalog: CatalogSupport::Accepted(args.catalog.catalog.clone()),
        },
        Command::Uninstall(args) => CommandInfo {
            name: "uninstall",
            catalog: CatalogSupport::Accepted(args.catalog.catalog.clone()),
        },
        Command::Lsp => CommandInfo {
            name: "lsp",
            catalog: CatalogSupport::Rejected,
        },
        Command::Mcp(args) => CommandInfo {
            name: "mcp",
            catalog: CatalogSupport::Accepted(args.catalog.clone()),
        },
        Command::Repl(args) => CommandInfo {
            name: "repl",
            catalog: CatalogSupport::Accepted(args.catalog.clone()),
        },
        Command::List(args) => CommandInfo {
            name: "list",
            catalog: CatalogSupport::Accepted(args.catalog.clone()),
        },
        Command::Check(args) => CommandInfo {
            name: "check",
            catalog: CatalogSupport::Accepted(args.catalog.clone()),
        },
        Command::Test(args) => CommandInfo {
            name: "test",
            catalog: CatalogSupport::Accepted(args.catalog.catalog.clone()),
        },
        Command::Paths(args) => CommandInfo {
            name: "paths",
            catalog: CatalogSupport::Accepted(args.catalog.catalog.clone()),
        },
        Command::Completions(_) => CommandInfo {
            name: "completions",
            catalog: CatalogSupport::Rejected,
        },
    }
}

fn run_with_context(command: ContextCommand, catalog: Option<PathBuf>) -> Result<(), CliError> {
    let catalog_path = catalog.unwrap_or_else(context::default_catalog_path);
    let ctx = Context::new(catalog_path)?;

    match command {
        ContextCommand::Analyze(args) => {
            let files = if args.files.is_empty() {
                discover_source_paths(&ctx, "analyze")?
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
                write_stderr(&format!("{:?}\n", miette::Report::new(diagnostic)))?;
            }
            if has_errors {
                return Err(CliError::message("analysis found errors"));
            }
        }
        ContextCommand::Fmt(args) => {
            let files = if args.files.is_empty() {
                discover_source_paths(&ctx, "format")?
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
                write_stderr(&render_format_check_failures(&changed))?;
                return Err(CliError::message(format!(
                    "{} files would be reformatted",
                    changed.len()
                )));
            }
        }
        ContextCommand::Eval(args) => {
            require_catalog_for_workspace(&ctx, "evaluate expressions")?;
            let session = dsl::session_with_catalog_path(&ctx.catalog_path, true)?;
            print_json_values(session.eval_json(&args.expression, Some("<eval>"))?)?;
        }
        ContextCommand::Install(args) => {
            let mut reporter = CliInstallReporter::default();
            let result = install::install_catalog_with_reporter(
                &ctx,
                if args.force {
                    Policy::Force
                } else {
                    Policy::Missing
                },
                &args.tools,
                &mut reporter,
            );
            write_stdout(&render_install_events(reporter.events()))?;
            result.map_err(|err| contextualize_install_error(&ctx, err))?;
        }
        ContextCommand::Uninstall(args) => {
            let mut reporter = CliInstallReporter::default();
            let result = install::uninstall_catalog_with_reporter(
                &ctx,
                uninstall_targets(&args)?,
                args.dry_run,
                &mut reporter,
            );
            write_stdout(&render_install_events(reporter.events()))?;
            result.map_err(|err| contextualize_install_error(&ctx, err))?;
        }
        ContextCommand::Repl => {
            require_catalog_for_workspace(&ctx, "start the REPL")?;
            repl::run_repl(&ctx)?;
        }
        ContextCommand::List => {
            let catalog = load_catalog(&ctx)?;
            let host = Host::current();
            write_stdout(&render_catalog_list(&catalog.tools, host))?;
        }
        ContextCommand::Check => {
            let catalog = load_catalog(&ctx)?;
            let host = Host::current();
            let (rows, missing) = catalog_check_rows(&ctx, &catalog.tools, host);
            write_stdout(&render_catalog_check(&rows))?;
            if missing != 0 {
                return Err(CliError::message(format!("{missing} tools missing")));
            }
        }
        ContextCommand::Test(args) => {
            let files = if args.files.is_empty() {
                discover_test_paths(&ctx)?
            } else {
                args.files
            };
            if files.is_empty() {
                return Err(CliError::message("no test files found"));
            }
            let mut rows = Vec::new();
            for path in files {
                let _values = dsl::values_from_path_with_catalog_path(&path, &ctx.catalog_path)?;
                rows.push(TestRow {
                    path: path.display().to_string(),
                    status: TestStatus::Ok,
                });
            }
            write_stdout(&render_test_results(&rows))?;
        }
        ContextCommand::Paths(args) => {
            write_stdout(&render_paths(&path_rows(&ctx, args.sources)))?;
        }
    }

    Ok(())
}

fn render_catalog_list(tools: &[Tool], host: Host) -> String {
    let mut table = output_table(Some(DEFAULT_TABLE_WIDTH));
    table.set_header(vec![
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
            Cell::new(tool.action.label()),
            Cell::new(tool.phase().label()),
            Cell::new(bin_summary(tool)),
            Cell::new(tool.meta.description.as_deref().unwrap_or_default()),
        ]);
    }

    format!("{}\n", table.trim_fmt())
}

#[derive(Debug)]
struct CatalogCheckRow {
    name: String,
    status: ToolPresenceStatus,
    version: String,
}

fn catalog_check_rows(ctx: &Context, tools: &[Tool], host: Host) -> (Vec<CatalogCheckRow>, usize) {
    let mut missing = 0;
    let rows = tools
        .iter()
        .map(|tool| {
            let status = install::tool_presence_status(ctx, tool, host);
            if status == ToolPresenceStatus::Missing {
                missing += 1;
            }
            let version = if status == ToolPresenceStatus::Present {
                tool.version_summary()
            } else {
                String::new()
            };
            CatalogCheckRow {
                name: tool.name.clone(),
                status,
                version,
            }
        })
        .collect();
    (rows, missing)
}

fn render_catalog_check(rows: &[CatalogCheckRow]) -> String {
    let mut table = output_table(Some(DEFAULT_TABLE_WIDTH));
    table.set_header(vec![
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
struct TestRow {
    path: String,
    status: TestStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TestStatus {
    Ok,
}

fn render_test_results(rows: &[TestRow]) -> String {
    let mut table = output_table(Some(DEFAULT_TABLE_WIDTH));
    table.set_header(vec![header_cell("test"), header_cell("status")]);

    for row in rows {
        table.add_row(vec![Cell::new(&row.path), test_status_cell(row.status)]);
    }

    format!("{}\n", table.trim_fmt())
}

fn render_format_check_failures(paths: &[PathBuf]) -> String {
    let mut table = output_table(Some(DEFAULT_TABLE_WIDTH));
    table.set_header(vec![header_cell("file"), header_cell("status")]);

    for path in paths {
        table.add_row(vec![
            Cell::new(path.display().to_string()),
            Cell::new("would reformat").fg(Color::Yellow),
        ]);
    }

    format!("{}\n", table.trim_fmt())
}

#[derive(Default)]
struct CliInstallReporter {
    events: Vec<install::InstallEvent>,
}

impl CliInstallReporter {
    fn events(&self) -> &[install::InstallEvent] {
        &self.events
    }
}

impl install::InstallReporter for CliInstallReporter {
    fn report(&mut self, event: install::InstallEvent) {
        self.events.push(event);
    }
}

fn render_install_events(events: &[install::InstallEvent]) -> String {
    if events.is_empty() {
        return String::new();
    }

    let mut table = output_table(Some(DEFAULT_TABLE_WIDTH));
    table.set_header(vec![
        header_cell("tool"),
        header_cell("action"),
        header_cell("detail"),
    ]);

    for event in events {
        table.add_row(vec![
            Cell::new(&event.tool),
            install_event_cell(event.action),
            Cell::new(&event.detail),
        ]);
    }

    format!("{}\n", table.trim_fmt())
}

#[derive(Debug)]
struct PathRow {
    kind: &'static str,
    path: String,
    status: PathStatus,
    resolved: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PathStatus {
    Exists,
    Missing,
}

fn path_rows(ctx: &Context, include_sources: bool) -> Vec<PathRow> {
    let mut rows = vec![
        PathRow::new("catalog", &ctx.catalog_path),
        PathRow::new("root", &ctx.root_dir),
        PathRow::new("bin", &ctx.bin_dir),
        PathRow::new("state", &ctx.state_dir),
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
            status: path_status(&dir),
            resolved,
        }
    }));
    if include_sources {
        rows.extend(
            ctx.source_paths()
                .into_iter()
                .map(|path| PathRow::new("source", &path)),
        );
        rows.extend(
            ctx.test_paths()
                .into_iter()
                .map(|path| PathRow::new("test", &path)),
        );
    }
    rows
}

impl PathRow {
    fn new(kind: &'static str, path: &std::path::Path) -> Self {
        Self {
            kind,
            path: path.display().to_string(),
            status: path_status(path),
            resolved: String::new(),
        }
    }
}

fn render_paths(rows: &[PathRow]) -> String {
    let mut table = output_table(Some(DEFAULT_TABLE_WIDTH));
    table.set_header(vec![
        header_cell("kind"),
        header_cell("path"),
        header_cell("status"),
        header_cell("resolved"),
    ]);

    for row in rows {
        table.add_row(vec![
            Cell::new(row.kind),
            Cell::new(&row.path),
            path_status_cell(row.status),
            Cell::new(&row.resolved),
        ]);
    }

    format!("{}\n", table.trim_fmt())
}

fn host_status_cell(supported: bool) -> Cell {
    if supported {
        Cell::new("supported").fg(Color::Green)
    } else {
        Cell::new("unsupported").fg(Color::Yellow)
    }
}

fn check_status_cell(status: ToolPresenceStatus) -> Cell {
    match status {
        ToolPresenceStatus::Present => Cell::new(status.label()).fg(Color::Green),
        ToolPresenceStatus::Missing => Cell::new(status.label()).fg(Color::Red),
        ToolPresenceStatus::Unsupported => Cell::new(status.label()).fg(Color::Yellow),
    }
}

fn test_status_cell(status: TestStatus) -> Cell {
    match status {
        TestStatus::Ok => Cell::new("ok").fg(Color::Green),
    }
}

fn path_status(path: &std::path::Path) -> PathStatus {
    if path.exists() {
        PathStatus::Exists
    } else {
        PathStatus::Missing
    }
}

fn path_status_cell(status: PathStatus) -> Cell {
    match status {
        PathStatus::Exists => Cell::new("exists").fg(Color::Green),
        PathStatus::Missing => Cell::new("missing").fg(Color::Yellow),
    }
}

fn install_event_cell(action: install::InstallEventKind) -> Cell {
    let cell = Cell::new(action.label());
    match action {
        install::InstallEventKind::Present => cell.fg(Color::Green),
        install::InstallEventKind::Skip => cell.fg(Color::Yellow),
        install::InstallEventKind::Run => cell.fg(Color::Cyan),
        install::InstallEventKind::Extract => cell.fg(Color::Cyan),
        install::InstallEventKind::Remove => cell.fg(Color::Red),
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
        write_stdout(&format!("{}\n", serde_json::to_string_pretty(&value)?))?;
    }
    Ok(())
}

fn write_output_file(path: &Path, text: &str) -> Result<(), CliError> {
    ensure_output_parent(path)?;
    let temporary_path = temporary_output_path(path);

    if let Err(err) = std::fs::write(&temporary_path, text) {
        drop(std::fs::remove_file(&temporary_path));
        return Err(CliError::Io(err));
    }

    if let Err(err) = std::fs::rename(&temporary_path, path) {
        drop(std::fs::remove_file(&temporary_path));
        return Err(CliError::Io(err));
    }

    Ok(())
}

fn temporary_output_path(path: &Path) -> PathBuf {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("output"));
    let mut temporary_name = OsString::from(".");
    temporary_name.push(file_name);
    temporary_name.push(format!(
        ".{}.{}.tmp",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos())
    ));
    parent.join(temporary_name)
}

fn ensure_output_parent(path: &Path) -> Result<(), CliError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

fn write_stdout(text: &str) -> Result<(), CliError> {
    write_stream(std::io::stdout().lock(), text)
}

fn write_stderr(text: &str) -> Result<(), CliError> {
    write_stream(std::io::stderr().lock(), text)
}

fn write_stream(mut stream: impl Write, text: &str) -> Result<(), CliError> {
    match stream.write_all(text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Err(err) => Err(CliError::Io(err)),
    }
}

fn discover_source_paths(ctx: &Context, action: &str) -> Result<Vec<PathBuf>, CliError> {
    require_catalog_for_discovery(ctx, action)?;
    Ok(ctx.source_paths())
}

fn discover_test_paths(ctx: &Context) -> Result<Vec<PathBuf>, CliError> {
    require_catalog_for_discovery(ctx, "run tests")?;
    Ok(ctx.test_paths())
}

fn require_catalog_for_discovery(ctx: &Context, action: &str) -> Result<(), CliError> {
    if ctx.catalog_path.is_file() {
        return Ok(());
    }

    Err(CliError::message(format!(
        "no catalog found at {}; pass files explicitly or choose a catalog with --catalog to {action}",
        ctx.catalog_path.display()
    )))
}

fn require_catalog_for_workspace(ctx: &Context, action: &str) -> Result<(), CliError> {
    if ctx.catalog_path.is_file() {
        return Ok(());
    }

    Err(CliError::message(format!(
        "no catalog found at {}; choose a catalog with --catalog to {action}",
        ctx.catalog_path.display()
    )))
}

fn uninstall_targets(args: &UninstallArgs) -> Result<&[String], CliError> {
    if args.all {
        return Ok(&[]);
    }
    if args.tools.is_empty() {
        return Err(CliError::message(
            "no tools selected for uninstall; pass TOOL names or use --all",
        ));
    }
    Ok(&args.tools)
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
    use clap::{CommandFactory, Parser};
    use scaffold_catalog::Catalog;
    use scaffold_platform::{Host, HostArch, HostOs};
    use serde_json::json;

    use super::{
        CatalogCheckRow, PathRow, PathStatus, TestRow, TestStatus, ToolPresenceStatus,
        catalog_check_rows, contextualize_catalog_error, install, path_rows, render_catalog_check,
        render_catalog_list, render_format_check_failures, render_install_events, render_paths,
        render_test_results, require_catalog_for_discovery, require_catalog_for_workspace,
        selected_catalog, uninstall_targets, write_output_file, write_stream,
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
        let root = unique_test_dir("output-file-writer");
        let output = root.join("nested").join("reference.md");

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

        drop(std::fs::remove_dir_all(root));
    }

    fn unique_test_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "scaffold-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos())
        ))
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
            status: TestStatus::Ok,
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
        let root = unique_test_dir("paths-sources");
        drop(std::fs::remove_dir_all(&root));
        let entries = root.join("extensions").join("entries");
        std::fs::create_dir_all(&entries).expect("entries");
        std::fs::write(root.join("scaffold.scm"), "(import (rnrs))\n").expect("catalog");
        std::fs::write(
            entries.join("demo.scm"),
            "(library (entries demo) (export demo) (import (rnrs)) (define demo 'ok))\n",
        )
        .expect("source");
        std::fs::write(entries.join("test.scm"), "(import (rnrs))\n").expect("test");
        let ctx = scaffold_context::Context {
            catalog_path: root.join("scaffold.scm"),
            root_dir: root.clone(),
            bin_dir: root.join("bin"),
            state_dir: root.join("state"),
        };

        let rows = path_rows(&ctx, true);

        assert!(rows.iter().any(|row| {
            row.kind == "source" && row.path.ends_with("extensions/entries/demo.scm")
        }));
        assert!(rows.iter().any(|row| {
            row.kind == "test" && row.path.ends_with("extensions/entries/test.scm")
        }));

        drop(std::fs::remove_dir_all(root));
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
        let err = scaffold_catalog::CatalogError::Dsl(scaffold_dsl::DslError::Io(
            std::io::Error::new(std::io::ErrorKind::NotFound, "missing"),
        ));

        let message = contextualize_catalog_error(&ctx, err).to_string();

        assert!(message.contains("/definitely/missing/scaffold.scm"));
        assert!(message.contains("failed while loading catalog"));
    }
}
