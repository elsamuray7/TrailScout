use std::io;
use std::time::Instant;
use env_logger::Env;
use std::env;
use trailscout_lib::data::graph::{Edge, Sight, Node, Graph};
use trailscout_lib::data::osm_graph_creator::{create_fmi_graph, parse_osm_data, write_graph_file};
use clap::Parser;
#[macro_use]
extern crate log;

pub fn main() -> Result<(), io::Error> {
    //initializing the logger
    let env_logger = Env::default()
    .filter_or("TRAILSCOUT_LOG_LEVEL", "trace")
    .write_style_or("TRAILSCOUT_LOG_STYLE", "always");
    env_logger::init_from_env(env_logger);
    info!("starting up");

    let args: Vec<String> = env::args().collect();

    println!("Input file is {}.", &args[1]);
    println!("Output file is {}.", &args[2]);

    //let in_graph = "./osm_graphs/bremen-latest.osm.pbf";
    //let out_graph = "./osm_graphs/bremen-latest.fmi";

    let in_graph = &args[1];
    let out_graph = &args[2];
    create_fmi_graph(in_graph,out_graph)
}