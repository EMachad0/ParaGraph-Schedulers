use para_graph::model::{Dependency, Task};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

pub fn parse_task_xml(xml: &str) -> DiGraph<Task, Dependency> {
    let doc = roxmltree::Document::parse(xml).expect("Failed to parse XML");

    let root = doc.root_element();

    let mut graph = DiGraph::<Task, Dependency>::new();
    let mut indexes = HashMap::<String, NodeIndex>::new();
    let mut dependencies = HashMap::<String, Vec<Dependency>>::new();

    for node in root.children().filter(|n| n.is_element()) {
        match node.tag_name().name() {
            "job" => {
                let id = node.attribute("id").expect("Job ID not found");
                let processing_density = node
                    .attribute("runtime")
                    .expect("Job runtime not found")
                    .parse::<f64>()
                    .expect("Failed to parse Job runtime");

                let mut job_deps = Vec::new();
                let mut total_data_size = 0;
                for uses in node.children().filter(|n| n.is_element()) {
                    if uses.tag_name().name() != "uses" {
                        unreachable!("Unexpected tag: {}", uses.tag_name().name());
                    }

                    let link = uses.attribute("link").expect("Link not found");
                    let data_size = uses
                        .attribute("size")
                        .expect("Data size not found")
                        .parse::<u64>()
                        .expect("Failed to parse data size");
                    let dependency = Dependency::new(data_size);
                    if link == "input" {
                        total_data_size += data_size;
                        job_deps.push(dependency);
                    }
                }
                let task = Task {
                    data_size: total_data_size,
                    processing_density,
                    parallel_fraction: 0.,
                    pin: None,
                };
                let index = graph.add_node(task);
                indexes.insert(id.to_string(), index);
                dependencies.insert(id.to_string(), job_deps);
            }
            "child" => {
                let child = node.attribute("ref").expect("Parent ref not found");
                let child_index = indexes.get(child).expect("Parent not found");

                let parents = node.children().filter(|n| n.is_element());
                let dependencies = dependencies
                    .get(child)
                    .expect("Parent dependencies not found");
                for (parent, dependency) in parents.zip(dependencies.iter()) {
                    if parent.tag_name().name() != "parent" {
                        unreachable!("Unexpected tag: {}", parent.tag_name().name());
                    }

                    let parent = parent.attribute("ref").expect("Parent ref not found");
                    let parent_index = indexes.get(parent).expect("Parent not found");

                    graph.add_edge(*parent_index, *child_index, *dependency);
                }
            }
            _ => unreachable!("Unexpected tag: {}", node.tag_name().name()),
        }
    }
    graph
}
