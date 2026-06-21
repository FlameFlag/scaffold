use comfy_table::{Attribute, Cell, ContentArrangement, Table, presets::UTF8_FULL_CONDENSED};

pub(super) const DEFAULT_TABLE_WIDTH: u16 = 120;

pub(super) fn render_output_table(
    width: Option<u16>,
    headers: &[&str],
    rows: impl IntoIterator<Item = Vec<Cell>>,
) -> String {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);
    if let Some(width) = width {
        table.set_width(width);
    }
    table.set_header(
        headers
            .iter()
            .map(|header| Cell::new(header).add_attribute(Attribute::Bold)),
    );
    table.add_rows(rows);
    let mut output = table.trim_fmt();
    output.push('\n');
    output
}
