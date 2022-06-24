use std::io;
use data_api::api::graph::{Edge, Sight, Node};
use data_api::api::osm_graph_creator::{parse_osm_data, write_graph_file};

fn main() -> Result<(), io::Error> {
    let in_graph = "./osm_graphs/bremen-latest.osm.pbf";
    let out_graph = "./osm_graphs/bremen-latest.fmi";
    let mut nodes : Vec<Node> = Vec::new();
    let mut edges : Vec<Edge> = Vec::new();
    let mut sights : Vec<Sight> = Vec::new();
    parse_osm_data(in_graph, &mut nodes, &mut edges, &mut sights);
    write_graph_file( out_graph, &mut nodes, &mut edges, &mut sights);

    let graph = data_api::api::graph::Graph::parse_from_file(out_graph).unwrap();

    println!("{}", graph.num_nodes);
    println!("{}", graph.num_sights);
    println!("{}", graph.num_edges);
    Ok(())
}