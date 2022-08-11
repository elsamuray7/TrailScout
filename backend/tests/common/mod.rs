use std::path::Path;

use log::info;
use trailscout_lib::data::{osm_graph_creator::{parse_osm_data, write_graph_file}, graph::{Sight, Edge, Node}};

//pub const PATH:(&str, &str) = ("./tests_data/bremen-latest.osm.pbf", "./tests_data/output/test-bremen-latest.fmi");
pub const PATH:(&str, &str) = ("./tests_data/stgcenter.pbf", "./tests_data/output/test-stgcenter.fmi");


pub fn parse_pbf_to_fmi_file() {
        info!("starting test setup");
        info!("current working dir: {}",std::env::current_dir().unwrap().to_str().unwrap());
        let in_graph = PATH.0;
        let out_graph = PATH.1;
        let mut nodes : Vec<Node> = Vec::new();
        let mut edges : Vec<Edge> = Vec::new();
        let mut sights : Vec<Sight> = Vec::new();
        parse_osm_data(in_graph, &mut nodes, &mut edges, &mut sights);
        write_graph_file( out_graph, &mut nodes, &mut edges, &mut sights);
}

pub fn check_if_fmi_file_exists_and_parse_if_not() {
    if Path::new(PATH.1).exists() {
        info!("Found fmi file, parsing skipped.");
    } else {
        parse_pbf_to_fmi_file();
    }
}

pub fn initialize_logger() {
    trailscout_lib::init_logging();
}