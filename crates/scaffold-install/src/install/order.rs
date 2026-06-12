use std::collections::{HashMap, HashSet};

use scaffold_catalog::{Catalog, Phase, Tool};

use super::InstallError;

fn validate_requested_tools(catalog: &Catalog, names: &[String]) -> Result<(), InstallError> {
    let available = catalog
        .tools
        .iter()
        .map(|tool| tool.name.as_str())
        .collect::<HashSet<_>>();
    for name in names {
        if !available.contains(name.as_str()) {
            return Err(InstallError::MissingNamedTool(name.clone()));
        }
    }
    Ok(())
}

pub(super) fn resolve_install_order<'a>(
    catalog: &'a Catalog,
    names: &[String],
) -> Result<Vec<&'a Tool>, InstallError> {
    validate_requested_tools(catalog, names)?;
    let indices = tool_indices(catalog)?;
    let mut selected = HashSet::new();
    if names.is_empty() {
        selected.extend(0..catalog.tools.len());
    } else {
        for name in names {
            let index = indices[name.as_str()];
            include_dependencies(catalog, &indices, index, &mut selected)?;
        }
    }

    let mut edges = HashMap::<usize, HashSet<usize>>::new();
    for &index in &selected {
        let _targets = edges.entry(index).or_default();
    }

    for &index in &selected {
        let tool = &catalog.tools[index];
        for dependency in &tool.depends {
            let Some(&dependency_index) = indices.get(dependency.as_str()) else {
                return Err(InstallError::MissingDependency {
                    tool: tool.name.clone(),
                    dependency: dependency.clone(),
                });
            };
            if selected.contains(&dependency_index) {
                let _inserted = edges.entry(dependency_index).or_default().insert(index);
            }
        }
        for target in &tool.after {
            let Some(&target_index) = indices.get(target.as_str()) else {
                return Err(InstallError::MissingOrderReference {
                    tool: tool.name.clone(),
                    reference: target.clone(),
                });
            };
            if selected.contains(&target_index) {
                let _inserted = edges.entry(target_index).or_default().insert(index);
            }
        }
        for target in tool
            .before
            .iter()
            .filter(|target| !matches!(target.as_str(), "none" | "first"))
        {
            let Some(&target_index) = indices.get(target.as_str()) else {
                return Err(InstallError::MissingOrderReference {
                    tool: tool.name.clone(),
                    reference: target.clone(),
                });
            };
            if selected.contains(&target_index) {
                let _inserted = edges.entry(index).or_default().insert(target_index);
            }
        }
    }

    let mut indegree = selected
        .iter()
        .copied()
        .map(|index| (index, 0usize))
        .collect::<HashMap<_, _>>();
    for targets in edges.values() {
        for &target in targets {
            if let Some(count) = indegree.get_mut(&target) {
                *count += 1;
            }
        }
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(&index, &count)| (count == 0).then_some(index))
        .collect::<Vec<_>>();
    let mut ordered = Vec::with_capacity(selected.len());
    while !ready.is_empty() {
        ready.sort_by_key(|&index| install_sort_key(&catalog.tools[index], index));
        let index = ready.remove(0);
        ordered.push(index);
        if let Some(targets) = edges.get(&index) {
            let mut targets = targets.iter().copied().collect::<Vec<_>>();
            targets.sort_by_key(|&target| install_sort_key(&catalog.tools[target], target));
            for target in targets {
                let count = indegree.get_mut(&target).expect("target indegree");
                *count -= 1;
                if *count == 0 {
                    ready.push(target);
                }
            }
        }
    }

    if ordered.len() != selected.len() {
        return Err(InstallError::CyclicInstallOrder);
    }

    Ok(ordered
        .into_iter()
        .map(|index| &catalog.tools[index])
        .collect())
}

fn tool_indices(catalog: &Catalog) -> Result<HashMap<&str, usize>, InstallError> {
    let mut indices = HashMap::new();
    for (index, tool) in catalog.tools.iter().enumerate() {
        if indices.insert(tool.name.as_str(), index).is_some() {
            return Err(InstallError::DuplicateTool(tool.name.clone()));
        }
    }
    Ok(indices)
}

fn include_dependencies(
    catalog: &Catalog,
    indices: &HashMap<&str, usize>,
    index: usize,
    selected: &mut HashSet<usize>,
) -> Result<(), InstallError> {
    if !selected.insert(index) {
        return Ok(());
    }
    let tool = &catalog.tools[index];
    for dependency in &tool.depends {
        let Some(&dependency_index) = indices.get(dependency.as_str()) else {
            return Err(InstallError::MissingDependency {
                tool: tool.name.clone(),
                dependency: dependency.clone(),
            });
        };
        include_dependencies(catalog, indices, dependency_index, selected)?;
    }
    Ok(())
}

fn install_sort_key(tool: &Tool, index: usize) -> (bool, Phase, i32, usize) {
    (
        !tool.wants_first(),
        tool.phase(),
        tool.order.unwrap_or(0),
        index,
    )
}
