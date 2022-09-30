extern crate core;

use std::env;
use std::time::Instant;
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use pathfinding::prelude::dijkstra_all;
use rand::{Rng, SeedableRng, rngs::StdRng};
use trailscout_lib::algorithm::{Algorithm, Area, SightCategoryPref, UserPreferences};
use trailscout_lib::data::graph::{Category, Graph};
use trailscout_lib::init_logging;
use trailscout_lib::utils::dijkstra;

/// Benchmarks a dijkstra implementation. Either the Trailscout implementation
/// (`dijkstra == "self"`) or the one of the `pathfinding` crate (`dijkstra == "pathfinding"`).
fn bench_dijkstra(seed: u64, graph_file: &str, dijkstra: &str, iter_warmup: usize,
                  iter_measure: usize) {
    let graph = Graph::parse_from_file(graph_file)
        .expect("Failed to parse graph file");

    log::info!("Benchmarking dijkstra implementation {dijkstra} \n\
        on graph {graph_file} \n\
        with seed {seed}, \n\
        {iter_warmup} warm up and \n\
        {iter_measure} measured iterations");

    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
    let mut do_iteration = || {
        let src_id = rng.gen_range(0..graph.num_nodes);

        match dijkstra {
            "self" => {
                // Trailscout one-to-all dijkstra
                let start = Instant::now();
                dijkstra::run_ota_dijkstra(&graph, src_id);
                start
            }
            "pathfinding" => {
                // pathfinding one-to-all dijkstra
                let successors = |node_id: usize|
                    graph.get_outgoing_edges(node_id)
                        .into_iter()
                        .map(|edge| (edge.tgt, edge.dist))
                        .collect::<Vec<(usize, usize)>>();
                let start = Instant::now();
                dijkstra_all(&src_id,
                             |&node_id| successors(node_id));
                start
            }
            _ => panic!("Unknown dijkstra impl.")
        }.elapsed().as_millis()
    };

    // first iter_warmup rounds system warm up
    for i in 0..iter_warmup {
        do_iteration();
        log::trace!("Finished {} of {} warmup rounds", i + 1, iter_warmup);
    }

    // iter_measure measured rounds
    let mut measurements = vec![0; iter_measure];
    for i in 0..iter_measure {
        let elapsed = do_iteration();
        measurements[i] = elapsed;
        log::trace!("Finished {} of {} measured rounds", i + 1, iter_measure);
    }

    let avg = measurements.iter().sum::<u128>() / iter_measure as u128;
    log::info!("Average run time: {avg} ms");
}

/// Benchmarks the score and runtime of given algorithm under the given parameters
fn bench_algo(graph_file: &str, algo_name: &str, iter_warmup: usize, iter_measure: usize,
              radius: f64, walking_time: i64, category_prefs: Vec<SightCategoryPref>) {
    let graph = Graph::parse_from_file(graph_file)
        .expect("Failed to parse graph file");

    log::info!("Benchmarking {algo_name} algorithm \n\
        on graph {graph_file} \n\
        with {iter_warmup} warm up iterations, \n\
        {iter_measure} measured iterations, \n\
        radius {radius} m, \n\
        walking time {walking_time} h and \n\
        category preferences {:?}", &category_prefs);

    let start_time = DateTime::parse_from_rfc3339("2022-07-01T14:00:00+01:00")
        .unwrap().with_timezone(&Utc);
    let end_time = start_time + Duration::hours(walking_time);
    let area = Area::from_coords_and_radius(48.777226, 9.173895, radius);
    let user_prefs = UserPreferences::from_category_and_sight_prefs(
        category_prefs, vec![]);

    let do_iteration = || {
        let start = Instant::now();
        let algo = Algorithm::from_name(
            algo_name, &graph, start_time, end_time, 5.0 / 3.6,
            area.clone(), user_prefs.clone())
            .expect("Unknown algorithm");
        let route = algo.compute_route().expect("Error during route computation");
        let elapsed = start.elapsed().as_millis();

        (algo.get_collected_score(&route), elapsed)
    };

    for i in 0..iter_warmup {
        do_iteration();
        log::trace!("Finished {} of {} warmup rounds", i + 1, iter_warmup);
    }

    let mut measure_score = vec![0; iter_measure];
    let mut measure_time = vec![0; iter_measure];
    for i in 0..iter_measure {
        let (score, elapsed) = do_iteration();
        measure_score[i] = score;
        measure_time[i] = elapsed;
        log::trace!("Finished {} of {} measured rounds", i + 1, iter_measure);
    }

    let avg_score = measure_score.iter().sum::<usize>() / iter_measure;
    let avg_time = measure_time.iter().sum::<u128>() / iter_measure as u128;
    log::info!("Average collected score: {avg_score}");
    log::info!("Average run time: {avg_time} ms");
}

/// Benchmarks an implementation to compute the nearest node to a geo location.
/// Either the naive implementation (`which_impl == "naive"`) or the efficient one
/// (`which_impl == "eff"`).
fn bench_nearest_node(seed: u64, graph_file: &str, which_impl: &str, iter_warmup: usize,
                      iter_measure: usize) {
    let graph = Graph::parse_from_file(graph_file)
        .expect("Failed to parse graph file");

    log::info!("Benchmarking implementation to compute the nearest node to a geo location \
        {which_impl} \n\
        on graph {graph_file} \n\
        with seed {seed}, \n\
        {iter_warmup} warm up and \n\
        {iter_measure} measured iterations");

    let mut rng: StdRng = SeedableRng::seed_from_u64(seed);
    let mut do_iteration = || {
        let rand_id = rng.gen_range(0..graph.num_nodes);
        let rand_node = graph.get_node(rand_id);
        let pos_lat = rand_node.lat + 0.000001;
        let pos_lon = rand_node.lon - 0.000001;

        match which_impl {
            "naive" => {
                // Naive way
                let start = Instant::now();
                graph.get_nearest_node_naive(pos_lat, pos_lon);
                start
            }
            "eff" => {
                // Efficient way
                let start = Instant::now();
                graph.get_nearest_node(pos_lat, pos_lon);
                start
            }
            _ => panic!("Unknown impl.")
        }.elapsed().as_micros()
    };

    for i in 0..iter_warmup {
        do_iteration();
        log::trace!("Finished {} of {} warmup rounds", i + 1, iter_warmup);
    }

    let mut measurements = vec![0; iter_measure];
    for i in 0..iter_measure {
        measurements[i] = do_iteration();
        log::trace!("Finished {} of {} measured rounds", i + 1, iter_measure);
    }

    let avg = measurements.iter().sum::<u128>() / iter_measure as u128;
    log::info!("Average run time: {avg} µs");
}

fn main() {
    init_logging();
    log::info!("Average run time:");

    let args = env::args().collect_vec();

    let bench = args[1].as_str();
    match bench {
        "dijkstra" => {
            let seed: u64 = args[2].parse().unwrap();
            let graph_file = args[3].as_str();
            let dijkstra = args[4].as_str();
            let iter_warmup: usize = args[5].parse().unwrap();
            let iter_measure: usize = args[6].parse().unwrap();
            bench_dijkstra(seed, graph_file, dijkstra, iter_warmup, iter_measure);
        }
        "algo" => {
            let graph_file = args[2].as_str();
            let algo_name = args[3].as_str();
            let iter_warmup: usize = args[4].parse().unwrap();
            let iter_measure: usize = args[5].parse().unwrap();
            let radius: f64 = args[6].parse().unwrap();
            let walking_time: i64 = args[7].parse().unwrap();
            let mut category_prefs = Vec::with_capacity(
                (args.len() - 8) / 2);
            for i in (8..args.len()).step_by(2) {
                let category: Category = args[i].parse().unwrap();
                let pref: usize = args[i+1].parse().unwrap();
                category_prefs.push(SightCategoryPref::new(category, pref));
            }
            bench_algo(graph_file, algo_name, iter_warmup, iter_measure, radius, walking_time,
                       category_prefs);
        }
        "nn" => {
            let seed: u64 = args[2].parse().unwrap();
            let graph_file = args[3].as_str();
            let which_impl = args[4].as_str();
            let iter_warmup: usize = args[5].parse().unwrap();
            let iter_measure: usize = args[6].parse().unwrap();
            bench_nearest_node(seed, graph_file, which_impl, iter_warmup, iter_measure);
        }
        "load" => {
            let path = args[2].as_str();
            log::info!("Starting to parsed graph from: {}", &path);
            let graph = Graph::parse_from_file(&path).expect("Error parsing graph from file");
            log::info!("Parsed graph from: {} with {} nodes", &path, &graph.num_nodes);
        }
        _ => ()
    }
}