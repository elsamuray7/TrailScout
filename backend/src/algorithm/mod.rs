pub mod greedy;
pub mod sa_lin_yu;

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::data::graph::{Graph, Node, Sight};
use serde::{Serialize, Deserialize};
use derive_more::{Display, Error};
use crate::algorithm::greedy::GreedyAlgorithm;
use crate::algorithm::sa_lin_yu::SimAnnealingLinYu;

/// Type alias for a mapping from node id's to scores, where the nodes represent sights / tourist
/// attractions
type ScoreMap = HashMap<usize, usize>;

/// Multiplier for the relevant (reachable) radius to get the radius in which outgoing edges for
/// nodes should be retrieved
const EDGE_RADIUS_MULTIPLIER: f64 = 1.1;

/// Circular area around a geographic coordinate
#[derive(Deserialize)]
pub struct Area {
    lat: f64,
    lon: f64,
    radius: f64,
}

/// Maximum value for user sight preferences
const USER_PREF_MAX: usize = 5;

/// User preference for a sight category
#[derive(Deserialize)]
pub struct SightCategoryPref {
    name: String,
    pref: usize,
}

impl SightCategoryPref {
    /// Returns a valid preference value for this sight category
    fn get_valid_pref(&self) -> usize {
        self.pref.min(USER_PREF_MAX)
    }
}

/// User preference for a specific sight
#[derive(Deserialize)]
pub struct SightPref {
    id: usize,
    category: String,
    pref: usize,
}

impl SightPref {
    /// Returns a valid preference value for this sight
    fn get_valid_pref(&self) -> usize {
        self.pref.min(USER_PREF_MAX)
    }
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
    ///
    /// # Returns
    /// * an `Ok` containing a new algorithm instance in case of no errors, or
    /// * an `Err` containing an `AlgorithmError`, otherwise
    fn new(graph: &'a Graph,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           walking_speed_mps: f64,
           area: Area,
           user_prefs: UserPreferences) -> Result<Self, AlgorithmError> where Self: Sized;

    /// Compute a route on a graph that visits tourist attractions in a specific area based on
    /// user preferences for these tourist attractions
    ///
    /// # Returns
    /// * an `Ok` containing the computed route in case of no errors, or
    /// * an `Err` containing an `AlgorithmError`, otherwise
    fn compute_route(&self) -> Result<Route, AlgorithmError>;

    /// Outputs the score collected by a route computed by this algorithm
    fn get_collected_score(&self, route: &Route) -> usize;

    /// Returns a reference to this concrete implementation of the `_Algorithm` trait
    /// as a generic trait object
    fn as_algorithm(&'a self) -> &'a dyn _Algorithm where Self: Sized {
       self as &dyn _Algorithm
    }
}

pub enum Algorithm<'a> {
    Greedy(GreedyAlgorithm<'a>),
    SimAnnealing(SimAnnealingLinYu<'a>),
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
    ///
    /// # Returns
    /// * an `Ok` containing a new algorithm instance with the provided `algorithm_name`
    /// if such an algorithm exists, or
    /// * an `Err` containing an `AlgorithmError`, if the specified name is unknown
    pub fn from_name(algorithm_name: &str,
                     graph: &'a Graph,
                     start_time: DateTime<Utc>,
                     end_time: DateTime<Utc>,
                     walking_speed_mps: f64,
                     area: Area,
                     user_prefs: UserPreferences) -> Result<Self, AlgorithmError> {
        match algorithm_name {
            GreedyAlgorithm::ALGORITHM_NAME => Ok(Self::Greedy(GreedyAlgorithm::new(
                graph, start_time, end_time, walking_speed_mps, area, user_prefs)?)),
            SimAnnealingLinYu::ALGORITHM_NAME => Ok(Self::SimAnnealing(SimAnnealingLinYu::new(
                graph, start_time, end_time, walking_speed_mps, area, user_prefs)?)),
            unknown_name => Err(AlgorithmError::UnknownAlgorithm {
                unknown_name: unknown_name.to_string(),
            })
        }
    }

    /// Compute a route on a graph that visits tourist attractions in a specific area based on
    /// user preferences for these tourist attractions
    ///
    /// # Returns
    /// * an `Ok` containing the computed route in case of no errors, or
    /// * an `Err` containing an `AlgorithmError`, otherwise
    pub fn compute_route(&self) -> Result<Route, AlgorithmError> {
        match self {
            Self::Greedy(inner) => inner.as_algorithm(),
            Self::SimAnnealing(inner) => inner.as_algorithm(),
        }.compute_route()
    }
}

/// Error type of `algorithm` module
#[derive(Debug, Display, Error)]
pub enum AlgorithmError {
    /// Error indicating that an unknown algorithm has been requested
    #[display(fmt = "Unknown algorithm name: {}", unknown_name)]
    UnknownAlgorithm {
        unknown_name: String,
    },
    /// Error indicating that an algorithm has been requested with `end_time` before `start_time`,
    /// i.e., a negative time interval
    #[display(fmt = "End time is before start time")]
    NegativeTimeInterval,
    /// Error indicating that no route between two nodes could be determined
    #[display(fmt = "No route found from node {} to {}", from, to)]
    NoRouteFound {
        from: usize,
        to: usize,
    },
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use crate::algorithm::{_Algorithm, Area, RouteSector, SightCategoryPref, SightPref, UserPreferences};
    use crate::algorithm::greedy::GreedyAlgorithm;
    use crate::data::graph::{Category, Graph};
    use crate::init_logging;

    /// Baba Hotel, ich schwör!!
    const RADISSON_BLU_HOTEL: Area = Area {
        lat: 53.074448,
        lon: 8.805105,
        radius: 22.0,
    };

    #[test]
    fn test_greedy() {
        init_logging();

        let graph = Graph::parse_from_file("./tests_data/output/bremen-latest.fmi")
            .expect("Failed to parse graph file");

        let start_time = DateTime::parse_from_rfc3339("2022-07-01T10:00:00+01:00")
            .unwrap().with_timezone(&Utc);
        let end_time = DateTime::parse_from_rfc3339("2022-07-01T13:00:00+01:00")
            .unwrap().with_timezone(&Utc);
        let algo = GreedyAlgorithm::new(
            &graph,
            start_time,
            end_time,
            7.0 / 3.6,
            RADISSON_BLU_HOTEL,
            UserPreferences {
                categories: vec![SightCategoryPref { name: "Restaurants".to_string(), pref: 3 },
                                 SightCategoryPref { name: "Sightseeing".to_string(), pref: 5 },
                                 SightCategoryPref { name: "Nightlife".to_string(), pref: 4 }],
                sights: vec![SightPref { id: 1274147, category: "Sightseeing".to_string(), pref: 0 }],
            }).unwrap();
        let route = algo.compute_route()
            .expect("Error during route computation");

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

        let total_time_budget: usize = route.iter()
            .map(|sector|
                match sector {
                    RouteSector::Start(sector) => sector,
                    RouteSector::Intermediate(sector) => sector,
                    RouteSector::End(sector) => sector,
                }.time_budget
            )
            .sum();
        let actual_time_budget = end_time.timestamp() - start_time.timestamp();
        assert!((total_time_budget as i64) < actual_time_budget,
                "Used time budget: {}. Actual time budget: {}.",
                total_time_budget, actual_time_budget);
    }
}