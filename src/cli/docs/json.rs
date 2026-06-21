use scaffold_docs::{DocEntry, reference_entry_json};
use serde_json::{Value, json};

pub(super) use crate::cli::io::pretty_json as to_pretty_json;

pub(super) fn entries_json_response(
    mode: &str,
    query: Option<&str>,
    limit: Option<usize>,
    entries: &[&DocEntry],
) -> Result<String, serde_json::Error> {
    to_pretty_json(&entries_json_value(mode, query, limit, entries))
}

pub(super) fn entries_json_value(
    mode: &str,
    query: Option<&str>,
    limit: Option<usize>,
    entries: &[&DocEntry],
) -> Value {
    let mut response = json!({
        "mode": mode,
        "count": entries.len(),
        "entries": json_entries(entries.iter().copied()),
    });

    if let Some(query) = query {
        response["query"] = json!(query);
    }
    if let Some(limit) = limit {
        response["limit"] = json!(limit);
    }

    response
}

pub(super) fn json_entries<'a>(entries: impl IntoIterator<Item = &'a DocEntry>) -> Value {
    json!(
        entries
            .into_iter()
            .map(reference_entry_json)
            .collect::<Vec<_>>()
    )
}

pub(super) fn group_counts_json<'a>(
    groups: impl IntoIterator<Item = (&'a str, usize)>,
) -> Vec<Value> {
    groups
        .into_iter()
        .map(|(group, count)| {
            json!({
                "name": group,
                "entries": count,
            })
        })
        .collect()
}
