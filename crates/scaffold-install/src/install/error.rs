use std::path::PathBuf;

use thiserror::Error;

use scaffold_archive as archive;
use scaffold_process as process;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error(transparent)]
    Catalog(#[from] scaffold_catalog::CatalogError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Process(#[from] process::ProcessError),
    #[error(transparent)]
    Archive(#[from] archive::ArchiveError),
    #[error("{tool}: unsupported on this host")]
    UnsupportedHost { tool: String },
    #[error("{tool}: missing required executable")]
    MissingRequired { tool: String },
    #[error("{tool}: no package install command matches this host")]
    NoInstaller { tool: String },
    #[error("{tool}: build action does not define argv")]
    NoBuildCommand { tool: String },
    #[error("catalog does not contain requested tool: {0}")]
    MissingNamedTool(String),
    #[error("{tool}: installed tool failed verification")]
    VerificationFailed { tool: String },
    #[error("catalog contains duplicate tool name: {0}")]
    DuplicateTool(String),
    #[error("{tool}: dependency does not exist: {dependency}")]
    MissingDependency { tool: String, dependency: String },
    #[error("{tool}: install order reference does not exist: {reference}")]
    MissingOrderReference { tool: String, reference: String },
    #[error("catalog install order contains a cycle")]
    CyclicInstallOrder,
    #[error("{tool}: refusing to remove unsafe uninstall path {path:?}")]
    UnsafeUninstallPath { tool: String, path: PathBuf },
}
