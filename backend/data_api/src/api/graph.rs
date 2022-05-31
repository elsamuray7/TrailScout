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

/// A graph node with id, tags, latitude, longitude and info
pub struct Node {
    id: usize,
    tags: Vec<(String, String)>,
    lat: f64,
    lon: f64,
    info: String,
}

/// A directed graph edge with source and target
#[derive(Debug, Serialize, Default)]
pub struct Edge {
    pub src: usize,
    pub tgt: usize,
    pub dist: usize,
}

/// A directed graph with nodes, edges and node offsets
#[derive(Debug, Serialize, Default)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub offsets: Vec<usize>,
    pub num_nodes: usize,
    pub num_edges: usize,
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
        }
    }

    /// Parse node and edge data from a file into a directed graph
    fn parse_graph(&mut self, graph_file_path: &str) -> Result<(), ParseError> {
        // TODO
        Ok(());
    }

    /// Get the number of outgoing edges of the node with id `node_id`
    pub fn get_degree(&self, node_id: usize) -> usize {
        self.offsets[node_id + 1] - self.offsets[node_id]
    }

    pub fn get_graph(minLat: f64, maxLat:f64, minLon: f64, maxLon:f64) -> Graph {
        let graph = self::Graph;
        return graph;
        // TODO
    }

    pub fn osmIDtoNodeID(&self, osm_id: usize) -> usize {
        return 0;
        // TODO
    }
}

