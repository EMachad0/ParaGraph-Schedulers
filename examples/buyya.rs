use scheduler_benchmark::parser::tasks_parser::parse_task_xml;
use scheduler_benchmark::parser::topology_parser::parse_topology_xml;
use scheduler_benchmark::schedulers::buyya::buyya_par_cpu;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <task_xml> <topology_xml>", args[0]);
        std::process::exit(1);
    }

    let task_xml_file = &args[1];
    let task_xml = std::fs::read_to_string(task_xml_file).expect("Failed to read XML file");
    let task_graph = parse_task_xml(&task_xml);

    let topology_xml_file = &args[2];
    let topology_xml = std::fs::read_to_string(topology_xml_file).expect("Failed to read XML file");
    let topology_graph = parse_topology_xml(&topology_xml);

    println!(
        "Task graph: Nodes={} Edges={}",
        task_graph.node_count(),
        task_graph.edge_count()
    );
    println!(
        "Topology graph: Nodes={} Edges={}",
        topology_graph.node_count(),
        topology_graph.edge_count()
    );

    let matching = buyya_par_cpu(&topology_graph, &task_graph);
    matching.iter().enumerate().for_each(|(i, m)| {
        println!("{:2}: {}", i, m);
    });

    para_graph::graph::dot::to_dot("task", &task_graph);
    para_graph::graph::dot::to_dot("topology", &topology_graph);
}
