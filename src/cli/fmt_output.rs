use std::path::PathBuf;

use comfy_table::{Cell, Color};

use super::table::{DEFAULT_TABLE_WIDTH, render_output_table};

pub(super) fn render_format_check_failures(paths: &[PathBuf]) -> String {
    render_output_table(
        Some(DEFAULT_TABLE_WIDTH),
        &["file", "status"],
        paths.iter().map(|path| {
            vec![
                Cell::new(path.display().to_string()),
                Cell::new("would reformat").fg(Color::Yellow),
            ]
        }),
    )
}
