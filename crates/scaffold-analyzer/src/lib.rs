use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    path::PathBuf,
};

use scheme_rs::syntax::Syntax;

use scaffold_diagnostic::SourceDiagnostic;
use scaffold_docs::DocIndex;
use scaffold_editor::diagnostics::missing_doc_message;
use scaffold_scheme::{
    identifier_text, is_identifier, parse_error_offset, parse_source, proper_list,
    source_position_byte_offset, wrapped_string_text,
};

pub fn analyze_paths(paths: &[PathBuf]) -> std::io::Result<Vec<SourceDiagnostic>> {
    let sources = paths
        .iter()
        .map(|path| {
            let source_name = path.display().to_string();
            std::fs::read_to_string(path).map(|source| (source_name, source))
        })
        .collect::<std::io::Result<Vec<_>>>()?;
    let mut docs = DocIndex::empty();
    for (source_name, source) in &sources {
        docs.extend_source(source_name, source);
    }

    Ok(sources
        .into_iter()
        .flat_map(|(source_name, source)| analyze_source_with_docs(&source_name, &source, &docs))
        .collect())
}

#[cfg(test)]
fn analyze_source(source_name: &str, source: &str) -> Vec<SourceDiagnostic> {
    let mut docs = DocIndex::empty();
    docs.extend_source(source_name, source);
    analyze_source_with_docs(source_name, source, &docs)
}

fn analyze_source_with_docs(
    source_name: &str,
    source: &str,
    docs: &DocIndex,
) -> Vec<SourceDiagnostic> {
    let syntax = match parse_source(source, source_name) {
        Ok(syntax) => syntax,
        Err(error) => {
            return vec![SourceDiagnostic::syntax(
                source_name,
                source.to_owned(),
                parse_error_offset(&error, source),
                1,
                format!("Scheme syntax failed: {error}"),
            )];
        }
    };

    let mut definitions = Vec::new();
    collect_definitions(source, &syntax, &mut definitions);
    let mut doc_entries = Vec::new();
    collect_doc_entries(source, &syntax, &mut doc_entries);
    dedup_doc_entries(&mut doc_entries);

    let mut diagnostics = Vec::new();
    diagnostics.extend(duplicate_definition_diagnostics(
        source_name,
        source,
        &definitions,
    ));
    diagnostics.extend(duplicate_doc_diagnostics(source_name, source, &doc_entries));
    diagnostics.extend(missing_summary_diagnostics(
        source_name,
        source,
        &doc_entries,
    ));
    diagnostics.extend(
        definitions
            .into_iter()
            .filter(|definition| docs.get(&definition.name).is_none())
            .map(|definition| {
                SourceDiagnostic::missing_doc(
                    source_name,
                    source.to_owned(),
                    definition.offset,
                    definition.length,
                    missing_doc_message(&definition.name),
                )
            }),
    );
    diagnostics
}

#[derive(Debug)]
struct Definition {
    name: String,
    offset: usize,
    length: usize,
}

#[derive(Debug)]
struct DocEntry {
    name: String,
    offset: usize,
    length: usize,
    has_summary: bool,
    hidden: bool,
}

fn duplicate_definition_diagnostics(
    source_name: &str,
    source: &str,
    definitions: &[Definition],
) -> Vec<SourceDiagnostic> {
    duplicate_name_diagnostics(
        definitions,
        |definition| definition.name.as_str(),
        |name, first, duplicate| {
            SourceDiagnostic::duplicate_definition(
                source_name,
                source.to_owned(),
                name,
                first.offset,
                first.length,
                duplicate.offset,
                duplicate.length,
            )
        },
    )
}

fn duplicate_doc_diagnostics(
    source_name: &str,
    source: &str,
    doc_entries: &[DocEntry],
) -> Vec<SourceDiagnostic> {
    duplicate_name_diagnostics(
        doc_entries,
        |entry| entry.name.as_str(),
        |name, first, duplicate| {
            SourceDiagnostic::duplicate_doc(
                source_name,
                source.to_owned(),
                name,
                first.offset,
                first.length,
                duplicate.offset,
                duplicate.length,
            )
        },
    )
}

fn duplicate_name_diagnostics<'a, T>(
    items: &'a [T],
    mut name: impl FnMut(&'a T) -> &'a str,
    mut diagnostic: impl FnMut(&'a str, &'a T, &'a T) -> SourceDiagnostic,
) -> Vec<SourceDiagnostic> {
    let mut seen = HashMap::<&str, &T>::new();
    let mut reported = HashSet::<&str>::new();
    let mut diagnostics = Vec::new();
    for item in items {
        let item_name = name(item);
        match seen.entry(item_name) {
            Entry::Occupied(first) if reported.insert(item_name) => {
                diagnostics.push(diagnostic(item_name, *first.get(), item));
            }
            Entry::Occupied(_) => {}
            Entry::Vacant(entry) => {
                let _inserted = entry.insert(item);
            }
        }
    }
    diagnostics
}

fn missing_summary_diagnostics(
    source_name: &str,
    source: &str,
    doc_entries: &[DocEntry],
) -> Vec<SourceDiagnostic> {
    doc_entries
        .iter()
        .filter(|entry| !entry.hidden && !entry.has_summary)
        .map(|entry| {
            SourceDiagnostic::missing_doc_summary(
                source_name,
                source.to_owned(),
                &entry.name,
                entry.offset,
                entry.length,
            )
        })
        .collect()
}

fn dedup_doc_entries(doc_entries: &mut Vec<DocEntry>) {
    let mut seen = HashSet::new();
    doc_entries.retain(|entry| seen.insert((entry.name.clone(), entry.offset, entry.length)));
}

fn collect_definitions(source: &str, syntax: &Syntax, output: &mut Vec<Definition>) {
    if let Some(definition) = definition_from_form(source, syntax) {
        output.push(definition);
    }

    if let Some(items) = proper_list(syntax) {
        for item in items {
            collect_definitions(source, item, output);
        }
    }
}

fn collect_doc_entries(source: &str, syntax: &Syntax, output: &mut Vec<DocEntry>) {
    if let Some(entry) = doc_entry_from_form(source, syntax) {
        output.push(entry);
        return;
    }

    if let Some(items) = proper_list(syntax) {
        for (index, item) in items.iter().enumerate() {
            if let Some(entry) = doc_next_entry_from_items(source, items, index) {
                output.push(entry);
                continue;
            }
            collect_doc_entries(source, item, output);
        }
    }
}

fn doc_entry_from_form(source: &str, form: &Syntax) -> Option<DocEntry> {
    let items = proper_list(form)?;
    let head = identifier_text(items.first()?)?;
    if head != "doc" && head != "typedoc" && head != "extern-doc" {
        return None;
    }

    let subject_syntax = items.get(1)?;
    let name = subject_text(subject_syntax)?;
    let mut entry = DocEntry {
        offset: source_position_byte_offset(
            source,
            subject_syntax.span().line,
            subject_syntax.span().column + quote_prefix_len(subject_syntax),
        ),
        length: name.len(),
        name,
        has_summary: false,
        hidden: false,
    };

    apply_doc_fields(&mut entry, &items[2..]);

    Some(entry)
}

fn doc_next_entry_from_items(source: &str, items: &[Syntax], index: usize) -> Option<DocEntry> {
    let doc_next_items = proper_list(items.get(index)?)?;
    if doc_next_items
        .first()
        .and_then(identifier_text)
        .is_none_or(|head| head != "doc-next")
    {
        return None;
    }
    let definition = items
        .iter()
        .skip(index + 1)
        .find_map(|item| definition_from_form(source, item))?;
    let mut entry = DocEntry {
        offset: definition.offset,
        length: definition.length,
        name: definition.name,
        has_summary: false,
        hidden: false,
    };
    apply_doc_fields(&mut entry, &doc_next_items[1..]);
    Some(entry)
}

fn apply_doc_fields(entry: &mut DocEntry, fields: &[Syntax]) {
    for field in fields {
        let Some(field_items) = proper_list(field) else {
            continue;
        };
        let Some(field_name) = field_items.first().and_then(identifier_text) else {
            continue;
        };
        match field_name.as_str() {
            "summary" => {
                entry.has_summary = field_items.get(1).and_then(wrapped_string_text).is_some();
            }
            "hidden" => entry.hidden = true,
            _ => {}
        }
    }
}

fn definition_from_form(source: &str, form: &Syntax) -> Option<Definition> {
    let items = proper_list(form)?;
    let head = identifier_text(items.first()?)?;
    if head != "define" && head != "define-syntax" {
        return None;
    }

    let subject = items.get(1)?;
    if let Some(name) = identifier_text(subject) {
        return Some(definition(source, subject, name));
    }

    let signature = proper_list(subject)?;
    let name_syntax = signature.first()?;
    let name = identifier_text(name_syntax)?;
    Some(definition(source, name_syntax, name))
}

fn definition(source: &str, syntax: &Syntax, name: String) -> Definition {
    Definition {
        offset: source_position_byte_offset(source, syntax.span().line, syntax.span().column),
        length: name.len(),
        name,
    }
}

fn subject_text(syntax: &Syntax) -> Option<String> {
    if let Some(text) = identifier_text(syntax) {
        return Some(text);
    }
    if let Some(text) = wrapped_string_text(syntax) {
        return Some(text);
    }

    let items = proper_list(syntax)?;
    match items {
        [head, subject] if is_identifier(head, "quote") => {
            identifier_text(subject).or_else(|| wrapped_string_text(subject))
        }
        _ => None,
    }
}

fn quote_prefix_len(syntax: &Syntax) -> usize {
    let Some(items) = proper_list(syntax) else {
        return 0;
    };
    match items {
        [head, _] if is_identifier(head, "quote") => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests;
