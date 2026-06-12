use rmcp::{handler::server::router::tool::ToolRouter, model::CallToolResult};

use super::server::ScaffoldMcp;

mod catalog;
mod evaluation;
mod quality;
mod reference;

pub(super) fn router() -> ToolRouter<ScaffoldMcp> {
    ToolRouter::new()
        + catalog::router()
        + evaluation::router()
        + quality::router()
        + reference::router()
}

fn structured(value: serde_json::Value) -> CallToolResult {
    CallToolResult::structured(value)
}
