pub mod greedy;

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::data::graph::{Graph, Node, Sight};
use serde::{Serialize, Deserialize};

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
///
/// # Attributes
/// * `sights` - A vector containing the sights on this sector in the order in which they occur in
/// `nodes` (may contain between 1 and 2 sights)
/// * `nodes` - A vector containing a sequence of nodes from the sectors source to its target node
/// (both inclusive), where at least one of them is a sight
#[derive(Serialize, Debug)]
pub struct Sector<'a> {
    sights: Vec<&'a Sight>,
    nodes: Vec<&'a Node>,
}

impl<'a> Sector<'a> {
    /// Creates a new sector from given `sights` and `nodes`
    fn new(sights: Vec<&'a Sight>, nodes: Vec<&'a Node>) -> Self {
        Self {
            sights,
            nodes,
        }
    }
}

/// Type alias for a vector of route sectors that form a contiguous route
pub type Route<'a> = Vec<Sector<'a>>;

/// Algorithm trait to be implemented by concrete algorithm implementations
pub trait Algorithm<'a> {
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
           user_prefs: UserPreferences) -> Self;

    /// Compute a route on a given graph that visits tourist attractions in a given area based on
    /// user preferences for these tourist attractions
    fn compute_route(&self) -> Route;

    /// Try to map a node to its sight instance.
    /// Returns a `Some` containing the sight instance or `None` if the node is not a sight.
    fn map_node_to_sight(&self, node: &Node) -> Option<&Sight>;
}