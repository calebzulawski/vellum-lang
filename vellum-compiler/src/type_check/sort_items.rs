use crate::parse::{Context, ast};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use petgraph::graph::DiGraph;
use std::collections::HashMap;

pub fn sort(
    context: &mut Context,
    mut items: HashMap<String, ast::Item>,
    dependencies: HashMap<String, Vec<String>>,
) -> Result<Vec<ast::Item>, ()> {
    // First step
    // * Add types to the directed graph.
    let mut graph = DiGraph::new();

    let mut name_to_index = HashMap::new();
    let mut index_to_name = HashMap::new();

    for name in dependencies.keys() {
        let index = graph.add_node(name.clone());
        name_to_index.insert(name.clone(), index);
        index_to_name.insert(index, name.clone());
    }

    // Second step
    // * Add edges between nodes for direct dependencies
    for (name, deps) in &dependencies {
        for dep in deps {
            graph.add_edge(
                *name_to_index.get(dep).unwrap(),
                *name_to_index.get(name).unwrap(),
                (),
            );
        }
    }

    // Third step
    // * Sort the types
    let mut sorted = Vec::new();
    match petgraph::algo::toposort(&graph, None) {
        Err(e) => {
            let location = items
                .get(index_to_name.get(&e.node_id()).unwrap())
                .unwrap()
                .location();
            context.report(
                &Diagnostic::error()
                    .with_message("cycle detected")
                    .with_labels(vec![
                        Label::primary(location.file_id, location.span.clone())
                            .with_message("this type contains itself"),
                    ]),
            );
        }
        Ok(order) => {
            for index in order {
                sorted.push(items.remove(index_to_name.get(&index).unwrap()).unwrap());
            }
        }
    }
    sorted.extend(items.into_iter().map(|(_, v)| v));

    Ok(sorted)
}
