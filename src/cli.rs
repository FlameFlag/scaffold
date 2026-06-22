mod args;
mod catalog_output;
mod catalog_selection;
mod context_commands;
mod docs;
mod error;
mod fmt_output;
mod install_output;
mod io;
mod path_output;
mod repl;
mod table;
mod test_output;
mod workspace;

use clap::{CommandFactory, Parser};
use clap_complete::{Shell, generate};

use args::{Cli, Command};
use catalog_selection::{ContextCommand, selected_catalog};
use context_commands::run_with_context;
pub use error::CliError;
use io::{write_output_file, write_stdout};

pub fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    let catalog = selected_catalog(&cli.command, cli.catalog, cli.catalog_mode)?;

    match cli.command {
        Command::Lsp => scaffold_lsp::run().map_err(CliError::Lsp),
        Command::Mcp(_) => scaffold_mcp::run(catalog.path, catalog.mode).map_err(CliError::Mcp),
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

#[cfg(test)]
mod tests;
