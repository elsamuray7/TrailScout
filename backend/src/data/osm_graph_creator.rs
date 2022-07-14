use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::{fs, io};
use std::io::{LineWriter, Write};
use crossbeam::thread;
use serde::Deserialize;
use std::time::{Instant};
use log::{info, error};
use osmpbf::{Element, BlobReader, BlobType};
use crate::data::graph::{calc_dist, Category, Edge, Node as GraphNode, Sight};

use super::graph::Graph;

const SIGHTS_CONFIG_PATH :&str = "./sights_config.json";
const EDGE_CONFIG_PATH :&str = "./edge_type_config.json";

//Deserialization of sights_config
#[derive(Deserialize)]
struct SightsConfig {
    category_tag_map: Vec<CategoryTagMap>
}

#[derive(Deserialize)]
struct CategoryTagMap {
    category: String,
    tags: Vec<Tag>
}

#[derive(Deserialize)]
struct Tag {
    key: String,
    value: String
}

//Deserialization of edge_type_config
#[derive(Deserialize)]
struct EdgeTypeConfig {
    edge_type_tag_map: Vec<EdgeTypeMap>
}

#[derive(Deserialize)]
struct EdgeTypeMap {
    edge_type: String,
    tag: Tag
}

//read config at SIGHTS_CONFIG_PATH and return it
fn get_sights_config() -> SightsConfig {
    let data = fs::read_to_string(SIGHTS_CONFIG_PATH).expect("Unable to read file");
    let sights_config: SightsConfig = serde_json::from_str(&data).expect("Unable to parse");
    return sights_config;
}

//read config at EDGE_CONFIG_PATH and return it
fn get_edge_type_config() -> EdgeTypeConfig {
    let data = fs::read_to_string(EDGE_CONFIG_PATH).expect("Unable to read file");
    let edge_type_config: EdgeTypeConfig = serde_json::from_str(&data).expect("Unable to parse");
    return edge_type_config;
}


pub fn parse_osm_data (osmpbf_file_path: &str, nodes: &mut Vec<GraphNode>, edges: &mut Vec<Edge>, sights: &mut Vec<Sight>) -> Result<(), io::Error> {

    let sight_config_orig = get_sights_config();
    //let edge_type_config = get_edge_type_config();

    let reader = BlobReader::from_path(osmpbf_file_path)?;    

    let mut osm_id_to_node_id: HashMap<usize, usize> = HashMap::new();
    let mut is_street_node: BTreeMap<usize, bool> = BTreeMap::new(); // TODO when parsing ways mark street ndoes, filter nodes that are neither street nodes nor sight nodes

 
    info!("Start reading the PBF file!");
    let time_start = Instant::now();
    //read the file into memory with multi threading
    thread::scope( |s| {
    let mut threads = Vec::new();
    reader.for_each(|result|{
        let blob = result.unwrap();
        let blob_type = blob.get_type();
        if blob_type == BlobType::OsmHeader {
            info!("This is a Header");
            let header = blob.to_headerblock().unwrap();
            info!("required Features: {:?}", header.required_features());
            info!("optional Features: {:?}", header.optional_features());
        } else if blob_type == BlobType::OsmData {
            let sight_config = &sight_config_orig;
            let thread_result = s.spawn(move |d| {
                let data = blob.to_primitiveblock().unwrap();
                let mut result = (Vec::<GraphNode>::new(), Vec::<Edge>::new(), Vec::<Sight>::new());
                //start iterating through the blob elements
                data.for_each_element(|element| {
                    match element {
                        Element::Node(n) => {
                            // TODO if no tags corrects tags for category + category enum
                            /*
                            let mut isSight = false;
                            for (key, value) in n.tags() {
                                match key {
                                    "amenity" => {
                                        isSight = true;
                                        match value {
                                            "restaurant" | "biergarten" | "cafe" | "fast_food" | "food_court" => {
                                                let mut sight = Sight {
                                                    node_id: n.id() as usize,
                                                    lat: n.lat(),
                                                    lon: n.lon(),
                                                    category: Category::Restaurants,
                                                };
                                                sights.push(sight);
                                                num_sights += 1;
                                                node_count += 1;
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if(!isSight) {
                                let mut node = GraphNode {
                                    osm_id: n.id() as usize,
                                    id: num_nodes,
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    //info: "".to_string()
                                };
                                osm_id_to_node_id.entry(node.osm_id)
                                    .or_insert(num_nodes);
                                nodes.push(node);
                                num_nodes += 1;
                                node_count += 1;
                            }

                            */

                            let mut is_sight = false;
                            for (key, value) in n.tags() {
                                /*
                                node.info.push_str("key: (");
                                node.info.push_str(key);
                                node.info.push_str(") value: (");
                                node.info.push_str(value);
                                node.info.push_str(")\n");

                                */
                                for cat_tag_map in &sight_config.category_tag_map {
                                    for tag in &cat_tag_map.tags {
                                        if key.eq(&tag.key) {
                                            if value.eq(&tag.value) {
                                                is_sight = true;
                                                let sight = Sight {
                                                    node_id: 0, // TODO change to nearest node
                                                    lat: n.lat(),
                                                    lon: n.lon(),
                                                    category: cat_tag_map.category.parse::<Category>().unwrap(),
                                                };
                                                result.2.push(sight);

                                                let node = GraphNode {
                                                    osm_id: n.id() as usize,
                                                    id: 0,
                                                    lat: n.lat(),
                                                    lon: n.lon(),
                                                    info: "".to_string()
                                                };
                                                result.0.push(node);
                                            }
                                        }
                                    }
                                }
                            }
                            if !is_sight {
                                let node = GraphNode {
                                    osm_id: n.id() as usize,
                                    id: 0,
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    info: "".to_string()
                                };
                                result.0.push(node);
                            }
                        },
                        Element::DenseNode(n) => {
                            // TODO if no tags corrects tags for category + category enum
                            /*
                            let mut isSight = false;
                            for (key, value) in n.tags() {
                                match key {
                                    "amenity" => {
                                        isSight = true;
                                        match value {
                                            "restaurant" | "biergarten" | "cafe" | "fast_food" | "food_court" => {
                                                let mut sight = Sight {
                                                    node_id: n.id() as usize,
                                                    lat: n.lat(),
                                                    lon: n.lon(),
                                                    category: Category::Restaurants,
                                                };
                                                sights.push(sight);
                                                num_sights += 1;
                                                node_count += 1;
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if(!isSight) {
                                let mut node = GraphNode {
                                    osm_id: n.id() as usize,
                                    id: num_nodes,
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    //info: "".to_string()
                                };
                                osm_id_to_node_id.entry(node.osm_id)
                                    .or_insert(num_nodes);
                                nodes.push(node);
                                num_nodes += 1;
                                node_count += 1;
                            }

                            */

                            let mut is_sight = false;
                            for (key, value) in n.tags() {
                                /*
                                node.info.push_str("key: (");
                                node.info.push_str(key);
                                node.info.push_str(") value: (");
                                node.info.push_str(value);
                                node.info.push_str(")\n");

                                */
                                for cat_tag_map in &sight_config.category_tag_map {
                                    for tag in &cat_tag_map.tags {
                                        if key.eq(&tag.key) {
                                            if value.eq(&tag.value) {
                                                is_sight = true;
                                                let sight = Sight {
                                                    node_id: 0, // TODO change to nearest node
                                                    lat: n.lat(),
                                                    lon: n.lon(),
                                                    category: cat_tag_map.category.parse::<Category>().unwrap(),
                                                };
                                                result.2.push(sight);

                                                let node = GraphNode {
                                                    osm_id: n.id() as usize,
                                                    id: 0,
                                                    lat: n.lat(),
                                                    lon: n.lon(),
                                                    info: "".to_string()
                                                };
                                                result.0.push(node);
                                            }
                                        }
                                    }
                                }
                            }
                            if !is_sight {
                                let node = GraphNode {
                                    osm_id: n.id() as usize,
                                    id: 0,
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    info: "".to_string()
                                };
                                result.0.push(node);
                            }
                        },
                        Element::Way(w) => {
                            // TODO way id; check way tags
                            let mut way_ref_iter = w.refs();
                            let mut osm_src = way_ref_iter.next().unwrap() as usize;
                            for node_id  in way_ref_iter {
                                let osm_tgt = node_id as usize;
                                let edge = Edge {
                                    osm_id: w.id() as usize,
                                    osm_src: osm_src,
                                    osm_tgt: osm_tgt,
                                    src: 0,
                                    tgt: 0,
                                    dist: 0
                                };
                                result.1.push(edge);
                                osm_src = osm_tgt;
                            }
                        },
                        Element::Relation(r) => {},
                        _ => error!("Unrecognized Element")
                    }
                });
                info!("Finished processing one blob!");
                result
            });
            threads.push(thread_result);
        }
    });
    //join all threads and accumulate results
    for t in threads  {
        let mut result = t.join().unwrap();
        nodes.append(&mut result.0);
        edges.append(&mut result.1);
        sights.append(&mut result.2);
    }
    let time_duration = time_start.elapsed();
    info!("Finished reading PBF file after {} seconds!", time_duration.as_secs());
    }).ok();
    //post processing of nodes
    let mut id_counter = 0;
    for node in nodes.iter_mut() {
        node.id = id_counter;
        osm_id_to_node_id.insert(node.osm_id, node.id);
        id_counter += 1;
    }
    let time_duration = time_start.elapsed();
    info!("Finished building osm_id_to_node_id after {} seconds!", time_duration.as_secs());

    //post processing of edges
    for edge in edges.iter_mut() {
        let src = *osm_id_to_node_id.get(&edge.osm_src).unwrap();
        let src_node = nodes.get(src).unwrap();

        let tgt = *osm_id_to_node_id.get(&edge.osm_tgt).unwrap();
        let tgt_node = nodes.get(tgt).unwrap();

        let mut edge = edge;
        edge.src = src;
        edge.tgt = tgt;
        edge.dist = calc_dist(src_node.lat, src_node.lon, tgt_node.lat, tgt_node.lon);
    }

    let time_duration = time_start.elapsed();
    info!("Finished post processing of edges after {} seconds!", time_duration.as_secs());

    edges.sort_unstable_by(|e1, e2| {
        let id1 = e1.src;
        let id2 = e2.src;
        id1.cmp(&id2).then_with(||{
            let id1 = e1.tgt;
            let id2 = e2.tgt;
            id1.cmp(&id2)
        })
    });

    let time_duration = time_start.elapsed();
    info!("Finished sorting edges after {} seconds!", time_duration.as_secs());

    let time_duration = time_start.elapsed();
    info!("End of PBF data parsing after {} seconds!", time_duration.as_secs());
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
        file.write(format!("{} {} {}\n", node.id, node.lat, node.lon).as_bytes())?;
    }
    for sight in &*sights {
        file.write(format!("{} {} {} {}\n", sight.node_id, sight.lat, sight.lon, sight.category.to_string()).as_bytes())?;
    }
    for edge in &*edges {
        file.write(format!("{} {} {}\n", edge.src, edge.tgt, edge.dist).as_bytes())?;
    }
    Ok(())
}