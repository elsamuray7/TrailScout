use std::io;
use env_logger::Env;
use std::env;
use trailscout_lib::data::graph::Graph;
use trailscout_lib::data::osm_graph_creator::{parse_and_write_osm_data};
#[macro_use]
extern crate log;

pub fn main() -> Result<(), io::Error> {
    //initializing the logger
    let env_logger = Env::default()
    .filter_or("TRAILSCOUT_LOG_LEVEL", "debug")
    .write_style_or("TRAILSCOUT_LOG_STYLE", "always");
    env_logger::init_from_env(env_logger);
    info!("starting up");

    let in_graph = env::var("i").unwrap_or("./osm_graphs/bremen-latest.osm.pbf".to_string());
    let out_graph = env::var("o").unwrap_or("./osm_graphs/bremen-latest.fmibin".to_string());

    println!("Input file is {}.", &in_graph);
    println!("Output file is {}.", &out_graph);

    parse_and_write_osm_data(&in_graph, &out_graph); // TODO result never used

    let graph = Graph::parse_from_file(&out_graph).unwrap();

    info!("Nodes: {}", graph.num_nodes);
    info!("Sights: {}", graph.num_sights);
    info!("Edges: {}", graph.num_edges);

    Ok(())
}