use std::collections::{HashMap, HashSet};
use std::fmt::Formatter;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::num::{ParseFloatError, ParseIntError};
use geoutils::{Distance, Location};
use itertools::Itertools;
use log::{debug, trace};
use serde::Serialize;
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};
use pathfinding::prelude::*;

/// Bounding box of a circular area around a coordinate
struct BoundingBox {
    min_lat: f64,
    max_lat: f64,
    min_lon: f64,
    max_lon: f64,
}

impl BoundingBox {
    /// Create the bounding box of a circular area, specified by `radius`, around a given
    /// coordinate
    fn from_coordinate_and_radius(lat: f64, lon: f64, radius: f64) -> Self {
        /*
        TODO
         compute bounding box of circular area
         Get the distance between two nodes
         pub fn get_dist(&self, src_id: usize, tgt_id: usize) between two nodes
         */
        todo!()
    }
}

#[derive(Deserialize_enum_str, Serialize_enum_str, PartialEq, Eq, Debug)]
#[serde(rename_all = "PascalCase")]
pub enum Category {
    ThemePark,
    Swimming,
    PicnicBarbequeSpot,
    MuseumExhibition,
    Nature,
    Nightlife,
    Restaurants,
    Sightseeing,
    Shopping,
    Animals,
    Other
}

#[derive(Deserialize_enum_str, Serialize_enum_str, PartialEq, Debug)]
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
    Tertiary // Straßen, die Dörfer verbinden
}

/// A graph node located at a specific coordinate
#[derive(Debug, Serialize)]
pub struct Node {
    pub osm_id: usize,
    pub id: usize,
    pub lat: f64,
    pub lon: f64,
    pub info: String,
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
#[derive(Clone, Copy)]
pub struct Edge {
    pub(crate) osm_id: usize, // TODO delete later!
    pub osm_src: usize,
    pub osm_tgt: usize,
    /// The id of the edge's source node
    pub src: usize,
    /// The id of the edge's target node
    pub tgt: usize,
    /// The edge's weight, i.e., the distance between its source and target
    pub dist: usize,
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
#[derive(Debug, Serialize)]
pub struct Sight {
    pub osm_id: usize,
    pub node_id: usize,
    pub lat: f64,
    pub lon: f64,
    //pub tags: Tags,
    //pub info: String,
    pub category: Category,
}

/// A directed graph. In addition to nodes and edges, the definition also contains a set of sights
/// mapped on their nearest nodes, respectively.
pub struct Graph {
    nodes: Vec<Node>,
    // TODO check if pub needed or pub (crate)
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
        let graph_file = File::open(graph_file_path)?;
        let graph_reader = BufReader::new(graph_file);

        let mut lines = graph_reader.lines();
        let mut line_no = 0_usize;

        let num_nodes: usize = lines.next()
            .expect("Unexpected EOF while parsing number of nodes")?
            .parse()?;
        let num_sights: usize = lines.next()
            .expect("Unexpected EOF while parsing number of sights")?
            .parse()?;
        let num_edges: usize = lines.next()
            .expect("Unexpected EOF while parsing number of edges")?
            .parse()?;
        line_no += 3;

        let mut nodes = Vec::with_capacity(num_nodes);
        for i in 0..num_nodes {
            let line = lines.next()
                .expect(&format!("Unexpected EOF while parsing nodes in line {}", line_no))?;
            let mut split = line.split(" ");
            line_no += 1;
            split.next(); // id

            let node = Node {
                osm_id: 0,
                id: i,
                lat: split.next()
                    .expect(&format!("Unexpected EOL while parsing node latitude in line {}",
                                     line_no))
                    .parse()?,
                lon: split.next()
                    .expect(&format!("Unexpected EOL while parsing node longitude in line {}",
                                     line_no))
                    .parse()?,
                info: "".to_string()
            };
            nodes.push(node);
        }

        let mut sights = Vec::with_capacity(num_sights);
        for _ in 0..num_sights {
            let line = lines.next()
                .expect(&format!("Unexpected EOF while parsing nodes in line {}", line_no))?;
            let mut split = line.split(" ");
            line_no += 1;

            let sight = Sight {
                osm_id: 0,
                node_id: split.next()
                    .expect(&format!("Unexpected EOL while parsing sight node id in line {}",
                                     line_no))
                    .parse()?,
                lat: split.next()
                    .expect(&format!("Unexpected EOL while parsing sight latitude in line {}",
                                     line_no))
                    .parse()?,
                lon: split.next()
                    .expect(&format!("Unexpected EOL while parsing sight longitude in line {}",
                                     line_no))
                    .parse()?,
                category: split.next()
                    .expect(&format!("Unexpected EOL while parsing sight category in line {}",
                                     line_no))
                    .parse()
                    .unwrap(),
            };
            sights.push(sight);
        }

        let mut next_src: usize = 0;
        let mut offset: usize = 0;
        let mut edges = Vec::with_capacity(num_edges);
        let mut offsets = vec![0; num_nodes + 1];
        for _ in 0..num_edges {
            let line = lines.next()
                .expect(&format!("Unexpected EOF while parsing edges in line {}", line_no))?;
            let mut split = line.split(" ");
            line_no += 1;

            let edge = Edge {
                osm_id: 0,
                osm_src: 0,
                osm_tgt: 0,
                src: split.next()
                    .expect(&format!("Unexpected EOL while parsing edge source in line {}",
                                     line_no))
                    .parse()?,
                tgt: split.next()
                    .expect(&format!("Unexpected EOL while parsing edge target in line {}",
                                     line_no))
                    .parse()?,
                dist: split.next()
                    .expect(&format!("Unexpected EOL while parsing edge weight in line {}",
                                     line_no))
                    .parse()?,
            };

            if edge.src >= next_src {
                for j in next_src..=edge.src {
                    offsets[j] = offset;
                }
                next_src = edge.src + 1;
            }
            offset += 1;

            edges.push(edge);
        }
        for i in next_src..=num_nodes {
            offsets[i] = num_edges;
        }

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
    pub fn get_sights_in_area(&self, lat: f64, lon: f64, radius: f64) -> HashMap<usize, &Sight> {
        debug!("Computing sights in area: lat: {}, lon: {}, radius: {}", lat, lon, radius);

        let successors = |node: &Node|
            self.get_outgoing_edges_in_area(node.id, lat, lon, radius)
                .into_iter()
                .map(|edge| (self.get_node(edge.tgt), edge.dist))
                .collect::<Vec<(&Node, usize)>>();

        // Get all nodes that are reachable from the node with the lowest distance to the center
        let center_id = self.get_nearest_node(lat, lon);
        let reachable_nodes: HashSet<&Node> = dijkstra_all(
            &self.get_node(center_id),
            |node| successors(node))
            .into_keys()
            .collect();

        //estimate bounding box with 111111 meters = 1 longitude degree
        //use binary search to find the range of elements that should be considered
        let lower_bound = binary_search_sights_vector(&self.sights, lat - radius / 111111.0);
        let upper_bound = binary_search_sights_vector(&self.sights, lat + radius / 111111.0);

        let slice = &self.sights[lower_bound..upper_bound];

        let center = Location::new(lat, lon);
        let radius = Distance::from_meters(radius);
        //iterate through the slice and check every sight whether it's in the target circle
        let sights_in_area: HashMap<usize, &Sight> = slice.iter()
            .filter(|sight| {
                let location = Location::new(sight.lat, sight.lon);
                location.is_in_circle(&center, radius)
                    .expect("Could not determine whether sight lies in given area")
            })
            .filter(|sight| reachable_nodes.contains(self.get_node(sight.node_id)))
            .map(|sight| (sight.node_id, sight))
            .collect();
        debug!("Found {} reachable sights within the given area (of a total of {} sights)",
            sights_in_area.len(), self.sights.len());

        sights_in_area
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
pub fn get_nearest_node(nodes_sorted_by_lat: &Vec<&Node>, id_filter: &HashSet<usize>, lat: f64, lon: f64) -> usize {
    // Location to find the nearest node for
    let location = Location::new(lat, lon);

    // Search the index of the node with the closest latitude coordinate within the list of nodes
    let result = nodes_sorted_by_lat.binary_search_by(|n| n.lat.total_cmp(&lat));
    let found_index = result.unwrap_or_else(|index| index);
    debug!("Starting to search for nearest node at index: {} for latitude: {}", found_index, lat);

    let mut min_dist = Distance::from_meters(f64::MAX);
    let mut min_id = 0;

    // Iterate over the left and right neighbour indices to determine the nearest node
    let mut left_index = found_index;
    let mut right_index = found_index;
    loop {
        for i in [left_index, right_index] {
            let node = nodes_sorted_by_lat[i];

            // If the distance with the current nodes longitude set to the longitude of the
            // location is greater than the minimum distance so far, abort and output the node
            // with the found distance
            let node_loc_lon_aligned = Location::new(node.lat, location.longitude());
            // Use haversine distance here for more efficiency
            let minimum_possible_distance = location.haversine_distance_to(&node_loc_lon_aligned);
            if minimum_possible_distance.meters() >= min_dist.meters() {
                trace!("Minimum possible distance: {} for node: {} greater/equal to minimum distance so far: {} for node: {}",
                minimum_possible_distance, node.id, min_dist, min_id);
                return min_id;
            }

            // If the node is not a sight and has a smaller distance to the location than the
            // minimum distance found so far, update the minimum distance and the id of the nearest
            // node
            if !id_filter.contains(&node.id) {
                let node_loc = Location::new(node.lat, node.lon);
                // Use haversine distance here for more efficiency
                let dist = location.haversine_distance_to(&node_loc);
                if dist.meters() < min_dist.meters() {
                    trace!("Updating minimum distance so far from: {} for node: {} to: {} for node: {}",
                    min_dist, min_id, dist, node.id);
                    min_dist = dist;
                    min_id = node.id;
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
    use env_logger::Env;
    use geoutils::{Distance, Location};
    use itertools::Itertools;
    use log::{debug, trace};
    use pathfinding::prelude::dijkstra;
    use rand::{Rng, thread_rng};
    use crate::data::graph::{Graph, Node};

    /// Baba Hotel, ich schwör!!
    const RADISSON_BLU_HOTEL: (f64, f64) = (53.074448, 8.805105);

    #[test]
    fn test_reverse_edges() {
        let graph = Graph::parse_from_file("./osm_graphs/bremen-latest.fmi")
            .expect("Failed to parse graph file");

        let mut rng = thread_rng();

        let successors = |node: &Node|
            graph.get_outgoing_edges(node.id)
                .into_iter()
                .map(|edge| (graph.get_node(edge.tgt), edge.dist))
                .collect::<Vec<(&Node, usize)>>();

        for round in 0..50 {
            println!("Round {} / {}", round, 50);

            let rand_src = rng.gen_range(0..graph.num_nodes);
            let rand_tgt = rng.gen_range(0..graph.num_nodes);

            let dijkstra_result = dijkstra(&graph.get_node(rand_src),
                                           |node| successors(node),
                                           |node| node.id == rand_tgt);
            let rev_dijkstra_result = dijkstra(&graph.get_node(rand_tgt),
                                               |node| successors(node),
                                               |node| node.id == rand_src);

            match dijkstra_result {
                Some((_, dist)) => {
                    println!("Route from {} to {} exists", rand_src, rand_tgt);
                    assert!(rev_dijkstra_result.is_some(),
                            "Route between {} and {} is directed", rand_src, rand_tgt);
                    let (_, rev_dist) = rev_dijkstra_result.unwrap();
                    assert_eq!(dist, rev_dist, "Distances do not match: {} vs. {}", dist, rev_dist);
                },
                None => {
                    println!("No route from {} to {}", rand_src, rand_tgt);
                    assert!(rev_dijkstra_result.is_none(),
                            "Route between {} and {} is directed", rand_tgt, rand_src);
                }
            }
        }
    }

    #[test]
    fn test_offsets() {
        let graph = Graph::parse_from_file("./osm_graphs/bremen-latest.fmi")
            .expect("Failed to parse graph file");

        let mut rng = thread_rng();
        let rand_id = rng.gen_range(0..graph.num_nodes);

        let outgoing_edges = graph.get_outgoing_edges(rand_id);
        for edge in outgoing_edges {
            assert_eq!(edge.src, rand_id, "Expected source: {}, got: {} via edge offsets",
                       rand_id, edge.src);
        }
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
        let env = Env::default()
            .filter_or("TRAILSCOUT_LOG_LEVEL", "trace")
            .write_style_or("TRAILSCOUT_LOG_STYLE", "always");
        env_logger::try_init_from_env(env).ok();

        let graph = Graph::parse_from_file("./osm_graphs/bremen-latest.fmi")
            .expect("Failed to parse graph file");

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
