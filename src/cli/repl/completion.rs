use nucleo_matcher::{
    Config as FuzzyConfig, Matcher,
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
};
use reedline::{Completer, Span, Suggestion};
use tower_lsp::lsp_types::{CompletionItem, Documentation};

use scaffold_docs::DocIndex;
use scaffold_lsp::completion_items;

use crate::cli::docs::doc_entry_group;

use super::commands::{REPL_COMMAND_SPECS, split_repl_command};

#[derive(Clone, Debug)]
struct ReplCompletion {
    value: String,
    display: Option<String>,
    description: Option<String>,
    extra: Option<Vec<String>>,
}

impl AsRef<str> for ReplCompletion {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

pub(super) struct ReplCompleter {
    commands: Vec<ReplCompletion>,
    doc_symbols: Vec<ReplCompletion>,
    doc_groups: Vec<ReplCompletion>,
    scheme_symbols: Vec<ReplCompletion>,
}

impl ReplCompleter {
    pub(super) fn new(docs: &DocIndex) -> Self {
        let commands = REPL_COMMAND_SPECS
            .iter()
            .map(|command| ReplCompletion {
                value: command.name.to_owned(),
                display: Some(command.usage.to_owned()),
                description: Some(command.description.to_owned()),
                extra: None,
            })
            .collect();
        let doc_symbols = completion_items(docs)
            .into_iter()
            .map(repl_completion_from_lsp_item)
            .collect::<Vec<_>>();
        let doc_groups = documented_groups(docs)
            .into_iter()
            .map(|group| ReplCompletion {
                value: group,
                display: None,
                description: Some("Documentation group.".to_owned()),
                extra: None,
            })
            .collect::<Vec<_>>();
        let mut scheme_symbols = doc_symbols.clone();
        scheme_symbols.sort_by(|left, right| left.value.cmp(&right.value));
        scheme_symbols.dedup_by(|left, right| left.value == right.value);

        Self {
            commands,
            doc_symbols,
            doc_groups,
            scheme_symbols,
        }
    }
}

impl Completer for ReplCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        let pos = safe_completion_pos(line, pos);
        let span = completion_span(line, pos);
        let prefix = &line[span.start..span.end];
        let context = self.completion_context(line, pos);
        context.suggestions(prefix, span)
    }
}

impl ReplCompleter {
    fn completion_context(&self, line: &str, pos: usize) -> ReplCompletionContext<'_> {
        if line.trim_start().starts_with(':') {
            let before_cursor = &line[..pos];
            let (command, rest) = split_repl_command(before_cursor);
            if rest.is_empty() && !before_cursor.ends_with(char::is_whitespace) {
                return ReplCompletionContext::prefix(&self.commands);
            }
            return match command {
                ":doc" | ":docs" | ":search" | ":source" => {
                    ReplCompletionContext::prefix_then_fuzzy(&self.doc_symbols)
                }
                ":group" => ReplCompletionContext::prefix_then_fuzzy(&self.doc_groups),
                _ => ReplCompletionContext::prefix(&self.commands),
            };
        }
        ReplCompletionContext::prefix_then_fuzzy(&self.scheme_symbols)
    }
}

struct ReplCompletionContext<'a> {
    candidates: &'a [ReplCompletion],
    fuzzy_fallback: bool,
}

impl<'a> ReplCompletionContext<'a> {
    const fn prefix(candidates: &'a [ReplCompletion]) -> Self {
        Self {
            candidates,
            fuzzy_fallback: false,
        }
    }

    const fn prefix_then_fuzzy(candidates: &'a [ReplCompletion]) -> Self {
        Self {
            candidates,
            fuzzy_fallback: true,
        }
    }

    fn suggestions(&self, prefix: &str, span: Span) -> Vec<Suggestion> {
        let prefix_matches = prefix_suggestions(prefix, span, self.candidates);
        if !prefix_matches.is_empty() || prefix.is_empty() || !self.fuzzy_fallback {
            return prefix_matches;
        }
        fuzzy_suggestions(prefix, span, self.candidates)
    }
}

fn repl_completion_from_lsp_item(item: CompletionItem) -> ReplCompletion {
    ReplCompletion {
        value: item.label,
        display: item.detail,
        description: item.documentation.and_then(completion_documentation_text),
        extra: None,
    }
}

fn completion_documentation_text(documentation: Documentation) -> Option<String> {
    let text = match documentation {
        Documentation::String(value) => value,
        Documentation::MarkupContent(markup) => markup.value,
    };
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with("```"))
        .map(str::to_owned)
}

fn documented_groups(docs: &DocIndex) -> Vec<String> {
    let mut groups = docs
        .visible_entries()
        .map(doc_entry_group)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    groups.sort();
    groups.dedup();
    groups
}

fn prefix_suggestions(prefix: &str, span: Span, candidates: &[ReplCompletion]) -> Vec<Suggestion> {
    candidates
        .iter()
        .filter(|candidate| prefix.is_empty() || starts_with_ignore_case(&candidate.value, prefix))
        .take(50)
        .map(|candidate| candidate.to_suggestion(span, None))
        .collect()
}

fn starts_with_ignore_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

fn fuzzy_suggestions(prefix: &str, span: Span, candidates: &[ReplCompletion]) -> Vec<Suggestion> {
    if prefix.is_empty() {
        return candidates
            .iter()
            .take(50)
            .map(|candidate| candidate.to_suggestion(span, None))
            .collect();
    }

    let pattern = Pattern::new(
        prefix,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let mut matcher = Matcher::new(FuzzyConfig::DEFAULT);
    pattern
        .match_list(candidates.iter(), &mut matcher)
        .into_iter()
        .take(50)
        .map(|(candidate, _score)| candidate.to_suggestion(span, None))
        .collect()
}

impl ReplCompletion {
    fn to_suggestion(&self, span: Span, match_indices: Option<Vec<usize>>) -> Suggestion {
        Suggestion {
            value: self.value.clone(),
            display_override: self.display.clone(),
            description: self.description.clone(),
            extra: self.extra.clone(),
            span,
            append_whitespace: self.value.starts_with(':'),
            match_indices,
            ..Default::default()
        }
    }
}

fn safe_completion_pos(line: &str, pos: usize) -> usize {
    let mut pos = pos.min(line.len());
    while pos > 0 && !line.is_char_boundary(pos) {
        pos -= 1;
    }
    pos
}

fn completion_span(line: &str, pos: usize) -> Span {
    let start = line[..pos]
        .char_indices()
        .rev()
        .find_map(|(index, ch)| completion_break_char(ch).then_some(index + ch.len_utf8()))
        .unwrap_or(0);
    Span::new(start, pos)
}

const fn completion_break_char(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '(' | ')' | '[' | ']' | '"' | '\'' | '`' | ',' | ';')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repl_completer_completes_commands_and_documented_symbols() {
        let docs = DocIndex::scaffold();
        let mut completer = ReplCompleter::new(&docs);

        let command_matches = completer.complete(":d", 2);
        assert!(
            command_matches
                .iter()
                .any(|item| item.value == ":doc" && item.description.is_some())
        );
        assert!(
            command_matches
                .iter()
                .any(|item| item.value == ":docs" && item.description.is_some())
        );

        let symbol_matches = completer.complete("(too", 4);
        assert!(
            symbol_matches
                .iter()
                .any(|item| item.value == "tool" && item.description.is_some())
        );
    }

    #[test]
    fn repl_completer_uses_fuzzy_matching_for_symbols() {
        let docs = DocIndex::scaffold();
        let mut completer = ReplCompleter::new(&docs);

        let matches = completer.complete("tl", 2);

        assert!(matches.iter().any(|item| item.value == "tool"));
    }

    #[test]
    fn repl_completer_prefers_prefix_matches_for_scheme_symbols() {
        let docs = DocIndex::scaffold();
        let mut completer = ReplCompleter::new(&docs);

        let matches = completer.complete("define", 6);
        let values = matches
            .iter()
            .map(|item| item.value.as_str())
            .collect::<Vec<_>>();

        assert!(!values.is_empty());
        assert!(values.contains(&"define"));
        assert!(values.contains(&"define-syntax"));
        assert!(values.iter().all(|value| value.starts_with("define")));
    }

    #[test]
    fn repl_completer_does_not_mix_path_commands_into_scheme_symbols() {
        let docs = DocIndex::scaffold();
        let mut completer = ReplCompleter::new(&docs);

        let matches = completer.complete("define", 6);

        assert!(
            matches
                .iter()
                .all(|item| item.description.as_deref() != Some("Executable available on PATH."))
        );
    }

    #[test]
    fn repl_completer_completes_doc_command_arguments() {
        let docs = DocIndex::scaffold();
        let mut completer = ReplCompleter::new(&docs);

        let doc_matches = completer.complete(":doc too", 8);
        assert!(doc_matches.iter().any(|item| item.value == "tool"));

        let group_matches = completer.complete(":group lang", 11);
        assert!(
            group_matches
                .iter()
                .any(|item| item.value.eq_ignore_ascii_case("language"))
        );
    }

    #[test]
    fn repl_completer_finds_completion_span_after_scheme_delimiters() {
        assert_eq!(completion_span("(tool \"rg\"", 5), Span::new(1, 5));
        assert_eq!(completion_span("(tool \"rg\"", 9), Span::new(7, 9));
    }
}
