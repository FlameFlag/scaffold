use scaffold_scheme::escape_string_literal_body;
use scaffold_template as template;

const CATALOG_CONTEXT_LIBRARY_TEMPLATE: &str = include_str!("std/catalog/context.scm");

pub(super) fn catalog_context_library_source(catalog_mode: Option<&str>) -> String {
    let catalog_mode_value = catalog_mode
        .map(escape_string_literal_body)
        .unwrap_or_default();
    let bindings = std::collections::HashMap::from([
        (
            "catalog_mode_present",
            template_bool(catalog_mode.is_some()),
        ),
        ("catalog_mode", catalog_mode_value.as_str()),
    ]);
    template::render(CATALOG_CONTEXT_LIBRARY_TEMPLATE, &bindings)
}

fn template_bool(value: bool) -> &'static str {
    if value { "#t" } else { "#f" }
}
