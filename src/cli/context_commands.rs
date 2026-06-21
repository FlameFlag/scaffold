use scaffold_context::{self as context, Context};
use scaffold_dsl as dsl;
use scaffold_fmt::FormatMode;
use scaffold_install::{self as install, Policy};
use scaffold_platform::Host;

use super::{
    CliError,
    catalog_output::{catalog_check_rows, render_catalog_check, render_catalog_list},
    catalog_selection::ContextCommand,
    fmt_output::render_format_check_failures,
    install_output::{CliInstallReporter, render_install_events, uninstall_targets},
    io::{print_json_values, write_stderr, write_stdout},
    path_output::{path_rows, render_paths},
    repl,
    test_output::{TestRow, render_test_results},
    workspace::{
        contextualize_install_error, discover_source_paths, discover_test_paths, load_catalog,
        require_catalog_for_workspace,
    },
};

pub(super) fn run_with_context(
    command: ContextCommand,
    catalog: Option<std::path::PathBuf>,
) -> Result<(), CliError> {
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
                write_stderr(&format!(
                    "{:?}
",
                    miette::Report::new(diagnostic)
                ))?;
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

            let mode = if args.check {
                FormatMode::Check
            } else {
                FormatMode::Write
            };
            let changed = files
                .into_iter()
                .filter_map(|path| match format_path_for_cli(&path, mode) {
                    Ok(true) => Some(Ok(path)),
                    Ok(false) => None,
                    Err(err) => Some(Err(err)),
                })
                .collect::<Result<Vec<_>, CliError>>()?;
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
            let rows = files
                .into_iter()
                .map(|path| {
                    dsl::values_from_path_with_catalog_path(&path, &ctx.catalog_path)?;
                    Ok(TestRow {
                        path: path.display().to_string(),
                    })
                })
                .collect::<Result<Vec<_>, CliError>>()?;
            write_stdout(&render_test_results(&rows))?;
        }
        ContextCommand::Paths(args) => {
            write_stdout(&render_paths(&path_rows(&ctx, args.sources)))?;
        }
    }

    Ok(())
}

fn format_path_for_cli(path: &std::path::Path, mode: FormatMode) -> Result<bool, CliError> {
    scaffold_fmt::format_path(path, mode).map_err(|err| match err {
        scaffold_fmt::FormatError::Io(err) => CliError::Io(err),
        err => {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            scaffold_diagnostic::SourceDiagnostic::syntax(
                path.display().to_string(),
                source,
                err.primary_offset(),
                1,
                err.to_string(),
            )
            .into()
        }
    })
}
