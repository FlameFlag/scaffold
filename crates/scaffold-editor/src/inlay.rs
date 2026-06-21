use crate::{
    TextPosition, sexpr,
    text::{clean_signature_parameter, signature_parameter_names},
};

#[derive(Clone, Copy)]
pub struct InlayParam<'a> {
    pub name: &'a str,
    pub summary: &'a str,
}

pub trait InlayEntry {
    fn signature(&self) -> Option<&str>;
    fn params(&self) -> impl Iterator<Item = InlayParam<'_>>;
}

impl<T: InlayEntry + ?Sized> InlayEntry for &T {
    fn signature(&self) -> Option<&str> {
        (**self).signature()
    }

    fn params(&self) -> impl Iterator<Item = InlayParam<'_>> {
        (**self).params()
    }
}

pub trait InlayIndex {
    type Entry<'a>: InlayEntry
    where
        Self: 'a;

    fn entry<'a>(&'a self, symbol: &str) -> Option<Self::Entry<'a>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextRange {
    pub start_line: u32,
    pub start_character: u32,
    pub end_line: u32,
    pub end_character: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InlayHint {
    pub line: u32,
    pub start: u32,
    pub label: String,
    pub tooltip: Option<String>,
}

pub fn inlay_hints(index: &impl InlayIndex, text: &str, range: TextRange) -> Vec<InlayHint> {
    let mut hints = Vec::new();
    for datum in sexpr::parse_datums(text) {
        collect_inlay_hints(index, text, range, datum.as_ref(), &mut hints);
    }
    hints.sort_by_key(|hint| (hint.line, hint.start));
    hints
}

pub fn parameter_names(entry: &impl InlayEntry) -> Vec<String> {
    let mut params = entry.params().peekable();
    if params.peek().is_some() {
        return params.map(|param| param.name.to_owned()).collect();
    }
    let Some(signature) = entry.signature() else {
        return Vec::new();
    };
    signature_parameter_names(signature)
        .skip(1)
        .filter(|part| *part != "...")
        .map(clean_signature_parameter)
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect()
}

pub fn is_variadic(entry: &impl InlayEntry) -> bool {
    entry
        .signature()
        .is_some_and(|signature| signature_parameter_names(signature).any(|part| part == "..."))
}

pub fn parameter_tooltip(entry: &impl InlayEntry, name: &str) -> Option<String> {
    entry
        .params()
        .find(|param| param.name == name)
        .map(|param| param.summary.to_owned())
}

fn collect_inlay_hints(
    index: &impl InlayIndex,
    text: &str,
    range: TextRange,
    datum: lexpr::datum::Ref<'_>,
    output: &mut Vec<InlayHint>,
) {
    let Some(items) = sexpr::list_items(datum) else {
        return;
    };
    if let Some(head) = items.first().and_then(|item| sexpr::symbol_text(*item))
        && let Some(entry) = index.entry(head)
    {
        inlay_hints_for_form(text, &items[1..], entry, range, output);
    }
    for item in items {
        collect_inlay_hints(index, text, range, item, output);
    }
}

fn inlay_hints_for_form(
    text: &str,
    args: &[lexpr::datum::Ref<'_>],
    entry: impl InlayEntry,
    range: TextRange,
    output: &mut Vec<InlayHint>,
) {
    let params = parameter_names(&entry);
    if params.is_empty() {
        return;
    }
    let variadic = is_variadic(&entry);
    for (index, arg) in args.iter().enumerate() {
        let Some(name) = params
            .get(index)
            .or_else(|| variadic.then(|| params.last()).flatten())
        else {
            continue;
        };
        let position = sexpr::span_start(text, *arg);
        if !range.contains(position) {
            continue;
        }
        output.push(InlayHint {
            line: position.line,
            start: position.character,
            label: format!("{name}:"),
            tooltip: parameter_tooltip(&entry, name),
        });
    }
}

impl TextRange {
    const fn contains(self, position: TextPosition) -> bool {
        (position.line > self.start_line
            || (position.line == self.start_line && position.character >= self.start_character))
            && (position.line < self.end_line
                || (position.line == self.end_line && position.character <= self.end_character))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Entry<'a> {
        signature: Option<&'a str>,
        params: Vec<InlayParam<'a>>,
    }

    impl InlayEntry for Entry<'_> {
        fn signature(&self) -> Option<&str> {
            self.signature
        }

        fn params(&self) -> impl Iterator<Item = InlayParam<'_>> {
            self.params.iter().map(|param| InlayParam {
                name: param.name,
                summary: param.summary,
            })
        }
    }

    struct Index<'a> {
        entry: Entry<'a>,
    }

    impl InlayIndex for Index<'_> {
        type Entry<'a>
            = &'a Entry<'a>
        where
            Self: 'a;

        fn entry<'a>(&'a self, symbol: &str) -> Option<Self::Entry<'a>> {
            (symbol == "demo").then_some(&self.entry)
        }
    }

    #[test]
    fn derives_parameter_names_from_structured_params() {
        assert_eq!(
            parameter_names(&Entry {
                signature: Some("(demo ignored)"),
                params: vec![InlayParam {
                    name: "name",
                    summary: "Demo name.",
                }],
            }),
            vec!["name"]
        );
    }

    #[test]
    fn derives_parameter_names_from_signature() {
        assert_eq!(
            parameter_names(&Entry {
                signature: Some("(demo name [action] field ...)"),
                params: Vec::new(),
            }),
            vec!["name", "action", "field"]
        );
    }

    #[test]
    fn identifies_variadic_signatures() {
        assert!(is_variadic(&Entry {
            signature: Some("(demo name ...)"),
            params: Vec::new(),
        }));
    }

    #[test]
    fn emits_inlay_hints_for_forms() {
        let hints = inlay_hints(
            &Index {
                entry: Entry {
                    signature: Some("(demo name [mode])"),
                    params: vec![InlayParam {
                        name: "name",
                        summary: "Demo name.",
                    }],
                },
            },
            "(demo \"x\")",
            TextRange {
                start_line: 0,
                start_character: 0,
                end_line: 0,
                end_character: 100,
            },
        );

        assert_eq!(
            hints,
            vec![InlayHint {
                line: 0,
                start: 6,
                label: "name:".to_owned(),
                tooltip: Some("Demo name.".to_owned()),
            }]
        );
    }
}
