pub mod greedy;
pub mod sa_lin_yu;

use std::collections::HashMap;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
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
#[derive(Deserialize, Clone)]
pub struct Area {
    lat: f64,
    lon: f64,
    radius: f64,
}

impl Area {
    /// Creates a new area instance from given coordinates and radius
    ///
    /// # Arguments
    /// * `lat` - The latitude coordinate
    /// * `lon` - The longitude coordinate
    /// * `radius` - The radius in meters
    pub fn from_coords_and_radius(lat: f64, lon: f64, radius: f64) -> Self {
        Self {
            lat,
            lon,
            radius,
        }
    }
}

/// Maximum value for user sight preferences
const USER_PREF_MAX: usize = 5;

/// User preference for a sight category
#[derive(Deserialize, Clone)]
pub struct SightCategoryPref {
    category: Category,
    pref: usize,
}

impl SightCategoryPref {
    /// Creates a new sight category preference
    ///
    /// # Arguments
    /// * `category` - The sight category
    /// * `pref` - The preference value between 1 (very low) and 5 (very high). 0 means
    /// that the category should be ignored.
    pub fn new(category: Category, pref: usize) -> Self {
        Self {
            category,
            pref,
        }
    }

    /// Returns a valid preference value for this sight category
    fn get_valid_pref(&self) -> usize {
        self.pref.min(USER_PREF_MAX)
    }
}

/// User preference for a specific sight
#[derive(Deserialize, Clone)]
pub struct SightPref {
    id: usize,
    pref: usize,
}

impl SightPref {
    /// Creates a new sight preference
    ///
    /// # Arguments
    /// * `id` - The sight node id
    /// * `pref` - The preference value between 1 (very low) and 5 (very high). 0 means
    /// that this particular sight should be ignored.
    pub fn new(id: usize, pref: usize) -> Self {
        Self {
            id,
            pref,
        }
    }

    /// Returns a valid preference value for this sight
    fn get_valid_pref(&self) -> usize {
        self.pref.min(USER_PREF_MAX)
    }
}

/// User preferences for sights and sight categories
#[derive(Deserialize, Clone)]
pub struct UserPreferences {
    categories: Vec<SightCategoryPref>,
    sights: Vec<SightPref>,
}

impl UserPreferences {
    /// Creates new user preferences from given category and sight preferences
    ///
    /// # Arguments
    /// * `category_prefs` - The sight category preferences
    /// * `sight_prefs` - The sight preferences
    pub fn from_category_and_sight_prefs(category_prefs: Vec<SightCategoryPref>, sight_prefs: Vec<SightPref>) -> Self {
        Self {
            categories: category_prefs,
            sights: sight_prefs,
        }
    }
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
    /// List of available algorithms
    const AVAILABLE_ALGORITHMS: [&'static str; 2] = [
        GreedyAlgorithm::ALGORITHM_NAME,
        SimAnnealingLinYu::ALGORITHM_NAME
    ];

    /// Returns a list of available algorithms specified by their respective names
    pub fn available_algorithms() -> &'static [&'static str] {
        &Self::AVAILABLE_ALGORITHMS
    }

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

    /// Returns a reference to the underlying implementation of the `_Algorithm` trait
    /// as a generic trait object
    fn inner(&self) -> &dyn _Algorithm {
        match self {
            Self::Greedy(inner) => inner.as_algorithm(),
            Self::SimAnnealing(inner) => inner.as_algorithm(),
        }
    }

    /// Compute a route on a graph that visits tourist attractions in a specific area based on
    /// user preferences for these tourist attractions
    ///
    /// # Returns
    /// * an `Ok` containing the computed route in case of no errors, or
    /// * an `Err` containing an `AlgorithmError`, otherwise
    pub fn compute_route(&self) -> Result<Route, AlgorithmError> {
        self.inner().compute_route()
    }

    /// Outputs the score collected by a route computed by this algorithm
    pub fn get_collected_score(&self, route: &Route) -> usize {
        self.inner().get_collected_score(route)
    }
}

/// Compute wait and service time for given sight based on the already used time budget
fn compute_wait_and_service_time(start_time: &DateTime<Utc>, end_time: &DateTime<Utc>, sight: &Sight,
                                 used_time_budget: i64, root_travel_time: i64) -> Option<(i64, i64)> {
    let time_window = sight.opening_hours();

    // Determine current time (after given used time budget)
    let curr_time = start_time.naive_utc() + Duration::seconds(used_time_budget);
    // Determine latest time such that root is still reachable
    let latest_time = end_time.naive_utc() - Duration::seconds(root_travel_time);
    if curr_time >= latest_time  {
        // Stay at sight not possible due to time restrictions
        return None;
    }

    // Determine the sights current open state
    // TODO handle DateLimitExceeded error properly
    let curr_state = time_window.state(curr_time)
        .expect("Failed to determine open state");

    // Initialize closure for computing the sights service time
    let compute_service_time = |service_start_time: NaiveDateTime, latest_service_end_time: NaiveDateTime| {
        let possible_service_time = latest_service_end_time.signed_duration_since(service_start_time)
            .num_seconds();
        sight.duration_of_stay_secs().min(possible_service_time)
    };

    // Determine wait and service time based on the sights current open state
    match curr_state {
        RuleKind::Open | RuleKind::Unknown => {
            let mut next_time = curr_time;
            loop {
                match time_window.next_change(next_time) {
                    Ok(close_time) => {
                        if time_window.is_closed(close_time) {
                            break Some((0, compute_service_time(
                                curr_time, close_time.min(latest_time))));
                        }
                        next_time = close_time;
                        if next_time >= curr_time + Duration::seconds(sight.duration_of_stay_secs()) {
                            break Some((0, compute_service_time(
                                curr_time, next_time.min(latest_time))));
                        }
                    }
                    _ => break Some((0, compute_service_time(curr_time, latest_time)))
                };
            }
        }
        RuleKind::Closed => {
            match time_window.next_change(curr_time) {
                Ok(open_time) => {
                    if open_time < latest_time {
                        let wait_time = open_time.signed_duration_since(curr_time).num_seconds();
                        let mut next_time = open_time;
                        loop {
                            match time_window.next_change(next_time) {
                                Ok(close_time) => {
                                    if time_window.is_closed(close_time) {
                                        break Some((wait_time, compute_service_time(
                                            open_time, close_time.min(latest_time))));
                                    }
                                    next_time = close_time;
                                    if next_time >= open_time + Duration::seconds(sight.duration_of_stay_secs()) {
                                        break Some((wait_time, compute_service_time(
                                            open_time, next_time.min(latest_time))));
                                    }
                                }
                                _ => break Some((wait_time, compute_service_time(
                                    open_time, latest_time)))
                            };
                        }
                    } else {
                        // Sight closed until (after) we have to leave
                        None
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
    /// Error indicating that no node has been found in the requested area
    #[display(fmt = "No nearest node found in requested area")]
    NoNearestNodeFound,
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Duration, Utc};
    use itertools::Itertools;
    use once_cell::sync::Lazy;
    use opening_hours_syntax::rules::RuleKind;
    use crate::algorithm::{Algorithm, Area, Route, RouteSector, Sector, SightCategoryPref, UserPreferences};
    use crate::data::graph::Category;
    use crate::init_logging;
    use crate::utils::test_setup;

    /// Start time of trip or hike used for testing
    pub const START_TIME: &str = "2022-07-01T14:00:00+01:00";
    /// End time of trip or hike used for testing
    pub const END_TIME: &str = "2022-07-01T20:00:00+01:00";

    /// https://www.youtube.com/watch?v=ExElCQwN3T8
    pub const WALKING_SPEED_MPS: f64 = 5.0 / 3.6;

    /// Baba Hotel, ich schwör!!
    const RADISSON_BLU_HOTEL: Area = Area {
        lat: 53.074448,
        lon: 8.805105,
        radius: 300.0,
    };

    /// User category preferences used for testing
    const CATEGORY_PREFS: [SightCategoryPref; 3] = [
        SightCategoryPref { category: Category::Sightseeing, pref: 5 },
        SightCategoryPref { category: Category::Nightlife, pref: 4 },
        SightCategoryPref { category: Category::Restaurants, pref: 2 }
    ];

    /// Lazily initialized vector with algorithm instances used for testing
    static ALGORITHMS: Lazy<Vec<Algorithm>> = Lazy::new(|| {
        let start_time = DateTime::parse_from_rfc3339(START_TIME).unwrap()
            .with_timezone(&Utc);
        let end_time = DateTime::parse_from_rfc3339(END_TIME).unwrap()
            .with_timezone(&Utc);
        let user_prefs = UserPreferences {
            categories: CATEGORY_PREFS.to_vec(),
            sights: vec![],
        };
        Algorithm::available_algorithms().iter().map(|&algo_name|
            Algorithm::from_name(
                algo_name, &test_setup::GRAPH, start_time, end_time,
                WALKING_SPEED_MPS, RADISSON_BLU_HOTEL, user_prefs.clone()
            ).unwrap()
        ).collect_vec()
    });

    /// Run given test with each algorithm instance in `ALGORITHMS`
    fn run_test_with_each_algorithm<T>(test: T) where T: Fn(&Algorithm) {
        init_logging();
        ALGORITHMS.iter().for_each(|algo| test(algo));
    }

    /// Compute a walking route with given algorithm and `panic` if the computed route is empty
    fn compute_route_with_empty_check<'a>(algo: &'a Algorithm) -> Route<'a> {
        let route = algo.compute_route().expect("Error during route computation");
        if route.is_empty() {
            panic!("Route empty");
        }
        route
    }

    #[test]
    fn test_route_contains_only_sights_with_category_pref() {
        let categories_with_prefs = CATEGORY_PREFS.iter()
            .map(|category_pref| &category_pref.category).collect_vec();
        let sector_ok = |sector: &Sector| {
            assert!(categories_with_prefs.contains(&&sector.sight.category),
                    "Route contains sight {} with category {:?}, which is not in user preferences",
                    sector.sight.node_id, sector.sight.category.to_string());
        };
        run_test_with_each_algorithm(|algo| {
            let route = compute_route_with_empty_check(algo);
            route.iter().for_each(|route_sector| match route_sector {
                RouteSector::Start(sector) => sector_ok(sector),
                RouteSector::Intermediate(sector) => sector_ok(sector),
                _ => (), // End sector has no target sight
            });
        });
    }

    #[test]
    fn test_route_travel_time_within_time_budget() {
        let start_time = DateTime::parse_from_rfc3339(START_TIME).unwrap()
            .with_timezone(&Utc);
        let end_time = DateTime::parse_from_rfc3339(END_TIME).unwrap()
            .with_timezone(&Utc);
        run_test_with_each_algorithm(|algo| {
            let route = compute_route_with_empty_check(algo);
            let route_end_time = match &route.last().unwrap() {
                RouteSector::End(end_sector) => end_sector.time_of_arrival,
                _ => panic!("Last sector must be end sector")
            };
            let route_travel_time = route_end_time.signed_duration_since(start_time)
                .num_seconds();

            let avail_time_budget = end_time.signed_duration_since(start_time).num_seconds();
            assert!(route_travel_time <= avail_time_budget, "Route travel time exceeds available budget");
        });
    }

    #[test]
    fn test_sights_on_route_within_opening_time() {
        let sector_ok = |sector: &Sector| {
            let service_start_time = sector.service_start_time.naive_utc();
            let service_end_time = sector.service_end_time.naive_utc();
            let sight_opening_hours = sector.sight.opening_hours();
            let state_at_start = sight_opening_hours.state(
                service_start_time).unwrap();
            let state_at_end = sight_opening_hours.state(
                service_end_time - Duration::seconds(1)).unwrap();
            assert!(matches!(state_at_start, RuleKind::Open | RuleKind::Unknown),
                    "Sight is not open at service start time {}: current state: {:?}, opening hours: {}",
                    sector.service_start_time.to_rfc3339(), state_at_start, &sector.sight.opening_hours);
            assert!(matches!(state_at_end, RuleKind::Open | RuleKind::Unknown),
                    "Sight is not open before service end time {}: current state: {:?}, opening hours: {}",
                    sector.service_end_time.to_rfc3339(), state_at_end, &sector.sight.opening_hours);
        };
        run_test_with_each_algorithm(|algo| {
            let route = compute_route_with_empty_check(algo);
            for route_sector in &route {
                match route_sector {
                    RouteSector::Start(sector) => sector_ok(sector),
                    RouteSector::Intermediate(sector) => sector_ok(sector),
                    _ => () // End sector has no target sight
                }
            }
        });
    }
}