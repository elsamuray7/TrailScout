use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::num::{ParseFloatError, ParseIntError};
use osmpbf::{ElementReader, Element, Node};
use crate::api::graph::{calc_dist, Edge, Node as GraphNode, Sight};

pub fn parse_osm_data (osmpbf_file_path: &str, nodes: &mut Vec<GraphNode>, edges: &mut Vec<Edge>, sights: &mut Vec<Sight>) -> Result<(), io::Error> {
    let mut num_nodes: usize = 0;
    let mut num_edges: usize = 0;
    //let mut num_sights: usize = 0;

    let reader = ElementReader::from_path(osmpbf_file_path)?;
    let mut node_count = 0;
    let mut way_count = 0;
    let mut dense_count = 0;
    let mut relation_count = 0;

    let mut osm_id_to_node_id: BTreeMap<usize, usize> = BTreeMap::new();

    reader.for_each(|element| {
        if let Element::Node(n) = element {
            // TODO if no tags corrects tags for category + category enum
            let mut node = GraphNode {
                osm_id: n.id() as usize,
                id: num_nodes,
                lat: n.lat(),
                lon: n.lon(),
                info: "".to_string()
            };
            for (key, value) in n.tags() {
                node.info.push_str("key: (");
                node.info.push_str(key);
                node.info.push_str(") value: (");
                node.info.push_str(value);
                node.info.push_str(")\n");
            }

            osm_id_to_node_id.entry(node.osm_id)
                .or_insert(num_nodes);

            nodes.push(node);
            node_count += 1;
            num_nodes += 1;
        } else if let Element::DenseNode(n) = element {
            // TODO if no tags corrects tags for category + category enum + compare node ids from denseNode and Node !!!
            let mut node = GraphNode {
                osm_id: n.id() as usize,
                id: num_nodes,
                lat: n.lat(),
                lon: n.lon(),
                info: "".to_string()
            };
            for (key, value) in n.tags() {
                node.info.push_str("key: (");
                node.info.push_str(key);
                node.info.push_str(") value: (");
                node.info.push_str(value);
                node.info.push_str(")\n");
            }
            osm_id_to_node_id.entry(node.osm_id)
                .or_insert(num_nodes);
            nodes.push(node);
            num_nodes += 1;
            dense_count += 1;
        } else if let Element::Way(w) = element {
            // TODO way id; check way tags
            let mut way_ref_iter = w.refs();
            let mut osm_src = way_ref_iter.next().unwrap() as usize;
            for node_id  in way_ref_iter {
                let osm_tgt = node_id as usize;
                let mut edge = Edge {
                    osm_id: w.id() as usize,
                    osm_src: osm_src,
                    osm_tgt: osm_tgt,
                    src: *osm_id_to_node_id.get(&osm_src).unwrap(),
                    tgt: *osm_id_to_node_id.get(&osm_tgt).unwrap(),
                    dist: 0
                };

                //let src_node = &nodes[edge.src];
                //let tgt_node = &nodes[edge.tgt];
                //edge.dist = calc_dist(&src_node.lat, &src_node.lon, &tgt_node.lat, &tgt_node.lon);

                edges.push(edge);
                num_edges += 1;
                way_count += 1;

                osm_src = osm_tgt;
            }
            /*
            if(w.id() == 3999579) {
                println!("way id 3999579:");
                for val in w.refs() {
                    println!("{}", val);
                }
            }
            */
        } else if let Element::Relation(_) = element {
            relation_count += 1;
        }
        //println!("nodes {} ways {} denses {} relations {}", node_count, way_count, dense_count, relation_count);
    })?;
    Ok(())
}

pub fn write_graph_file(graph_file_path_out: &str, nodes: &mut Vec<GraphNode>, edges: &mut Vec<Edge>, sights: &mut Vec<Sight>) -> std::io::Result<()> {
    let file = File::create(graph_file_path_out)?;
    let mut file = LineWriter::new(file);
    /*
    file.write((format!("Number of Nodes: {}\n", nodes.len())).as_bytes())?;
    file.write((format!("Number of Edges: {}\n", edges.len())).as_bytes())?;
    file.write((format!("osm_id node_id lat lon\n")).as_bytes())?;
    for node in &*nodes {
        file.write((format!("{} {} {} {}\n", node.osm_id, node.id, node.lat, node.lon).as_bytes()))?;
        file.write((format!("info\n{}\n", node.info)).as_bytes())?;
    }
    file.write((format!("osm_id osm_src osm_tgt src tgt dist\n")).as_bytes())?;
    for edge in &*edges {
        file.write((format!("{} {} {} {} {} {}\n", edge.osm_id, edge.osm_src, edge.osm_tgt, edge.src, edge.tgt, edge.dist)).as_bytes())?;
    }
     */
    file.write((format!("{}\n", nodes.len())).as_bytes())?;
    file.write((format!("{}\n", sights.len())).as_bytes())?;
    file.write((format!("{}\n", edges.len())).as_bytes())?;
    for node in &*nodes {
        file.write((format!("{} {} {}\n", node.id, node.lat, node.lon).as_bytes()))?;
    }
    /*
    for sight in &*sights {
        file.write((format!("{} {} {}\n", node.id, node.lat, node.lon).as_bytes()))?;
    }
     */
    for edge in &*edges {
        file.write((format!("{} {} {}\n", edge.src, edge.tgt, edge.dist)).as_bytes())?;
    }
    Ok(())
}