use lexpr::datum::Ref;

use super::sexpr;

#[derive(Debug, serde::Serialize)]
pub struct DefinitionSymbol {
    pub name: String,
    pub offset: usize,
    pub line: u32,
    pub start: u32,
    pub length: u32,
}

#[must_use]
pub fn definitions(text: &str) -> Vec<DefinitionSymbol> {
    let mut definitions = Vec::new();
    for datum in sexpr::parse_datums(text) {
        collect_definitions(text, datum.as_ref(), &mut definitions);
    }
    definitions
}

#[allow(dead_code)]
#[must_use]
pub fn definition_names(text: &str) -> Vec<String> {
    definitions(text)
        .into_iter()
        .map(|definition| definition.name)
        .collect()
}

#[must_use]
pub fn documented_symbols(text: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let datums = sexpr::parse_datums(text);
    let datum_refs = datums
        .iter()
        .map(|datum| datum.as_ref())
        .collect::<Vec<_>>();
    for (index, datum) in datum_refs.iter().copied().enumerate() {
        if let Some(symbol) = doc_next_symbol(&datum_refs, index) {
            symbols.push(symbol);
            continue;
        }
        collect_documented_symbols(datum, &mut symbols);
    }
    symbols
}

fn collect_definitions(text: &str, datum: Ref<'_>, output: &mut Vec<DefinitionSymbol>) {
    let Some(items) = sexpr::list_items(datum) else {
        return;
    };
    if let Some(head) = items.first().and_then(|item| sexpr::symbol_text(*item)) {
        match head {
            "define" => {
                if let Some(symbol) = items.get(1).and_then(|item| define_subject(*item)) {
                    output.push(definition_symbol(text, symbol));
                }
            }
            "define-syntax" => {
                if let Some(symbol) = items.get(1).and_then(|item| symbol_subject(*item)) {
                    output.push(definition_symbol(text, symbol));
                }
            }
            _ => {}
        }
    }
    for item in items {
        collect_definitions(text, item, output);
    }
}

fn collect_documented_symbols(datum: Ref<'_>, output: &mut Vec<String>) {
    let Some(items) = sexpr::list_items(datum) else {
        return;
    };
    if let Some(head) = items.first().and_then(|item| sexpr::symbol_text(*item))
        && (head == "doc" || head == "typedoc" || head == "extern-doc")
        && let Some(symbol) = items.get(1).and_then(|item| doc_subject(*item))
    {
        output.push(symbol);
    }
    for (index, item) in items.iter().copied().enumerate() {
        if let Some(symbol) = doc_next_symbol(&items, index) {
            output.push(symbol);
            continue;
        }
        collect_documented_symbols(item, output);
    }
}

fn doc_next_symbol(items: &[Ref<'_>], index: usize) -> Option<String> {
    let doc_next_items = sexpr::list_items(*items.get(index)?)?;
    if doc_next_items
        .first()
        .and_then(|item| sexpr::symbol_text(*item))
        .is_none_or(|head| head != "doc-next")
    {
        return None;
    }
    items
        .iter()
        .copied()
        .skip(index + 1)
        .find_map(definition_name)
}

fn define_subject(datum: Ref<'_>) -> Option<Ref<'_>> {
    if symbol_subject(datum).is_some() {
        return Some(datum);
    }
    sexpr::list_items(datum)?.first().copied()
}

fn doc_subject(datum: Ref<'_>) -> Option<String> {
    if let Some(symbol) = sexpr::symbol_text(datum) {
        return Some(symbol.to_owned());
    }
    if let Some(string) = sexpr::string_text(datum) {
        return Some(string.to_owned());
    }
    let items = sexpr::list_items(datum)?;
    match items.as_slice() {
        [head, subject] if sexpr::symbol_text(*head).is_some_and(|head| head == "quote") => {
            sexpr::symbol_text(*subject)
                .or_else(|| sexpr::string_text(*subject))
                .map(str::to_owned)
        }
        _ => None,
    }
}

fn symbol_subject(datum: Ref<'_>) -> Option<Ref<'_>> {
    sexpr::symbol_text(datum).map(|_| datum)
}

fn definition_name(datum: Ref<'_>) -> Option<String> {
    let items = sexpr::list_items(datum)?;
    let head = items.first().and_then(|item| sexpr::symbol_text(*item))?;
    match head {
        "define" => items.get(1).and_then(|item| define_subject(*item)),
        "define-syntax" => items.get(1).and_then(|item| symbol_subject(*item)),
        _ => None,
    }
    .and_then(|subject| sexpr::symbol_text(subject).map(str::to_owned))
}

fn definition_symbol(text: &str, datum: Ref<'_>) -> DefinitionSymbol {
    let name = sexpr::symbol_text(datum).unwrap_or_default().to_owned();
    let position = sexpr::span_start(text, datum);
    DefinitionSymbol {
        length: name.encode_utf16().count() as u32,
        name,
        offset: sexpr::utf16_offset_at_span_start(text, datum),
        line: position.line,
        start: position.character,
    }
}
