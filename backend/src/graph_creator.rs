use std::io;
use trailscout_lib::data::graph::{Edge, Sight, Node, Graph};
use trailscout_lib::data::osm_graph_creator::{parse_osm_data, write_graph_file};

fn main() -> Result<(), io::Error> {
    let in_graph = "./osm_graphs/bremen-latest.osm.pbf";
    let out_graph = "./osm_graphs/bremen-latest.fmi";
    let mut nodes : Vec<Node> = Vec::new();
    let mut edges : Vec<Edge> = Vec::new();
    let mut sights : Vec<Sight> = Vec::new();
    parse_osm_data(in_graph, &mut nodes, &mut edges, &mut sights);
    write_graph_file( out_graph, &mut nodes, &mut edges, &mut sights);

    let graph = Graph::parse_from_file(out_graph).unwrap();

    println!("{}", graph.num_nodes);
    println!("{}", graph.num_sights);
    println!("{}", graph.num_edges);
    Ok(())
}