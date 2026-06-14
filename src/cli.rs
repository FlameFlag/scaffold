mod args;
mod docs;
mod repl;

use std::path::PathBuf;

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};

use scaffold_catalog::Catalog;
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
            let rendered = docs::render_docs(args.format)?;
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
            let rendered = docs::render_docs(args.format)?;
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
            )?;
        }
        Command::Uninstall(args) => {
            install::uninstall_catalog(&ctx, &args.tools, args.dry_run)?;
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
            let catalog = Catalog::load(&ctx.catalog_path)?;
            let host = Host::current();
            for tool in catalog.tools {
                let host_status = if tool.supports_host(host) {
                    "yes"
                } else {
                    "no"
                };
                println!("{}\t{}", tool.name, host_status);
            }
        }
        Command::Check => {
            let catalog = Catalog::load(&ctx.catalog_path)?;
            let host = Host::current();
            let mut missing = 0;
            for tool in catalog.tools {
                let status = if !tool.supports_host(host) {
                    "unsupported"
                } else if install::tool_is_present(&ctx, &tool) {
                    "present"
                } else {
                    missing += 1;
                    "missing"
                };
                let version = tool.version_summary();
                if version.is_empty() {
                    println!("{}\t{}", tool.name, status);
                } else {
                    println!("{}\t{}\t{}", tool.name, status, version);
                }
            }
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
            println!("catalog\t{}", ctx.catalog_path.display());
            println!("root\t{}", ctx.root_dir.display());
            println!("bin\t{}", ctx.bin_dir.display());
            println!("state\t{}", ctx.state_dir.display());
            for dir in ctx.extension_dirs() {
                let canonical = std::fs::canonicalize(&dir).ok();
                if let Some(canonical) = canonical
                    && canonical != dir
                {
                    println!("extension\t{}\t{}", dir.display(), canonical.display());
                    continue;
                }
                println!("extension\t{}", dir.display());
            }
        }
        Command::Completions(args) => {
            let mut command = Cli::command();
            let shell: Shell = args.shell.into();
            generate(shell, &mut command, "scaffold", &mut std::io::stdout());
        }
    }

    Ok(())
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
