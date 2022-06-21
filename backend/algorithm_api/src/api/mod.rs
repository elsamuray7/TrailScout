pub mod greedy;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use data_api::api::graph::Graph;
use serde::{Serialize, Deserialize};

/// Type alias for a mapping from node id's to scores, where the nodes represent sights / tourist
/// attractions
type ScoreMap = HashMap<usize, usize>;

/// Geographic coordinate
#[derive(Serialize)]
pub struct Coordinate {
    lat: f64,
    lon: f64,
}

/// Circular area around a coordinate
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

/// Type alias for a sequence of coordinates that form a contiguous route
pub type Route = Vec<Coordinate>;

/// Algorithm trait to be implemented by concrete algorithm implementations
trait Algorithm {
    /// Create a new algorithm instance
    ///
    /// # Arguments
    /// * `graph_ref` - A readable/writable atomic reference to the graph on which to run the
    /// algorithm
    /// * `start_time` - The intended start time of the walk
    /// * `end_time` - The intended end time of the walk
    /// * `walking_speed_mps` - The walking speed in meters per second
    /// * `area` - The area in which the walking route should lie
    /// * `user_prefs` - The users preferences for sight categories and sights, respectively
    fn new(graph_ref: Arc<RwLock<Graph>>,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           walking_speed_mps: f64,
           area: Area,
           user_prefs: UserPreferences) -> Self;

    /// Compute a route on a given graph that visits tourist attractions in a given area based on
    /// user preferences for these tourist attractions
    fn compute_route(&self) -> Route;
}