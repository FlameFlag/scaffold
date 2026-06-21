use std::path::PathBuf;

use super::CliError;
use super::args::{self, Command};

pub(super) enum ContextCommand {
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

pub(super) fn selected_catalog(
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
