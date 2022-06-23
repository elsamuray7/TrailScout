use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use data_api::api::graph::{Graph, Node, Sight};
use itertools::Itertools;
use pathfinding::prelude::*;
use crate::api::{Algorithm, Area, Coordinate, Route, ScoreMap, UserPreferences};

/// Compute scores for tourist attractions based on user preferences for categories or specific
/// tourist attractions, respectively
///
/// TODO map user preference number to algorithm internal score number
fn compute_scores(sights: &HashMap<usize, Sight>, user_prefs: UserPreferences) -> ScoreMap {
    let mut scores = sights.iter()
        .map(|(&sight_id, _)| (sight_id, 0_usize))
        .collect::<ScoreMap>();
    for sight in &user_prefs.sights {
        // TODO implement check whether SightPref really corresponds to sight
        scores.insert(sight.id, sight.pref);
    }
    for category in &user_prefs.categories {
        sights.iter()
            // TODO filter sights by category name and set score for all sights in category
            .filter(|(_, sight)| false)
            .for_each(|(&sight_id, _)| {
                scores.entry(sight_id).or_insert(category.pref);
            });
    }
    scores
}

pub struct GreedyAlgorithm<'a> {
    graph: &'a Graph,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    /// Walking speed in meters per second
    walking_speed_mps: f64,
    area: Area,
    sights: HashMap<usize, Sight>,
    root_id: usize,
    scores: ScoreMap,
}

impl<'a> Algorithm<'a> for GreedyAlgorithm<'a> {
    fn new(graph: &'a Graph,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           walking_speed_mps: f64,
           area: Area,
           user_prefs: UserPreferences) -> Self {
        if end_time < start_time {
            panic!("End time before start time");
        }

        let sights = graph.get_sights_in_area(area.lat, area.lon, area.radius);
        let root_id = graph.get_nearest_node(area.lat, area.lon);
        let scores = compute_scores(&sights, user_prefs);
        Self {
            graph,
            start_time,
            end_time,
            walking_speed_mps,
            area,
            sights,
            root_id,
            scores,
        }
    }

     fn compute_route(&self) -> Route {
        let mut curr_node_id = self.root_id;
        let successors = |node: &Node|
            self.graph.get_outgoing_edges_in_area(node.id, self.area.lat, self.area.lon, self.area.radius)
                .into_iter()
                .map(|edge| (self.graph.get_node(edge.tgt), edge.dist))
                .collect::<Vec<(&Node, usize)>>();

        let root = self.graph.get_node(self.root_id);
        let mut route: Route = vec![Coordinate { lat: root.lat, lon: root.lon }];
        let mut time_budget_left = (self.end_time.timestamp() - self.start_time.timestamp()) as usize;
        loop {
            // calculate distances from curr_node to all sight nodes
            let result_to_sights: HashMap<&Node, (&Node, usize)> =
                dijkstra_all(&self.graph.get_node(curr_node_id),
                             |&node| successors(node));

            // sort sight nodes by their distance to curr_node
            let sorted_dist_vec: Vec<_> = result_to_sights.values()
                .filter(|(node, _)| self.sights.contains_key(&node.id))
                .sorted_unstable_by(|(node1, dist1), (node2, dist2)| {
                    let score1 = self.scores[&node1.id];
                    let score2 = self.scores[&node2.id];
                    (score1 / dist1).cmp(&(score2 / dist2))
                })
                .collect();

            // for each sight node, check whether sight can be included in route without violating time budget
            let len_route_before = route.len();
            for &(sight_node, dist) in sorted_dist_vec {
                let secs_needed_to_sight = dist as f64 / self.walking_speed_mps;
                let result_sight_to_root =
                    dijkstra(&self.graph.get_node(sight_node.id),
                             |&node| successors(node),
                             |&node| node.id == self.root_id);
                match result_sight_to_root {
                    Some((_, dist_sight_to_root)) => {
                        let secs_needed_sight_to_root = dist_sight_to_root as f64 / self.walking_speed_mps;
                        let secs_total = (secs_needed_to_sight + secs_needed_sight_to_root) as usize + 1;
                        if secs_total <= time_budget_left {
                            // add sight and all intermediate nodes to route
                            let mut new_route_tail =
                                VecDeque::from([Coordinate {lat: sight_node.lat, lon: sight_node.lon}]);
                            let mut curr_pred = sight_node;
                            while curr_pred.id != curr_node_id {
                                curr_pred = result_to_sights[&curr_pred].0;
                                new_route_tail.push_front(Coordinate { lat: curr_pred.lat, lon: curr_pred.lon });
                            }
                            for coord in new_route_tail {
                                route.push(coord);
                            }

                            time_budget_left -= secs_total;
                            curr_node_id = sight_node.id;
                            break;
                        }
                    }
                    None => continue // No path from sight to root found. Continue.
                };
            }

            // check whether any sight has been included in route and if not, go back to root
            if route.len() == len_route_before {
                let result_to_root =
                    dijkstra(&self.graph.get_node(curr_node_id),
                             |&node| successors(node),
                             |&node| node.id == self.root_id)
                        .expect("No path from last visited sight to root");
                let (new_route_tail, _) = result_to_root;
                for &elem in &new_route_tail[1..] {
                    route.push(Coordinate { lat: elem.lat, lon: elem.lon });
                }
                break;
            }
        }

        route
    }

    fn map_node_to_sight(&self, node: &Node) -> Option<&Sight> {
        self.sights.get(&node.id)
    }
}