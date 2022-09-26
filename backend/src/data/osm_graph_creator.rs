use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, File};
use std::io;
use std::hash::{Hash, Hasher};
use std::io::BufWriter;
use std::path::Path;
use crossbeam::thread;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use geoutils::Location;
use log::{info, trace};
use osmpbf::{BlobReader, BlobType, Element, Way};
use crate::data;
use crate::data::graph::{Category, EdgeType, get_nearest_node, INode};
use crate::data::{EdgeTypeConfig, SightsConfig};

/// An osm node located at a specific coordinate extraced from the osm data.
#[derive(Debug, Serialize, Deserialize)]
pub struct OSMNode {
    #[serde(skip_serializing, skip_deserializing)]
    /// The osm id of the node extracted from the osm data. Later mapped into id.
    osm_id: usize,
    /// The id of the node.
    id: usize,
    /// The latitude of the Location.
    lat: f64,
    /// The longitude of the Location.
    lon: f64
}

impl INode for OSMNode {
    fn id(&self) -> usize {
        self.id
    }
    fn lat(&self) -> f64 {
        self.lat
    }
    fn lon(&self) -> f64 {
        self.lon
    }
}

impl PartialEq<Self> for OSMNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for OSMNode {}

impl Hash for OSMNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// A directed and weighted osm edge extracted from the osm data.
#[derive(Clone, Copy, Serialize)]
struct OSMEdge {
    #[serde(skip_serializing)]
    /// The osm id of the edge's source node from the osm data.
    osm_src: usize,

    /// The osm id of the edge's target node from the osm data.
    #[serde(skip_serializing)]
    osm_tgt: usize,
    /// The id of the edge's source node
    src: usize,
    /// The id of the edge's target node
    tgt: usize,
    /// The edge's weight, i.e., the distance between its source and target
    dist: usize,
    /// The street type of the edge.
    edge_type: EdgeType,
}

impl PartialEq<Self> for OSMEdge {
    fn eq(&self, other: &Self) -> bool {
        self.src == other.src && self.tgt == other.tgt
    }
}

impl Eq for OSMEdge {}

impl Hash for OSMEdge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.src.hash(state);
        self.tgt.hash(state);
    }
}

/// A sight node mapped on its nearest node
#[derive(Debug, Serialize)]
struct OSMSight {
    #[serde(skip_serializing)]
    osm_id: usize,
    node_id: usize,
    lat: f64,
    lon: f64,
    category: Category,
    name: String,
    opening_hours: String,
    wikidata_id: String
}

/// Parse given `graph_file`. If it does not exist yet, build it from `source_file` first.
pub fn checked_create_fmi_graph(graph_file: &str, osm_source_file: &str) -> io::Result<()> {
    if !Path::new(graph_file).exists() && Path::new(osm_source_file).exists() {
        parse_and_write_osm_data(osm_source_file, graph_file)?
    }
    Ok(())
}

/// Parse osmpbf data given in `osmpbf_file_path`.
/// Extract and filter the osm data to create a directed weighted fmi graph containing sights.
/// The data is filtered by the sights_config and edge_type_config files.
/// Writes and saves the created graph data in `fmi_file_path`.
pub fn parse_and_write_osm_data (osmpbf_file_path: &str, fmi_file_path: &str) -> Result<(), io::Error> {
    let mut osm_nodes: Vec<OSMNode> = Vec::new();
    let mut osm_edges: Vec<OSMEdge> = Vec::new();
    let mut osm_sights: Vec<OSMSight> = Vec::new();

    let sight_config_orig = data::get_sights_config();
    let edge_type_config_orig = data::get_edge_type_config();

    let reader = BlobReader::from_path(osmpbf_file_path)?;

    info!("Start reading the PBF file!");
    let time_start = Instant::now();
    //read the file into memory with multi threading
    thread::scope( |s| {
    let mut threads = Vec::new();
    reader.for_each(|result|{
        let blob = result.unwrap();
        let blob_type = blob.get_type();
        if blob_type == BlobType::OsmHeader {
            trace!("This is a Header");
            let header = blob.to_headerblock().unwrap();
            trace!("required Features: {:?}", header.required_features());
            trace!("optional Features: {:?}", header.optional_features());
        } else if blob_type == BlobType::OsmData {
            let sight_config = &sight_config_orig;
            let edge_type_config = &edge_type_config_orig;
            let thread_result = s.spawn(move |_| {
                let data = blob.to_primitiveblock().unwrap();
                let mut result = (Vec::<OSMNode>::new(), Vec::<OSMEdge>::new(), Vec::<OSMSight>::new());
                //start iterating through the blob elements
                data.for_each_element(|element| {
                    match element {
                        Element::Node(n) => {
                            create_osm_node(n.id() as usize, n.lat(), n.lon(), n.tags().collect(), &sight_config, &mut result);
                        },
                        Element::DenseNode(n) => {
                            create_osm_node(n.id() as usize, n.lat(), n.lon(), n.tags().collect(), &sight_config, &mut result);
                        },
                        Element::Way(w) => {
                            create_osm_edges(w, &edge_type_config, &mut result);
                        },
                        Element::Relation(_) => {
                            trace!("Relation element not implemented yet")
                        },

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
        osm_nodes.append(&mut result.0);
        osm_edges.append(&mut result.1);
        osm_sights.append(&mut result.2);
    }
    let time_duration = time_start.elapsed();
    info!("Finished reading PBF file after {} seconds!", time_duration.as_millis() as f32 / 1000.0);
    }).ok();

    //Remove more unwanted sights
    remove_some_sights_without_name(&mut osm_sights);

    let nodes_before_pruning = osm_nodes.len();
    prune_nodes_without_edges(&mut osm_nodes, &osm_edges, &osm_sights);
    let time_duration = time_start.elapsed();
    info!("Finished pruning of {} nodes without edges after {} seconds!", nodes_before_pruning - osm_nodes.len(), time_duration.as_millis() as f32 / 1000.0);



    // HashSet to check whether a node is a sight or not
    let mut is_sight_node = HashSet::new();
    id_post_processing(&mut osm_nodes, &mut osm_edges, &mut osm_sights, &mut is_sight_node);
    let time_duration = time_start.elapsed();
    info!("Finished id post processing after {} seconds!", time_duration.as_millis() as f32 / 1000.0);

    integrate_sights_into_graph(&osm_nodes, &mut osm_edges, &osm_sights, &is_sight_node);
    let time_duration = time_start.elapsed();
    info!("Finished mapping sights into graph after {} seconds!", time_duration.as_millis() as f32 / 1000.0);

    osm_edges.sort_unstable_by(|e1, e2| {
        let id1 = e1.src;
        let id2 = e2.src;
        id1.cmp(&id2).then_with(||{
            let id1 = e1.tgt;
            let id2 = e2.tgt;
            id1.cmp(&id2).then_with(|| {
                let dist1 = e1.dist;
                let dist2 = e2.dist;
                dist1.cmp(&dist2)
            })
        })
    });
    let time_duration = time_start.elapsed();
    info!("Finished sorting edges after {} seconds!", time_duration.as_millis() as f32 / 1000.0);

    let edges_before_pruning = osm_edges.len();
    prune_edges(&mut osm_edges);
    let time_duration = time_start.elapsed();
    info!("Finished pruning of {} identical edges after {} seconds!", edges_before_pruning - osm_edges.len(), time_duration.as_millis() as f32 / 1000.0);

    osm_edges.sort_unstable_by(|e1, e2| {
        let id1 = e1.src;
        let id2 = e2.src;
        id1.cmp(&id2).then_with(||{
            let id1 = e1.tgt;
            let id2 = e2.tgt;
            id1.cmp(&id2)
        })
    });
    let time_duration = time_start.elapsed();
    info!("Finished resorting edges after {} seconds!", time_duration.as_millis() as f32 / 1000.0);

    osm_sights.sort_unstable_by( |s1, s2| {
        if s1.lat > s2.lat {
            Ordering::Greater
        } else if s1.lat < s2.lat {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    });
    let time_duration = time_start.elapsed();
    info!("Finished sorting sights after {} seconds!", time_duration.as_millis() as f32 / 1000.0);

    let time_duration = time_start.elapsed();
    info!("End of PBF data parsing after {} seconds!", time_duration.as_millis() as f32 / 1000.0);

    info!("Start writing the fmi binary file!");
    let path = Path::new(fmi_file_path);
    let prefix = path.parent().unwrap();
    create_dir_all(prefix)?;

    let file = File::create(fmi_file_path)?;
    let mut file = BufWriter::new(file);
    bincode::serialize_into(&mut file, &osm_nodes).expect("Error serializing nodes");
    bincode::serialize_into(&mut file, &osm_sights).expect("Error serializing sights");
    bincode::serialize_into(&mut file, &osm_edges).expect("Error serializing edges");

    let time_duration = time_start.elapsed();
    info!("End of writing fmi binary file after {} seconds!", time_duration.as_millis() as f32 / 1000.0);
    Ok(())
}

/// This is used in the parsing process when reading Nodes / Densenodes from the osm data to create an OSMNode and the corresponding Sight.
/// By using the `sight_config` and the given `tags`, the method detects whether the node is a OSMSight or not.
/// Only creates OSMSights with a specific tag defined in the `sight_config`.
fn create_osm_node(osm_id: usize, lat: f64, lon: f64, tags: Vec<(&str, &str)>, sight_config: &SightsConfig, result: &mut (Vec<OSMNode>, Vec<OSMEdge>, Vec<OSMSight>)) {
    // if sight has no name, osm_id is shown
    let mut osm_name = "None".to_string(); // default
    let mut osm_opening_hours = "empty".to_string(); // default
    let mut categories: HashSet<Category> = HashSet::new();
    let mut osm_wikidata_id = "empty".to_string();
    let mut is_sight = false;
    for (key, value) in tags {
        for cat_tag_map in &sight_config.category_tag_map {
            let category = cat_tag_map.category.parse::<Category>().unwrap();
            for tag in &cat_tag_map.tags {
                if key.eq(&tag.key) {
                    if value.eq(&tag.value) {
                        is_sight = true;
                        categories.insert(category);
                    }
                }
            }
        }
        if key.eq("name") {
            osm_name = value.parse().unwrap();
        }
        if key.eq("opening_hours") {
            osm_opening_hours = value.parse().unwrap();
        }
        if key.eq("wikidata"){
            osm_wikidata_id = value.parse().unwrap();
        }
    }
    let osm_node = OSMNode {
        osm_id,
        id: 0,
        lat,
        lon
    };
    result.0.push(osm_node);

    if is_sight {
        //we are saving the osm id because it's needed in the post processing
        for category in categories {
            let name = osm_name.clone();
            let opening_hours = osm_opening_hours.clone();
            let wikidata_id = osm_wikidata_id.clone();
            let osm_sight = OSMSight {
                osm_id,
                node_id: 0,
                lat,
                lon,
                category,
                name,
                opening_hours,
                wikidata_id
            };
            result.2.push(osm_sight);
        }
    }
}

/// This is used in the parsing process when reading Ways from the osm data to create OSMEdges using the given Way `w`.
/// Only creates OSMEdges with a specific type defined in `edge_type_config`.
/// A Way consists of several osm ids in a specific order, for example (0, 3, 5, 9, 4, ..., 10).
/// This method separates this sequence of osm ids into OSMEdges. In this example: (0,3), (3,5), (5,9) and so on.
/// Since a Way is directed, it creates one OSMEdge for each direction.
fn create_osm_edges(w: Way, edge_type_config: &EdgeTypeConfig, result: &mut (Vec<OSMNode>, Vec<OSMEdge>, Vec<OSMSight>)) {
    let way_tags = w.tags();
    for (key, value) in way_tags {
        for et_tag_map in &edge_type_config.edge_type_tag_map {
            let edge_type = et_tag_map.edge_type.parse::<EdgeType>().unwrap();
            for tag in &et_tag_map.tags {
                if key == tag.key && value == tag.value {
                    let mut way_ref_iter = w.refs();
                    let mut osm_src = way_ref_iter.next().unwrap() as usize;
                    for node_id  in way_ref_iter {
                        // undirected graph, create in and out edges
                        let osm_tgt = node_id as usize;
                        let out_edge = OSMEdge {
                            osm_src,
                            osm_tgt,
                            src: 0,
                            tgt: 0,
                            dist: 0,
                            edge_type
                        };
                        result.1.push(out_edge);

                        let in_edge = OSMEdge {
                            osm_src: osm_tgt,
                            osm_tgt: osm_src,
                            src: 0,
                            tgt: 0,
                            dist: 0,
                            edge_type
                        };
                        result.1.push(in_edge);

                        osm_src = osm_tgt;
                    }
                }
            }
        }
    }
}

/// Remove Sights when they do not have a name, except when they are of category nature or
/// PicnicBarbequeSpot (These types of sights rarely have names but are still cool).
fn remove_some_sights_without_name(osm_sights: &mut Vec<OSMSight>){

    info!("Nodes Before remove_some_sights_without_name: {}", osm_sights.len());
    osm_sights.retain(
        |sight| !sight.name.eq("None") | matches!(sight.category, Category::Nature)
            | matches!(sight.category, Category::PicnicBarbequeSpot)
    );
    info!("Nodes After remove_some_sights_without_name: {}", osm_sights.len());


}

/// Removes every node from `osm_nodes` which has no edge in `osm_edges`.
/// Iterates through `osm_edges` and saves every source node in `nodes_with_outgoing_edge`.
/// Checks if nodes in `osm_nodes` are in `nodes_with_outgoing_edge`. Prunes them if not.
fn prune_nodes_without_edges(osm_nodes: &mut Vec<OSMNode>, osm_edges: &Vec<OSMEdge>, osm_sights: &Vec<OSMSight>) {
    let mut nodes_with_outgoing_edge = HashMap::<usize, bool>::new();
    for edge in osm_edges.iter() {
        nodes_with_outgoing_edge.insert(edge.osm_src, true);
    }
    for sight in osm_sights.iter() { // TODO wieso sight?
        nodes_with_outgoing_edge.insert(sight.osm_id, true);
    }

    let mut i = osm_nodes.len();
    while i > 0 {
        i -= 1;
        if !nodes_with_outgoing_edge.contains_key(&osm_nodes.get(i).unwrap().osm_id) {
            osm_nodes.swap_remove(i);
        }
    }
}

/// Post processing of `osm_nodes`, `osm_edges` and `osm_sights`.
/// A HashMap `osm_id_to_node_id` is created to map all osm_ids from the osm data to the correct graph node_id.
/// While creating `osm_id_to_node_id` duplicate nodes are detected and removed from `osm_nodes`.
/// Afterwards assign sights in `osm_sights` the same node_id as their corresponding nodes by using `osm_id_to_node_id`.
/// In the last step after this whole id mapping process, the edges in `osm_edges` can be set correctly.
fn id_post_processing(osm_nodes: &mut Vec<OSMNode>, osm_edges: &mut Vec<OSMEdge>, osm_sights: &mut Vec<OSMSight>, is_sight_node: &mut HashSet<usize>) {
    let mut osm_id_to_node_id: HashMap<usize, usize> = HashMap::new();
    let mut id_counter = 0;
    let mut duplicate_position_list : Vec<usize> = Vec::new();
    for node in osm_nodes.iter_mut() {
        node.id = id_counter;
        //check for duplicate nodes
        if osm_id_to_node_id.contains_key(&node.osm_id) {
            // info!("duplicate node with id {} and osm_id {}", node.id, node.osm_id);
            // safe position of duplicate (position is for all wright, when deleting starts with first one)
            duplicate_position_list.push(id_counter);
        } else {
            // add new node and increase counter for id
            osm_id_to_node_id.insert(node.osm_id, node.id);
            id_counter += 1;
        }
    }

    // remove duplicate nodes
    for current_id in duplicate_position_list {
        osm_nodes.remove(current_id);
    }

    // assign the same id as the corresponding node (sight and node should have the same osm_id)
    is_sight_node.reserve(osm_sights.len());
    for sight in osm_sights.iter_mut() {
        sight.node_id = *osm_id_to_node_id.get(&sight.osm_id).unwrap();
        is_sight_node.insert(sight.node_id);
    }

    // post processing of edges
    for edge in osm_edges.iter_mut() {
        let src = *osm_id_to_node_id.get(&edge.osm_src).unwrap();
        let src_node = osm_nodes.get(src).unwrap();

        let tgt = *osm_id_to_node_id.get(&edge.osm_tgt).unwrap();
        let tgt_node = osm_nodes.get(tgt).unwrap();

        let src_loc = Location::new(src_node.lat, src_node.lon);
        let tgt_loc = Location::new(tgt_node.lat, tgt_node.lon);

        let mut edge = edge;
        edge.src = src;
        edge.tgt = tgt;
        edge.dist = src_loc.distance_to(&tgt_loc)
            .expect("Could not determine distance between edge source and target")
            .meters() as usize
    }
}

/// Creates one edge (`osm_edges`) for each direction from a sight (`osm_sights`) and the nearest non sight node (`osm_nodes`).
fn integrate_sights_into_graph(osm_nodes: &Vec<OSMNode>, osm_edges: &mut Vec<OSMEdge>, osm_sights: &Vec<OSMSight>, is_sight_node: &HashSet<usize>) {

    //create node list sorted by lat
    let mut nodeIds_by_lat:Vec<usize> = (0..osm_nodes.len()).collect();
    nodeIds_by_lat.sort_unstable_by(|x, y| 
        osm_nodes.get(*x).unwrap().lat.total_cmp(&osm_nodes.get(*y).unwrap().lat));

    let mut n = 0 as f64;
    for sight in osm_sights.iter() {
        n += 1.0;
        let nearest_node_id = get_nearest_node(&osm_nodes, &nodeIds_by_lat, &is_sight_node, sight.lat, sight.lon);
        let nearest_node = &osm_nodes[nearest_node_id];
        let sight_loc = Location::new(sight.lat, sight.lon);
        let nearest_node_loc = Location::new(nearest_node.lat, nearest_node.lon);
        let nearest_dist = sight_loc.distance_to(&nearest_node_loc)
            .expect("Could not determine distance between sight and its nearest node")
            .meters() as usize;
        let out_edge = OSMEdge {
            osm_src: 0,
            osm_tgt: 0,
            src: sight.node_id,
            tgt: nearest_node.id,
            dist: nearest_dist,
            edge_type: EdgeType::SightEdge
        };
        let in_edge = OSMEdge {
            osm_src: 0,
            osm_tgt: 0,
            src: nearest_node.id,
            tgt: sight.node_id,
            dist: nearest_dist,
            edge_type: EdgeType::SightEdge
        };
        osm_edges.push(out_edge);
        osm_edges.push(in_edge);
        trace!("Progress: {}", n / (osm_sights.len() as f64));
    }
}

/// Removes all duplicate OSMEdges in `osm_edges` and keeps the OSMEdge with the lowest dist.
/// This is guarenteed if `osm_edges` is sorted by src, then by tgt and then by dist.
fn prune_edges(osm_edges: &mut Vec<OSMEdge>) {
    let mut i = osm_edges.len()-1;
    while i > 0 {
        let edge_a = osm_edges.get(i-1).unwrap();
        let edge_b = osm_edges.get(i).unwrap();
        if edge_a == edge_b {
            osm_edges.swap_remove(i);
        }
        i -= 1;
    }
}