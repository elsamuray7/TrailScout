use env_logger::Env;
use log::info;
use trailscout_lib::data::{osm_graph_creator::{parse_osm_data, write_graph_file}, graph::{Sight, Edge, Node, Graph}};

pub fn parse_pbf_to_fmi_file() {
        initializeLogger();
        info!("starting test setup");
        info!("current working dir: {}",std::env::current_dir().unwrap().to_str().unwrap());
        let in_graph = "./osm_graphs/bremen-latest.osm.pbf";
        let out_graph = "./tests_data/output/test-bremen-latest.fmi";
        let mut nodes : Vec<Node> = Vec::new();
        let mut edges : Vec<Edge> = Vec::new();
        let mut sights : Vec<Sight> = Vec::new();
        parse_osm_data(in_graph, &mut nodes, &mut edges, &mut sights);
        write_graph_file( out_graph, &mut nodes, &mut edges, &mut sights);
}

fn initializeLogger() {
    //initializing the logger
    let env = Env::default()
    .filter_or("TRAILSCOUT_LOG_LEVEL", "trace")
    .write_style_or("TRAILSCOUT_LOG_STYLE", "always");
    env_logger::init_from_env(env);
}