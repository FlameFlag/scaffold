use comfy_table::{Attribute, Cell, ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};

pub(super) const DEFAULT_TABLE_WIDTH: u16 = 120;

pub(super) fn output_table(width: Option<u16>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);
    if let Some(width) = width {
        table.set_width(width);
    }
    table
}

pub(super) fn header_cell(label: &str) -> Cell {
    Cell::new(label).add_attribute(Attribute::Bold)
}
