use comfy_table::{Cell, Color};
use scaffold_catalog::Tool;
use scaffold_context::Context;
use scaffold_install::{self as install, ToolPresenceStatus};
use scaffold_platform::Host;

use super::table::{DEFAULT_TABLE_WIDTH, render_output_table};

pub(super) fn render_catalog_list(tools: &[Tool], host: Host) -> String {
    render_output_table(
        Some(DEFAULT_TABLE_WIDTH),
        &["tool", "host", "action", "phase", "bins", "description"],
        tools.iter().map(|tool| {
            vec![
                Cell::new(&tool.name),
                host_status_cell(tool.supports_host(host)),
                Cell::new(tool.action.label()),
                Cell::new(tool.phase().label()),
                Cell::new(tool.bin_summary()),
                Cell::new(tool.meta.description.as_deref().unwrap_or_default()),
            ]
        }),
    )
}

#[derive(Debug)]
pub(super) struct CatalogCheckRow {
    pub(super) name: String,
    pub(super) status: ToolPresenceStatus,
    pub(super) version: String,
}

pub(super) fn catalog_check_rows(
    ctx: &Context,
    tools: &[Tool],
    host: Host,
) -> (Vec<CatalogCheckRow>, usize) {
    let rows = tools
        .iter()
        .map(|tool| {
            let presence = install::tool_presence_summary(ctx, tool, host);
            CatalogCheckRow {
                name: tool.name.clone(),
                status: presence.status,
                version: presence.version,
            }
        })
        .collect::<Vec<_>>();
    let missing = rows
        .iter()
        .filter(|row| row.status == ToolPresenceStatus::Missing)
        .count();
    (rows, missing)
}

pub(super) fn render_catalog_check(rows: &[CatalogCheckRow]) -> String {
    render_output_table(
        Some(DEFAULT_TABLE_WIDTH),
        &["tool", "status", "version"],
        rows.iter().map(|row| {
            vec![
                Cell::new(&row.name),
                check_status_cell(row.status),
                Cell::new(&row.version),
            ]
        }),
    )
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
