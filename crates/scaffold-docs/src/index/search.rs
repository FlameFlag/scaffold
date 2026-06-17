use nucleo_matcher::{
    Config as FuzzyConfig, Matcher, Utf32Str,
    pattern::{CaseMatching, Normalization, Pattern},
};
use scaffold_editor::reference::{ReferenceItem, same_markdown_paragraph};

use super::model::{DocEntry, DocIndex};

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
                    reference_entry_search_score(entry, query, pattern, &mut matcher)
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
    let mut variants = vec![query.to_owned()];
    if let Some(stripped) = source_location_query_without_column(query)
        && stripped != query
    {
        variants.push(stripped.to_owned());
    }
    variants
}

fn source_location_query_without_column(query: &str) -> Option<&str> {
    let (source_and_line, column) = query.rsplit_once(':')?;
    if column.is_empty() || !column.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    let (source, line) = source_and_line.rsplit_once(':')?;
    if source.is_empty() || line.is_empty() || !line.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    Some(source_and_line)
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
    let query = normalize_search_token(query);
    let max_distance = suggestion_distance_threshold(&query);
    if max_distance == 0 {
        return Vec::new();
    }

    let mut suggestions = entries
        .into_iter()
        .filter(|entry| !entry.hidden())
        .filter_map(|entry| {
            reference_entry_suggestion_score(entry, &query, max_distance)
                .map(|score| (entry, score))
        })
        .collect::<Vec<_>>();
    suggestions.sort_by(|(left_entry, left_score), (right_entry, right_score)| {
        left_score
            .cmp(right_score)
            .then_with(|| left_entry.name().cmp(right_entry.name()))
    });
    suggestions
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
) -> Option<u32> {
    let mut buf = Vec::new();
    let searchable = reference_entry_search_text(entry);
    let mut score = pattern.score(Utf32Str::new(&searchable, &mut buf), matcher)?;

    if let Some(name_score) = pattern.score(Utf32Str::new(entry.name(), &mut buf), matcher) {
        score += name_score * 8;
    }
    score += reference_entry_name_match_bonus(entry.name(), query);
    if let Some(signature) = entry.signature()
        && let Some(signature_score) = pattern.score(Utf32Str::new(signature, &mut buf), matcher)
    {
        score += signature_score * 4;
    }
    if let Some(summary) = entry.summary()
        && let Some(summary_score) = pattern.score(Utf32Str::new(summary, &mut buf), matcher)
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
        .into_iter()
        .filter(|(candidate, _)| !candidate.is_empty())
        .filter_map(|(candidate, priority)| {
            let distance = edit_distance(query, &candidate, max_distance);
            (distance <= max_distance).then_some(distance * 10 + priority)
        })
        .min()
}

fn reference_entry_suggestion_candidates(entry: &impl ReferenceItem) -> Vec<(String, usize)> {
    let mut candidates = Vec::new();
    candidates.push((normalize_search_token(entry.name()), 0));
    candidates.extend(
        search_field_tokens(entry.name())
            .into_iter()
            .map(|token| (token, 1)),
    );
    candidates
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
    let query = normalize_search_token(query);
    if query.is_empty() {
        return 0;
    }

    let name = normalize_search_token(name);
    if name == query {
        return 1_000_000 + query.len() as u32;
    }
    if name.starts_with(&query) {
        return 100_000 + query.len() as u32;
    }

    0
}

fn reference_entry_is_plausible_fuzzy_match(entry: &impl ReferenceItem, query: &str) -> bool {
    let tokens = query
        .split_whitespace()
        .map(normalize_search_token)
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    if tokens.is_empty() {
        return false;
    }

    let symbolic_fields = reference_entry_symbolic_search_fields(entry);
    let prose_fields = reference_entry_prose_search_fields(entry)
        .iter()
        .map(|field| normalize_search_token(field))
        .collect::<Vec<_>>();

    tokens.iter().all(|token| {
        symbolic_fields
            .iter()
            .any(|field| symbolic_field_matches_token(field, token))
            || prose_fields.iter().any(|field| field.contains(token))
    })
}

fn symbolic_field_matches_token(field: &str, token: &str) -> bool {
    let normalized_field = normalize_search_token(field);
    if normalized_field == token
        || normalized_field.starts_with(token)
        || normalized_field.contains(token)
    {
        return true;
    }

    search_field_tokens(field).into_iter().any(|field_token| {
        field_token == token
            || field_token.starts_with(token)
            || field_token.contains(token)
            || (token.len() >= 3 && is_subsequence(token, &field_token))
    })
}

fn reference_entry_search_text(entry: &impl ReferenceItem) -> String {
    let mut fields = reference_entry_symbolic_search_fields(entry);
    fields.extend(reference_entry_prose_search_fields(entry));
    fields.join(" ")
}

fn reference_entry_symbolic_search_fields(entry: &impl ReferenceItem) -> Vec<String> {
    let mut parts = vec![
        entry.name().to_owned(),
        entry.group().unwrap_or("Language").to_owned(),
        entry.signature().unwrap_or_default().to_owned(),
        entry.source().unwrap_or_default().to_owned(),
        entry.source_location().unwrap_or_default(),
        entry.effect().unwrap_or_default().to_owned(),
        entry.since().unwrap_or_default().to_owned(),
        entry.stability().unwrap_or_default().to_owned(),
    ];
    parts.extend(entry.params().iter().map(|param| param.name.to_owned()));
    parts.extend(entry.see().into_iter().map(str::to_owned));
    parts.extend(entry.requires_capability().into_iter().map(str::to_owned));
    parts
}

pub(super) fn reference_entry_prose_search_fields(entry: &impl ReferenceItem) -> Vec<String> {
    let summary = entry.summary().unwrap_or_default();
    let markdown = entry.markdown().unwrap_or_default();
    let mut parts = vec![summary.to_owned()];
    if !same_markdown_paragraph(summary, markdown) {
        parts.push(markdown.to_owned());
    }
    parts.extend([
        entry.example().unwrap_or_default().to_owned(),
        entry.returns().unwrap_or_default().to_owned(),
        entry.deprecated().unwrap_or_default().to_owned(),
    ]);
    parts.extend(entry.params().iter().map(|param| param.summary.to_owned()));
    parts
}

fn normalize_search_token(value: &str) -> String {
    value
        .chars()
        .filter(|ch| ch.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn search_field_tokens(value: &str) -> Vec<String> {
    value
        .split(|ch: char| !ch.is_alphanumeric())
        .map(normalize_search_token)
        .filter(|token| !token.is_empty())
        .collect()
}

fn is_subsequence(needle: &str, haystack: &str) -> bool {
    let mut chars = haystack.chars();
    needle
        .chars()
        .all(|needle_ch| chars.any(|ch| ch == needle_ch))
}

fn edit_distance(left: &str, right: &str, max_distance: usize) -> usize {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    if left.len().abs_diff(right.len()) > max_distance {
        return max_distance + 1;
    }

    let mut distances = vec![vec![0; right.len() + 1]; left.len() + 1];
    for (index, row) in distances.iter_mut().enumerate() {
        row[0] = index;
    }
    for (index, cell) in distances[0].iter_mut().enumerate() {
        *cell = index;
    }

    for left_index in 1..=left.len() {
        let mut row_min = distances[left_index][0];
        for right_index in 1..=right.len() {
            let cost = usize::from(left[left_index - 1] != right[right_index - 1]);
            let insert = distances[left_index][right_index - 1] + 1;
            let delete = distances[left_index - 1][right_index] + 1;
            let replace = distances[left_index - 1][right_index - 1] + cost;
            let mut distance = insert.min(delete).min(replace);

            if left_index > 1
                && right_index > 1
                && left[left_index - 1] == right[right_index - 2]
                && left[left_index - 2] == right[right_index - 1]
            {
                distance = distance.min(distances[left_index - 2][right_index - 2] + 1);
            }

            distances[left_index][right_index] = distance;
            row_min = row_min.min(distance);
        }

        if row_min > max_distance {
            return max_distance + 1;
        }
    }

    distances[left.len()][right.len()]
}
