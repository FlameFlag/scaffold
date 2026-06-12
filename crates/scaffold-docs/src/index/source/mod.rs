use lexpr::datum::{Datum, Ref};

use scaffold_editor::sexpr;

use super::model::{DocEntry, DocKind, DocParam, SourceDocs, SourcePosition, SourceRange};

mod libraries;

use libraries::{imported_libraries, library_name};

#[must_use]
pub fn source_docs(source_name: &str, source: &str) -> SourceDocs {
    if !sexpr::parses_completely(source) {
        return SourceDocs {
            library_name: None,
            imports: Vec::new(),
            entries: Vec::new(),
        };
    }
    let datums = sexpr::parse_datums(source);
    let defaults = module_defaults(&datums);
    let datum_refs = datums
        .iter()
        .map(|datum| datum.as_ref())
        .collect::<Vec<_>>();
    let mut entries = Vec::new();
    for (index, datum) in datum_refs.iter().copied().enumerate() {
        if let Some(entry) =
            doc_next_entry_from_items(source_name, source, &datum_refs, index, &defaults)
        {
            merge_or_push_doc_entry(&mut entries, entry);
            continue;
        }
        collect_doc_entries(source_name, source, datum, &defaults, &mut entries);
    }
    attach_definition_ranges(source_name, source, &mut entries);
    SourceDocs {
        library_name: library_name(&datums),
        imports: imported_libraries(&datums),
        entries,
    }
}

fn attach_definition_ranges(source_name: &str, source: &str, entries: &mut [DocEntry]) {
    let definitions = definition_entries(source_name, source);
    for entry in entries {
        if let Some(definition) = definitions.iter().find(|definition| {
            definition.name == entry.name && definition.source.as_deref() == entry.source.as_deref()
        }) {
            entry.range = definition.range;
        }
    }
}

#[must_use]
pub fn source_docs_with_definitions(source_name: &str, source: &str) -> SourceDocs {
    let mut source_docs = source_docs(source_name, source);
    if !source_docs.entries.is_empty() || sexpr::parses_completely(source) {
        let mut entries = definition_entries(source_name, source);
        for entry in source_docs.entries {
            merge_or_push_doc_entry(&mut entries, entry);
        }
        source_docs.entries = entries;
    }
    source_docs
}

#[cfg(test)]
pub(super) fn docs_from_source(source_name: &str, source: &str) -> Vec<DocEntry> {
    source_docs(source_name, source).entries
}

fn definition_entries(source_name: &str, source: &str) -> Vec<DocEntry> {
    scaffold_editor::syntax::definitions(source)
        .into_iter()
        .map(|definition| {
            let mut entry = DocEntry::new(definition.name, DocKind::Function);
            entry.source = Some(source_name.to_owned());
            entry.range = Some(SourceRange {
                start: SourcePosition {
                    line: definition.line,
                    character: definition.start,
                },
                end: SourcePosition {
                    line: definition.line,
                    character: definition.start + definition.length,
                },
            });
            entry
        })
        .collect()
}

fn collect_doc_entries(
    source_name: &str,
    text: &str,
    datum: Ref<'_>,
    defaults: &DocDefaults,
    output: &mut Vec<DocEntry>,
) {
    if let Some(entry) = doc_entry_from_form(source_name, text, datum, defaults) {
        merge_or_push_doc_entry(output, entry);
        return;
    }

    if let Some(items) = sexpr::list_items(datum) {
        for (index, item) in items.iter().copied().enumerate() {
            if let Some(entry) =
                doc_next_entry_from_items(source_name, text, &items, index, defaults)
            {
                merge_or_push_doc_entry(output, entry);
                continue;
            }
            collect_doc_entries(source_name, text, item, defaults, output);
        }
    }
}

fn doc_entry_from_form(
    source_name: &str,
    text: &str,
    form: Ref<'_>,
    defaults: &DocDefaults,
) -> Option<DocEntry> {
    let items = sexpr::list_items(form)?;
    let head = items.first().and_then(|item| sexpr::symbol_text(*item))?;
    if head != "doc" && head != "typedoc" && head != "extern-doc" {
        return None;
    }

    let subject = items.get(1).and_then(|item| doc_subject(text, *item))?;
    let mut entry = DocEntry::new(subject.name, DocKind::Function);
    entry.source = Some(source_name.to_owned());
    entry.range = Some(subject.range);

    for field in &items[2..] {
        apply_doc_field(&mut entry, *field);
    }

    defaults.apply(&mut entry);
    Some(entry)
}

fn doc_next_entry_from_items(
    source_name: &str,
    text: &str,
    items: &[Ref<'_>],
    index: usize,
    defaults: &DocDefaults,
) -> Option<DocEntry> {
    let doc_next = items.get(index).copied()?;
    let doc_next_items = sexpr::list_items(doc_next)?;
    let head = doc_next_items
        .first()
        .and_then(|item| sexpr::symbol_text(*item))?;
    if head != "doc-next" {
        return None;
    }
    let definition = items
        .iter()
        .copied()
        .skip(index + 1)
        .find_map(|item| definition_from_form(text, item))?;
    let mut entry = DocEntry::new(definition.name, definition.kind);
    entry.signature = Some(definition.signature);
    entry.source = Some(source_name.to_owned());
    entry.range = Some(definition.range);
    for field in &doc_next_items[1..] {
        apply_doc_field(&mut entry, *field);
    }
    defaults.apply(&mut entry);
    Some(entry)
}

fn apply_doc_field(entry: &mut DocEntry, field: Ref<'_>) {
    let Some(field_items) = sexpr::list_items(field) else {
        return;
    };
    let Some(field_name) = field_items
        .first()
        .and_then(|item| sexpr::symbol_text(*item))
    else {
        return;
    };
    match field_name {
        "signature" => entry.signature = structured_signature(&field_items),
        "summary" => entry.summary = field_items.get(1).and_then(|item| string_text(*item)),
        "markdown" => entry.markdown = field_items.get(1).and_then(|item| string_text(*item)),
        "example" => entry.example = field_items.get(1).and_then(|item| string_text(*item)),
        "returns" => entry.returns = field_items.get(1).and_then(|item| string_text(*item)),
        "group" => entry.group = field_items.get(1).and_then(|item| string_text(*item)),
        "effect" => entry.effect = field_items.get(1).and_then(|item| subject_name(*item)),
        "requires-capability" => {
            if let Some(capability) = field_items.get(1).and_then(|item| subject_name(*item)) {
                entry.requires_capability.push(capability);
            }
        }
        "stability" => entry.stability = field_items.get(1).and_then(|item| string_text(*item)),
        "since" => entry.since = field_items.get(1).and_then(|item| string_text(*item)),
        "deprecated" => {
            entry.deprecated = field_items.get(1).and_then(|item| string_text(*item));
        }
        "param" => {
            let Some(name) = field_items.get(1).and_then(|item| subject_name(*item)) else {
                return;
            };
            let Some(summary) = field_items.get(2).and_then(|item| string_text(*item)) else {
                return;
            };
            entry.params.push(DocParam { name, summary });
        }
        "see" => {
            if let Some(reference) = field_items.get(1).and_then(|item| subject_name(*item)) {
                entry.see.push(reference);
            }
        }
        "hidden" => entry.hidden = true,
        _ => {}
    }
}

fn merge_or_push_doc_entry(output: &mut Vec<DocEntry>, doc_entry: DocEntry) {
    let Some(existing) = output.iter_mut().find(|entry| {
        entry.name == doc_entry.name && entry.source.as_deref() == doc_entry.source.as_deref()
    }) else {
        output.push(doc_entry);
        return;
    };

    existing.signature = doc_entry.signature;
    existing.summary = doc_entry.summary;
    existing.markdown = doc_entry.markdown;
    existing.example = doc_entry.example;
    existing.params = doc_entry.params;
    existing.returns = doc_entry.returns;
    existing.group = doc_entry.group;
    existing.see = doc_entry.see;
    existing.effect = doc_entry.effect;
    existing.requires_capability = doc_entry.requires_capability;
    existing.stability = doc_entry.stability;
    existing.since = doc_entry.since;
    existing.deprecated = doc_entry.deprecated;
    existing.hidden = doc_entry.hidden;
}

#[derive(Debug)]
struct DocSubject {
    name: String,
    range: SourceRange,
}

#[derive(Debug, Default)]
struct DocDefaults {
    group: Option<String>,
    effect: Option<String>,
    requires_capability: Vec<String>,
    stability: Option<String>,
    since: Option<String>,
}

impl DocDefaults {
    fn apply(&self, entry: &mut DocEntry) {
        if entry.group.is_none() {
            entry.group = self.group.clone();
        }
        if entry.effect.is_none() {
            entry.effect = self.effect.clone();
        }
        if entry.requires_capability.is_empty() {
            entry.requires_capability = self.requires_capability.clone();
        }
        if entry.stability.is_none() {
            entry.stability = self.stability.clone();
        }
        if entry.since.is_none() {
            entry.since = self.since.clone();
        }
    }
}

struct DefinitionDoc {
    name: String,
    signature: String,
    range: SourceRange,
    kind: DocKind,
}

fn module_defaults(datums: &[Datum]) -> DocDefaults {
    let mut defaults = DocDefaults::default();
    for datum in datums {
        collect_module_defaults(datum.as_ref(), &mut defaults);
    }
    defaults
}

fn collect_module_defaults(datum: Ref<'_>, defaults: &mut DocDefaults) {
    let Some(items) = sexpr::list_items(datum) else {
        return;
    };
    if items
        .first()
        .and_then(|item| sexpr::symbol_text(*item))
        .is_some_and(|head| head == "moduledoc")
    {
        for field in &items[1..] {
            let Some(field_items) = sexpr::list_items(*field) else {
                continue;
            };
            let Some(field_name) = field_items
                .first()
                .and_then(|item| sexpr::symbol_text(*item))
            else {
                continue;
            };
            match field_name {
                "group" => defaults.group = field_items.get(1).and_then(|item| string_text(*item)),
                "effect" => {
                    defaults.effect = field_items.get(1).and_then(|item| subject_name(*item));
                }
                "requires-capability" => {
                    if let Some(capability) =
                        field_items.get(1).and_then(|item| subject_name(*item))
                    {
                        defaults.requires_capability.push(capability);
                    }
                }
                "stability" => {
                    defaults.stability = field_items.get(1).and_then(|item| string_text(*item));
                }
                "since" => defaults.since = field_items.get(1).and_then(|item| string_text(*item)),
                _ => {}
            }
        }
        return;
    }
    for item in items {
        collect_module_defaults(item, defaults);
    }
}

fn definition_from_form(text: &str, form: Ref<'_>) -> Option<DefinitionDoc> {
    let items = sexpr::list_items(form)?;
    let head = items.first().and_then(|item| sexpr::symbol_text(*item))?;
    match head {
        "define" => {
            let subject = *items.get(1)?;
            if let Some(name) = sexpr::symbol_text(subject) {
                return Some(DefinitionDoc {
                    name: name.to_owned(),
                    signature: name.to_owned(),
                    range: symbol_range(text, subject, name),
                    kind: DocKind::Function,
                });
            }
            let signature_items = sexpr::list_items(subject)?;
            let name_datum = *signature_items.first()?;
            let name = sexpr::symbol_text(name_datum)?;
            Some(DefinitionDoc {
                name: name.to_owned(),
                signature: render_signature_list(&signature_items),
                range: symbol_range(text, name_datum, name),
                kind: DocKind::Function,
            })
        }
        "define-syntax" => {
            let subject = *items.get(1)?;
            let name = sexpr::symbol_text(subject)?;
            Some(DefinitionDoc {
                name: name.to_owned(),
                signature: name.to_owned(),
                range: symbol_range(text, subject, name),
                kind: DocKind::Keyword,
            })
        }
        _ => None,
    }
}

fn doc_subject(text: &str, datum: Ref<'_>) -> Option<DocSubject> {
    if let Some(name) = direct_subject_name(datum) {
        return Some(DocSubject {
            range: symbol_range(text, datum, &name),
            name,
        });
    }
    let items = sexpr::list_items(datum)?;
    match items.as_slice() {
        [head, subject] if sexpr::symbol_text(*head).is_some_and(|head| head == "quote") => {
            let name = direct_subject_name(*subject)?;
            Some(DocSubject {
                range: symbol_range(text, *subject, &name),
                name,
            })
        }
        _ => None,
    }
}

fn subject_name(datum: Ref<'_>) -> Option<String> {
    direct_subject_name(datum).or_else(|| {
        let items = sexpr::list_items(datum)?;
        match items.as_slice() {
            [head, subject] if sexpr::symbol_text(*head).is_some_and(|head| head == "quote") => {
                direct_subject_name(*subject)
            }
            _ => None,
        }
    })
}

fn direct_subject_name(datum: Ref<'_>) -> Option<String> {
    sexpr::symbol_text(datum)
        .or_else(|| sexpr::string_text(datum))
        .map(str::to_owned)
}

fn string_text(datum: Ref<'_>) -> Option<String> {
    sexpr::string_text(datum).map(str::to_owned)
}

fn structured_signature(items: &[Ref<'_>]) -> Option<String> {
    let first = *items.get(1)?;
    if let Some(text) = sexpr::string_text(first) {
        return Some(text.to_owned());
    }
    if let Some(signature_items) = sexpr::list_items(first) {
        return Some(render_signature_list(&signature_items));
    }
    match sexpr::symbol_text(first)? {
        "value" | "syntax" => items.get(2).and_then(|item| subject_name(*item)),
        _ => Some(render_datum(first)),
    }
}

fn render_signature_list(items: &[Ref<'_>]) -> String {
    format!(
        "({})",
        items
            .iter()
            .map(|item| render_datum(*item))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn render_datum(datum: Ref<'_>) -> String {
    if let Some(symbol) = sexpr::symbol_text(datum) {
        return symbol.to_owned();
    }
    if let Some(string) = sexpr::string_text(datum) {
        return format!("\"{}\"", string.replace('\\', "\\\\").replace('"', "\\\""));
    }
    if let Some(items) = sexpr::list_items(datum) {
        return render_signature_list(&items);
    }
    datum.to_string()
}

fn symbol_range(text: &str, datum: Ref<'_>, symbol: &str) -> SourceRange {
    let position = sexpr::span_start(text, datum);
    let start = SourcePosition {
        line: position.line,
        character: position.character,
    };
    let end = SourcePosition {
        line: start.line,
        character: start.character + symbol.encode_utf16().count() as u32,
    };
    SourceRange { start, end }
}
