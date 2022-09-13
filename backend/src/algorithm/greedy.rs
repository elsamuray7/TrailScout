use std::collections::HashMap;
use std::rc::Rc;
use chrono::{DateTime, Utc};
use crate::data::graph::{Graph, Sight};
use itertools::Itertools;
use crate::algorithm::{_Algorithm, AlgorithmError, Area, Route, RouteSector, ScoreMap, Sector, UserPreferences, USER_PREF_MAX, compute_wait_and_service_time, EndSector};
use crate::utils::dijkstra;

/// Greedy internal user preference to score mapping
const USER_PREF_TO_SCORE: [usize; USER_PREF_MAX + 1] = [0, 1, 2, 4, 8, 16];

/// Compute scores for tourist attractions based on user preferences for categories or specific
/// tourist attractions, respectively
fn compute_scores(sights: &Vec<&Sight>, user_prefs: UserPreferences) -> ScoreMap {
    let mut scores: ScoreMap = sights.iter()
        .map(|sight| (sight.node_id, (0_usize, sight.category))).collect();

    for category_pref in &user_prefs.categories {
        let category_score = USER_PREF_TO_SCORE[category_pref.get_valid_pref()];
        sights.iter()
            .filter(|sight| sight.category == category_pref.category)
            .for_each(|sight| {
                let (prev_score, prev_category) = scores.get_mut(
                    &sight.node_id).unwrap();
                if category_score > *prev_score {
                    *prev_score = category_score;
                    *prev_category = category_pref.category;
                }
            });
    }

    let sight_id_category_map: HashMap<_, _> = sights.iter()
        .map(|sight| (sight.node_id, sight.category)).collect();
    for sight_pref in &user_prefs.sights {
        // Ignore nodes and sights that are not in the fetched sights
        if sight_id_category_map.contains_key(&sight_pref.id) {
            let sight_pref_score = USER_PREF_TO_SCORE[sight_pref.get_valid_pref()];
            let (prev_score, _) = scores.get_mut(&sight_pref.id).unwrap();
            if sight_pref_score > *prev_score {
                *prev_score = sight_pref_score;
            }
        }
    }

    log::trace!("Computed scores: {:?}", &scores);

    scores
}

/// Greedy implementation of the `Algorithm` trait.
///
/// The greedy algorithm tries to find the best route by including sights into the route based on
/// their score-distance ratio at that time until the time budget is used up.
pub struct GreedyAlgorithm<'a> {
    graph: &'a Graph,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    /// Walking speed in meters per second
    walking_speed_mps: f64,
    area: Area,
    sights: Vec<&'a Sight>,
    root_id: usize,
    scores: ScoreMap,
}

impl GreedyAlgorithm<'_> {
    /// Unique string identifier of this algorithm implementation
    pub const ALGORITHM_NAME: &'static str = "Greedy";
}

impl<'a> _Algorithm<'a> for GreedyAlgorithm<'a> {
    fn new(graph: &'a Graph,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           walking_speed_mps: f64,
           area: Area,
           user_prefs: UserPreferences) -> Result<Self, AlgorithmError> {
        if end_time < start_time {
            return Err(AlgorithmError::NegativeTimeInterval);
        }

        let time_budget = end_time.signed_duration_since(start_time).num_seconds() as f64;
        let edge_radius = walking_speed_mps * time_budget / std::f64::consts::PI / 2.0;
        let sights_radius = edge_radius.min(area.radius);
        let sights = graph.get_reachable_sights_in_area(area.lat, area.lon,
                                                        sights_radius, edge_radius);
        if sights.is_empty() {
            return Err(AlgorithmError::NoSightsFound)
        }

        let root_id = match graph.get_nearest_node_in_area(area.lat, area.lon, area.radius) {
            Some(nearest_node) => nearest_node,
            None => return Err(AlgorithmError::NoNearestNodeFound)
        };

        let scores = compute_scores(&sights, user_prefs);

        Ok(Self {
            graph,
            start_time,
            end_time,
            walking_speed_mps,
            area,
            sights,
            root_id,
            scores,
        })
    }

     fn compute_route(&self) -> Result<Route, AlgorithmError> {
         let total_time_budget = self.end_time.signed_duration_since(self.start_time)
             .num_seconds();
         let mut time_budget_left = total_time_budget;

         let edge_radius = self.walking_speed_mps * time_budget_left as f64 / std::f64::consts::PI / 2.0;

         log::debug!("Starting greedy search");

         let mut route: Route = vec![];
         // Get all sights that can potentially be visited
         let mut unvisited_sights: HashMap<_, _> = self.sights.iter()
             .filter(|&sight| {
                 let (score, category) = self.scores[&sight.node_id];
                 score > 0 && sight.category == category
             })
             .map(|&sight| (sight.node_id, sight))
             .collect();
         if unvisited_sights.is_empty() {
             return Err(AlgorithmError::NoPreferencesProvided);
         }
         let mut curr_node_id = self.root_id;
         let result_from_root = Rc::new(dijkstra::run_ota_dijkstra_in_area(
             self.graph, curr_node_id, self.area.lat, self.area.lon, edge_radius));
         let mut result_to_sights;
         loop {
             // calculate distances from curr_node to all sight nodes
             if curr_node_id == self.root_id {
                 result_to_sights = result_from_root.clone();
             } else {
                 result_to_sights = Rc::new(dijkstra::run_ota_dijkstra_in_area(
                     self.graph, curr_node_id, self.area.lat, self.area.lon, edge_radius));
             }

             // sort sight nodes by a metric derived from the sights score and its distance to
             // the current node
             let sorted_dist_vec = unvisited_sights.iter()
                 .filter_map(|(&sight_id, &sight)| result_to_sights.dist_to(sight_id)
                     .and_then(|dist| Some((sight, dist))))
                 .sorted_unstable_by(|&(sight1, dist1), &(sight2, dist2)| {
                     let (score1, _) = self.scores[&sight1.node_id];
                     let (score2, _) = self.scores[&sight2.node_id];
                     let metric1 = score1 as f64 / dist1 as f64;
                     let metric2 = score2 as f64 / dist2 as f64;
                     metric2.total_cmp(&metric1)
                 })
                 .collect_vec();
             log::trace!("Sorted {} sights by greedy metric", sorted_dist_vec.len());

             // for each sight node, check whether sight can be included in route without violating time budget
             let len_route_before = route.len();
             for (sight, dist) in sorted_dist_vec {
                 let sight_travel_time = (dist as f64 / self.walking_speed_mps) as i64 + 1;

                 let used_time_budget = total_time_budget - time_budget_left + sight_travel_time;
                 let (wait_time, service_time) = match compute_wait_and_service_time(
                     &self.start_time, sight, used_time_budget) {
                     Some(result) => result,
                     None => continue
                 };

                 // Works because graph is undirected
                 match result_from_root.dist_to(sight.node_id) {
                     Some(dist_to_root) => {
                         let sight_total_time = sight_travel_time + wait_time + service_time;
                         let root_travel_time = (dist_to_root as f64 / self.walking_speed_mps) as i64 + 1;

                         if sight_total_time + root_travel_time <= time_budget_left {
                             log::trace!("Appending sight {} (secs to include sight: {} <= left time budget: {}) with score: {}",
                                 sight.node_id, sight_total_time + root_travel_time, time_budget_left, self.scores[&sight.node_id].0);

                             // add sector containing sight and all intermediate nodes to route
                             let path = result_to_sights.build_path(self.graph,
                                                                    sight.node_id);
                             let sector = Sector::new(
                                 &self.start_time, total_time_budget - time_budget_left,
                                 sight_travel_time, wait_time, service_time, sight, path);
                             route.push(if curr_node_id == self.root_id {
                                 RouteSector::Start(sector)
                             } else {
                                 RouteSector::Intermediate(sector)
                             });

                             time_budget_left -= sight_total_time;
                             unvisited_sights.remove(&sight.node_id);
                             curr_node_id = sight.node_id;
                             break;
                         }
                     }
                     None => continue // No path from sight to root found. Continue.
                 };
             }

             // check whether any sight has been included in route and if not, go back to root
             let len_route_after = route.len();
             if len_route_after == len_route_before && len_route_after > 0 {
                 log::trace!("Traveling back to root");

                 // Path from sight to root must exist because otherwise, we would have skipped sight
                 // Works because graph is undirected
                 let result_to_root = result_from_root.result_of(
                     self.graph, curr_node_id).unwrap();

                 let secs_to_root = (result_to_root.dist() as f64 / self.walking_speed_mps) as i64;
                 let mut path = result_to_root.consume_path();
                 // Reverse because path is in reverse direction
                 path.reverse();
                 route.push(RouteSector::End(EndSector::new(
                     &self.start_time, total_time_budget - time_budget_left,
                     secs_to_root, path)));
                 break;
             }
         }

         let collected_score = self.get_collected_score(&route);
         log::debug!("Finished greedy search. Computed walking route from node: {} including {} sights with total score: {}.",
             self.root_id, route.len() - 1, collected_score);

         Ok(route)
    }

    fn get_collected_score(&self, route: &Route) -> usize {
        route.iter()
            .map(|route_sec| {
                match route_sec {
                    // Start and intermediate sectors contain a sight per definition
                    RouteSector::Start(sector) => self.scores[&sector.sight.node_id].0,
                    RouteSector::Intermediate(sector) => self.scores[&sector.sight.node_id].0,
                    _ => 0,
                }
            })
            .sum()
    }
}