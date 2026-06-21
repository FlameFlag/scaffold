use comfy_table::{Cell, Color};
use scaffold_context::Context;

use super::table::{DEFAULT_TABLE_WIDTH, render_output_table};

#[derive(Debug)]
pub(super) struct PathRow {
    pub(super) kind: &'static str,
    pub(super) path: String,
    pub(super) status: PathStatus,
    pub(super) resolved: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PathStatus {
    Exists,
    Missing,
}

pub(super) fn path_rows(ctx: &Context, include_sources: bool) -> Vec<PathRow> {
    let fixed_rows = [
        PathRow::new("catalog", &ctx.catalog_path),
        PathRow::new("root", &ctx.root_dir),
        PathRow::new("bin", &ctx.bin_dir),
        PathRow::new("state", &ctx.state_dir),
    ];
    let discovered_rows = include_sources
        .then(|| {
            ctx.source_paths()
                .into_iter()
                .map(|path| PathRow::new("source", &path))
                .chain(
                    ctx.test_paths()
                        .into_iter()
                        .map(|path| PathRow::new("test", &path)),
                )
        })
        .into_iter()
        .flatten();

    fixed_rows
        .into_iter()
        .chain(ctx.extension_dirs().into_iter().map(extension_path_row))
        .chain(discovered_rows)
        .collect()
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

fn extension_path_row(dir: std::path::PathBuf) -> PathRow {
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
}

pub(super) fn render_paths(rows: &[PathRow]) -> String {
    render_output_table(
        Some(DEFAULT_TABLE_WIDTH),
        &["kind", "path", "status", "resolved"],
        rows.iter().map(|row| {
            vec![
                Cell::new(row.kind),
                Cell::new(&row.path),
                path_status_cell(row.status),
                Cell::new(&row.resolved),
            ]
        }),
    )
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
