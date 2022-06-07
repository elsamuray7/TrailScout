mod greedy;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use lib::api::graph::Graph;
use crate::api::greedy::GreedyAlgorithm;

type Coordinate = (f64, f64);

pub trait Algorithm {
    /// Compute a walking route along tourist attractions based on their scores that starts and
    /// ends at `root`
    fn compute_route(&self, root: Coordinate);
}

/// Creates a new instance of the default algorithm
pub fn default(graph: Arc<RwLock<Graph>>) -> Box<dyn Algorithm> {
    Box::new(GreedyAlgorithm::new(graph))
}

/// Type alias for a mapping from node id's to scores, where the nodes represent sights / tourist
/// attractions
type Scores = HashMap<usize, usize>;

/// Default implementation for computing scores for tourist attractions based on user preferences
/// for categories or specific tourist attractions, respectively
fn compute_scores() -> Scores {
    todo!()
}