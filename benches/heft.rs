use criterion::{criterion_group, criterion_main, Criterion};
use scheduler_benchmark::parser::tasks_parser::parse_task_xml;
use scheduler_benchmark::parser::topology_parser::parse_topology_xml;
use scheduler_benchmark::schedulers::heft::{heft_par_cpu, heft_par_gpu, heft_serial};

const TASK_XML: &str = "xml/bin/task.xml";
const TOPOLOGY_XML: &str = "xml/bin/topology.xml";

fn bench_buyya(c: &mut Criterion) {
    let mut group = c.benchmark_group("Heft");
    // group.sampling_mode(criterion::SamplingMode::Flat);
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(60));

    let topology_xml = std::fs::read_to_string(TOPOLOGY_XML).expect("Failed to read XML file");
    let topology_graph = parse_topology_xml(&topology_xml);
    let task_xml = std::fs::read_to_string(TASK_XML).expect("Failed to read XML file");
    let task_graph = parse_task_xml(&task_xml);

    let input = (topology_graph, task_graph);

    group.bench_with_input("Serial", &input, |b, (topology_graph, task_graph)| {
        b.iter(|| heft_serial(topology_graph, task_graph))
    });
    group.bench_with_input("CPU", &input, |b, (topology_graph, task_graph)| {
        b.iter(|| heft_par_cpu(topology_graph, task_graph))
    });
    group.bench_with_input("GPU", &input, |b, (topology_graph, task_graph)| {
        b.iter(|| heft_par_gpu(topology_graph, task_graph))
    });
    group.finish();
}

criterion_group!(benches, bench_buyya);
criterion_main!(benches);
