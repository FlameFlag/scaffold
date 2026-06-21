use comfy_table::{Cell, Color};

use super::table::{DEFAULT_TABLE_WIDTH, render_output_table};

#[derive(Debug)]
pub(super) struct TestRow {
    pub(super) path: String,
}

pub(super) fn render_test_results(rows: &[TestRow]) -> String {
    render_output_table(
        Some(DEFAULT_TABLE_WIDTH),
        &["test", "status"],
        rows.iter()
            .map(|row| vec![Cell::new(&row.path), test_status_cell()]),
    )
}

fn test_status_cell() -> Cell {
    Cell::new("ok").fg(Color::Green)
}
