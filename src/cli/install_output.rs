use comfy_table::{Cell, Color};
use scaffold_install as install;

use super::{
    CliError,
    args::UninstallArgs,
    table::{DEFAULT_TABLE_WIDTH, render_output_table},
};

#[derive(Default)]
pub(super) struct CliInstallReporter {
    events: Vec<install::InstallEvent>,
}

impl CliInstallReporter {
    pub(super) fn events(&self) -> &[install::InstallEvent] {
        &self.events
    }
}

impl install::InstallReporter for CliInstallReporter {
    fn report(&mut self, event: install::InstallEvent) {
        self.events.push(event);
    }
}

pub(super) fn render_install_events(events: &[install::InstallEvent]) -> String {
    if events.is_empty() {
        return String::new();
    }

    render_output_table(
        Some(DEFAULT_TABLE_WIDTH),
        &["tool", "action", "detail"],
        events.iter().map(|event| {
            vec![
                Cell::new(&event.tool),
                install_event_cell(event.action),
                Cell::new(&event.detail),
            ]
        }),
    )
}

pub(super) fn uninstall_targets(args: &UninstallArgs) -> Result<&[String], CliError> {
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
