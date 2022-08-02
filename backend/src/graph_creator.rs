use std::io;
use std::time::Instant;
use env_logger::Env;
use trailscout_lib::data::graph::{Edge, Sight, Node, Graph};
use trailscout_lib::data::osm_graph_creator::{parse_osm_data, write_graph_file};
#[macro_use]
extern crate log;

fn main() -> Result<(), io::Error> {
    //initializing the logger
    let env = Env::default()
    .filter_or("TRAILSCOUT_LOG_LEVEL", "trace")
    .write_style_or("TRAILSCOUT_LOG_STYLE", "always");
    env_logger::init_from_env(env);
    info!("starting up");

    let in_graph = "./osm_graphs/bremen-latest.osm.pbf";
    let out_graph = "./osm_graphs/bremen-latest.fmi";
    let mut nodes : Vec<Node> = Vec::new();
    let mut edges : Vec<Edge> = Vec::new();
    let mut sights : Vec<Sight> = Vec::new();
    parse_osm_data(in_graph, &mut nodes, &mut edges, &mut sights);
    write_graph_file( out_graph, &mut nodes, &mut edges, &mut sights);

    info!("Start creating the graph from fmi file!");
    let time_start = Instant::now();

    let graph = Graph::parse_from_file(out_graph).unwrap();

    let time_duration = time_start.elapsed();
    info!("End graph creation after {} seconds!", time_duration.as_secs());

    println!("{}", graph.num_nodes);
    println!("{}", graph.num_sights);
    println!("{}", graph.num_edges);
    Ok(())
}