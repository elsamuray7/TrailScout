pub mod greedy;

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::data::graph::{Graph, Node, Sight};
use serde::{Serialize, Deserialize};
use crate::algorithm::greedy::GreedyAlgorithm;

/// Type alias for a mapping from node id's to scores, where the nodes represent sights / tourist
/// attractions
type ScoreMap = HashMap<usize, usize>;

/// Circular area around a geographic coordinate
#[derive(Deserialize)]
pub struct Area {
    lat: f64,
    lon: f64,
    radius: f64,
}

/// User preference for a sight category
#[derive(Deserialize)]
pub struct SightCategoryPref {
    name: String,
    pref: usize,
}

/// User preference for a specific sight
#[derive(Deserialize)]
pub struct SightPref {
    id: usize,
    category: String,
    pref: usize,
}

/// User preferences for sights and sight categories
#[derive(Deserialize)]
pub struct UserPreferences {
    categories: Vec<SightCategoryPref>,
    sights: Vec<SightPref>,
}

/// A sector within a route
#[derive(Serialize, Debug)]
#[serde(tag = "type")]
pub enum RouteSector<'a> {
    /// The first sector in a route
    Start(Sector<'a>),
    /// An intermediate sector that has a predecessor and a successor
    Intermediate(Sector<'a>),
    /// The last sector in a route
    End(Sector<'a>),
}

/// Concrete representation of a route sector
#[derive(Serialize, Debug)]
pub struct Sector<'a> {
    time_budget: usize,
    sight: Option<&'a Sight>,
    nodes: Vec<&'a Node>,
}

impl<'a> Sector<'a> {
    /// Creates a new route sector
    ///
    /// # Arguments
    /// * `time_budget` - The required time budget in seconds to travel from the sectors source
    /// to its target node
    /// * `nodes` - A vector containing a sequence of nodes from the sectors source to its
    /// target node (both inclusive)
    fn new(time_budget: usize, nodes: Vec<&'a Node>) -> Self {
        Self {
            time_budget,
            sight: None,
            nodes,
        }
    }

    /// Creates a new route sector with a target sight
    ///
    /// # Arguments
    /// * `time_budget` - The required time budget in seconds to travel from the sectors source
    /// to its target node
    /// * `sight` - The target sight of this sector
    /// * `nodes` - A vector containing a sequence of nodes from the sectors source to its
    /// target node (both inclusive)
    fn with_sight(time_budget: usize, sight: &'a Sight, nodes: Vec<&'a Node>) -> Self {
        Self {
            time_budget,
            sight: Some(sight),
            nodes,
        }
    }
}

/// Type alias for a vector of route sectors that form a contiguous route
pub type Route<'a> = Vec<RouteSector<'a>>;

/// Algorithm trait to be implemented by concrete algorithm implementations
trait _Algorithm<'a> {
    /// Create a new algorithm instance
    ///
    /// # Arguments
    /// * `graph` - A reference to the graph on which to run the algorithm
    /// * `start_time` - The intended start time of the walk
    /// * `end_time` - The intended end time of the walk
    /// * `walking_speed_mps` - The walking speed in meters per second
    /// * `area` - The area in which the walking route should lie
    /// * `user_prefs` - The users preferences for sight categories and sights, respectively
    fn new(graph: &'a Graph,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           walking_speed_mps: f64,
           area: Area,
           user_prefs: UserPreferences) -> Self where Self: Sized;

    /// Compute a route on a graph that visits tourist attractions in a specific area based on
    /// user preferences for these tourist attractions
    fn compute_route(&self) -> Route;

    /// Returns a reference to this concrete implementation of the `_Algorithm` trait
    /// as a generic trait object
    fn as_algorithm(&'a self) -> &'a dyn _Algorithm where Self: Sized {
       self as &dyn _Algorithm
    }
}

pub enum Algorithm<'a> {
    Greedy(GreedyAlgorithm<'a>),
}

impl<'a> Algorithm<'a> {
    /// Create a new algorithm instance with the provided `algorithm_name`
    ///
    /// # Arguments
    /// * `algorithm_name` - The name of the algorithm implementation
    /// * `graph` - A reference to the graph on which to run the algorithm
    /// * `start_time` - The intended start time of the walk
    /// * `end_time` - The intended end time of the walk
    /// * `walking_speed_mps` - The walking speed in meters per second
    /// * `area` - The area in which the walking route should lie
    /// * `user_prefs` - The users preferences for sight categories and sights, respectively
    pub fn from_name(algorithm_name: &str,
                 graph: &'a Graph,
                 start_time: DateTime<Utc>,
                 end_time: DateTime<Utc>,
                 walking_speed_mps: f64,
                 area: Area,
                 user_prefs: UserPreferences) -> Self {
        match algorithm_name {
            GreedyAlgorithm::ALGORITHM_NAME => Algorithm::Greedy(GreedyAlgorithm::new(
                graph, start_time, end_time, walking_speed_mps, area, user_prefs)),
            _ => panic!("Unknown algorithm")
        }
    }

    /// Compute a route on a graph that visits tourist attractions in a specific area based on
    /// user preferences for these tourist attractions
    pub fn compute_route(&self) -> Route {
        match self {
            Self::Greedy(inner) => inner.as_algorithm(),
        }.compute_route()
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;
    use chrono::{DateTime, Utc};
    use crate::algorithm::{_Algorithm, Area, RouteSector, SightCategoryPref, SightPref, UserPreferences};
    use crate::algorithm::greedy::GreedyAlgorithm;
    use crate::data::graph::{Category, Graph};
    use crate::data::osm_graph_creator::{parse_osm_data, write_graph_file};

    fn get_graph() -> std::io::Result<Graph> {
        let pbf_path = "./osm_graphs/bremen-latest.osm.pbf";
        let fmi_path = "./osm_graphs/bremen-latest.fmi";
        if !Path::new(fmi_path).exists() {
            let mut nodes = Vec::new();
            let mut edges = Vec::new();
            let mut sights = Vec::new();
            parse_osm_data(pbf_path, &mut nodes, &mut edges, &mut sights)?;
            write_graph_file(fmi_path, &mut nodes, &mut edges, &mut sights)?;
        }
        let graph = Graph::parse_from_file("./osm_graphs/bremen-latest.fmi")
            .expect("Failed to parse graph file");
        Ok(graph)
    }

    #[test]
    fn test_greedy() -> std::io::Result<()> {
        let graph = get_graph()?;

        let algo = GreedyAlgorithm::new(
            &graph,
            DateTime::parse_from_rfc3339("2022-06-29T00:00:00+01:00")
                .unwrap().with_timezone(&Utc),
            DateTime::parse_from_rfc3339("2022-07-01T00:00:00+01:00")
                .unwrap().with_timezone(&Utc),
            7.0 / 3.6,
            Area {
                lat: 53.14519850000001,
                lon: 8.8384274,
                radius: 5.0,
            },
            UserPreferences {
                categories: vec![SightCategoryPref { name: "Restaurants".to_string(), pref: 3 },
                                 SightCategoryPref { name: "Sightseeing".to_string(), pref: 5 },
                                 SightCategoryPref { name: "Nightlife".to_string(), pref: 4 }],
                sights: vec![SightPref { id: 1274147, category: "Sightseeing".to_string(), pref: 0 }],
            });
        let route = algo.compute_route();

        // Route should only contain sectors that include sights with categories restaurant,
        // sightseeing or nightlife
        let invalid_category = |category: &Category| {
            *category != Category::Restaurants && *category != Category::Sightseeing &&
                *category != Category::Nightlife
        };
        let sector = route.iter().find(|&sector| match sector {
            RouteSector::Start(sector) => invalid_category(&sector.sight.unwrap().category),
            RouteSector::Intermediate(sector) => invalid_category(&sector.sight.unwrap().category),
            _ => false,
        });
        assert!(sector.is_none());

        // Route should not contain a sector with sight node 1274147
        let sector = route.iter().find(|&sector| match sector {
            RouteSector::Start(sector) => sector.sight.unwrap().node_id == 1274147,
            RouteSector::Intermediate(sector) => sector.sight.unwrap().node_id == 1274147,
            _ => false,
        });
        assert!(sector.is_none());

        Ok(())
    }
}