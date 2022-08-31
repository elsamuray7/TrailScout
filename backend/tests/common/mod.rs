use std::path::Path;

use log::info;
use trailscout_lib::data::osm_graph_creator::parse_and_write_osm_data;

pub const PATH:(&str, &str) = ("./tests_data/bremen-latest.osm.pbf", "./tests_data/output/test-bremen-latest.fmibin");
//pub const PATH:(&str, &str) = ("./tests_data/stgcenter.pbf", "./tests_data/output/test-stgcenter.fmibin");


pub fn parse_pbf_to_fmi_file() {
        info!("starting test setup");
        info!("current working dir: {}",std::env::current_dir().unwrap().to_str().unwrap());
        let in_graph = PATH.0;
        let out_graph = PATH.1;
        parse_and_write_osm_data(in_graph, out_graph);
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