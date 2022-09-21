use std::env;
use std::time::Instant;
use chrono::{DateTime, Duration, Utc};
use itertools::Itertools;
use pathfinding::prelude::dijkstra_all;
use rand::{Rng, thread_rng};
use trailscout_lib::algorithm::{Algorithm, Area, SightCategoryPref, UserPreferences};
use trailscout_lib::algorithm::greedy::GreedyAlgorithm;
use trailscout_lib::algorithm::sa_lin_yu::SimAnnealingLinYu;
use trailscout_lib::data::graph::{Category, Graph};
use trailscout_lib::init_logging;
use trailscout_lib::utils::dijkstra;

fn bench_dijkstra(graph_file: &str, iter_warmup: usize, iter_measure: usize) {
    let graph = Graph::parse_from_file(graph_file)
        .expect("Failed to parse graph file");

    log::info!("Benchmarking dijkstra implementations on graph {graph_file} with \
        {iter_warmup} warm up and {iter_measure} measured iterations");

    let mut own_res = vec![0; iter_measure];
    let mut pathfinding_res = vec![0; iter_measure];

    let mut rng = thread_rng();
    let mut do_iteration = || {
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

fn bench_algo(graph_file: &str, algo_name: &str, iter: usize, radius: f64, walking_time: i64) {
    let graph = Graph::parse_from_file(graph_file)
        .expect("Failed to parse graph file");

    log::info!("Benchmarking {algo_name} on graph {graph_file} with {iter} iterations, \
                radius {radius} m and walking_time {walking_time} h");

    let start_time = DateTime::parse_from_rfc3339("2022-07-01T14:00:00+01:00")
        .unwrap().with_timezone(&Utc);
    let end_time = start_time + Duration::hours(walking_time);
    let area = Area::from_coords_and_radius(48.777226, 9.173895, radius);
    let category_prefs = vec![
        SightCategoryPref::new(Category::MuseumExhibition, 5),
        SightCategoryPref::new(Category::Activities, 4),
        SightCategoryPref::new(Category::Nightlife, 3),
        SightCategoryPref::new(Category::Restaurants, 1),
    ];
    let user_prefs = UserPreferences::from_category_and_sight_prefs(
        category_prefs, vec![]);
    let algo = Algorithm::from_name(
        algo_name, &graph, start_time, end_time, 5.0 / 3.6, area, user_prefs)
        .expect("Unknown algorithm");

    let mut measurements = vec![0; iter];
    for i in 0..iter {
        let route = algo.compute_route().expect("Error during route computation");
        measurements[i] = algo.get_collected_score(&route);
    }
    let avg = measurements.iter().sum::<usize>() / iter;
    log::info!("Average collected score: {}", avg);
}

fn bench_sa_vs_greedy(graph_file: &str, settings: usize, iter: usize, radius: f64, walking_time: i64) {
    let graph = Graph::parse_from_file(graph_file)
        .expect("Failed to parse graph file");

    log::info!("Benchmarking Sim. Annealing vs. Greedy on graph {graph_file} with {iter} iterations, \
                radius {radius} m and walking_time {walking_time} h");

    let start_time = DateTime::parse_from_rfc3339("2022-07-01T14:00:00+01:00")
        .unwrap().with_timezone(&Utc);
    let end_time = start_time + Duration::hours(walking_time);
    let area = Area::from_coords_and_radius(48.777226, 9.173895, radius);
    let category_prefs = match settings {
        1 => vec![
            SightCategoryPref::new(Category::MuseumExhibition, 5),
            SightCategoryPref::new(Category::Activities, 4),
            SightCategoryPref::new(Category::Sightseeing, 3),
            SightCategoryPref::new(Category::Restaurants, 1)
        ],
        2 => vec![
            SightCategoryPref::new(Category::Activities, 5),
            SightCategoryPref::new(Category::Nightlife, 4),
            SightCategoryPref::new(Category::Sightseeing, 2)
        ],
        _ => vec![
            SightCategoryPref::new(Category::Activities, 5),
            SightCategoryPref::new(Category::Nightlife, 1),
        ]
    };
    let user_prefs = UserPreferences::from_category_and_sight_prefs(
        category_prefs, vec![]);

    let mut measure_score_greedy = vec![0; iter];
    let mut measure_score_sa = vec![0; iter];
    let mut measure_runt_greedy = vec![0; iter];
    let mut measure_runt_sa = vec![0; iter];
    for i in 0..iter {
        let start = Instant::now();
        let greedy = Algorithm::from_name(
            GreedyAlgorithm::ALGORITHM_NAME, &graph, start_time, end_time,
            5.0 / 3.6, area.clone(), user_prefs.clone()).unwrap();
        let route = greedy.compute_route().expect("Error during route computation");
        let elapsed = start.elapsed().as_millis();
        measure_score_greedy[i] = greedy.get_collected_score(&route);
        measure_runt_greedy[i] = elapsed;

        let start = Instant::now();
        let sa = Algorithm::from_name(
            SimAnnealingLinYu::ALGORITHM_NAME, &graph, start_time, end_time,
            5.0 / 3.6, area.clone(), user_prefs.clone()).unwrap();
        let route = sa.compute_route().expect("Error during route computation");
        let elapsed = start.elapsed().as_millis();
        measure_score_sa[i] = sa.get_collected_score(&route);
        measure_runt_sa[i] = elapsed;
    }

    let avg_score_greedy = measure_score_greedy.iter().sum::<usize>() / iter;
    let avg_score_sa = measure_score_sa.iter().sum::<usize>() / iter;
    let avg_runt_greedy = measure_runt_greedy.iter().sum::<u128>() / iter as u128;
    let avg_runt_sa = measure_runt_sa.iter().sum::<u128>() / iter as u128;
    log::info!("Greedy avg. score: {avg_score_greedy}, runtime: {avg_runt_greedy} ms");
    log::info!("Sim. Annealing avg. score: {avg_score_sa}, runtime: {avg_runt_sa} ms");
}

fn bench_nearest_node(graph_file: &str, iter_warmup: usize, iter_measure: usize) {
    let graph = Graph::parse_from_file(graph_file)
        .expect("Failed to parse graph file");

    log::info!("Benchmarking implementations to compute the nearest node to a position on graph \
        {graph_file} with {iter_warmup} warm up and {iter_measure} measured iterations");

    let mut measure_naive = vec![0; iter_measure];
    let mut measure_eff = vec![0; iter_measure];

    let mut rng = thread_rng();
    let mut do_iteration = || {
        let rand_id = rng.gen_range(0..graph.num_nodes);
        let rand_node = graph.get_node(rand_id);
        let pos_lat = rand_node.lat + 0.000001;
        let pos_lon = rand_node.lon - 0.000001;

        let start = Instant::now();
        graph.get_nearest_node_naive(pos_lat, pos_lon);
        let elapsed_naive = start.elapsed().as_micros();

        let start = Instant::now();
        graph.get_nearest_node(pos_lat, pos_lon);
        let elapsed_eff = start.elapsed().as_micros();

        (elapsed_naive, elapsed_eff)
    };

    for _ in 0..iter_warmup {
        do_iteration();
    }
    for i in 0..iter_measure {
        let (naive, eff) = do_iteration();
        measure_naive[i] = naive;
        measure_eff[i] = eff;
    }

    let avg_naive: u128 = measure_naive.iter().sum::<u128>() / iter_measure as u128;
    let avg_eff: u128 = measure_eff.iter().sum::<u128>() / iter_measure as u128;
    log::info!("Avg. naive: {avg_naive} µs vs. avg eff.: {avg_eff} µs");
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
            bench_dijkstra(graph_file, iter_warmup, iter_measure);
        }
        "algo" => {
            let graph_file = args[2].as_str();
            let algo_name = args[3].as_str();
            let iter: usize = args[4].parse().unwrap();
            let radius: f64 = args[5].parse().unwrap();
            let walking_time: i64 = args[6].parse().unwrap();
            bench_algo(graph_file, algo_name, iter, radius, walking_time);
        }
        "sa_vs_greedy" => {
            let graph_file = args[2].as_str();
            let settings: usize = args[3].parse().unwrap();
            let iter: usize = args[4].parse().unwrap();
            let radius: f64 = args[5].parse().unwrap();
            let walking_time: i64 = args[6].parse().unwrap();
            bench_sa_vs_greedy(graph_file, settings, iter, radius, walking_time);
        }
        "nn" => {
            let graph_file = args[2].as_str();
            let iter_warmup: usize = args[3].parse().unwrap();
            let iter_measure: usize = args[4].parse().unwrap();
            bench_nearest_node(graph_file, iter_warmup, iter_measure);
        }
        _ => ()
    }
}