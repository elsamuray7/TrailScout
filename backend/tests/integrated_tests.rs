use log::info;
use trailscout_lib::data::graph::Graph;

mod common;

#[test]
fn test_parsing_process_to_produce_graph_with_proper_number_of_elements() {
    common::parse_pbf_to_fmi_file();
    info!("Creating graph"); 
    let graph = Graph::parse_from_file("./tests_data/output/test-bremen-latest.fmi").unwrap();
    info!("Asserting graph properties"); 
    assert_eq!(graph.num_nodes, 1565587);
    assert_eq!(graph.num_sights, 3014);
    assert_eq!(graph.num_edges, 1942587);
}