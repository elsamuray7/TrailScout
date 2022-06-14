use std::fmt::Formatter;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::num::{ParseFloatError, ParseIntError};

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

/// A graph node located at a specific coordinate
pub struct Node {
    pub id: usize,
    pub lat: f64,
    pub lon: f64,
    pub info: String,
}

/// A directed and weighted (dist) graph edge between a source (src) and a target (tgt) node
pub struct Edge {
    pub(crate) id: usize, // TODO delete later!
    pub src: usize,
    pub tgt: usize,
    pub dist: usize,
}

/// Type alias for a vector containing sight tags with a key and value
pub type Tags = Vec<(String, String)>; // TODO are tags needed or just categories

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
    pub offsets: Vec<usize>,
    pub num_nodes: usize,
    pub num_edges: usize,
    sights: Vec<Sight>,
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
    pub fn parse_from_file(&mut self, graph_file_path: &str) -> Result<Self, ParseError> {
        // TODO parse osm graph creator output into graph
        todo!()
        /*
        let graph_file = File::open(graph_file_path)?;
        let graph_reader = BufReader::new(graph_file);

        let mut lines = graph_reader.lines();
        let mut line_no = 0;

        loop {
            let line = lines.next()
                .expect(&format!("Unexpected EOF while parsing header in line {}", line_no))?;
            line_no += 1;

            self.meta.push_str(&line);
            self.meta.push_str("\n");

            if !line.starts_with("#") {
                break;
            }
        }

        self.num_nodes = lines.next()
            .expect("Unexpected EOF while parsing number of nodes")?
            .parse()?;
        self.num_edges = lines.next()
            .expect("Unexpected EOF while parsing number of edges")?
            .parse()?;
        line_no += 3;

        self.nodes.reserve_exact(self.num_nodes);
        for i in 0..self.num_nodes {
            let line = lines.next()
                .expect(&format!("Unexpected EOF while parsing nodes in line {}", line_no))?;
            let mut split = line.split(" ");
            line_no += 1;
            split.next(); // id

            let node = Node {
                id: i,
                id2: split.next()
                    .expect(&format!("Unexpected EOL while parsing node latitude in line {}",
                                     line_no))
                    .parse()?,
                lat: split.next()
                    .expect(&format!("Unexpected EOL while parsing node latitude in line {}",
                                     line_no))
                    .to_string(),
                lon: split.next()
                    .expect(&format!("Unexpected EOL while parsing node longitude in line {}",
                                     line_no))
                    .to_string(),
                elevation: split.next()
                    .expect(&format!("Unexpected EOL while parsing node latitude in line {}",
                                     line_no))
                    .to_string(),
            };
            self.nodes.push(node);
        }

        self.edges.reserve_exact(self.num_edges);
        let mut new_temp_edges: BTreeMap<(usize, usize), Vec<(usize, usize, usize, String, String)>> = BTreeMap::new();
        for _ in 0..self.num_edges {
            let line = lines.next()
                .expect(&format!("Unexpected EOF while parsing edges in line {}", line_no))?;
            let mut split = line.split(" ");
            line_no += 1;

            let edge = Edge {
                a: split.next()
                    .expect(&format!("Unexpected EOL while parsing edge source in line {}",
                                     line_no))
                    .parse()?,
                b: split.next()
                    .expect(&format!("Unexpected EOL while parsing edge target in line {}",
                                     line_no))
                    .parse()?,
                dist: split.next()
                    .expect(&format!("Unexpected EOL while parsing edge weight in line {}",
                                     line_no))
                    .parse()?,
                edge_type: split.next()
                    .expect(&format!("Unexpected EOL while parsing edge weight in line {}",
                                     line_no))
                    .to_string(),
                maxspeed: split.next()
                    .expect(&format!("Unexpected EOL while parsing edge weight in line {}",
                                     line_no))
                    .to_string(),
            };

            let min_vertex = edge.a.min(edge.b);
            let max_vertex = edge.a.max(edge.b);
            new_temp_edges.entry((min_vertex, max_vertex))
                .and_modify(|edges|{
                    if edge.dist < edges[0].2 {
                        edges[0].2 = edge.dist;
                        edges[1].2 = edge.dist;
                    }
                })
                .or_insert(vec![(edge.a, edge.b, edge.dist, edge.edge_type.clone(), edge.maxspeed.clone()), (edge.b, edge.a, edge.dist, edge.edge_type, edge.maxspeed)]);
        }

        self.new_edges.reserve_exact(new_temp_edges.len() * 2);
        for edge_values in new_temp_edges.values() {
            for (a, b, dist, edge_type, maxspeed) in edge_values {
                self.new_edges.push((*a, *b, *dist, edge_type.clone(), maxspeed.clone()));
            }
        }
        self.new_edges.sort_unstable_by(|e1, e2| {
            let id1 = e1.0;
            let id2 = e2.0;
            id1.cmp(&id2).then_with(||{
                let id1 = e1.1;
                let id2 = e2.1;
                id1.cmp(&id2)
            })
        });
        self.new_num_edges = self.new_edges.len();

        Ok(())

         */
    }

    /// Get the nearest node to a given coordinate (latitude / longitude)
    fn get_nearest_node(&self, lat: f64, lon: f64) -> usize {
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

/// Calculates the distance between two given coordinates (latitude / longitude) in metres. TODO make it changeable later?
pub(crate) fn calc_dist(lat1: f64, lon1: f64, lat2: f64, lon2: f64) {

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