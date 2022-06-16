use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use data_api::api::graph::{Graph, Node, Sight};
use itertools::Itertools;
use pathfinding::prelude::*;
use crate::api::{Algorithm, Area, Coordinate, Route, ScoreMap, UserPreferences};

pub struct GreedyAlgorithm {
    graph_ref: Arc<RwLock<Graph>>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    /// Walking speed in meters per second
    walking_speed_mps: f64,
    area: Area,
    sights: HashMap<usize, Sight>,
    root_id: usize,
    user_prefs: UserPreferences,
}

impl GreedyAlgorithm {
    /// Compute scores for tourist attractions based on user preferences for categories or specific
    /// tourist attractions, respectively
    fn compute_scores(&self) -> ScoreMap {
        // TODO initialize score map with initial value 0 for all sights
        let mut scores = ScoreMap::with_capacity(self.sights.len());
        for _ in &self.user_prefs.categories {
            // TODO filter sights by category name and insert score for all sights in category
        }
        for sight in &self.user_prefs.sights {
            // TODO (maybe) compute algorithm internal score from user preference
            scores.insert(sight.id, sight.pref);
        }
        scores
    }

    /// Try to map a node to its sight instance.
    /// Returns `None` if the node is not a sight.
    fn map_node_to_sight(&self, node: &Node) -> Option<&Sight> {
        self.sights.get(&node.id)
    }
}

impl Algorithm for GreedyAlgorithm {
    fn new(graph_ref: Arc<RwLock<Graph>>,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           walking_speed_mps: f64,
           area: Area,
           user_prefs: UserPreferences) -> Self {
        if end_time < start_time {
            panic!("End time before start time");
        }
        let sights;
        let root_id;
        {
            let graph = graph_ref.read().unwrap();
            sights = graph.get_sights_in_area(area.lat, area.lon, area.radius);
            root_id = graph.get_nearest_node(area.lat, area.lon);
        }
        Self {
            graph_ref,
            start_time,
            end_time,
            walking_speed_mps,
            area,
            sights,
            root_id,
            user_prefs,
        }
    }

    fn compute_route(&self) -> Route {
        let graph = self.graph_ref.read().unwrap();
        let scores = self.compute_scores();
        let mut time_budget_left = (self.end_time.timestamp() - self.start_time.timestamp()) as usize;
        let mut route: Route = Vec::new();
        let mut curr_node_id = self.root_id;
        while {
            // calculate distances from curr_node to all sight nodes
            let dist_map: HashMap<&Node, (&Node, usize)> =
                dijkstra_all(&graph.get_node(curr_node_id),
                             |&node|
                                 graph.get_outgoing_edges_in_area(node.id, self.area.lat, self.area.lon, self.area.radius)
                                     .into_iter()
                                     .map(|edge| (graph.get_node(edge.tgt), edge.dist))
                                     .collect::<Vec<(&Node, usize)>>());

            // sort sight nodes by their distance to curr_node
            let sorted_dist_vec: Vec<_> = dist_map.values()
                .filter(|(node, _)| self.sights.contains_key(&node.id))
                .sorted_unstable_by(|(node1, dist1), (node2, dist2)| {
                    let score1 = scores[&node1.id];
                    let score2 = scores[&node2.id];
                    (score1 / dist1).cmp(&(score2 / dist2))
                })
                .collect();

            // for each sight node, check whether sight can be included in route without violating time budget
            let len_route_before = route.len();
            for &(node, dist) in sorted_dist_vec {
                let secs_needed_to_sight = dist as f64 / self.walking_speed_mps;
                let result =
                    dijkstra(&graph.get_node(node.id),
                            |&node|
                                graph.get_outgoing_edges_in_area(node.id, self.area.lat, self.area.lon, self.area.radius)
                                    .into_iter()
                                    .map(|edge| (graph.get_node(edge.tgt), edge.dist))
                                    .collect::<Vec<(&Node, usize)>>(),
                            |&node| node.id == self.root_id);
                match result {
                    Some((_, dist_sight_to_root)) => {
                        let secs_needed_sight_to_root = dist_sight_to_root as f64 / self.walking_speed_mps;
                        let secs_total = (secs_needed_to_sight + secs_needed_sight_to_root) as usize + 1;
                        if secs_total <= time_budget_left {
                            // add sight to route
                            time_budget_left -= secs_total;
                            curr_node_id = node.id;
                            route.push(Coordinate { lat: node.lat, lon: node.lon });
                        }
                    }
                    None => {
                        // TODO handle error
                    }
                }
            }

            // check whether any sight has been included in route
            route.len() > len_route_before
        } {}

        // go back to root
        let root = graph.get_node(self.root_id);
        route.push(Coordinate { lat: root.lat, lon: root.lon });

        route
    }
}