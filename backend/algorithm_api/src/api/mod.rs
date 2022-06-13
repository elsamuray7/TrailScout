pub mod route_provider;
pub mod greedy;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use data_api::api::graph::Graph;
use serde::{Serialize, Deserialize};

/// Type alias for a mapping from node id's to scores, where the nodes represent sights / tourist
/// attractions
type ScoreMap = HashMap<usize, usize>;

/// Immutable data structure to access scores for sights / tourist attractions
struct Scores {
    score_map: ScoreMap,
}

impl Scores {
    /// Create a `Scores` instance from a score map
    fn from_map(score_map: ScoreMap) -> Self {
        Self {
            score_map,
        }
    }

    /// Get the score of the node with id `node_id`
    fn get_score(&self, node_id: &usize) -> Option<usize> {
        match self.score_map.get(node_id) {
            Some(&score) => Some(score),
            _ => None,
        }
    }
}

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
    fn new(graph_ref: Arc<RwLock<Graph>>,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           area: Area,
           user_prefs: UserPreferences) -> Self;

    /// Compute a route on a given graph that visits tourist attractions in a given area based on
    /// user preferences for these tourist attractions
    fn compute_route(&self) -> Route;
}