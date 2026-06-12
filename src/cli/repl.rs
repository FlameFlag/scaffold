mod commands;
mod completion;
mod validation;

use super::{CliError, print_json_values};
use commands::{ReplControl, handle_repl_command};
use completion::ReplCompleter;
use reedline::{
    DefaultHinter, DefaultPrompt, DefaultPromptSegment, DescriptionMode, EditCommand, Emacs,
    FileBackedHistory, IdeMenu, KeyCode, KeyModifiers, Keybindings, MenuBuilder, Reedline,
    ReedlineEvent, ReedlineMenu, Signal, default_emacs_keybindings,
};
use scaffold_context::Context;
use scaffold_docs::DocIndex;
use scaffold_dsl as dsl;
use validation::ReplValidator;

pub(super) fn run_repl(ctx: &Context) -> Result<(), CliError> {
    let session = dsl::session_with_extension_root(&ctx.root_dir, true)?;
    let docs = DocIndex::scaffold();
    let mut editor = repl_editor(&docs, &ctx.state_dir)?;
    let prompt = DefaultPrompt::new(
        DefaultPromptSegment::Basic("scaffold".to_owned()),
        DefaultPromptSegment::Empty,
    );

    loop {
        match editor.read_line(&prompt) {
            Ok(Signal::Success(line)) => {
                let trimmed = line.trim();
                if matches!(trimmed, "(exit)" | ",q") {
                    break;
                }
                if trimmed.starts_with(':') || trimmed == ",help" {
                    match handle_repl_command(trimmed, &docs) {
                        ReplControl::Continue => continue,
                        ReplControl::Break => break,
                    }
                } else if trimmed.is_empty() {
                    continue;
                }
                match session.eval_json(&line, Some("<repl>")) {
                    Ok(values) => print_json_values(values)?,
                    Err(err) => eprintln!("{:?}", miette::Report::new(err)),
                }
            }
            Ok(Signal::CtrlC | Signal::CtrlD) => break,
            Ok(_) => {}
            Err(err) => return Err(err.into()),
        }
    }

    Ok(())
}

fn repl_editor(docs: &DocIndex, state_dir: &std::path::Path) -> Result<Reedline, CliError> {
    let history = Box::new(FileBackedHistory::with_file(
        1000,
        state_dir.join("repl.history"),
    )?);
    let completion_menu = Box::new(
        IdeMenu::default()
            .with_name("completion_menu")
            .with_max_completion_width(48)
            .with_max_description_width(72)
            .with_description_mode(DescriptionMode::PreferRight)
            .with_default_border(),
    );
    let mut keybindings = default_emacs_keybindings();
    add_repl_menu_keybindings(&mut keybindings);

    Ok(Reedline::create()
        .with_history(history)
        .with_history_exclusion_prefix(Some(" ".to_owned()))
        .with_hinter(Box::new(DefaultHinter::default()))
        .with_completer(Box::new(ReplCompleter::new(docs)))
        .with_validator(Box::new(ReplValidator))
        .with_quick_completions(true)
        .with_partial_completions(true)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(Box::new(Emacs::new(keybindings)))
        .with_ansi_colors(true))
}

fn add_repl_menu_keybindings(keybindings: &mut Keybindings) {
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_owned()),
            ReedlineEvent::MenuNext,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::ALT,
        KeyCode::Enter,
        ReedlineEvent::Edit(vec![EditCommand::InsertNewline]),
    );
}
