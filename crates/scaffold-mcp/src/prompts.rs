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
                "Start with project_paths and search_reference for targeted docs; use render_reference only when the full reference export is needed. ",
                "Then use eval_catalog. ",
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
                "Debug Scaffold Scheme by reading the active catalog and using search_reference for targeted docs, ",
                "then use eval_expression or eval_file to isolate the failing form. ",
                "Use analyze for static diagnostics and run_tests for regression coverage."
            ),
        )])
        .with_description("debug-scaffold-eval")
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rmcp::model::PromptMessageContent;

    use super::*;

    #[test]
    fn write_catalog_prompt_prefers_targeted_reference_search() {
        let server = ScaffoldMcp::new(PathBuf::from("/workspace/scaffold.scm"));
        let prompt = server.write_scaffold_catalog();
        let message = prompt.messages.first().expect("prompt message");
        let PromptMessageContent::Text { text } = &message.content else {
            panic!("expected text prompt message");
        };

        assert!(text.contains("search_reference for targeted docs"));
        assert!(text.contains("render_reference only when the full reference export is needed"));
        assert!(!text.contains("search_reference or render_reference"));
    }

    #[test]
    fn debug_prompt_prefers_targeted_reference_search() {
        let server = ScaffoldMcp::new(PathBuf::from("/workspace/scaffold.scm"));
        let prompt = server.debug_scaffold_eval();
        let message = prompt.messages.first().expect("prompt message");
        let PromptMessageContent::Text { text } = &message.content else {
            panic!("expected text prompt message");
        };

        assert!(text.contains("search_reference for targeted docs"));
        assert!(!text.contains("reference resources"));
    }
}
