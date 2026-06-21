use scaffold_catalog::CatalogError;
use scaffold_dsl as dsl;

#[derive(Debug, thiserror::Error, miette::Diagnostic)]
pub enum CliError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Context(#[from] scaffold_context::ContextError),
    #[error(transparent)]
    Catalog(#[from] scaffold_catalog::CatalogError),
    #[error(transparent)]
    Install(#[from] scaffold_install::InstallError),
    #[error(transparent)]
    Format(#[from] scaffold_fmt::FormatError),
    #[error(transparent)]
    Reedline(#[from] reedline::ReedlineError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Dsl(#[from] dsl::DslError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Source(Box<scaffold_diagnostic::SourceDiagnostic>),
    #[error("language server failed: {0}")]
    Lsp(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("MCP server failed: {0}")]
    Mcp(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    Message(String),
}

impl CliError {
    pub(super) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl From<scaffold_diagnostic::SourceDiagnostic> for CliError {
    fn from(value: scaffold_diagnostic::SourceDiagnostic) -> Self {
        Self::Source(Box::new(value))
    }
}

pub(super) fn contextualize_catalog_error(
    catalog_path: &std::path::Path,
    err: CatalogError,
) -> CliError {
    match err {
        CatalogError::Dsl(dsl::DslError::Io(source)) => CliError::message(format!(
            "failed while loading catalog {}: {source}",
            catalog_path.display()
        )),
        err => CliError::Catalog(err),
    }
}
