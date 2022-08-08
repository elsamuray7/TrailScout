use std::path::Path;

use env_logger::Env;
use log::info;
use trailscout_lib::data::{osm_graph_creator::{parse_osm_data, write_graph_file}, graph::{Sight, Edge, Node, Graph}};

pub fn parse_pbf_to_fmi_file() {
        info!("starting test setup");
        info!("current working dir: {}",std::env::current_dir().unwrap().to_str().unwrap());
        let in_graph = "./tests_data/bremen-latest.osm.pbf";
        let out_graph = "./tests_data/output/test-bremen-latest.fmi";
        let mut nodes : Vec<Node> = Vec::new();
        let mut edges : Vec<Edge> = Vec::new();
        let mut sights : Vec<Sight> = Vec::new();
        parse_osm_data(in_graph, &mut nodes, &mut edges, &mut sights);
        write_graph_file( out_graph, &mut nodes, &mut edges, &mut sights);
}

pub fn check_if_fmi_file_exists_and_parse_if_not() {
    if Path::new("./tests_data/output/test-bremen-latest.fmi").exists() {
        info!("Found fmi file, parsing skipped.");
    } else {
        parse_pbf_to_fmi_file();
    }
}

pub fn initialize_logger() {
    //initializing the logger
    let env = Env::default()
    .filter_or("TRAILSCOUT_LOG_LEVEL", "trace")
    .write_style_or("TRAILSCOUT_LOG_STYLE", "always");
    env_logger::try_init_from_env(env).ok();
}