use para_graph::model::{Device, Transmission};
use petgraph::prelude::*;
use std::collections::HashMap;

pub fn parse_topology_xml(xml: &str) -> UnGraph<Device, Transmission> {
    let doc = roxmltree::Document::parse(xml).expect("Failed to parse XML");

    let root = doc.root_element();

    let mut graph = UnGraph::<Device, Transmission>::new_undirected();
    let mut indexes = HashMap::<String, NodeIndex>::new();
    let mut edges = HashMap::<String, Vec<Transmission>>::new();

    for node in root.children().filter(|n| n.is_element()) {
        match node.tag_name().name() {
            "job" => {
                let id = node.attribute("id").expect("Job ID not found");
                let cpu_frequency = node
                    .attribute("runtime")
                    .expect("Job runtime not found")
                    .parse::<f64>()
                    .expect("Failed to parse Job runtime");

                let mut job_deps = Vec::new();
                for uses in node.children().filter(|n| n.is_element()) {
                    if uses.tag_name().name() != "uses" {
                        unreachable!("Unexpected tag: {}", uses.tag_name().name());
                    }

                    let link = uses.attribute("link").expect("Link not found");
                    let data_size = uses
                        .attribute("size")
                        .expect("Data size not found")
                        .parse::<f64>()
                        .expect("Failed to parse data size");
                    let transmission = Transmission::new(data_size / 1e9);
                    if link == "input" {
                        job_deps.push(transmission);
                    }
                }
                let task = Device {
                    number_of_cores: 1,
                    cpu_frequency: cpu_frequency / 1_000.,
                };
                let index = graph.add_node(task);
                indexes.insert(id.to_string(), index);
                edges.insert(id.to_string(), job_deps);
            }
            "child" => {
                let child = node.attribute("ref").expect("Parent ref not found");
                let child_index = indexes.get(child).expect("Parent not found");

                let parents = node.children().filter(|n| n.is_element());
                let dependencies = edges.get(child).expect("Parent dependencies not found");
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
