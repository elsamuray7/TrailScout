use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use data_api::api::graph::{Graph, Node, Sight};
use pathfinding::prelude::*;
use crate::api::{Algorithm, Area, Coordinate, Route, ScoreMap, Scores, UserPreferences};

pub struct GreedyAlgorithm {
    graph_ref: Arc<RwLock<Graph>>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    sights: HashMap<usize, Sight>,
    root_id: usize,
    user_prefs: UserPreferences,
}

impl GreedyAlgorithm {
    /// Compute scores for tourist attractions based on user preferences for categories or specific
    /// tourist attractions, respectively
    fn compute_scores(&self) -> Scores {
        let mut score_map = ScoreMap::with_capacity(self.sights.len());
        for _ in &self.user_prefs.categories {
            // TODO filter sights by category name and insert score for all sights in category
        }
        for sight in &self.user_prefs.sights {
            // TODO (maybe) compute algorithm internal score from user preference
            score_map.insert(sight.id, sight.pref);
        }
        Scores::from_map(score_map)
    }
}

impl Algorithm for GreedyAlgorithm {
    fn new(graph_ref: Arc<RwLock<Graph>>,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           area: Area,
           user_prefs: UserPreferences) -> Self {
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
            sights,
            root_id,
            user_prefs,
        }
    }

    fn compute_route(&self) -> Route {
        let graph = self.graph_ref.read().unwrap();
        let scores = self.compute_scores();
        let mut route: Vec<Sight> = Vec::new();
        let mut curr_node = &&graph.nodes[self.root_id];
        loop {
            let mut unvisited_sights: Vec<_> = self.sights.keys()
                .map(|sight_id| *sight_id)
                .collect();
            let (mut dist_map, _): (HashMap<&Node, (&Node, usize)>, Option<&Node>) = dijkstra_partial(curr_node,
                                                               |&node| graph.get_outgoing_edges(node.id)
                                                                   .into_iter()
                                                                   .map(|edge| (&graph.nodes[edge.tgt], edge.dist))
                                                                   .collect::<Vec<(&Node, usize)>>(),
                                                               |&node| {
                                                                    unvisited_sights.retain(|&sight_id| sight_id != node.id);
                                                                    unvisited_sights.is_empty()
                                                                });
        }
        todo!()
    }
}