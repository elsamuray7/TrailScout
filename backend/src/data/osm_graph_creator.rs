use std::collections::BTreeMap;
use std::{fs, io};
use std::fs::File;
use std::io::{LineWriter, Write};
use log::{info,trace};
use osmpbf::{ElementReader, Element};
use serde::Deserialize;
use crate::data::graph::{calc_dist, Category, Edge, Node as GraphNode, Sight};

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
    let mut num_nodes: usize = 0;
    let mut num_edges: usize = 0;
    let mut num_sights: usize = 0;

    let sight_config = get_sights_config();
    //let edge_type_config = get_edge_type_config();

    let reader = ElementReader::from_path(osmpbf_file_path)?;
    let mut node_count = 0;
    let mut way_count = 0;
    let mut dense_count = 0;
    let mut relation_count = 0;

    let mut progress_counter = 0;

    let mut osm_id_to_node_id: BTreeMap<usize, usize> = BTreeMap::new();
    let mut is_street_node: BTreeMap<usize, bool> = BTreeMap::new(); // TODO when parsing ways mark street ndoes, filter nodes that are neither street nodes nor sight nodes

    info!("Start reading the PBF file!");
    reader.for_each(|element| {
        if let Element::Node(n) = element {
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
                                    node_id: num_nodes, // TODO change to nearest node
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    category: cat_tag_map.category.parse::<Category>().unwrap(),
                                };
                                sights.push(sight);
                                num_sights += 1;

                                let node = GraphNode {
                                    osm_id: n.id() as usize,
                                    id: num_nodes,
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    info: "".to_string()
                                };

                                osm_id_to_node_id.entry(node.osm_id)
                                    .or_insert(num_nodes);
                                nodes.push(node);
                                num_nodes += 1;
                                node_count += 1;
                            }
                        }
                    }
                }
            }
            if !is_sight {
                let node = GraphNode {
                    osm_id: n.id() as usize,
                    id: num_nodes,
                    lat: n.lat(),
                    lon: n.lon(),
                    info: "".to_string()
                };

                osm_id_to_node_id.entry(node.osm_id)
                    .or_insert(num_nodes);
                nodes.push(node);
                num_nodes += 1;
                node_count += 1;
            }
        } else if let Element::DenseNode(n) = element {
            // TODO if no tags corrects tags for category + category enum + compare node ids from denseNode and Node !!!
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
                                    node_id: num_nodes, // TODO change to nearest node
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    category: cat_tag_map.category.parse::<Category>().unwrap(),
                                };
                                sights.push(sight);
                                num_sights += 1;

                                let node = GraphNode {
                                    osm_id: n.id() as usize,
                                    id: num_nodes,
                                    lat: n.lat(),
                                    lon: n.lon(),
                                    info: "".to_string()
                                };

                                osm_id_to_node_id.entry(node.osm_id)
                                    .or_insert(num_nodes);
                                nodes.push(node);
                                num_nodes += 1;
                                node_count += 1;
                            }
                        }
                    }
                }
            }
            if !is_sight {
                let node = GraphNode {
                    osm_id: n.id() as usize,
                    id: num_nodes,
                    lat: n.lat(),
                    lon: n.lon(),
                    info: "".to_string()
                };

                osm_id_to_node_id.entry(node.osm_id)
                    .or_insert(num_nodes);
                nodes.push(node);
                num_nodes += 1;
                node_count += 1;
            }
        } else if let Element::Way(w) = element {
            // TODO way id; check way tags for edge type
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
                // TODO set edge_type
                let src_node = &nodes[edge.src];
                let tgt_node = &nodes[edge.tgt];
                edge.dist = calc_dist(src_node.lat, src_node.lon, tgt_node.lat, tgt_node.lon);
                //let srcNode = &nodes[edge.src];
                //let tgtNode = &nodes[edge.tgt];
                //let dist = calc_dist(srcNode.lat, srcNode.lon), tgt.;

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
        if progress_counter % 40000 == 0 {
            trace!("finished processing {} elements", progress_counter);
        }
        progress_counter += 1;
        //println!("nodes {} ways {} denses {} relations {}", node_count, way_count, dense_count, relation_count);
    })?;
    info!("Finished reading PBF file!");
    edges.sort_unstable_by(|e1, e2| {
        let id1 = e1.src;
        let id2 = e2.src;
        id1.cmp(&id2).then_with(||{
            let id1 = e1.tgt;
            let id2 = e2.tgt;
            id1.cmp(&id2)
        })
    });
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