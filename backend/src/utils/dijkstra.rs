use crate::data::graph::{Graph, Node};
use crate::utils::binary_minheap::BinaryMinHeap;

/// Dijkstra result of a single node
pub struct NodeResult<'a>(usize, Vec<&'a Node>);

impl<'a> NodeResult<'a> {
    /// Creates a new node result with distance `dist` and path `path` to the associated node
    fn new(dist: usize, path: Vec<&'a Node>) -> Self {
        Self(dist, path)
    }

    /// Returns the distance to the associated node
    pub fn dist(&self) -> usize {
        self.0
    }

    /// Returns the path from the source node to the associated node
    pub fn path(&self) -> &Vec<&'a Node> {
        &self.1
    }

    /// Consumes the path from the source node to the associated node
    pub fn consume_path(self) -> Vec<&'a Node> {
        self.1
    }
}

/// Struct to hold the result of a run of the Dijkstra algorithm
pub struct DijkstraResult {
    dists: Vec<usize>,
    preds: Vec<usize>,
}

impl DijkstraResult {
    /// Creates a new `DijkstraResult` instance for given graph size
    fn new(num_nodes: usize) -> Self {
        Self {
            dists: vec![usize::MAX; num_nodes],
            preds: vec![usize::MAX; num_nodes],
        }
    }

    /// Returns the dijkstra result for the node with id `node_id` in a `Some` or `None` if the
    /// node is not reachable from the source node
    pub fn result_of<'a>(&self, graph: &'a Graph, node_id: usize) -> Option<NodeResult<'a>> {
        match self.dists[node_id] {
            usize::MAX => None,
            dist => Some(NodeResult::new(dist, self.build_path(graph, node_id)))
        }
    }

    /// Returns the distance to the node with id `node_id` in a `Some` or `None` if the node is
    /// not reachable from the source node
    pub fn dist_to(&self, node_id: usize) -> Option<usize> {
        match self.dists[node_id] {
            usize::MAX => None,
            dist => Some(dist),
        }
    }

    /// Build the path from the source node to the node with id `tgt_id`.
    /// This method assumes that the target can be reached from the source, otherwise it will
    /// output a path that solely consists of the target.
    pub fn build_path<'a>(&self, graph: &'a Graph, tgt_id: usize) -> Vec<&'a Node> {
        let mut path = vec![];
        let mut curr_pred = tgt_id;
        // source node has no predecessor
        while curr_pred < usize::MAX {
            path.push(graph.get_node(curr_pred));
            curr_pred = self.preds[curr_pred];
        }
        path.reverse();
        path
    }
}

/// Initialize the `DijkstraResult` instance and the priority queue for a run of the Dijkstra algorithm
fn init_result_and_pq(graph: &Graph, src_id: usize) -> (DijkstraResult, BinaryMinHeap) {
    let mut result = DijkstraResult::new(graph.num_nodes);
    result.dists[src_id] = 0;

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

/// Process the outgoing edges of the node with id `node_id` in the given area
fn process_edges_in_area(graph: &Graph, node_id: usize, result: &mut DijkstraResult, pq: &mut BinaryMinHeap,
                         lat: f64, lon: f64, radius: f64) {
    let node_dist = result.dists[node_id];
    for edge in graph.get_outgoing_edges_in_area(node_id, lat, lon, radius) {
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
            break;
        } else {
            process_edges(graph, node_id, &mut result, &mut pq);
        }
    }

    result.result_of(graph, tgt_id)
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

/// Run a Dijkstra from the source node with id `src_id` to the target node with id `tgt_id`
/// in the given area
pub fn run_dijkstra_in_area(graph: &Graph, src_id: usize, tgt_id: usize,
                            lat: f64, lon: f64, radius: f64) -> Option<NodeResult> {
    let (mut result, mut pq) = init_result_and_pq(graph, src_id);

    while !pq.is_empty() {
        let node_id = pq.pop(&result.dists);
        if node_id == tgt_id {
            break;
        } else {
            process_edges_in_area(graph, node_id, &mut result, &mut pq, lat, lon, radius);
        }
    }

    result.result_of(graph, tgt_id)
}

/// Run a Dijkstra from the source node with id `src_id` to all other nodes in the given area
pub fn run_ota_dijkstra_in_area(graph: &Graph, src_id: usize,
                                lat: f64, lon: f64, radius: f64) -> DijkstraResult {
    let (mut result, mut pq) = init_result_and_pq(graph, src_id);

    while !pq.is_empty() {
        let node_id = pq.pop(&result.dists);
        process_edges_in_area(graph, node_id, &mut result, &mut pq, lat, lon, radius);
    }

    result
}

#[cfg(test)]
mod test {
    use pathfinding::prelude::{dijkstra, dijkstra_all};
    use rand::{Rng, thread_rng};
    use crate::data::graph::Graph;
    use crate::init_logging;
    use crate::utils::dijkstra::{run_dijkstra, run_ota_dijkstra, run_ota_dijkstra_in_area};
    use crate::utils::test_setup;

    #[test]
    fn test_dijkstra() {
        init_logging();

        let graph = &test_setup::GRAPH;

        let mut rng = thread_rng();
        let src_id = rng.gen_range(0..graph.num_nodes);
        let tgt_id = rng.gen_range(0..graph.num_nodes);

        let result = run_dijkstra(&graph, src_id, tgt_id);

        let successors = |node_id: usize|
            graph.get_outgoing_edges(node_id)
                .into_iter()
                .map(|edge| (edge.tgt, edge.dist))
                .collect::<Vec<(usize, usize)>>();
        let exp_result = dijkstra(&src_id,
                                  |&node_id| successors(node_id),
                                  |&node_id| node_id == tgt_id);

        match exp_result {
            Some((_, exp_dist)) => {
                assert!(result.is_some());
                let actual_dist = result.unwrap().dist();
                assert_eq!(actual_dist, exp_dist, "Distances differ: actual: {}, expected: {}",
                           actual_dist, exp_dist);
            }
            None => assert!(result.is_none())
        }
    }

    #[test]
    fn test_ota_dijkstra() {
        init_logging();

        let graph = &test_setup::GRAPH;

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
            let actual_dist = result.dist_to(node_id);
            let expected = exp_result.get(&node_id);

            if node_id == src_id {
                assert!(actual_dist.is_some());
                assert!(expected.is_none());
                let actual_dist = actual_dist.unwrap();
                assert_eq!(actual_dist, 0);
            } else {
                match expected {
                    Some(&(_, exp_dist)) => {
                        assert!(actual_dist.is_some());
                        let actual_dist = actual_dist.unwrap();
                        assert_eq!(actual_dist, exp_dist, "Distances differ: actual: {}, expected: {}",
                                   actual_dist, exp_dist);
                    }
                    None => assert!(actual_dist.is_none())
                }
            }
        }
    }

    #[test]
    fn test_path_length() {
        init_logging();

        let graph = &test_setup::GRAPH;

        let mut rng = thread_rng();
        let src_id = rng.gen_range(0..graph.num_nodes);
        let tgt_id = rng.gen_range(0..graph.num_nodes);

        let result = run_dijkstra(&graph, src_id, tgt_id);

        let successors = |node_id: usize|
            graph.get_outgoing_edges(node_id)
                .into_iter()
                .map(|edge| (edge.tgt, edge.dist))
                .collect::<Vec<(usize, usize)>>();
        let exp_result = dijkstra(&src_id,
                                  |&node_id| successors(node_id),
                                  |&node_id| node_id == tgt_id);

        match exp_result {
            Some((exp_path, _)) => {
                assert!(result.is_some());
                let path = result.unwrap().consume_path();

                let mut exp_len = 0;
                for (i, &node_id) in exp_path.iter().enumerate() {
                    if i == exp_path.len() - 1 {
                        break;
                    }

                    let edge = graph.get_outgoing_edges(node_id).into_iter()
                        .find(|&edge| edge.tgt == exp_path[i + 1]);
                    assert!(edge.is_some());
                    let edge = edge.unwrap();
                    exp_len += edge.dist;
                }

                let mut actual_len = 0;
                for (i, &node) in path.iter().enumerate() {
                    if i == path.len() - 1 {
                        break;
                    }

                    let edge = graph.get_outgoing_edges(node.id).into_iter()
                        .find(|&edge| edge.tgt == path[i + 1].id);
                    assert!(edge.is_some());
                    let edge = edge.unwrap();
                    actual_len += edge.dist;
                }

                assert_eq!(actual_len, exp_len, "Path length differs: actual: {}, expected: {}",
                           actual_len, exp_len);
            }
            None => assert!(result.is_none())
        }
    }

    #[test]
    fn test_ota_dijkstra_in_area() {
        init_logging();

        let graph = &test_setup::GRAPH;

        let mut rng = thread_rng();
        let src_id = rng.gen_range(0..graph.num_nodes);
        let src = graph.get_node(src_id);

        let result = run_ota_dijkstra_in_area(&graph, src_id, src.lat, src.lon,1000.0);

        let successors = |node_id: usize|
            graph.get_outgoing_edges_in_area(node_id, src.lat, src.lon, 1000.0)
                .into_iter()
                .map(|edge| (edge.tgt, edge.dist))
                .collect::<Vec<(usize, usize)>>();
        let exp_result = dijkstra_all(&src_id,
                                      |&node_id| successors(node_id));

        for node_id in 0..graph.num_nodes {
            let actual_dist = result.dist_to(node_id);
            let expected = exp_result.get(&node_id);

            if node_id == src_id {
                assert!(actual_dist.is_some());
                assert!(expected.is_none());
                let actual_dist = actual_dist.unwrap();
                assert_eq!(actual_dist, 0);
            } else {
                match expected {
                    Some(&(_, exp_dist)) => {
                        assert!(actual_dist.is_some());
                        let actual_dist = actual_dist.unwrap();
                        assert_eq!(actual_dist, exp_dist, "Distances differ: actual: {}, expected: {}",
                                   actual_dist, exp_dist);
                    }
                    None => assert!(actual_dist.is_none())
                }
            }
        }
    }
}