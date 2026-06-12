use std::path::{Path, PathBuf};

use miette::Diagnostic;
use thiserror::Error;

use scaffold_diagnostic::SourceDiagnostic;

mod bundled;
mod eval;
mod fs;
mod host;
mod json;
mod libraries;
mod literal;
mod path;
mod stdlib;
mod workspace;

#[cfg(test)]
mod tests;

pub use eval::DslSession;
pub use stdlib::CapabilityDescriptor;

pub struct DocumentationSource {
    pub path: &'static str,
    pub source: &'static str,
}

pub struct CatalogDocument {
    pub value: serde_json::Value,
    pub source_name: String,
    pub source_text: String,
    pub value_spans: Vec<SourceSpan>,
    pub implicit_tools: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SourceSpan {
    pub offset: usize,
    pub len: usize,
}

#[must_use]
pub fn documentation_sources() -> Vec<DocumentationSource> {
    let mut sources = stdlib::core_documentation_sources();
    sources.extend(
        bundled::BUNDLED_EXTENSION_SOURCES
            .iter()
            .map(|source| DocumentationSource {
                path: source.path,
                source: source.source,
            }),
    );
    sources
}

#[must_use]
pub const fn rust_backed_capabilities() -> &'static [CapabilityDescriptor] {
    stdlib::rust_backed_capabilities()
}

#[derive(Debug, Error, Diagnostic)]
pub enum DslError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    #[diagnostic(transparent)]
    Diagnostic(Box<SourceDiagnostic>),
    #[error("Scheme evaluation failed: {0}")]
    Eval(String),
    #[error("Scheme returned unsupported data at {path}: {message}")]
    Shape { path: String, message: String },
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl From<SourceDiagnostic> for DslError {
    fn from(value: SourceDiagnostic) -> Self {
        Self::Diagnostic(Box::new(value))
    }
}

pub type Result<T> = std::result::Result<T, DslError>;

pub fn values_from_path(path: impl AsRef<Path>) -> Result<Vec<serde_json::Value>> {
    let path = path.as_ref();
    let root = parent_dir(path);
    let extension_dirs = libraries::extension_dirs_for_root(root);
    let source = std::fs::read_to_string(path)?;
    let root = absolute_lexical_path(root)?;
    let path = absolute_lexical_path(path)?;
    let source_name = path.to_string_lossy().into_owned();
    eval::values_from_str_with_context(
        &source,
        Some(source_name.as_ref()),
        &extension_dirs,
        eval::DslEvalContext::new(Some(root), Some(path)),
    )
}

pub fn values_from_path_with_extension_root(
    path: impl AsRef<Path>,
    extension_root: impl AsRef<Path>,
) -> Result<Vec<serde_json::Value>> {
    let path = path.as_ref();
    let extension_root = extension_root.as_ref();
    let extension_dirs = libraries::extension_dirs_for_root(extension_root);
    let source = std::fs::read_to_string(path)?;
    let context_root = absolute_lexical_path(extension_root)?;
    let context_path = absolute_lexical_path(path)?;
    let source_name = context_path.to_string_lossy().into_owned();
    eval::values_from_str_with_context(
        &source,
        Some(source_name.as_ref()),
        &extension_dirs,
        eval::DslEvalContext::new(Some(context_root), Some(context_path)),
    )
}

pub fn session_with_extension_root(
    extension_root: impl AsRef<Path>,
    default_imports: bool,
) -> Result<DslSession> {
    let extension_root = extension_root.as_ref();
    let extension_dirs = libraries::extension_dirs_for_root(extension_root);
    let context_root = absolute_lexical_path(extension_root)?;
    DslSession::with_context(
        &extension_dirs,
        default_imports,
        eval::DslEvalContext::new(Some(context_root), None),
    )
}

pub fn catalog_value_from_path(path: impl AsRef<Path>) -> Result<serde_json::Value> {
    catalog_document_from_path(path).map(|document| document.value)
}

pub fn catalog_document_from_path(path: impl AsRef<Path>) -> Result<CatalogDocument> {
    let path = path.as_ref();
    let root = parent_dir(path);
    let extension_dirs = libraries::extension_dirs_for_root(root);
    let source = std::fs::read_to_string(path)?;
    let root = absolute_lexical_path(root)?;
    let path = absolute_lexical_path(path)?;
    let source_name = path.to_string_lossy().into_owned();
    eval::catalog_document_from_str_with_context(
        source,
        source_name,
        &extension_dirs,
        eval::DslEvalContext::new(Some(root), Some(path)),
    )
}

#[cfg(any(test, feature = "test-support"))]
pub fn catalog_value_from_str(text: &str) -> Result<serde_json::Value> {
    eval::catalog_value_from_values(values_from_str(text)?)
}

#[cfg(any(test, feature = "test-support"))]
pub fn values_from_str(text: &str) -> Result<Vec<serde_json::Value>> {
    eval::values_from_str_with_extension_dirs(text, Some("<string>"), &[])
}

fn parent_dir(path: &Path) -> &Path {
    path.parent().unwrap_or_else(|| Path::new("."))
}

fn absolute_lexical_path(path: &Path) -> Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(path::normalize_lexically(&path))
}
