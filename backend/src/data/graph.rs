use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufReader};
use std::num::{ParseFloatError, ParseIntError};
use std::time::Instant;
use strum_macros::EnumString;
use geoutils::{Distance, Location};
use itertools::Itertools;
use log::{debug, trace, info};
use serde::{Serialize, Deserialize};
use opening_hours::OpeningHours;
use crate::data;
use crate::data::SightsConfig;
use crate::utils::dijkstra;

#[derive(EnumString, Deserialize, Serialize, PartialEq, Eq, Debug, Copy, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum Category {
    Activities,
    Swimming,
    PicnicBarbequeSpot,
    MuseumExhibition,
    Nature,
    Nightlife,
    Restaurants,
    Sightseeing,
    Shopping,
    Animals
}

#[derive(strum_macros::Display, EnumString, Deserialize, Serialize, PartialEq, Debug, Copy, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum EdgeType {
    Unclassified, // Öffentlich befahrbare Nebenstraßen
    Residential, // Tempo-30-Zonen
    Service, // Privatgelände)
    LivingStreet, // Verkehrsberuhigter Bereich
    Pedestrian, // Fußgängerzone
    Track, // Wirtschafts-, Feld- oder Waldweg
    Road, // Straße unbekannter Klassifikation)
    Footway, // Gehweg
    Bridleway, // Reitweg
    Steps, // Treppen auf Fuß-/Wanderwegen
    Corridor, // Ein Gang im Inneren eines Gebäudes
    Path, // Wanderwege oder Trampelpfade
    Primary, // Straßen von nationaler Bedeutung
    Secondary, // Straßen von überregionaler Bedeutung
    Tertiary, // Straßen, die Dörfer verbinden
    SightEdge // Selbst erzeugte Kanten von einer Sight zum nächsten Straßenknoten
}

pub trait INode {
    fn id(&self) -> usize;
    fn lat(&self) -> f64;
    fn lon(&self) -> f64;
}

/// A graph node located at a specific coordinate
#[derive(Debug, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub lat: f64,
    pub lon: f64
}

impl INode for Node {
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

impl PartialEq<Self> for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Node {}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// A directed and weighted graph edge
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Edge {
    /// The id of the edge's source node
    pub src: usize,
    /// The id of the edge's target node
    pub tgt: usize,
    /// The edge's weight, i.e., the distance between its source and target
    pub dist: usize,
    /// The street type of the edge.
    pub edge_type: EdgeType,
}

impl PartialEq<Self> for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.src == other.src && self.tgt == other.tgt
    }
}

impl Eq for Edge {}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.src.hash(state);
        self.tgt.hash(state);
    }
}

/// A sight node mapped on its nearest node
//#[derive(Debug, Serialize, Deserialize)]
#[derive(Serialize, Deserialize)]
pub struct Sight {
    pub node_id: usize,
    pub lat: f64,
    pub lon: f64,
    pub category: Category,
    pub name: String,
    pub opening_hours: String,
    #[serde(skip)]
    pub opening_hours_parsed: Option<OpeningHours>,
    #[serde(skip_deserializing)]
    pub duration_of_stay_minutes : usize, //default 0 when not overwritten by set_config_duration_of_stay
    pub wikidata_id: String
}

impl Sight{
    ///Tries to parse opening hours from osm and then sets opening_hours_parsed.
    ///If osm value cannot be parsed use default value from sights config
    ///Also overwrites opening_hours if default value is used
    pub fn parse_opening_hours(&mut self, sights_config: &SightsConfig){
        //Try to parse OSM opening hours
        let opening_hours_parsed = match OpeningHours::parse(&self.opening_hours){
            Ok(res) => {
                trace!("Using OSM Opening Times");
                Some(res)
            }
            //Get default value if could not parse
            _ => {
                trace!("Parsing Default Opening Times");
                let default_openings = self.get_default_opening_hour(&sights_config);
                let parse = OpeningHours::parse(&default_openings)
                    .expect("Could not parse default opening hours");

                //override old (invalid) opening times with default
                self.opening_hours = default_openings;

                Some(parse)
            }
        };
        self.opening_hours_parsed = opening_hours_parsed;
    }

    /// Get default opening hours from sights_config
    fn get_default_opening_hour(&self, sights_config : &SightsConfig) -> String {
        for cat_tag_map in &sights_config.category_tag_map{
            let cat = cat_tag_map.category.parse::<Category>().unwrap();
            if self.category == cat {
                let default_opening_hours = cat_tag_map.opening_hours.clone();
                return default_opening_hours
            }
        }
        panic!("Error while parsing default time from config") // If this happens sights_config.json is wrong
    }

    ///Sets the duration_of_stay_minutes value according to value in sights config
    pub fn set_config_duration_of_stay(&mut self, sights_config : &SightsConfig) {
        // viel Code duplication mit get_default_opening_hour - vielleicht irgendwie refactoren?
        for cat_tag_map in &sights_config.category_tag_map{
            let cat = cat_tag_map.category.parse::<Category>().unwrap();
            if self.category == cat {
                let default_opening_hours = cat_tag_map.duration_of_stay_minutes.clone();
                self.duration_of_stay_minutes = default_opening_hours;
                break
            }
        }
    }
}

// Need custom debug to ignore parsed OpeningHours because it does not implement it
impl Debug for Sight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sight")
            .field("node_id", &self.node_id)
            .field("lat", &self.lat)
            .field("lon", &self.lon)
            .field("category", &self.category)
            .field("name", &self.name)
            .field("opening_hours", &self.opening_hours)
            .field("duration_of_stay_minutes", &self.duration_of_stay_minutes)
            .finish()
    }
}



/// A directed graph. In addition to nodes and edges, the definition also contains a set of sights
/// mapped on their nearest nodes, respectively.
pub struct Graph {
    nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub offsets: Vec<usize>,
    pub num_nodes: usize,
    pub num_edges: usize,
    pub sights: Vec<Sight>,
    pub num_sights: usize,
}


impl Graph {
    /// Parse graph data (in particular, nodes, edges and sights) from a file and create a new
    /// graph from it
    pub fn parse_from_file(graph_file_path: &str) -> Result<Self, ParseError> {
        info!("Start creating the graph from fmi binary file!");
        let time_start = Instant::now();

        let graph_file = File::open(graph_file_path)?;
        let mut graph_reader = BufReader::new(graph_file);

        let nodes:Vec<Node> = bincode::deserialize_from(&mut graph_reader).unwrap();
        let mut sights:Vec<Sight> = bincode::deserialize_from(&mut graph_reader).unwrap();
        let edges:Vec<Edge> = bincode::deserialize_from(&mut graph_reader).unwrap();

        let num_nodes = nodes.len();
        let num_sights = sights.len();
        let num_edges = edges.len();

        let mut next_src: usize = 0;
        let mut offset: usize = 0;
        let mut offsets = vec![0; num_nodes + 1];
        for edge in edges.iter() {
            if edge.src >= next_src {
                for j in next_src..=edge.src {
                    offsets[j] = offset;
                }
                next_src = edge.src + 1;
            }
            offset += 1;
        }
        for i in next_src..= num_nodes {
            offsets[i] = num_edges;
        }

        //Parse the opening hours to fill Opening_hours_parsed: Option<OpeningHours>
        //Also read duration_of_stay_minutes from sights config and set the value for the sight
        let sights_config = data::get_sights_config();
        for sight in &mut sights{
            sight.parse_opening_hours(&sights_config);
            sight.set_config_duration_of_stay(&sights_config);
        }

        
        let time_duration = time_start.elapsed();
        info!("End graph creation after {} seconds!", time_duration.as_millis() as f32 / 1000.0);

        Ok(Self {
            nodes,
            edges,
            offsets,
            num_nodes,
            num_edges,
            sights,
            num_sights,
        })
    }

    /// Returns a reference to the vector containing all nodes in this graph
    pub fn nodes(&self) -> &Vec<Node> {
        &self.nodes
    }

    /// Get the node with id `node_id`
    pub fn get_node(&self, node_id: usize) -> &Node {
        &self.nodes[node_id]
    }

    /// Get the nearest non-sight reachable graph node to a given coordinate (latitude / longitude)
    pub fn get_nearest_node(&self, lat: f64, lon: f64) -> usize {
        let nodes_sorted_by_lat = self.nodes.iter()
            .sorted_unstable_by(|node1, node2| node1.lat.total_cmp(&node2.lat))
            .collect_vec();
        let id_filter = self.sights.iter().map(|sight| sight.node_id)
            //.merge(self.nodes.iter().filter(|node| self.get_degree(node.id) > 0)
            //    .map(|node| node.id))
            .collect();
        get_nearest_node(&nodes_sorted_by_lat, &id_filter, lat, lon)
    }

    /// Get the nearest non-sight reachable graph node to a given coordinate (latitude / longitude)
    /// Also checks if the nearest node is in a given area. Returns None if not in given area.
    pub fn get_nearest_node_in_area(&self, lat: f64, lon: f64, radius: f64) -> Option<usize> {
        let nodes_sorted_by_lat = self.nodes.iter()
            .sorted_unstable_by(|node1, node2| node1.lat.total_cmp(&node2.lat))
            .collect_vec();
        let id_filter = self.sights.iter().map(|sight| sight.node_id)
            //.merge(self.nodes.iter().filter(|node| self.get_degree(node.id) > 0)
            //    .map(|node| node.id))
            .collect();
        let nearest_node_id = get_nearest_node(&nodes_sorted_by_lat, &id_filter, lat, lon);
        let nearest_node = self.get_node(nearest_node_id);
        let nearest_node_location = Location::new(nearest_node.lat(), nearest_node.lon());

        let center = Location::new(lat, lon);
        let radius = Distance::from_meters(radius);

        let mut min_id: Option<usize> = None;
        if nearest_node_location.is_in_circle(&center, radius).unwrap() {
            min_id = Some(nearest_node_id);
            min_id
        } else {
            min_id
        }
    }

    /// Get the number of outgoing edges of the node with id `node_id`
    pub fn get_degree(&self, node_id: usize) -> usize {
        self.offsets[node_id + 1] - self.offsets[node_id]
    }

    /// Get all outgoing edges of a particular node
    pub fn get_outgoing_edges(&self, node_id: usize) -> &[Edge] {
        &self.edges[self.offsets[node_id]..self.offsets[node_id + 1]]
    }

    /// Get all outgoing edges of a particular node where the edge target lies within given area
    pub fn get_outgoing_edges_in_area(&self, node_id: usize, lat: f64, lon: f64, radius: f64) -> Vec<&Edge> {
        let center = Location::new(lat, lon);
        let out_edges = self.get_outgoing_edges(node_id);
        out_edges.iter()
            .filter(|&edge| {
                let tgt_node = self.get_node(edge.tgt);
                let tgt_loc = Location::new(tgt_node.lat, tgt_node.lon);
                // Use haversine distance here for more efficiency
                center.haversine_distance_to(&tgt_loc).meters() <= radius
            })
            .collect()
    }

    /// Get all sights within a circular area, specified by `radius` (in meters), around a given coordinate
    /// (latitude / longitude)
    pub fn get_sights_in_area(&self, lat: f64, lon: f64, radius: f64) -> Vec<&Sight> {
        debug!("Computing sights in area: lat: {}, lon: {}, radius: {}", lat, lon, radius);

        //estimate bounding box with 111111 meters = 1 longitude degree
        //use binary search to find the range of elements that should be considered
        let lower_bound = binary_search_sights_vector(&self.sights, lat - radius / 111111.0);
        let upper_bound = binary_search_sights_vector(&self.sights, lat + radius / 111111.0);

        let slice = &self.sights[lower_bound..upper_bound];

        let center = Location::new(lat, lon);
        let radius = Distance::from_meters(radius);
        //iterate through the slice and check every sight whether it's in the target circle
        let sights_in_area: Vec<&Sight> = slice.iter()
            .filter(|sight| {
                let location = Location::new(sight.lat, sight.lon);
                location.is_in_circle(&center, radius)
                    .expect("Could not determine whether sight lies in given area")
            })
            .collect();
        debug!("Found {} sights within the given area (of a total of {} sights)",
            sights_in_area.len(), self.sights.len());

        sights_in_area
    }

    /// Get all reachable sights within a circular area, specified by `radius` (in meters), around a given coordinate
    /// (latitude / longitude).
    /// `reachable_with` specifies within which radius reachability must be tested.
    pub fn get_reachable_sights_in_area(&self, lat: f64, lon: f64, radius: f64, reachable_within: f64) -> Vec<&Sight> {
        // Get all nodes that are reachable from the node with the lowest distance to the center
        let center_id = self.get_nearest_node(lat, lon);
        let reachable_nodes = dijkstra::run_ota_dijkstra_in_area(
            &self, center_id, lat, lon, reachable_within);

        let reachable_sights: Vec<&Sight> = self.get_sights_in_area(lat, lon, radius).into_iter()
            .filter(|sight | reachable_nodes.dist_to(sight.node_id).is_some())
            .collect();
        debug!("Found {} reachable sights within the given area (of a total of {} sights)",
            reachable_sights.len(), self.sights.len());

        reachable_sights
    }
}

/// Helper method to estimate index bounds within the sights vector for latitude coordinates
fn binary_search_sights_vector(sights: &Vec<Sight>, target_latitude: f64) -> usize {
    let result = sights.binary_search_by(|sight|
        sight.lat.total_cmp(&target_latitude));
    result.unwrap_or_else(|index| index)
}

/// Get the nearest node (that is not in `id_filter`) to a given coordinate (latitude / longitude).
/// The function expects a node vector sorted by latitude.
pub fn get_nearest_node(nodes_sorted_by_lat: &Vec<&impl INode>, id_filter: &HashSet<usize>, lat: f64, lon: f64) -> usize {
    // Location to find the nearest node for
    let location = Location::new(lat, lon);

    // Search the index of the node with the closest latitude coordinate within the list of nodes
    let result = nodes_sorted_by_lat.binary_search_by(|n| n.lat().total_cmp(&lat));
    let found_index = result.unwrap_or_else(|index| index);
    trace!("Starting to search for nearest node at index: {} for latitude: {}", found_index, lat);

    let mut min_dist = Distance::from_meters(f64::MAX);
    let mut min_id = usize::MAX;

    // Iterate over the left and right neighbour indices to determine the nearest node
    let mut left_index = found_index;
    let mut right_index = found_index;
    loop {
        for i in [left_index, right_index] {
            let node = nodes_sorted_by_lat[i];

            // If the distance with the current nodes longitude set to the longitude of the
            // location is greater than the minimum distance so far, abort and output the node
            // with the found distance
            let node_loc_lon_aligned = Location::new(node.lat(), location.longitude());
            // Use haversine distance here for more efficiency
            let minimum_possible_distance = location.haversine_distance_to(&node_loc_lon_aligned);
            if minimum_possible_distance.meters() >= min_dist.meters() {
                trace!("Minimum possible distance: {} for node: {} greater/equal to minimum distance so far: {} for node: {}",
                minimum_possible_distance, node.id(), min_dist, min_id);
                return min_id;
            }

            // If the node is not a sight and has a smaller distance to the location than the
            // minimum distance found so far, update the minimum distance and the id of the nearest
            // node
            if !id_filter.contains(&node.id()) {
                let node_loc = Location::new(node.lat(), node.lon());
                // Use haversine distance here for more efficiency
                let dist = location.haversine_distance_to(&node_loc);
                if dist.meters() < min_dist.meters() {
                    trace!("Updating minimum distance so far from: {} for node: {} to: {} for node: {}",
                    min_dist, min_id, dist, node.id());
                    min_dist = dist;
                    min_id = node.id();
                }
            }
        }

        if left_index == 0 && right_index == nodes_sorted_by_lat.len() - 1 {
            break;
        }

        left_index = if left_index == 0 { 0 } else { left_index - 1 };
        right_index = if right_index == nodes_sorted_by_lat.len() - 1 { right_index } else { right_index + 1 };
    }

    min_id
}

#[derive(Debug)]
pub enum ParseError {
    IO(std::io::Error),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "{}", err.to_string()),
            Self::ParseInt(err) => write!(f, "{}", err.to_string()),
            Self::ParseFloat(err) => write!(f, "{}", err.to_string()),
        }
    }
}

impl std::error::Error for ParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Self::IO(ref err) => Some(err),
            Self::ParseInt(ref err) => Some(err),
            Self::ParseFloat(ref err) => Some(err),
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<ParseIntError> for ParseError {
    fn from(err: ParseIntError) -> Self {
        Self::ParseInt(err)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(err: ParseFloatError) -> Self {
        Self::ParseFloat(err)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;
    use std::time::Instant;
    use geoutils::{Distance, Location};
    use log::{debug, trace};
    use rand::{Rng, thread_rng};
    use crate::data::graph::{Graph, Node};
    use crate::init_logging;
    use crate::utils::test_setup;

    /// Baba Hotel, ich schwör!!
    const RADISSON_BLU_HOTEL: (f64, f64) = (53.074448, 8.805105);

    #[test]
    fn test_offsets() {
        init_logging();

        let graph = &test_setup::GRAPH;

        let mut rng = thread_rng();
        let rand_id = rng.gen_range(0..graph.num_nodes);

        let outgoing_edges = graph.get_outgoing_edges(rand_id);
        for edge in outgoing_edges {
            assert_eq!(edge.src, rand_id, "Expected source: {}, got: {} via edge offsets",
                       rand_id, edge.src);
        }

        let mut offsets_clone = graph.offsets.clone();
        offsets_clone.sort();
        assert_eq!(offsets_clone, graph.offsets, "Offsets are not in ascending order");
    }

    fn get_nearest_node_naive(nodes: &Vec<&Node>, id_filter: &HashSet<usize>, lat: f64, lon: f64) -> usize {
        let location = Location::new(lat, lon);

        let mut min_dist = Distance::from_meters(f64::MAX);
        let mut min_id = 0;

        for (id, node) in nodes.iter().enumerate() {
            if !id_filter.contains(&id) {
                let node_loc = Location::new(node.lat, node.lon);
                let dist = location.haversine_distance_to(&node_loc);
                if dist.meters() < min_dist.meters() {
                    trace!("Updating minimum distance so far from: {} for node: {} to: {} for node: {}",
                        min_dist, min_id, dist, id);
                    min_dist = dist;
                    min_id = id;
                }
            }
        }

        min_id
    }

    #[test]
    fn test_nearest_node() {
        init_logging();

        let graph = &test_setup::GRAPH;

        let (lat, lon) = RADISSON_BLU_HOTEL;
        let location = Location::new(lat, lon);

        let start = Instant::now();
        let actual = graph.get_nearest_node(lat, lon);
        let elapsed = start.elapsed().as_millis();
        debug!("Efficient implementation took {} ms", elapsed);

        let start = Instant::now();
        let id_filter = graph.sights.iter().map(|sight| sight.node_id)
            //.merge(graph.nodes.iter().filter(|node| graph.get_degree(node.id) > 0)
            //    .map(|node| node.id))
            .collect();
        let expected = get_nearest_node_naive(&graph.nodes.iter().collect(),
                                              &id_filter, lat, lon);
        let elapsed = start.elapsed().as_millis();
        debug!("Naive implementation took {} ms", elapsed);


        let actual_node = graph.get_node(actual);
        let expected_node = graph.get_node(expected);
        let actual_loc = Location::new(actual_node.lat, actual_node.lon);
        let expected_loc = Location::new(expected_node.lat, expected_node.lon);
        let actual_dist = location.haversine_distance_to(&actual_loc);
        let expected_dist = location.haversine_distance_to(&expected_loc);

        assert_eq!(actual, expected, "Expected nearest node: {} with dist: {}, got: {} with dist: {} from efficient implementation",
                   expected, expected_dist, actual, actual_dist);
    }
}
