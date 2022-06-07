use std::sync::{Arc, RwLock};
use data_api::api::graph::Graph;
use crate::api::{Algorithm, Coordinate};

pub struct GreedyAlgorithm {
    graph: Arc<RwLock<Graph>>,
}

impl GreedyAlgorithm {
    /// Creates a new instance of `GreedyAlgorithm`
    pub fn new(graph: Arc<RwLock<Graph>>) -> Self {
        Self {
            graph,
        }
    }
}

impl Algorithm for GreedyAlgorithm {
    fn compute_route(&self, root: Coordinate) {
        todo!()
    }
}