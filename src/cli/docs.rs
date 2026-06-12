use super::{CliError, args::DocsFormat};

pub(super) fn render_docs(format: DocsFormat) -> Result<String, CliError> {
    match format {
        DocsFormat::Markdown => Ok(scaffold_docs::scaffold_reference_markdown()),
        DocsFormat::Json => Ok(scaffold_docs::scaffold_reference_json()?),
    }
}
