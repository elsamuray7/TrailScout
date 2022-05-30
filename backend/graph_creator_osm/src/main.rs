use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::num::{ParseFloatError, ParseIntError};
use osmpbf::{ElementReader, Element};

/*
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

 */

/// An undirected graph edge between two nodes a and b
struct Edge {
    a: usize,
    b: usize,
    dist: usize,
    edge_type: String,
    maxspeed: String,
}

/// A graph node with id, latitude and longitude
struct Node {
    id: usize,
    tags: Vec<(String, String)>,
    lat: f64,
    lon: f64,
    info: String,
}

/// An undirected graph with nodes and edges
struct Graph {
    //meta: String,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    num_nodes: usize,
    num_edges: usize,
}

impl Graph {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            //offsets: Vec::new(),
            num_nodes: 0,
            num_edges: 0,
        }
    }

    fn parse_graph (&mut self, graph_file_path: &str) -> Result<(), io::Error> {
        let reader = ElementReader::from_path(graph_file_path)?;

        reader.for_each(|element| {
            if let Element::Node(n) = element {
                let mut node = Node {
                    id: n.id() as usize,
                    tags: vec![],
                    lat: n.lat(),
                    lon: n.lon(),
                    info: "".to_string()
                };
                for (key, value) in n.tags() {
                    node.tags.push((key.parse().unwrap(), value.parse().unwrap()));
                }
                self.nodes.push(node);
                self.num_nodes += 1;
            }
            /*else if let Element::Way(_) = element {
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
                self.edges.push(edge);
            }

             */
        })?;

        Ok(())
    }

    fn write_graph(&mut self, graph_file_path_out: &str) -> std::io::Result<()> {
        let file = File::create(graph_file_path_out)?;
        let mut file = LineWriter::new(file);

        //file.write((format!("{}", self.meta)).as_bytes())?;
        //file.write((format!("{}\n", self.num_nodes)).as_bytes())?;
        //file.write((format!("{}\n", self.new_num_edges)).as_bytes())?;

        for node in &self.nodes {
            file.write((format!("{} {} {} {}", node.id, node.lat, node.lon, node.info)).as_bytes())?;
            for (key, value) in &node.tags {
                file.write((format!(" {} {}\n", key, value)).as_bytes())?;
            }
        }
/*
        for (a, b, dist, edge_type, maxspeed) in &self.new_edges {
            file.write((format!("{} {} {} {} {}\n", a, b, dist, edge_type, maxspeed)).as_bytes())?;
        }
*/
        Ok(())
    }
}

fn main() -> Result<(), io::Error> {
    let in_graph = "C:/Users/Acer/Documents/EnProFMI2022/backend/graph_creator_osm/osm_graphs/bremen-latest.osm.pbf";
    let out_graph = "C:/Users/Acer/Documents/EnProFMI2022/backend/graph_creator_osm/osm_graphs/bremen-latest.fmi";

    let mut graph = Graph::new();
    Graph::parse_graph(&mut graph, &in_graph);
    Graph::write_graph(&mut graph, &out_graph);

    Ok(())
}