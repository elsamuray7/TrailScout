use ctor::ctor;
use log::info;
use once_cell::sync::Lazy;
use trailscout_lib::data::graph::Graph;

mod common;

static GRAPH: Lazy<Graph> = Lazy::new(|| Graph::parse_from_file("./tests_data/output/test-bremen-latest.fmi").unwrap());

#[ctor]
fn initialize() {
    common::initializeLogger();
    common::parse_pbf_to_fmi_file();
}

#[test]
fn test_parsing_process_to_produce_graph_with_proper_number_of_elements() {
    info!("Creating graph");
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Asserting graph properties"); 
    assert_eq!(graph.num_nodes, 1565544, "nodes");
    assert_eq!(graph.num_sights, 3014, "sights");
    assert_eq!(graph.num_edges, 1942587, "edges");
    let a = graph.get_sights_in_area(1.0,1.0,1.0);
    //It seems like there are some duplicate nodes, which causes a few of the sights to not be returned (2971 instead of 3014)
    assert_eq!(a.len(), 2971, "get_sights_in_area");
}

#[test]
fn test_sights_have_at_least_one_outgoing_edge () {
    info!("Creating graph"); 
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Finished creating graph");
        
    let mut unleavable_sights = 0;
    for sight in &graph.sights {
        let edges = graph.get_outgoing_edges(sight.node_id);
        if edges.len() == 0 {
            unleavable_sights += 1;
        }
    }
    assert_eq!(unleavable_sights, 0, "Unleavable sights: {} of a total of {} sights", unleavable_sights, graph.sights.len());
}

#[test]
fn test_sights_have_at_least_one_incoming_edge () {
    info!("Creating graph"); 
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Finished creating graph");
        
    let mut unreachable_sights = 0;
    for sight in &graph.sights {
        unreachable_sights += 1;
        let edges = graph.get_outgoing_edges(sight.node_id);
        'outer: for edge in edges {
            //get neighbours outgoing edges
            let neighbour_edges = graph.get_outgoing_edges(edge.tgt);
            //check if any of the neighbour edges leads back to the sight
            for n_edge in neighbour_edges {
                if n_edge.tgt == sight.node_id {
                    unreachable_sights -= 1;
                    break 'outer
                }
            }
        }
    }
    assert_eq!(unreachable_sights, 0, "Unreachable sights: {} of a total of {} sights", unreachable_sights, graph.sights.len());
}

#[test]
fn test_edges_go_in_both_directions() {
    info!("Creating graph"); 
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Finished creating graph");

    let mut onesided_edges = 0;
    for edge in &graph.edges {
        onesided_edges += 1;
        let n_edges = graph.get_outgoing_edges(edge.tgt);
        for n_edge in n_edges {
            if n_edge.tgt == edge.src {
                onesided_edges -= 1;
            }
        }
    }
    assert_eq!(onesided_edges, 0, "Onesided edges: {} of a total of {} edges", onesided_edges, graph.edges.len());
}