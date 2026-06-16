use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum, ValueHint};

#[derive(Debug, Parser)]
#[command(
    name = "scaffold",
    about = "Small Scheme-driven system scaffolding tool",
    version,
    arg_required_else_help = true,
    subcommand_required = true
)]
pub(super) struct Cli {
    #[command(subcommand)]
    pub(super) command: Command,

    #[arg(
        long,
        global = true,
        env = "SCAFFOLD_CATALOG",
        value_name = "FILE",
        value_hint = ValueHint::FilePath
    )]
    pub(super) catalog: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub(super) enum Command {
    #[command(about = "Analyze Scaffold Scheme files for static issues")]
    Analyze(AnalyzeArgs),
    #[command(about = "Browse or export Scaffold Scheme reference documentation")]
    Docs(DocsArgs),
    #[command(about = "Evaluate a Scaffold Scheme expression")]
    Eval(EvalArgs),
    #[command(about = "Format Scaffold Scheme files")]
    Fmt(FmtArgs),
    #[command(about = "Install tools from the catalog")]
    Install(InstallArgs),
    #[command(about = "Uninstall tools from the catalog")]
    Uninstall(UninstallArgs),
    #[command(about = "Run the Scaffold Scheme language server")]
    Lsp,
    #[command(about = "Run the Scaffold MCP server over stdio")]
    Mcp,
    #[command(about = "Start an interactive Scaffold Scheme REPL")]
    Repl,
    #[command(about = "List catalog tools")]
    List,
    #[command(about = "Check whether catalog tools are present")]
    Check,
    #[command(about = "Run Scheme tests")]
    Test(TestArgs),
    #[command(about = "Print resolved paths")]
    Paths,
    #[command(about = "Generate shell completions")]
    Completions(CompletionArgs),
}

#[derive(Debug, Args)]
pub(super) struct AnalyzeArgs {
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub(super) files: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub(super) struct EvalArgs {
    #[arg(value_name = "EXPR")]
    pub(super) expression: String,
}

#[derive(Debug, Args)]
pub(super) struct InstallArgs {
    #[arg(value_name = "TOOL")]
    pub(super) tools: Vec<String>,
    #[arg(long)]
    pub(super) force: bool,
}

#[derive(Debug, Args)]
pub(super) struct UninstallArgs {
    #[arg(value_name = "TOOL")]
    pub(super) tools: Vec<String>,
    #[arg(
        long,
        help = "Print uninstall actions without running or removing anything"
    )]
    pub(super) dry_run: bool,
}

#[derive(Debug, Args)]
pub(super) struct DocsArgs {
    #[arg(value_name = "QUERY", help = "Show one symbol, or fuzzy-search docs")]
    pub(super) query: Vec<String>,
    #[arg(long, help = "Show the full generated reference")]
    pub(super) all: bool,
    #[arg(long, value_name = "GROUP", help = "List docs in one group")]
    pub(super) group: Option<String>,
    #[arg(
        long,
        value_name = "NAME",
        help = "Show where a documented symbol comes from"
    )]
    pub(super) source: Option<String>,
    #[arg(
        long,
        default_value_t = 20,
        value_parser = parse_positive_usize,
        help = "Maximum search results to show"
    )]
    pub(super) limit: usize,
    #[arg(
        long,
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        help = "Write the generated full reference to a file"
    )]
    pub(super) output: Option<PathBuf>,
    #[arg(
        long,
        value_enum,
        help = "Export generated reference instead of browsing"
    )]
    pub(super) format: Option<DocsFormat>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(super) enum DocsFormat {
    Markdown,
    Json,
}

#[derive(Debug, Args)]
pub(super) struct FmtArgs {
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub(super) files: Vec<PathBuf>,
    #[arg(long, help = "Exit with an error if any file is not formatted")]
    pub(super) check: bool,
}

#[derive(Debug, Args)]
pub(super) struct TestArgs {
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub(super) files: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub(super) struct CompletionArgs {
    #[arg(value_enum)]
    pub(super) shell: CompletionShell,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(super) enum CompletionShell {
    Bash,
    Elvish,
    Fish,
    PowerShell,
    Zsh,
}

impl From<CompletionShell> for clap_complete::Shell {
    fn from(value: CompletionShell) -> Self {
        match value {
            CompletionShell::Bash => Self::Bash,
            CompletionShell::Elvish => Self::Elvish,
            CompletionShell::Fish => Self::Fish,
            CompletionShell::PowerShell => Self::PowerShell,
            CompletionShell::Zsh => Self::Zsh,
        }
    }
}

fn parse_positive_usize(value: &str) -> Result<usize, String> {
    let parsed = value
        .parse::<usize>()
        .map_err(|err| format!("invalid positive integer: {err}"))?;
    if parsed == 0 {
        return Err("value must be at least 1".to_owned());
    }
    Ok(parsed)
}
