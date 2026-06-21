use std::path::PathBuf;

use scaffold_catalog::{Catalog, CatalogError};
use scaffold_context::Context;
use scaffold_install as install;

use super::{CliError, error};

pub(super) fn discover_source_paths(ctx: &Context, action: &str) -> Result<Vec<PathBuf>, CliError> {
    require_catalog_for_discovery(ctx, action)?;
    Ok(ctx.source_paths())
}

pub(super) fn discover_test_paths(ctx: &Context) -> Result<Vec<PathBuf>, CliError> {
    require_catalog_for_discovery(ctx, "run tests")?;
    Ok(ctx.test_paths())
}

pub(super) fn require_catalog_for_discovery(ctx: &Context, action: &str) -> Result<(), CliError> {
    if ctx.catalog_path.is_file() {
        return Ok(());
    }

    Err(CliError::message(format!(
        "no catalog found at {}; pass files explicitly or choose a catalog with --catalog to {action}",
        ctx.catalog_path.display()
    )))
}

pub(super) fn require_catalog_for_workspace(ctx: &Context, action: &str) -> Result<(), CliError> {
    if ctx.catalog_path.is_file() {
        return Ok(());
    }

    Err(CliError::message(format!(
        "no catalog found at {}; choose a catalog with --catalog to {action}",
        ctx.catalog_path.display()
    )))
}

pub(super) fn load_catalog(ctx: &Context) -> Result<Catalog, CliError> {
    Catalog::load(&ctx.catalog_path).map_err(|err| contextualize_catalog_error(ctx, err))
}

pub(super) fn contextualize_install_error(ctx: &Context, err: install::InstallError) -> CliError {
    match err {
        install::InstallError::Catalog(err) => contextualize_catalog_error(ctx, err),
        err => CliError::Install(err),
    }
}

pub(super) fn contextualize_catalog_error(ctx: &Context, err: CatalogError) -> CliError {
    error::contextualize_catalog_error(&ctx.catalog_path, err)
}
