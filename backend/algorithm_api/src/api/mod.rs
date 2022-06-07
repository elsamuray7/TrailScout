pub mod route_provider;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use data_api::api::graph::Graph;
use crate::api::route_provider::{RouteProviderReq, RouteProviderReqUserPrefs, RouteProviderRes};

/// Type alias for a mapping from node id's to scores, where the nodes represent sights / tourist
/// attractions
type Scores = HashMap<usize, usize>;

/// Compute scores for tourist attractions based on user preferences for categories or specific
/// tourist attractions, respectively
fn compute_scores(graph: Arc<RwLock<Graph>>, user_prefs: &RouteProviderReqUserPrefs) -> Scores {
    todo!()
}

/// Compute a route on a given graph for the tourist attractions and start and end point
/// contained in `data` via a greedy approach.
pub fn compute_route_greedy(graph: Arc<RwLock<Graph>>, data: RouteProviderReq) -> RouteProviderRes {
    todo!()
}