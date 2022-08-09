use std::time::Instant;
use itertools::assert_equal;
use log::info;
use once_cell::sync::Lazy;
use trailscout_lib::data::graph::Graph;

mod common;

static GRAPH: Lazy<Graph> = Lazy::new(|| Graph::parse_from_file(common::PATH.1).unwrap());


#[test]
fn test_parsing_process_to_produce_graph_with_proper_number_of_elements() {
    common::initialize_logger();
    common::parse_pbf_to_fmi_file();
    info!("Creating graph");
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Finished creating graph with {} nodes, {} sights and {} edges", graph.num_nodes, graph.num_sights, graph.num_edges);
    if common::PATH.0.contains("bremen") {
        assert_eq!(graph.num_nodes, 1565544, "nodes");
        assert_eq!(graph.num_sights, 3014, "sights");
        assert_eq!(graph.num_edges, 3352214, "edges");
    } else if common::PATH.0.contains("stg") {
        assert_eq!(graph.num_nodes, 22175, "nodes");
        assert_eq!(graph.num_sights, 356, "sights");
        assert_eq!(graph.num_edges, 44836, "edges");
    }
}

#[test]
fn test_graph_connection() {
    // common::initialize_logger();
    // common::check_if_fmi_file_exists_and_parse_if_not();
    // info!("Creating graph");
    // let graph: &Lazy<Graph> = &GRAPH;
    // info!("Finished creating graph with {} nodes, {} sights and {} edges", graph.num_nodes, graph.num_sights, graph.num_edges);
    //
    // let mut visit_result: Vec<bool> = Vec::new();
    // visit_result.resize(graph.num_nodes, false);
    //
    // for (id, node) in graph.nodes().iter().enumerate() {
    //     if !visit_result[id] {
    //         let mut next_nodes: Vec<usize> = Vec::new();
    //         next_nodes.push(id);
    //         visit_result[id] = true;
    //         let mut num_visited_nodes = 1;
    //         while !next_nodes.is_empty() {
    //             let n_id = next_nodes.pop().unwrap();
    //             let outgoing_edges = graph.get_outgoing_edges(n_id);
    //             for edge in outgoing_edges {
    //                 if !visit_result[edge.tgt] {
    //                     next_nodes.push(edge.tgt);
    //                     visit_result[edge.tgt] = true;
    //                     num_visited_nodes += 1;
    //                 }
    //             }
    //         }
    //
    //         for n in graph.nodes() {
    //
    //         }
    //         info!("Der Teilgraph besteht aus {} Knoten von insgesamt {}", num_visited_nodes, graph.num_nodes);
    //     }
    // }

}

#[test]
fn test_sights_have_at_least_one_outgoing_edge () {
    common::initialize_logger();
    common::check_if_fmi_file_exists_and_parse_if_not();
    info!("Creating graph"); 
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Finished creating graph with {} nodes, {} sights and {} edges", graph.num_nodes, graph.num_sights, graph.num_edges);
        
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
    common::initialize_logger();
    common::check_if_fmi_file_exists_and_parse_if_not();
    info!("Creating graph"); 
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Finished creating graph with {} nodes, {} sights and {} edges", graph.num_nodes, graph.num_sights, graph.num_edges);
        
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
    common::initialize_logger();
    common::check_if_fmi_file_exists_and_parse_if_not();
    info!("Creating graph"); 
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Finished creating graph with {} nodes, {} sights and {} edges", graph.num_nodes, graph.num_sights, graph.num_edges);

    let mut onesided_edges = 0;
    for edge in &graph.edges {
        onesided_edges += 1;
        let n_edges = graph.get_outgoing_edges(edge.tgt);
        for n_edge in n_edges {
            if n_edge.tgt == edge.src {
                onesided_edges -= 1;
                break;
            }
        }
    }
    assert_eq!(onesided_edges, 0, "Onesided edges: {} of a total of {} edges", onesided_edges, graph.edges.len());
}

#[test]
fn get_sights_with_radius_1000_meters() {
    common::initialize_logger();
    common::check_if_fmi_file_exists_and_parse_if_not();
    info!("Creating graph"); 
    let graph: &Lazy<Graph> = &GRAPH;
    info!("Finished creating graph with {} nodes, {} sights and {} edges", graph.num_nodes, graph.num_sights, graph.num_edges);

    if common::PATH.0.contains("bremen") {
        //when you google "bremen lat long" then 53.0793° N, 8.8017° E is the result
        let sights_bremen_1000 = graph.get_sights_in_area(53.0793, 8.8017, 1000.0);
        assert_eq!(sights_bremen_1000.len(), 452, "Bremen doesn't have the correct number of sights");
    } else if common::PATH.0.contains("stg") {
        //when you google "stuttgart lat long" then 48.7758° N, 9.1829° E is the result
        let sights_stg_1000 = graph.get_sights_in_area(48.7758, 9.1829, 1000.0);
        assert_eq!(sights_stg_1000.len(), 350, "Stuttgart doesn't have the correct number of sights");
    }
}