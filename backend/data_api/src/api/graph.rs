use std::fmt::Formatter;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::num::{ParseFloatError, ParseIntError};

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
        // TODO compute bounding box of circular area
        todo!()
    }
}

/// A graph node located at a specific coordinate
pub struct Node {
    pub id: usize,
    pub lat: f64,
    pub lon: f64,
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
pub struct Edge {
    pub src: usize,
    pub tgt: usize,
    pub dist: usize,
}

/// Type alias for a vector containing sight tags with a key and value
pub type Tags = Vec<(String, String)>;

/// A sight node mapped on its nearest node
pub struct Sight {
    lat: f64,
    lon: f64,
    pub node_id: usize,
    pub tags: Tags,
    pub info: String,
}

/// A directed graph. In addition to nodes and edges, the definition also contains a set of sights
/// mapped on their nearest nodes, respectively.
pub struct Graph {
    pub nodes: Vec<Node>,
    // TODO check if pub needed or pub (crate)
    edges: Vec<Edge>,
    offsets: Vec<usize>,
    pub num_nodes: usize,
    pub num_edges: usize,
    sights: Vec<Sight>,
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
            sights: Vec::new()
        }
    }

    /// Parse graph data (in particular, nodes, edges and sights) from a file and create a new
    /// graph from it
    pub fn parse_from_file(file_path: &str) -> Result<Self, ParseError> {
        // TODO parse osm graph creator output into graph
        todo!()
    }

    /// Get the nearest node to a given coordinate (latitude / longitude)
    pub fn get_nearest_node(&self, lat: f64, lon: f64) -> usize {
        // TODO compute nearest node to given coordinate
        todo!()
    }

    /// Get the number of outgoing edges of the node with id `node_id`
    pub fn get_degree(&self, node_id: usize) -> usize {
        self.offsets[node_id+1] - self.offsets[node_id]
    }

    /// Get all outgoing edges of a particular node
    pub fn get_outgoing_edges(&self, node_id: usize) -> &[Edge] {
        &self.edges[self.offsets[node_id]..self.offsets[node_id+1]]
    }

    /// Get all sights within a circular area, specified by `radius`, around a given coordinate
    /// (latitude / longitude)
    pub fn get_sights_in_area(&self, lat: f64, lon: f64, radius: f64) -> Vec<Sight> {
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
        todo!()
    }
}