use crate::computing_time::computing_time;
use crate::matching::Matching;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use para_graph::algorithms::floyd_warshall::floyd_warshall_par_gpu;
use para_graph::graph::adj_matrix;
use para_graph::model::{Dependency, Device, Task, Transmission};
use petgraph::algo::toposort;
use petgraph::prelude::*;
use petgraph::visit::IntoNodeIdentifiers;

pub fn buyya_par_gpu(
    topology: &UnGraph<Device, Transmission>,
    tasks: &DiGraph<Task, Dependency>,
) -> Vec<Matching> {
    let topo = toposort(&tasks, None).unwrap();
    let mut assignments: Vec<Matching> = vec![Matching::default(); tasks.node_count()];
    let mut delay: Vec<f64> = vec![0.; topology.node_count()];

    let dist = floyd_warshall(topology);

    for u in topo {
        let task = tasks[u];

        let deps = tasks.edges_directed(u, Incoming).collect_vec();
        let calc_delay = |mu: NodeIndex| {
            let max_t_up = deps
                .iter()
                .map(|e| {
                    let v = e.source();
                    let mv = assignments[v.index()].node;
                    let t_up = dist[mu.index()][mv] * e.weight().data_size as f64;
                    let delay_v = delay[mv];
                    delay_v + t_up
                })
                .max_by_key(|f| OrderedFloat(*f))
                .unwrap_or_default();
            let t_mu = delay[mu.index()];
            let t_delay = max_t_up.max(t_mu);

            let device = topology[mu];
            let t = computing_time(&device, &task);
            t_delay + t
        };

        let assign_node = match task.pin {
            None => topology
                .node_identifiers()
                .min_by(|a, b| calc_delay(*a).total_cmp(&calc_delay(*b)))
                .unwrap(),
            Some(node) => node,
        };

        let finish_time = calc_delay(assign_node);
        delay[assign_node.index()] = finish_time;
        assignments[u.index()] = Matching {
            node: assign_node.index(),
            finish_time,
        };
    }

    assignments
}

fn floyd_warshall(graph: &UnGraph<Device, Transmission>) -> Vec<Vec<f64>> {
    let n = graph.node_count();
    let graph = graph.map(|_, n| n, |_, e| 1. / (e.transmission_rate * 1_000_000_000.));
    let mat = adj_matrix::get_adj_matrix(&graph);
    floyd_warshall_par_gpu(n, &mat)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::{make_tasks_graph, make_topology_graph};

    #[test]
    fn test_buyya() {
        let tasks = make_tasks_graph();
        let topo = make_topology_graph();
        let assignments = buyya_par_gpu(&topo, &tasks);

        let node_ids = assignments.iter().map(|m| m.node).collect_vec();
        let expected = vec![0, 1, 1, 4, 4, 5, 0, 0, 0, 3, 1];
        assert_eq!(node_ids, expected);

        let finish_times = assignments.iter().map(|m| m.finish_time).collect_vec();
        let expected = vec![
            0.0158125,
            13.557479166666665,
            16.213729166666667,
            18.481586307255952,
            20.295202378684525,
            20.695202378684524,
            0.0,
            0.0158125,
            0.01,
            0.013541666666666667,
            0.0153125,
        ];
        assert_eq!(finish_times, expected);
    }
}
