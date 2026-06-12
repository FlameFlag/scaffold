use rmcp::{
    handler::server::router::prompt::PromptRouter,
    model::{GetPromptResult, PromptMessage, PromptMessageRole},
    prompt, prompt_router,
};

use super::server::ScaffoldMcp;

pub(super) fn router() -> PromptRouter<ScaffoldMcp> {
    ScaffoldMcp::prompt_router()
}

#[prompt_router]
impl ScaffoldMcp {
    #[prompt(
        name = "write-scaffold-catalog",
        description = "Plan or edit a Scaffold Scheme catalog using project reference docs."
    )]
    fn write_scaffold_catalog(&self) -> GetPromptResult {
        GetPromptResult::new(vec![PromptMessage::new_text(
            PromptMessageRole::User,
            concat!(
                "Use the Scaffold MCP tools to work on catalog Scheme. ",
                "Start with project_paths, search_reference or render_reference, then eval_catalog. ",
                "After edits, run analyze and run_tests. Do not request install or uninstall through MCP."
            ),
        )])
        .with_description("write-scaffold-catalog")
    }

    #[prompt(
        name = "debug-scaffold-eval",
        description = "Debug a Scaffold Scheme evaluation, catalog, or test failure."
    )]
    fn debug_scaffold_eval(&self) -> GetPromptResult {
        GetPromptResult::new(vec![PromptMessage::new_text(
            PromptMessageRole::User,
            concat!(
                "Debug Scaffold Scheme by reading the active catalog and reference resources, ",
                "then use eval_expression or eval_file to isolate the failing form. ",
                "Use analyze for static diagnostics and run_tests for regression coverage."
            ),
        )])
        .with_description("debug-scaffold-eval")
    }
}
