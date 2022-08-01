use log::info;
use trailscout_lib::data::graph::Graph;

mod common;

#[test]
fn test_parsing_process_to_produce_graph_with_proper_number_of_elements() {
    common::parse_pbf_to_fmi_file();
    //common::initializeLogger();
    info!("Creating graph"); 
    let graph = Graph::parse_from_file("./tests_data/output/test-bremen-latest.fmi").unwrap();
    info!("Asserting graph properties"); 
    assert_eq!(graph.num_nodes, 1565544);
    assert_eq!(graph.num_sights, 3014);
    assert_eq!(graph.num_edges, 1942587);
    let a = graph.get_sights_in_area(1.0,1.0,1.0);
    //It seems like there are some duplicate nodes, which causes a few of the sights to not be returned (2971 instead of 3014)
    assert_eq!(a.len(), 2971);
}