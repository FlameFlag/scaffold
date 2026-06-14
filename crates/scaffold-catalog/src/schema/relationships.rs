use std::collections::BTreeMap;

use petgraph::algo::toposort;
use petgraph::graphmap::DiGraphMap;
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
    let mut graph = DiGraphMap::<usize, ()>::new();
    for index in 0..tools.len() {
        graph.add_node(index);
    }

    for (index, tool) in tools.iter().enumerate() {
        let object = tool.as_object().expect("tool shape already validated");
        for (_, dependency) in string_array_items(object, "depends") {
            graph.add_edge(names[dependency], index, ());
        }
        for (_, target) in string_array_items(object, "after") {
            graph.add_edge(names[target], index, ());
        }
        for (_, target) in string_array_items(object, "before") {
            if !matches!(target, "first" | "none") {
                graph.add_edge(index, names[target], ());
            }
        }
    }

    if toposort(&graph, None).is_err() {
        return Err("catalog install order contains a dependency cycle".to_owned());
    }
    Ok(())
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
