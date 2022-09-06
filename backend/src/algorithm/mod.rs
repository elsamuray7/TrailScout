pub mod greedy;
pub mod sa_lin_yu;

use std::collections::HashMap;
use chrono::{DateTime, Duration, NaiveDateTime, SecondsFormat, Utc};
use crate::data::graph::{Category, Graph, Node, Sight};
use serde::{Serialize, Deserialize, Serializer};
use derive_more::{Display, Error};
use opening_hours_syntax::rules::RuleKind;
use crate::algorithm::greedy::GreedyAlgorithm;
use crate::algorithm::sa_lin_yu::SimAnnealingLinYu;

/// Type alias for a mapping from node id's to scores, where the nodes represent sights / tourist
/// attractions
type ScoreMap = HashMap<usize, (usize, Category)>;

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
    category: String, // TODO never used
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
    End(EndSector<'a>),
}

/// Helper function to serialize date times as RFC 3339 (ISO 8601) date time strings
fn serialize_date_time<S>(date_time: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
    serializer.serialize_str(date_time.to_rfc3339().as_str())
}

/// Concrete representation of a route sector
///
/// # Fields
/// * `time_of_arrival` - The time of arrival at the sight
/// * `service_start_time` - The time at which the service at the sight can start, i.e. the
/// first time when the sight is open after the arrival
/// * `service_end_time` - The time at which the service at the sight ends, either because the
/// estimated duration of stay has been reached or because the sight closes
/// * `sight` - The target sight of this sector
/// * `nodes` - A vector containing a sequence of nodes from the sectors source to its
/// target sight (both inclusive)
#[derive(Serialize, Debug)]
pub struct Sector<'a> {
    #[serde(serialize_with = "serialize_date_time")]
    time_of_arrival: DateTime<Utc>,
    #[serde(serialize_with = "serialize_date_time")]
    service_start_time: DateTime<Utc>,
    #[serde(serialize_with = "serialize_date_time")]
    service_end_time: DateTime<Utc>,
    sight: &'a Sight,
    nodes: Vec<&'a Node>,
}

impl<'a> Sector<'a> {
    /// Creates a new route sector with a target sight
    ///
    /// # Arguments
    /// * `start_time` - The start time of the trip or hike
    /// * `used_time_budget` - The number of seconds passed since the start of the trip
    /// * `sight_travel_time` - The number of seconds to travel from the sectors source node to its
    /// target sight
    /// * `wait_time` - The number of seconds to wait until the service at the sight can start
    /// * `service_time` - The number of seconds to spend at the sectors target sight
    fn new(start_time: &DateTime<Utc>, used_time_budget: i64, sight_travel_time: i64, wait_time: i64,
           service_time: i64, sight: &'a Sight, nodes: Vec<&'a Node>) -> Self {
        let curr_time = *start_time + Duration::seconds(used_time_budget);
        let time_of_arrival = curr_time + Duration::seconds(sight_travel_time);
        let service_start_time = time_of_arrival + Duration::seconds(wait_time);
        let service_end_time = service_start_time + Duration::seconds(service_time);
        Self {
            time_of_arrival,
            service_start_time,
            service_end_time,
            sight,
            nodes,
        }
    }
}

/// Concrete representation of a route end sector
///
/// # Fields
/// * `time_of_arrival` - The time of arrival at the target node
/// * `nodes` - A vector containing a sequence of nodes from the sectors source to its
/// target node (both inclusive)
#[derive(Serialize, Debug)]
pub struct EndSector<'a> {
    #[serde(serialize_with = "serialize_date_time")]
    time_of_arrival: DateTime<Utc>,
    nodes: Vec<&'a Node>,
}

impl<'a> EndSector<'a> {
    /// Creates a new route end sector
    ///
    /// # Arguments
    /// * `start_time` - The start time of the trip or hike
    /// * `used_time_budget` - The number of seconds passed since the start of the trip
    /// * `tgt_travel_time` - The number of seconds to travel from the sectors source node to its
    /// target node
    fn new(start_time: &DateTime<Utc>, used_time_budget: i64, tgt_travel_time: i64,
           nodes: Vec<&'a Node>) -> Self {
        let curr_time = *start_time + Duration::seconds(used_time_budget);
        let time_of_arrival = curr_time + Duration::seconds(tgt_travel_time);
        Self {
            time_of_arrival,
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

/// Compute wait and service time for given sight based on the already used time budget
fn compute_wait_and_service_time(start_time: &DateTime<Utc>, sight: &Sight, used_time_budget: i64) -> Option<(i64, i64)> {
    let time_window = sight.opening_hours();

    // Determine current time (after given used time budget)
    let curr_time: NaiveDateTime = (*start_time + Duration::seconds(used_time_budget)).naive_utc();

    // Determine the sights current open state
    // TODO handle DateLimitExceeded error properly
    let curr_state = time_window.state(curr_time)
        .expect("Failed to determine open state");

    // Initialize closure for computing the sights service time
    let compute_service_time = |close_time: NaiveDateTime| {
        let possible_service_time = close_time.signed_duration_since(curr_time)
            .num_seconds();
        sight.duration_of_stay_secs().min(possible_service_time)
    };

    // Determine wait and service time based on the sights current open state
    match curr_state {
        RuleKind::Open | RuleKind::Unknown => {
            match time_window.next_change(curr_time) {
                Ok(close_time) => {
                    let service_time = if !time_window.is_closed(close_time) {
                        // log::trace!("Unknown time window change at {close_time}: {}",
                        //     &sight.opening_hours);
                        sight.duration_of_stay_secs()
                    } else {
                        compute_service_time(close_time)
                    };
                    Some((0, service_time))
                }
                _ => Some((0, sight.duration_of_stay_secs()))
            }
        }
        RuleKind::Closed => {
            match time_window.next_change(curr_time) {
                Ok(open_time) => {
                    // if !time_window.is_open(open_time) {
                    //     log::trace!("Unknown time window change at {open_time}: {}",
                    //         &sight.opening_hours);
                    // }
                    let wait_time = open_time.signed_duration_since(curr_time).num_seconds();
                    match time_window.next_change(open_time) {
                        Ok(close_time) => {
                            let service_time = if !time_window.is_closed(close_time) {
                                // log::trace!("Unknown time window change at {close_time}: {}",
                                //     &sight.opening_hours);
                                sight.duration_of_stay_secs()
                            } else {
                                compute_service_time(close_time)
                            };
                            Some((wait_time, service_time))
                        }
                        _ => Some((wait_time, sight.duration_of_stay_secs()))
                    }
                }
                _ => {
                    // Closed forever?
                    // log::trace!("Time window never opens after {curr_time}: {}",
                    //     &sight.opening_hours);
                    None
                }
            }
        }
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
    /// Error indicating that no reachable sights have been found in the requested area or in the
    /// area that can be traveled in the requested time interval
    #[display(fmt = "No sights found in area that are reachable within time interval")]
    NoSightsFound,
    /// Error indicating that an unknown category was assigned a preference
    #[display(fmt = "Unknown category name: {}", unknown_name)]
    UnknownCategory {
        unknown_name: String,
    },
    /// Error indicating that no route between two nodes could be determined
    #[display(fmt = "No route found from node {} to {}", from, to)]
    NoRouteFound {
        from: usize,
        to: usize,
    },
    /// Error indicating that an algorithm has been requested without category or sight preferences
    #[display(fmt = "No preferences for categories or sights provided")]
    NoPreferencesProvided,
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use crate::algorithm::{_Algorithm, Area, RouteSector, SightCategoryPref, UserPreferences};
    use crate::algorithm::greedy::GreedyAlgorithm;
    use crate::data::graph::Category;
    use crate::init_logging;
    use crate::utils::test_setup;

    /// Baba Hotel, ich schwör!!
    const RADISSON_BLU_HOTEL: Area = Area {
        lat: 53.074448,
        lon: 8.805105,
        radius: 1000.0,
    };

    #[test]
    fn test_greedy() {
        init_logging();

        let graph = &test_setup::GRAPH;

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
                sights: vec![],
            }).unwrap();
        let route = algo.compute_route()
            .expect("Error during route computation");

        // Route should not be empty
        assert!(route.len() > 0, "Route is empty");

        // Route should only contain sectors that include sights with categories restaurant,
        // sightseeing or nightlife
        let invalid_category = |category: &Category| {
            *category != Category::Restaurants && *category != Category::Sightseeing &&
                *category != Category::Nightlife
        };
        let sector = route.iter().find(|&sector| match sector {
            RouteSector::Start(sector) => invalid_category(&sector.sight.category),
            RouteSector::Intermediate(sector) => invalid_category(&sector.sight.category),
            _ => false,
        });
        assert!(sector.is_none(), "Route includes sight with category not in user preferences");

        // The number of seconds between the time of arrival at the start sector and end sector
        // should not exceed the available time budget
        let route_end_time = match &route.last().unwrap() {
            RouteSector::End(end_sector) => end_sector.time_of_arrival,
            _ => panic!("Last sector must be end sector")
        };
        let route_time_budget = route_end_time.signed_duration_since(start_time)
            .num_seconds();
        let avail_time_budget = end_time.signed_duration_since(start_time).num_seconds();
        assert!(route_time_budget <= avail_time_budget, "Route time budget exceeds available budget");
    }
}