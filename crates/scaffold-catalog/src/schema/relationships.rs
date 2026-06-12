use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

pub(super) fn validate_tool_references(
    tools: &[Value],
    names: &BTreeMap<String, usize>,
) -> Result<(), String> {
    for (index, tool) in tools.iter().enumerate() {
        let object = tool.as_object().expect("tool shape already validated");
        let tool_name = object
            .get("name")
            .and_then(Value::as_str)
            .expect("tool name already validated");
        for field in ["depends", "after"] {
            for (ref_index, reference) in string_array_items(object, field) {
                if !names.contains_key(reference) {
                    return Err(format!(
                        "$.tools[{index}].{field}[{ref_index}] references unknown tool {reference:?} from {tool_name:?}"
                    ));
                }
            }
        }
        for (ref_index, reference) in string_array_items(object, "before") {
            if matches!(reference, "first" | "none") {
                continue;
            }
            if !names.contains_key(reference) {
                return Err(format!(
                    "$.tools[{index}].before[{ref_index}] references unknown tool {reference:?} from {tool_name:?}"
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_install_order(
    tools: &[Value],
    names: &BTreeMap<String, usize>,
) -> Result<(), String> {
    let mut edges = BTreeMap::<usize, BTreeSet<usize>>::new();
    for index in 0..tools.len() {
        let _targets = edges.entry(index).or_default();
    }

    for (index, tool) in tools.iter().enumerate() {
        let object = tool.as_object().expect("tool shape already validated");
        for (_, dependency) in string_array_items(object, "depends") {
            let _inserted = edges.entry(names[dependency]).or_default().insert(index);
        }
        for (_, target) in string_array_items(object, "after") {
            let _inserted = edges.entry(names[target]).or_default().insert(index);
        }
        for (_, target) in string_array_items(object, "before") {
            if !matches!(target, "first" | "none") {
                let _inserted = edges.entry(index).or_default().insert(names[target]);
            }
        }
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for index in 0..tools.len() {
        if visit_install_order(index, &edges, &mut visiting, &mut visited) {
            return Err("catalog install order contains a dependency cycle".to_owned());
        }
    }
    Ok(())
}

fn visit_install_order(
    index: usize,
    edges: &BTreeMap<usize, BTreeSet<usize>>,
    visiting: &mut BTreeSet<usize>,
    visited: &mut BTreeSet<usize>,
) -> bool {
    if visited.contains(&index) {
        return false;
    }
    if !visiting.insert(index) {
        return true;
    }
    for target in edges.get(&index).into_iter().flatten() {
        if visit_install_order(*target, edges, visiting, visited) {
            return true;
        }
    }
    let _removed = visiting.remove(&index);
    let _inserted = visited.insert(index);
    false
}

fn string_array_items<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> impl Iterator<Item = (usize, &'a str)> {
    object
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(index, value)| value.as_str().map(|value| (index, value)))
}
