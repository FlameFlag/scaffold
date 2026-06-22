use std::path::PathBuf;

use rmcp::{ServiceExt, transport::stdio};
use scaffold_context as context;

mod prompts;
mod resources;
mod server;
mod tools;

use server::ScaffoldMcp;

pub fn run(
    catalog: Option<PathBuf>,
    catalog_mode: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let catalog_path = catalog.unwrap_or_else(context::default_catalog_path);
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        let service = ScaffoldMcp::new(catalog_path, catalog_mode)
            .serve(stdio())
            .await?;
        let _quit_reason = service.waiting().await?;
        Ok(())
    })
}

fn internal_error(error: impl std::fmt::Display) -> rmcp::ErrorData {
    rmcp::ErrorData::internal_error(error.to_string(), None)
}
