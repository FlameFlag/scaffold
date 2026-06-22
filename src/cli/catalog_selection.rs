use std::path::PathBuf;

use super::CliError;
use super::args::{self, Command};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SelectedCatalog {
    pub(super) path: Option<PathBuf>,
    pub(super) mode: Option<String>,
}

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
    Accepted {
        path: Option<PathBuf>,
        mode: Option<String>,
    },
    Rejected,
}

pub(super) fn selected_catalog(
    command: &Command,
    top_level_catalog: Option<PathBuf>,
    top_level_catalog_mode: Option<String>,
) -> Result<SelectedCatalog, CliError> {
    let info = command_info(command);
    match info.catalog {
        CatalogSupport::Accepted { path, mode } => Ok(SelectedCatalog {
            path: path.or(top_level_catalog).or_else(env_catalog),
            mode: mode.or(top_level_catalog_mode),
        }),
        CatalogSupport::Rejected
            if top_level_catalog.is_some() || top_level_catalog_mode.is_some() =>
        {
            Err(CliError::message(format!(
                "`{}` does not use --catalog",
                info.name
            )))
        }
        CatalogSupport::Rejected => Ok(SelectedCatalog {
            path: None,
            mode: None,
        }),
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
            catalog: catalog_support(&args.catalog),
        },
        Command::Docs(_) => CommandInfo {
            name: "docs",
            catalog: CatalogSupport::Rejected,
        },
        Command::Eval(args) => CommandInfo {
            name: "eval",
            catalog: catalog_support(&args.catalog),
        },
        Command::Fmt(args) => CommandInfo {
            name: "fmt",
            catalog: catalog_support(&args.catalog),
        },
        Command::Install(args) => CommandInfo {
            name: "install",
            catalog: catalog_support(&args.catalog),
        },
        Command::Uninstall(args) => CommandInfo {
            name: "uninstall",
            catalog: catalog_support(&args.catalog),
        },
        Command::Lsp => CommandInfo {
            name: "lsp",
            catalog: CatalogSupport::Rejected,
        },
        Command::Mcp(args) => CommandInfo {
            name: "mcp",
            catalog: catalog_support(args),
        },
        Command::Repl(args) => CommandInfo {
            name: "repl",
            catalog: catalog_support(args),
        },
        Command::List(args) => CommandInfo {
            name: "list",
            catalog: catalog_support(args),
        },
        Command::Check(args) => CommandInfo {
            name: "check",
            catalog: catalog_support(args),
        },
        Command::Test(args) => CommandInfo {
            name: "test",
            catalog: catalog_support(&args.catalog),
        },
        Command::Paths(args) => CommandInfo {
            name: "paths",
            catalog: catalog_support(&args.catalog),
        },
        Command::Completions(_) => CommandInfo {
            name: "completions",
            catalog: CatalogSupport::Rejected,
        },
    }
}

fn catalog_support(args: &args::CatalogArgs) -> CatalogSupport {
    CatalogSupport::Accepted {
        path: args.catalog.clone(),
        mode: args.catalog_mode.clone(),
    }
}
