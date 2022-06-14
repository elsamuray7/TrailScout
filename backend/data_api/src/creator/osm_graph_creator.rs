use std::any::Any;
use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::num::{ParseFloatError, ParseIntError};
use osmpbf::{ElementReader, Element, Node};
use crate::api::graph::{calc_dist, Edge, Node as GraphNode, Sight};
use crate::api::graph;

fn parse_osm_data (osmpbf_file_path: &str) -> Result<(), io::Error> {
    let mut nodes : Vec<GraphNode> = Vec::new();
    let mut edges : Vec<Edge> = Vec::new();
    let mut sights : Vec<Sight> = Vec::new();
    let mut num_nodes : usize = 0;
    let mut num_edges : usize = 0;
    let mut num_sights : usize = 0;

    let reader = ElementReader::from_path(osmpbf_file_path)?;
    let mut node_count = 0;
    let mut way_count = 0;
    let mut dense_count = 0;
    let mut relation_count = 0;

    reader.for_each(|element| {
        if let Element::Node(n) = element {
            // TODO if no tags corrects tags for category + category enum
            let mut node = GraphNode {
                id: n.id() as usize,
                lat: n.lat(),
                lon: n.lon(),
                info: "".to_string()
            };
            for (key, value) in n.tags() {
                node.info = "key: (" + key + ") value: (" + value + ")\n";
            }
            nodes.push(node);
            node_count += 1;
            num_nodes += 1;
        } else if let Element::DenseNode(n) = element {
            // TODO if no tags corrects tags for category + category enum + compare node ids from denseNode and Node !!!
            let mut node = GraphNode {
                id: n.id() as usize,
                lat: n.lat(),
                lon: n.lon(),
                info: "".to_string()
            };
            for (key, value) in n.tags() {
                node.info = "key: (" + key.parse().unwrap() + ") value: (" + value.parse().unwrap() + ")\n";
            }
            nodes.push(node);
            num_nodes += 1;
            dense_count += 1;
        } else if let Element::Way(w) = element {
            // TODO way id; check way tags
            let mut way_iter = w.refs();
            let mut src = way_iter.next().unwrap() as usize;
            for node_id  in way_iter {
                let tgt = node_id as usize;
                let edge = Edge {
                    id: w.id() as usize,
                    src: src,
                    tgt: tgt,
                    dist: 0 // TODO calc dist with graph api method calc_distance
                };
                edges.push(edge);
                num_edges += 1;
                way_count += 1;

                src = tgt;
            }
        } else if let Element::Relation(_) = element {
            relation_count += 1;
        }
        println!("nodes {} ways {} denses {} relations {}", node_count, way_count, dense_count, relation_count);
    })?;
    // TODO sort nodes by osm_id
    println!("nodes {} ways {} denses {} relations {}", node_count, way_count, dense_count, relation_count);
    Ok(())
}

fn write_graph_file(graph_file_path_out: &str) -> std::io::Result<()> {
    let file = File::create(graph_file_path_out)?;
    let mut file = LineWriter::new(file);
    for node in &nodes {
        file.write((format!("node lat lon info \n{} {} {} {}\ntags\n", node.id, node.lat, node.lon, node.info)).as_bytes())?;
        for (key, value) in &node.tags {
            file.write((format!("key:{} value:{}\n\n", key, value)).as_bytes())?;
        }
    }
    for edge in &edges {
        file.write((format!("id src tgt \n{} {} {}\n", edge.id, edge.src, edge.tgt)).as_bytes())?;
    }
    Ok(())
}

fn main() -> Result<(), io::Error> {
    let in_graph = "./osm_graphs/bremen-latest.osm.pbf";
    let out_graph = "./osm_graphs/bremen-latest.fmi";
    parse_osm_data(&in_graph);
    write_graph_file( &out_graph);
    Ok(())
}