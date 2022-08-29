use std::env;
use std::time::Instant;
use itertools::Itertools;
use pathfinding::prelude::dijkstra_all;
use rand::{Rng, thread_rng};
use trailscout_lib::data::graph::Graph;
use trailscout_lib::init_logging;
use trailscout_lib::utils::dijkstra;

fn bench_dijkstra(graph_file: &str, iter_warmup: usize, iter_measure: usize) {
    let graph = Graph::parse_from_file(graph_file)
        .expect("Failed to parse graph file");

    log::info!("Benchmarking dijkstra implementations on graph {graph_file} with \
        {iter_warmup} warm up and {iter_measure} measured iterations");

    let mut own_res = vec![0; iter_measure];
    let mut pathfinding_res = vec![0; iter_measure];

    let do_iteration = || {
        let mut rng = thread_rng();
        let src_id = rng.gen_range(0..graph.num_nodes);

        let start = Instant::now();
        dijkstra::run_ota_dijkstra(&graph, src_id);
        let elapsed = start.elapsed().as_millis();

        let successors = |node_id: usize|
            graph.get_outgoing_edges(node_id)
                .into_iter()
                .map(|edge| (edge.tgt, edge.dist))
                .collect::<Vec<(usize, usize)>>();
        let start = Instant::now();
        dijkstra_all(&src_id,
                     |&node_id| successors(node_id));
        let pathfinding_elapsed = start.elapsed().as_millis();

        (elapsed, pathfinding_elapsed)
    };

    // first iter_warmup rounds system warm up
    for i in 0..iter_warmup {
        do_iteration();
        log::trace!("Finished {} of {} warmup rounds", i + 1, iter_warmup);
    }

    for i in 0..iter_measure {
        let (elapsed, pathfinding_elapsed) = do_iteration();
        own_res[i] = elapsed;
        pathfinding_res[i] = pathfinding_elapsed;
        log::trace!("Finished {} of {} measured rounds", i + 1, iter_measure);
    }

    let avg = own_res.iter().sum::<u128>() / iter_measure as u128;
    let pathfinding_avg = pathfinding_res.iter().sum::<u128>() / iter_measure as u128;
    log::info!("Own dijkstra impl. average run time: {} ms", avg);
    log::info!("pathfinding crate's dijkstra impl. average run time: {} ms", pathfinding_avg);
}

fn main() {
    init_logging();

    let args = env::args().collect_vec();

    let bench = args[1].as_str();
    match bench {
        "dijkstra" => {
            let graph_file = args[2].as_str();
            let iter_warmup: usize = args[3].parse().unwrap();
            let iter_measure: usize = args[4].parse().unwrap();
            bench_dijkstra(graph_file, iter_warmup, iter_measure)
        }
        _ => ()
    }
}