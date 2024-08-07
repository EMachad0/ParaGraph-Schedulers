use crate::computing_time::computing_time;
use crate::matching::Matching;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use para_graph::graph::adj_matrix;
use para_graph::{
    algorithms::floyd_warshall::floyd_warshall_par_gpu,
    model::{Dependency, Device, Task, Transmission},
};
use petgraph::algo::toposort;
use petgraph::prelude::*;
use petgraph::visit::IntoNodeIdentifiers;
use rayon::prelude::ParallelSliceMut;

pub fn heft_par_gpu(
    topology: &UnGraph<Device, Transmission>,
    tasks: &DiGraph<Task, Dependency>,
) -> Vec<Matching> {
    let ranking = mean_ranking(topology, tasks, computing_time);
    let ranking = heft_prioritize(&ranking, topology, tasks);
    heft_assign(&ranking, topology, tasks)
}

fn mean_ranking(
    topology: &UnGraph<Device, Transmission>,
    tasks: &DiGraph<Task, Dependency>,
    rank_function: fn(&Device, &Task) -> f64,
) -> Vec<f64> {
    let indices = tasks.node_indices();
    indices
        .map(|u| {
            let task = tasks[u];
            let rank = topology
                .node_weights()
                .map(|d| rank_function(d, &task))
                .sum::<f64>()
                / topology.node_count() as f64;
            rank
        })
        .collect_vec()
}

fn heft_prioritize(
    ranking: &[f64],
    topology: &UnGraph<Device, Transmission>,
    tasks: &DiGraph<Task, Dependency>,
) -> Vec<f64> {
    let topo = toposort(&tasks, None).unwrap();
    let dist = floyd_warshall(topology);
    let mean_dist = dist
        .into_iter()
        .map(|v| v.into_iter().sum::<f64>())
        .sum::<f64>();
    let mut new_ranking = ranking.to_vec();
    topo.into_iter().rev().for_each(|u| {
        let w = tasks
            .edges(u)
            .map(|e| {
                let v = e.target();
                let avg_com_cost = mean_dist * e.weight().data_size as f64;
                new_ranking[v.index()] + avg_com_cost
            })
            .max_by_key(|v| OrderedFloat(*v))
            .unwrap_or_default();
        new_ranking[u.index()] += w;
    });
    new_ranking
}

fn heft_assign(
    ranking: &[f64],
    topology: &UnGraph<Device, Transmission>,
    tasks: &DiGraph<Task, Dependency>,
) -> Vec<Matching> {
    let mut assignments: Vec<Matching> = vec![Matching::default(); tasks.node_count()];
    let mut delay: Vec<f64> = vec![0.; topology.node_count()];

    let mut ranks_and_tasks = tasks
        .node_indices()
        .map(|u| (ranking[u.index()].round() as usize, u))
        .collect_vec();
    // radix_sort_par_cpu(&mut ranks_and_tasks);
    ranks_and_tasks.par_sort_by_key(|(r, _)| *r);
    let tasks_by_rank = ranks_and_tasks
        .into_iter()
        .rev()
        .map(|(_, u)| u)
        .collect_vec();

    let dist = floyd_warshall(topology);

    for u in tasks_by_rank {
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
    fn test_heft() {
        let tasks = make_tasks_graph();
        let topo = make_topology_graph();
        let assignments = heft_par_gpu(&topo, &tasks);

        let node_ids = assignments.iter().map(|m| m.node).collect_vec();
        let expected = vec![0, 1, 1, 4, 4, 5, 0, 0, 0, 6, 3];
        assert_eq!(node_ids, expected);

        let finish_times = assignments.iter().map(|m| m.finish_time).collect_vec();
        let expected = vec![
            0.0,
            13.541666666666666,
            16.197916666666664,
            18.46577380725595,
            20.279389878684523,
            20.67938987868452,
            0.0158125,
            0.0158125,
            0.01,
            0.013901785714285715,
            0.0153125,
        ];
        assert_eq!(finish_times, expected);
    }
}
