use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};
use data_api::api::graph::{Graph, Sight};
use pathfinding::prelude::*;
use crate::api::{Algorithm, Area, Route, ScoreMap, Scores, UserPreferences};

pub struct GreedyAlgorithm {
    graph_ref: Arc<RwLock<Graph>>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    sights: Vec<Sight>,
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
        {
            let graph = graph_ref.read().unwrap();
            sights = graph.get_sights_in_area(area.lat, area.lon, area.radius);
        }
        Self {
            graph_ref,
            start_time,
            end_time,
            sights,
            user_prefs,
        }
    }

    fn compute_route(&self) -> Route {
        let graph = self.graph_ref.read().unwrap();
        let scores = self.compute_scores();
        todo!()
    }
}