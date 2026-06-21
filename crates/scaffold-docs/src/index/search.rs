use nucleo_matcher::{
    Config as FuzzyConfig, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};
use scaffold_editor::reference::{ReferenceItem, same_markdown_paragraph};

use super::{
    join_text,
    model::{DocEntry, DocIndex},
};

#[must_use]
pub fn search_doc_entries<'a>(docs: &'a DocIndex, query: &str, limit: usize) -> Vec<&'a DocEntry> {
    search_reference_entries(docs.entries(), query, limit)
}

#[must_use]
pub fn suggest_doc_entries<'a>(docs: &'a DocIndex, query: &str, limit: usize) -> Vec<&'a DocEntry> {
    suggest_reference_entries(docs.entries(), query, limit)
}

#[must_use]
pub fn search_reference_entries<'a, E>(
    entries: impl IntoIterator<Item = &'a E>,
    query: &str,
    limit: usize,
) -> Vec<&'a E>
where
    E: ReferenceItem + 'a,
{
    let query_patterns = search_query_variants(query)
        .into_iter()
        .map(|query| {
            let pattern = Pattern::parse(&query, CaseMatching::Ignore, Normalization::Smart);
            (query, pattern)
        })
        .collect::<Vec<_>>();
    let mut matcher = Matcher::new(FuzzyConfig::DEFAULT);
    let mut buf = Vec::new();
    let mut matches = entries
        .into_iter()
        .filter(|entry| !entry.hidden())
        .filter(|entry| {
            query_patterns
                .iter()
                .any(|(query, _)| reference_entry_is_plausible_fuzzy_match(*entry, query))
        })
        .filter_map(|entry| {
            query_patterns
                .iter()
                .filter_map(|(query, pattern)| {
                    reference_entry_search_score(entry, query, pattern, &mut matcher, &mut buf)
                })
                .max()
                .map(|score| (entry, score))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|(left_entry, left_score), (right_entry, right_score)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_entry.name().cmp(right_entry.name()))
    });
    matches
        .into_iter()
        .take(limit)
        .map(|(entry, _score)| entry)
        .collect()
}

fn search_query_variants(query: &str) -> Vec<String> {
    std::iter::once(query.to_owned())
        .chain(
            source_location_query_without_column(query)
                .filter(|stripped| *stripped != query)
                .map(str::to_owned),
        )
        .collect()
}

#[must_use]
pub fn source_path_from_location_query(query: &str) -> Option<&str> {
    parse_source_location_query(query).map(|query| query.source)
}

fn source_location_query_without_column(query: &str) -> Option<&str> {
    parse_source_location_query(query)?.source_with_line
}

struct SourceLocationQuery<'a> {
    source: &'a str,
    source_with_line: Option<&'a str>,
}

fn parse_source_location_query(query: &str) -> Option<SourceLocationQuery<'_>> {
    let (source_or_source_with_line, line_or_column) = query.rsplit_once(':')?;
    if source_or_source_with_line.is_empty() || !is_ascii_digits(line_or_column) {
        return None;
    }

    if let Some((source, line)) = source_or_source_with_line.rsplit_once(':')
        && !source.is_empty()
        && is_ascii_digits(line)
    {
        return Some(SourceLocationQuery {
            source,
            source_with_line: Some(source_or_source_with_line),
        });
    }

    Some(SourceLocationQuery {
        source: source_or_source_with_line,
        source_with_line: None,
    })
}

fn is_ascii_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

#[must_use]
pub fn suggest_reference_entries<'a, E>(
    entries: impl IntoIterator<Item = &'a E>,
    query: &str,
    limit: usize,
) -> Vec<&'a E>
where
    E: ReferenceItem + 'a,
{
    let query = normalize_reference_query_token(query);
    let max_distance = suggestion_distance_threshold(&query);
    if max_distance == 0 {
        return Vec::new();
    }

    let mut matches = entries
        .into_iter()
        .filter(|entry| !entry.hidden())
        .filter_map(|entry| {
            reference_entry_suggestion_score(entry, &query, max_distance)
                .map(|score| (entry, score))
        })
        .collect::<Vec<_>>();
    matches.sort_by(|(left_entry, left_score), (right_entry, right_score)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_entry.name().cmp(right_entry.name()))
    });
    matches
        .into_iter()
        .take(limit)
        .map(|(entry, _score)| entry)
        .collect()
}

fn reference_entry_search_score(
    entry: &impl ReferenceItem,
    query: &str,
    pattern: &Pattern,
    matcher: &mut Matcher,
    buf: &mut Vec<char>,
) -> Option<u32> {
    let searchable = reference_entry_search_text(entry);
    let mut score = pattern.score(Utf32Str::new(&searchable, buf), matcher)?;

    if let Some(name_score) = pattern.score(Utf32Str::new(entry.name(), buf), matcher) {
        score += name_score * 8;
    }
    score += reference_entry_name_match_bonus(entry.name(), query);
    if let Some(signature) = entry.signature()
        && let Some(signature_score) = pattern.score(Utf32Str::new(signature, buf), matcher)
    {
        score += signature_score * 4;
    }
    if let Some(summary) = entry.summary()
        && let Some(summary_score) = pattern.score(Utf32Str::new(summary, buf), matcher)
    {
        score += summary_score * 2;
    }

    Some(score)
}

fn reference_entry_suggestion_score(
    entry: &impl ReferenceItem,
    query: &str,
    max_distance: usize,
) -> Option<usize> {
    reference_entry_suggestion_candidates(entry)
        .filter(|(candidate, _)| !candidate.is_empty())
        .filter_map(|(candidate, priority)| {
            let distance = typo_distance(query, &candidate, max_distance);
            (distance <= max_distance).then_some(distance * 10 + priority)
        })
        .min()
}

fn reference_entry_suggestion_candidates(
    entry: &impl ReferenceItem,
) -> impl Iterator<Item = (String, usize)> + '_ {
    std::iter::once((normalize_reference_query_token(entry.name()), 0))
        .chain(search_field_tokens(entry.name()).map(|token| (token, 1)))
}

fn suggestion_distance_threshold(query: &str) -> usize {
    match query.chars().count() {
        0..=3 => 0,
        4..=7 => 1,
        8..=12 => 2,
        _ => 3,
    }
}

fn reference_entry_name_match_bonus(name: &str, query: &str) -> u32 {
    let query = normalize_reference_query_token(query);
    if query.is_empty() {
        return 0;
    }

    let name = normalize_reference_query_token(name);
    if name == query {
        return 1_000_000 + query.len() as u32;
    }
    if name.starts_with(&query) {
        return 100_000 + query.len() as u32;
    }

    0
}

fn reference_entry_is_plausible_fuzzy_match(entry: &impl ReferenceItem, query: &str) -> bool {
    let mut tokens = query
        .split_whitespace()
        .map(normalize_reference_query_token)
        .filter(|token| !token.is_empty())
        .peekable();
    if tokens.peek().is_none() {
        return false;
    }

    tokens.all(|token| {
        reference_entry_symbolic_search_fields(entry)
            .any(|field| symbolic_field_matches_token(&field, &token))
            || reference_entry_prose_search_fields(entry)
                .map(|field| normalize_reference_query_token(&field))
                .any(|field| field.contains(&token))
    })
}

fn symbolic_field_matches_token(field: &str, token: &str) -> bool {
    let normalized_field = normalize_reference_query_token(field);
    if normalized_field == token
        || normalized_field.starts_with(token)
        || normalized_field.contains(token)
    {
        return true;
    }

    search_field_tokens(field).any(|field_token| {
        field_token == token
            || field_token.starts_with(token)
            || field_token.contains(token)
            || (token.len() >= 3 && is_subsequence(token, &field_token))
    })
}

fn reference_entry_search_text(entry: &impl ReferenceItem) -> String {
    join_text(
        reference_entry_symbolic_search_fields(entry)
            .chain(reference_entry_prose_search_fields(entry)),
        " ",
    )
}

fn reference_entry_symbolic_search_fields(
    entry: &impl ReferenceItem,
) -> impl Iterator<Item = String> + '_ {
    [
        entry.name().to_owned(),
        entry.group().unwrap_or("Language").to_owned(),
        entry.signature().unwrap_or_default().to_owned(),
        entry.source().unwrap_or_default().to_owned(),
        entry.source_location().unwrap_or_default(),
        entry.effect().unwrap_or_default().to_owned(),
        entry.since().unwrap_or_default().to_owned(),
        entry.stability().unwrap_or_default().to_owned(),
    ]
    .into_iter()
    .chain(entry.params().map(|param| param.name.to_owned()))
    .chain(entry.see().map(str::to_owned))
    .chain(entry.requires_capability().map(str::to_owned))
}

pub(super) fn reference_entry_prose_search_fields(
    entry: &impl ReferenceItem,
) -> impl Iterator<Item = String> + '_ {
    let summary = entry.summary().unwrap_or_default();
    let markdown = entry.markdown().unwrap_or_default();
    std::iter::once(summary.to_owned())
        .chain((!same_markdown_paragraph(summary, markdown)).then_some(markdown.to_owned()))
        .chain([
            entry.example().unwrap_or_default().to_owned(),
            entry.returns().unwrap_or_default().to_owned(),
            entry.deprecated().unwrap_or_default().to_owned(),
        ])
        .chain(entry.params().map(|param| param.summary.to_owned()))
}

#[must_use]
pub fn normalize_reference_query_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn search_field_tokens(value: &str) -> impl Iterator<Item = String> + '_ {
    value
        .split(|ch: char| !ch.is_alphanumeric())
        .map(normalize_reference_query_token)
        .filter(|token| !token.is_empty())
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    needle
        .chars()
        .all(|needle_ch| chars.any(|ch| ch == needle_ch))
}

fn typo_distance(left: &str, right: &str, max_distance: usize) -> usize {
    if left.chars().count().abs_diff(right.chars().count()) > max_distance {
        return max_distance + 1;
    }

    strsim::osa_distance(left, right)
}
