use std::collections::HashMap;
use std::fmt::Formatter;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::num::{ParseFloatError, ParseIntError};
use serde::{Serialize};
use serde_enum_str::{Deserialize_enum_str, Serialize_enum_str};

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
    /// Create a new graph without any nodes, edges or sights
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            offsets: Vec::new(),
            num_nodes: 0,
            num_edges: 0,
            sights: Vec::new(),
            num_sights: 0
        }
    }

    /// Parse graph data (in particular, nodes, edges and sights) from a file and create a new
    /// graph from it
    pub fn parse_from_file(graph_file_path: &str) -> Result<Self, ParseError> {
        let mut graph = Graph::new();
        let graph_file = File::open(graph_file_path)?;
        let graph_reader = BufReader::new(graph_file);

        let mut lines = graph_reader.lines();
        let mut line_no = 0;

        graph.num_nodes = lines.next()
            .expect("Unexpected EOF while parsing number of nodes")?
            .parse()?;
        graph.num_sights = lines.next()
            .expect("Unexpected EOF while parsing number of sights")?
            .parse()?;
        graph.num_edges = lines.next()
            .expect("Unexpected EOF while parsing number of edges")?
            .parse()?;
        line_no += 3;

        graph.nodes.reserve_exact(graph.num_nodes);
        for i in 0..graph.num_nodes {
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
            graph.nodes.push(node);
        }

        graph.sights.reserve_exact(graph.num_sights);
        for i in 0..graph.num_sights {
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
            graph.sights.push(sight);
        }

        let mut last_src: i64 = -1;
        let mut offset: usize = 0;
        graph.edges.reserve_exact(graph.num_edges);
        graph.offsets.resize(graph.num_nodes + 1, 0);
        for _ in 0..graph.num_edges {
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

            if edge.src as i64 > last_src {
                for j in (last_src + 1) as usize..=edge.src {
                    graph.offsets[j] = offset;
                }
                last_src = edge.src as i64;
            }
            offset += 1;

            graph.edges.push(edge);
        }
        graph.offsets[graph.num_nodes] = graph.num_edges;

        Ok(graph)
    }

    /// Returns a reference to the vector containing all nodes in this graph
    pub fn nodes(&self) -> &Vec<Node> {
        &self.nodes
    }

    /// Get the node with id `node_id`
    pub fn get_node(&self, node_id: usize) -> &Node {
        &self.nodes[node_id]
    }

    /// Get the nearest node to a given coordinate (latitude / longitude)
    pub fn get_nearest_node(&self, lat: f64, lon: f64) -> usize {
        // TODO compute nearest node to given coordinate
        let mut min_dist = usize::MAX;
        let mut min_id = self.nodes[0].id;
        for (id, node) in self.nodes.iter().enumerate() {
            let dist = calc_dist(lat, lon, node.lat, node.lon);
            if dist < min_dist {
                min_dist = dist;
                min_id = id;
            }
        }
        min_id
    }

    /// Get the number of outgoing edges of the node with id `node_id`
    pub fn get_degree(&self, node_id: usize) -> usize {
        self.offsets[node_id+1] - self.offsets[node_id]
    }

    /// Get all outgoing edges of a particular node
    pub fn get_outgoing_edges(&self, node_id: usize) -> &[Edge] {
        &self.edges[self.offsets[node_id]..self.offsets[node_id+1]]
    }

    /// Get all outgoing edges of a particular node where the edge target lies within given area
    pub fn get_outgoing_edges_in_area(&self, node_id: usize, lat: f64, lon: f64, radius: f64) -> Vec<&Edge> {
        let out_edges = self.get_outgoing_edges(node_id);
        out_edges.iter()
            .filter(|&edge| {
                let tgt_node = self.get_node(edge.tgt);
                // TODO check whether target node lies in area
                true
            })
            .collect()
    }

    /// Get all sights within a circular area, specified by `radius`, around a given coordinate
    /// (latitude / longitude)
    pub fn get_sights_in_area(&self, lat: f64, lon: f64, radius: f64) -> HashMap<usize, &Sight> {
        /*
        TODO
            - get bbox of area around coordinate
            - get slice of sights within min/max latitude of bbox, e.g. with binary search
            (precondition: sights sorted by latitude, should already be the case in graph
            creator output file)
            - create mutable vector with fetched sights
            - sort sights by longitude
            - get slice of sights within min/max longitude of bbox, e.g. with binary search
            - return new vector with fetched sights
         */
        self.sights.iter()
            .map(|sight| (sight.node_id, sight))
            .collect()
    }
}

/// Calculates the distance between two given coordinates (latitude / longitude) in metres. TODO make metre changeable later?
pub(crate) fn calc_dist(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> usize {
    let r: f64 = 6371000.0;

    let d_lat: f64 = (lat2 - lat1).to_radians();
    let d_lon: f64 = (lon2 - lon1).to_radians();
    let lat1: f64 = lat1.to_radians();
    let lat2: f64 = lat2.to_radians();

    let a: f64 = ((d_lat/2.0).sin()) * ((d_lat/2.0).sin()) + ((d_lon/2.0).sin()) * ((d_lon/2.0).sin()) * (lat1.cos()) * (lat2.cos());
    let c: f64 = 2.0 * ((a.sqrt()).atan2((1.0-a).sqrt()));

    (r * c) as usize
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
