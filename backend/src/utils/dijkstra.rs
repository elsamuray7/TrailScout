use crate::data::graph::Graph;
use crate::utils::binary_minheap::BinaryMinHeap;

/// Struct to hold the result of a run of the Dijkstra algorithm
pub struct DijkstraResult {
    dists: Vec<usize>,
    preds: Vec<usize>,
}

/// Dijkstra result of a single node
pub struct NodeResult(usize, usize);

impl NodeResult {
    /// Create a new node result with distance `dist` to and best predecessor `pred` of the
    /// associated node
    fn new(dist: usize, pred: usize) -> Self {
        Self(dist, pred)
    }

    /// Returns the distance to the associated node
    pub fn dist(&self) -> usize {
        self.0
    }

    /// Returns the best predecessor of the associated node
    pub fn pred(&self) -> usize {
        self.1
    }
}

impl DijkstraResult {
    /// Creates a new `DijkstraResult` instance for given graph size
    fn new(num_nodes: usize) -> Self {
        Self {
            dists: vec![usize::MAX; num_nodes],
            preds: vec![usize::MAX; num_nodes],
        }
    }

    /// Returns the dijkstra result of the node with id `node_id` in a `Some` or `None` if the node
    /// is not reachable from the source node
    pub fn result_of(&self, node_id: usize) -> Option<NodeResult> {
        match self.dists[node_id] {
            usize::MAX => None,
            dist => Some(NodeResult::new(dist, self.preds[node_id])),
        }
    }
}

/// Initialize the `DijkstraResult` instance and the priority queue for a run of the Dijkstra algorithm
fn init_result_and_pq(graph: &Graph, src_id: usize) -> (DijkstraResult, BinaryMinHeap) {
    let mut result = DijkstraResult::new(graph.num_nodes);
    result.dists[src_id] = 0;
    result.preds[src_id] = 0;

    let mut pq = BinaryMinHeap::with_capacity(graph.num_nodes);
    pq.push(src_id, &result.dists);

    (result, pq)
}

/// Process the outgoing edges of the node with id `node_id`
fn process_edges(graph: &Graph, node_id: usize, result: &mut DijkstraResult, pq: &mut BinaryMinHeap) {
    let node_dist = result.dists[node_id];
    for edge in graph.get_outgoing_edges(node_id) {
        let dist = node_dist + edge.dist;

        if dist < result.dists[edge.tgt] {
            result.dists[edge.tgt] = dist;
            result.preds[edge.tgt] = node_id;

            pq.insert_or_update(edge.tgt, &result.dists);
        }
    }
}

/// Run a Dijkstra from the source node with id `src_id` to the target node with id `tgt_id`
pub fn run_dijkstra(graph: &Graph, src_id: usize, tgt_id: usize) -> Option<NodeResult> {
    let (mut result, mut pq) = init_result_and_pq(graph, src_id);

    while !pq.is_empty() {
        let node_id = pq.pop(&result.dists);
        if node_id == tgt_id {
            Some(NodeResult::new(result.dists[tgt_id], result.preds[tgt_id]));
        } else {
            process_edges(graph, node_id, &mut result, &mut pq);
        }
    }

    None
}

/// Run a Dijkstra from the source node with id `src_id` to all other nodes
pub fn run_ota_dijkstra(graph: &Graph, src_id: usize) -> DijkstraResult {
    let (mut result, mut pq) = init_result_and_pq(graph, src_id);

    while !pq.is_empty() {
        let node_id = pq.pop(&result.dists);
        process_edges(graph, node_id, &mut result, &mut pq);
    }

    result
}

#[cfg(test)]
mod test {
    use std::time::Instant;
    use pathfinding::prelude::dijkstra_all;
    use rand::{Rng, thread_rng};
    use crate::data::graph::Graph;
    use crate::init_logging;
    use crate::utils::dijkstra::run_ota_dijkstra;

    #[test]
    fn test_ota_dijkstra() {
        init_logging();

        let graph = Graph::parse_from_file("./tests_data/output/bremen-latest.fmibin")
            .expect("Failed to parse graph file");

        let mut rng = thread_rng();
        let src_id = rng.gen_range(0..graph.num_nodes);

        let result = run_ota_dijkstra(&graph, src_id);

        let successors = |node_id: usize|
            graph.get_outgoing_edges(node_id)
                .into_iter()
                .map(|edge| (edge.tgt, edge.dist))
                .collect::<Vec<(usize, usize)>>();
        let exp_result = dijkstra_all(&src_id,
                                      |&node_id| successors(node_id));

        for node_id in 0..graph.num_nodes {
            let actual = result.result_of(node_id);
            let expected = exp_result.get(&node_id);

            if node_id == src_id {
                assert!(actual.is_some());
                assert!(expected.is_none());
                let actual = actual.unwrap();
                assert_eq!(actual.dist(), 0);
                assert_eq!(actual.pred(), 0);
            } else {
                match expected {
                    Some(&(exp_pred, exp_dist)) => {
                        assert!(actual.is_some());
                        let actual = actual.unwrap();
                        assert_eq!(actual.dist(), exp_dist, "Distances differ: actual: {}, expected: {}",
                                   actual.dist(), exp_dist);
                    }
                    None => assert!(actual.is_none())
                }
            }
        }
    }
}