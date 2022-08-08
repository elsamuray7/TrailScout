use std::collections::{BTreeMap, HashMap};
use std::fs::{File, create_dir_all};
use std::{fs, io};
use std::io::{Write, BufWriter};
use crossbeam::thread;
use serde::Deserialize;
use std::time::{Instant};
use log::{info, error, trace, debug};
use osmpbf::{Element, BlobReader, BlobType};
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
                                                //we are saving the osm id because it's needed in the post processing
                                                let sight = Sight {
                                                    osm_id: n.id() as usize,
                                                    node_id: 0,
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
                                                //we are saving the osm id because it's needed in the post processing
                                                let sight = Sight {
                                                    osm_id: n.id() as usize,
                                                    node_id: 0,
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
                                // undirected graph, create in and out edges
                                let osm_tgt = node_id as usize;
                                let out_edge = Edge {
                                    osm_id: w.id() as usize,
                                    osm_src: osm_src,
                                    osm_tgt: osm_tgt,
                                    src: 0,
                                    tgt: 0,
                                    dist: 0
                                };
                                result.1.push(out_edge);

                                let in_edge = Edge {
                                    osm_id: w.id() as usize,
                                    osm_src: osm_tgt,
                                    osm_tgt: osm_src,
                                    src: 0,
                                    tgt: 0,
                                    dist: 0
                                };
                                result.1.push(in_edge);

                                osm_src = osm_tgt;
                            }
                        },
                        Element::Relation(r) => {},
                        _ => error!("Unrecognized Element")
                    }
                });
                trace!("Finished processing one blob!");
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
    //post processing of nodes and sights
    let mut id_counter = 0;
    for node in nodes.iter_mut() {
        node.id = id_counter;
        //check for duplicate nodes
        if(osm_id_to_node_id.contains_key(&node.osm_id)) {
            info!("duplicate node with id {} and osm_id {}", node.id, node.osm_id);
        }
        osm_id_to_node_id.insert(node.osm_id, node.id);
        id_counter += 1;
    }
    //assign the same id as the corresponding node (sight and node should have the same osm_id)
    for sight in sights.iter_mut() {
        sight.node_id = *osm_id_to_node_id.get(&sight.osm_id).unwrap();
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

    let mut number_of_edges = edges.len();
    info!("Start pruning identical edges!");

    // prune double edges
    let prune_edges: HashMap<Edge, usize> =
    {
    let mut prune_edges: HashMap<&Edge, usize> = HashMap::new();
    let mut edge_a = edges.first().unwrap();
    let mut first_edge = true;
    // find all edges to be pruned
    for edge in &*edges {
        if (first_edge) {
            edge_a = edge;
            first_edge = false;
        } else {
            let edge_b = edge;
            // edges are sorted by src, then by tgt, check for same (src, tgt) edges
            if (edge_a.src == edge_b.src) && (edge_a.tgt == edge_b.tgt) {
                trace!("Found two identical edges! \n Edge a: src: {} tgt: {} dist: {} \n Edge b: src: {} tgt: {} dist: {}", edge_a.src, edge_a.tgt, edge_a.dist, edge_b.src, edge_b.tgt, edge_b.dist);
                // if several identical edges exist, save the lowest dist
                if prune_edges.contains_key(&edge_a) {
                    let prune_dist = prune_edges.get_key_value(&edge_a).unwrap().0.dist;
                    if (edge_a.dist < prune_dist) && (edge_a.dist <= edge_b.dist) {
                        trace!("Updating edge dist ({}, {}): {} -> {}", edge_a.src, edge_a.tgt, prune_dist, edge_a.dist);
                        prune_edges.insert(edge_a, edge_a.dist);
                    } else if (edge_b.dist < prune_dist) && (edge_a.dist > edge_b.dist) {
                        trace!("Updating edge dist ({} / {}): {} -> {}", edge_a.src, edge_a.tgt, prune_dist, edge_b.dist);
                        prune_edges.insert(edge_a, edge_b.dist);
                    }
                } else {  // save lowest dist edge to prune later
                    /*
                    if edge_a.dist < edge_b.dist {
                        info!("Different distance: edge_a: {}, edge_b: {}", edge_a.dist, edge_b.dist);
                    } else if edge_b.dist < edge_a.dist {
                        info!("Different distance: edge_a: {}, edge_b: {}", edge_a.dist, edge_b.dist);
                    }
                    */
                    if edge_a.dist <= edge_b.dist {
                        trace!("Inserting edge: ({}, {}) with dist: {}", edge_a.src, edge_a.tgt, edge_a.dist);
                        prune_edges.insert(edge_a, edge_a.dist);
                    } else {
                        trace!("Inserting edge: ({}, {}) with dist: {}", edge_a.src, edge_a.tgt, edge_b.dist);
                        prune_edges.insert(edge_a, edge_b.dist);
                    }
                }
            }
            edge_a = edge_b;
        }
    }
    prune_edges.iter().map(|(&edge, &dist)| (edge.clone(), dist)).collect()
    };
    // prune identical edges and keep one edge with lowest dist
    edges.retain(|edge| !prune_edges.contains_key(&edge));
    let prune_edges_len = prune_edges.len();
    for (edge, dist) in prune_edges {
        if dist < edge.dist {
            let mut new_edge = edge.clone();
            new_edge.dist = dist;
            edges.push(new_edge);
        } else {
            edges.push(edge);
        }
    }

    let time_duration = time_start.elapsed();
    info!("Finished pruning identical edges after {} seconds!", time_duration.as_secs());

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

    info!("Number of edges before pruning: {}", number_of_edges);
    info!("Number of edges after pruning: {}", edges.len());
    number_of_edges = number_of_edges - edges.len();
    info!("Number of edges pruned: {}", number_of_edges);
    info!("Prune edges: {}", prune_edges_len);

    let time_duration = time_start.elapsed();
    info!("End of PBF data parsing after {} seconds!", time_duration.as_secs());
    Ok(())
}


pub fn write_graph_file(graph_file_path_out: &str, nodes: &mut Vec<GraphNode>, edges: &mut Vec<Edge>, sights: &mut Vec<Sight>) -> std::io::Result<()> {
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
    info!("Start writing the fmi file!");
    let time_start = Instant::now();

    let mut text = "".to_owned();

    for node in &*nodes {
        text.push_str(&format!("{} {} {}\n", node.id, node.lat, node.lon));
    }
    for sight in &*sights {
        text.push_str(&format!("{} {} {} {}\n", sight.node_id, sight.lat, sight.lon, sight.category.to_string()));
    }
    for edge in &*edges {
        text.push_str(&format!("{} {} {}\n", edge.src, edge.tgt, edge.dist));
    }
    
    let time_duration = time_start.elapsed();
    info!("Created text after {} seconds!", time_duration.as_secs());

    let path = std::path::Path::new(graph_file_path_out);
    let prefix = path.parent().unwrap();
    create_dir_all(prefix)?;
    let file = File::create(graph_file_path_out)?;
    let mut file = BufWriter::new(file);

    file.write((format!("{}\n", nodes.len())).as_bytes())?;
    file.write((format!("{}\n", sights.len())).as_bytes())?;
    file.write((format!("{}\n", edges.len())).as_bytes())?;

    file.write(text.as_bytes())?;

    let time_duration = time_start.elapsed();
    info!("End of writing fmi file after {} seconds!", time_duration.as_secs());
    Ok(())
}