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
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        help = "Use this catalog instead of auto-discovery; catalog commands also read \
                SCAFFOLD_CATALOG"
    )]
    pub(super) catalog: Option<PathBuf>,

    #[arg(
        long,
        value_name = "MODE",
        help = "Select a catalog mode exposed to Scheme as catalog/mode"
    )]
    pub(super) catalog_mode: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct CatalogArgs {
    #[arg(
        long,
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        help = "Use this catalog instead of auto-discovering scaffold.scm or catalog.scm"
    )]
    pub(super) catalog: Option<PathBuf>,

    #[arg(
        long,
        value_name = "MODE",
        help = "Select a catalog mode exposed to Scheme as catalog/mode"
    )]
    pub(super) catalog_mode: Option<String>,
}

#[derive(Debug, Args)]
pub(super) struct PathsArgs {
    #[command(flatten)]
    pub(super) catalog: CatalogArgs,
    #[arg(
        long,
        help = "Include discovered Scheme source and test files under the catalog root"
    )]
    pub(super) sources: bool,
}

#[derive(Debug, Subcommand)]
pub(super) enum Command {
    #[command(about = "Analyze Scaffold Scheme files for static issues")]
    Analyze(AnalyzeArgs),
    #[command(
        about = "Browse or export Scaffold Scheme reference documentation",
        long_about = "Browse targeted Scaffold Scheme reference docs by symbol, search query, group, \
                      source file, or source location. Use --all or --output only when you need \
                      the complete generated reference.",
        after_help = "Examples:\n  scaffold docs\n  scaffold docs tool\n  scaffold docs --search \"ctlg tool\"\n  scaffold docs --group Catalog\n  scaffold docs --source tool\n  scaffold docs --source src/dsl/std/catalog/tool.scm\n  scaffold docs --source src/dsl/std/catalog/tool.scm:16:1\n  scaffold docs --all\n  scaffold docs --output reference.md\n  scaffold docs --output reference.json"
    )]
    Docs(DocsArgs),
    #[command(about = "Evaluate a Scaffold Scheme expression")]
    Eval(EvalArgs),
    #[command(about = "Format Scaffold Scheme files")]
    Fmt(FmtArgs),
    #[command(about = "Install named tools, or every supported tool when TOOL is omitted")]
    Install(InstallArgs),
    #[command(about = "Uninstall named tools, or every tool with --all")]
    Uninstall(UninstallArgs),
    #[command(about = "Run the Scaffold Scheme language server")]
    Lsp,
    #[command(about = "Run the Scaffold MCP server over stdio")]
    Mcp(CatalogArgs),
    #[command(about = "Start an interactive Scaffold Scheme REPL")]
    Repl(CatalogArgs),
    #[command(about = "List catalog tools")]
    List(CatalogArgs),
    #[command(about = "Check whether catalog tools are present")]
    Check(CatalogArgs),
    #[command(about = "Run Scheme tests")]
    Test(TestArgs),
    #[command(about = "Print resolved paths")]
    Paths(PathsArgs),
    #[command(about = "Generate shell completions")]
    Completions(CompletionArgs),
}

#[derive(Debug, Args)]
pub(super) struct AnalyzeArgs {
    #[command(flatten)]
    pub(super) catalog: CatalogArgs,
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub(super) files: Vec<PathBuf>,
}

#[derive(Debug, Args)]
pub(super) struct EvalArgs {
    #[command(flatten)]
    pub(super) catalog: CatalogArgs,
    #[arg(
        value_name = "EXPR",
        help = "Scheme expression to evaluate with the active catalog libraries"
    )]
    pub(super) expression: String,
}

#[derive(Debug, Args)]
pub(super) struct InstallArgs {
    #[command(flatten)]
    pub(super) catalog: CatalogArgs,
    #[arg(
        value_name = "TOOL",
        help = "Catalog tool name to install; omit to install every supported tool"
    )]
    pub(super) tools: Vec<String>,
    #[arg(long, help = "Run install actions even when the tool appears present")]
    pub(super) force: bool,
}

#[derive(Debug, Args)]
pub(super) struct UninstallArgs {
    #[command(flatten)]
    pub(super) catalog: CatalogArgs,
    #[arg(
        value_name = "TOOL",
        help = "Catalog tool name to uninstall; use --all to uninstall every tool"
    )]
    pub(super) tools: Vec<String>,
    #[arg(long, conflicts_with = "tools", help = "Uninstall every catalog tool")]
    pub(super) all: bool,
    #[arg(
        long,
        help = "Print uninstall actions without running or removing anything"
    )]
    pub(super) dry_run: bool,
}

#[derive(Debug, Args)]
pub(super) struct DocsArgs {
    #[arg(
        value_name = "QUERY",
        help = "Show one symbol, or search reference docs"
    )]
    pub(super) query: Vec<String>,
    #[arg(long, help = "Show the full generated reference")]
    pub(super) all: bool,
    #[arg(long, value_name = "QUERY", help = "Search reference docs")]
    pub(super) search: Option<String>,
    #[arg(long, value_name = "GROUP", help = "List docs in one group")]
    pub(super) group: Option<String>,
    #[arg(
        long,
        value_name = "SYMBOL_OR_SOURCE",
        help = "Show where a documented symbol comes from, or list docs from a source file or location"
    )]
    pub(super) source: Option<String>,
    #[arg(
        long,
        value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..),
        help = "Maximum search results to show; defaults to 20, range 1-100"
    )]
    pub(super) limit: Option<usize>,
    #[arg(
        long,
        value_name = "FILE",
        value_hint = ValueHint::FilePath,
        help = "Write the generated full reference; use .md, .markdown, .json, or pass --format"
    )]
    pub(super) output: Option<PathBuf>,
    #[arg(
        long,
        value_enum,
        help = "Render browse output or full-reference exports as markdown or json"
    )]
    pub(super) format: Option<DocsFormat>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(super) enum DocsFormat {
    #[value(alias("md"))]
    Markdown,
    Json,
}

#[derive(Debug, Args)]
pub(super) struct FmtArgs {
    #[command(flatten)]
    pub(super) catalog: CatalogArgs,
    #[arg(value_name = "FILE", value_hint = ValueHint::FilePath)]
    pub(super) files: Vec<PathBuf>,
    #[arg(long, help = "Exit with an error if any file is not formatted")]
    pub(super) check: bool,
}

#[derive(Debug, Args)]
pub(super) struct TestArgs {
    #[command(flatten)]
    pub(super) catalog: CatalogArgs,
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
