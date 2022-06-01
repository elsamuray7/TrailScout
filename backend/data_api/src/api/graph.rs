use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::num::{ParseFloatError, ParseIntError};
use osmpbf::{ElementReader, Element};

#[derive(Debug)]
enum ParseError {
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

/// Struct to hold Bounding Box of a circuit around coordinates
pub struct BoundingBox {
    pub min_lat: f64,
    pub max_lat: f64,
    pub min_lon: f64,
    pub max_lon: f64,
}

/// A graph node with id, tags, latitude, longitude and info
pub struct Node {
    id: usize,
    tags: Vec<(String, String)>,
    lat: f64,
    lon: f64,
    info: String,
}

/// An directed graph edge with source, target and distance
pub struct Edge {
    pub src: usize,
    pub tgt: usize,
    pub dist: usize,
}

/// A graph sight node with nearest node id, latitude, longitude, tags and info
pub struct Sight {
    id: usize,
    lat: f64,
    lon: f64,
    tags: Vec<(String, String)>,
    info: String,
}

/// A directed graph with nodes, edges, offsets and sights
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    offsets: Vec<usize>, // TODO check if pub needed or pub (crate)
    num_nodes: usize,
    num_edges: usize,
    sights: Vec<Sight>,
}

impl Graph {
    /// Create a new directed graph without any nodes or edges
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            offsets: Vec::new(),
            num_nodes: 0,
            num_edges: 0,
            sights: Vec::new()
        }
    }

    /// Parse graph data (nodes, edges, ...) from a file into a directed graph
    fn parse_graph(&mut self, graph_file_path: &str) -> Result<(), ParseError> {
        todo!(parse osm graph creator output into a graph);
        Ok(());
    }

    /// Create a graph from a file that contains graph data (nodes, edges, ...)
    pub fn from_file(file_path: &str) -> Self {
        let mut graph = Graph::new();
        match graph.parse_graph(file_path) {
            Ok(_) => (),
            Err(err) => panic!("Failed to create graph from files at {}: {}", file_path,
                               err.to_string())
        }
        graph
    }

    /// Get the number of outgoing edges of the node with id `node_id`
    pub fn get_degree(&self, node_id: usize) -> usize {
        self.offsets[node_id + 1] - self.offsets[node_id]
    }

    /// Get the nearest node to specific coordinates (lat / lon)
    pub fn get_nearest_node(&self, lat: f64, lon: f64) -> usize {
        todo!(return nearest node to these cooridnates)
    }

    /// Get outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_id: usize) -> &[Edge] {
        &self.edges[self.offsets[node_id]..self.offsets[node_id+1]]
    }

    /// Get all sights within a radius around coordinates (lat/lon)
    pub fn get_sights_from(&self, lat: f64, lon: f64, radius: f64) -> Vec<Sight> {
        todo!(get min/max lat/lon from cooridnates and radius, then sort sights by lat, get slice, sort by lon, get slice, return slice)
    }
}

/// Get the minimum and maximum latitude and longitude from given coordinates and a radius around it
fn get_bounding_box(lat: f64, lon: f64, radius: f64) -> BoundingBox {
    todo!(calculate the bounding box of a circle)
}