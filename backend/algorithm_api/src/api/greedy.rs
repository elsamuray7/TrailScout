use crate::api::{Algorithm, Coordinate};

pub struct GreedyAlgorithm {

}

impl GreedyAlgorithm {
    /// Creates a new instance of `GreedyAlgorithm`
    pub fn new() -> Self {
        Self {}
    }
}

impl Algorithm for GreedyAlgorithm {
    fn compute_route(&self, root: Coordinate) {
        todo!()
    }
}