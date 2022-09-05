use log::info;
use once_cell::sync::Lazy;
use crate::{data::{osm_graph_creator::parse_and_write_osm_data, graph::Graph}};

pub const GRAPH_PATH:(&str, &str) = ("./tests_data/bremen-latest.osm.pbf", "./tests_data/output/test-bremen-latest.fmibin");
//pub const GRAPH_PATH:(&str, &str) = ("./tests_data/stgcenter.pbf", "./tests_data/output/test-stgcenter.fmibin");

pub static GRAPH: Lazy<Graph> = Lazy::new(|| {
    parse_pbf_to_fmi_file();
    Graph::parse_from_file(GRAPH_PATH.1).unwrap()
});

fn parse_pbf_to_fmi_file() {
        info!("starting test setup");
        info!("current working dir: {}",std::env::current_dir().unwrap().to_str().unwrap());
        let in_graph = GRAPH_PATH.0;
        let out_graph = GRAPH_PATH.1;
        parse_and_write_osm_data(in_graph, out_graph).ok();
}