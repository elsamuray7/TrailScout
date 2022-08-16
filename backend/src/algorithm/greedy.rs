use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use crate::data::graph::{Category, Graph, Node, Sight};
use itertools::Itertools;
use pathfinding::prelude::*;
use crate::algorithm::{_Algorithm, AlgorithmError, Area, Route, RouteSector, ScoreMap, Sector, UserPreferences,
                       USER_PREF_MAX};

/// Greedy internal user preference to score mapping
const USER_PREF_TO_SCORE: [usize; USER_PREF_MAX + 1] = [0, 1, 2, 4, 8, 16];

/// Compute scores for tourist attractions based on user preferences for categories or specific
/// tourist attractions, respectively
///
/// TODO map user preference number to algorithm internal score number
fn compute_scores(sights: &HashMap<usize, &Sight>, user_prefs: UserPreferences) -> ScoreMap {
    let mut scores: ScoreMap = sights.iter()
        .map(|(&sight_id, _)| (sight_id, 0_usize))
        .collect();
    for category in &user_prefs.categories {
        let category_enum = category.name.parse::<Category>()
            .unwrap_or(Category::Other);
        sights.iter()
            .filter(|(_, sight)| sight.category == category_enum)
            .for_each(|(&sight_id, _)| {
                scores.insert(sight_id, USER_PREF_TO_SCORE[category.get_valid_pref()]);
            });
    }
    for sight in &user_prefs.sights {
        // TODO implement check whether SightPref really corresponds to sight
        scores.insert(sight.id, USER_PREF_TO_SCORE[sight.get_valid_pref()]);
    }
    log::debug!("Computed scores: {:?}", &scores);

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
    sights: HashMap<usize, &'a Sight>,
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

        let sights = graph.get_sights_in_area(area.lat, area.lon, area.radius);
        let root_id = graph.get_nearest_node(area.lat, area.lon);
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
         let successors = |node: &Node|
             self.graph.get_outgoing_edges_in_area(node.id, self.area.lat, self.area.lon, self.area.radius)
                 .into_iter()
                 .map(|edge| (self.graph.get_node(edge.tgt), edge.dist))
                 .collect::<Vec<(&Node, usize)>>();

         log::debug!("Starting greedy search");

         let mut route: Route = vec![];
         let mut time_budget_left = (self.end_time.timestamp() - self.start_time.timestamp()) as usize;
         // Get all sights that can potentially be visited
         let mut unvisited_sights: HashSet<_> = self.sights.keys()
             .filter(|&sight_id| self.scores[sight_id] > 0)
             .map(usize::to_owned)
             .collect();
         let mut curr_node_id = self.root_id;
         loop {
             // calculate distances from curr_node to all sight nodes
             let result_to_sights: HashMap<&Node, (&Node, usize)> =
                 dijkstra_all(&self.graph.get_node(curr_node_id),
                              |&node| successors(node));

             // sort sight nodes by their distance to curr_node
             let sorted_dist_vec: Vec<_> = result_to_sights.iter()
                 .filter(|(node, _)| unvisited_sights.contains(&node.id))
                 .sorted_unstable_by(|(node1, (_, dist1)), (node2, (_, dist2))| {
                     let score1 = self.scores[&node1.id];
                     let score2 = self.scores[&node2.id];

                     log::trace!("Comparing nodes {} and {}", node1.id, node2.id);
                     log::trace!("Node1: score: {}, distance to current position: {}", score1, dist1);
                     log::trace!("Node2: score: {}, distance to current position: {}", score2, dist2);

                     let metric2 = score2 as f64 / *dist2.max(&1) as f64;
                     let metric1 = score1 as f64 / *dist1.max(&1) as f64;
                     metric2.total_cmp(&metric1)
                 })
                 .map(|(&node, &(_, dist))| (node, dist))
                 .collect();
             log::trace!("Number of unvisited reachable sights from current node {}: {}",
                 curr_node_id, sorted_dist_vec.len());
             log::trace!("Sorted sights:\n{:?}", &sorted_dist_vec);

             // for each sight node, check whether sight can be included in route without violating time budget
             let len_route_before = route.len();
             for (sight_node, dist) in sorted_dist_vec {
                 let secs_needed_to_sight = dist as f64 / self.walking_speed_mps;
                 let result_sight_to_root =
                     dijkstra(&self.graph.get_node(sight_node.id),
                              |&node| successors(node),
                              |&node| node.id == self.root_id);
                 match result_sight_to_root {
                     Some((_, dist_sight_to_root)) => {
                         let secs_needed_sight_to_root = dist_sight_to_root as f64 / self.walking_speed_mps;
                         let secs_total = (secs_needed_to_sight + secs_needed_sight_to_root) as usize + 1;

                         log::trace!("Checking sight {}: secs to include sight: {}, left time budget: {}",
                             sight_node.id, secs_total, time_budget_left);

                         if secs_total <= time_budget_left {
                             log::trace!("Adding sight to route");

                             // add sector containing sight and all intermediate nodes to route
                             let sector_nodes = build_path(&sight_node, &result_to_sights);
                             log::trace!("Appending sector to route:\n{:?}", &sector_nodes);

                             let sector = Sector::with_sight(secs_needed_to_sight as usize,
                                                             self.sights[&sight_node.id],
                                                             sector_nodes);
                             route.push(if curr_node_id == self.root_id {
                                 RouteSector::Start(sector)
                             } else {
                                 RouteSector::Intermediate(sector)
                             });

                             time_budget_left -= secs_total;
                             unvisited_sights.remove(&sight_node.id);
                             curr_node_id = sight_node.id;
                             break;
                        }
                    }
                    None => continue // No path from sight to root found. Continue.
                };
            }

            // check whether any sight has been included in route and if not, go back to root
            if route.len() == len_route_before {
                log::trace!("Traveling back to root");

                let result_to_root =
                    dijkstra(&self.graph.get_node(curr_node_id),
                             |&node| successors(node),
                             |&node| node.id == self.root_id)
                        .expect("No path from last visited sight to root");

                let (sector_nodes, dist_to_root) = result_to_root;
                let secs_to_root = (dist_to_root as f64 / self.walking_speed_mps) as usize;
                log::trace!("Appending sector to route:\n{:?}", &sector_nodes);

                route.push(RouteSector::End(Sector::new(secs_to_root, sector_nodes)));
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
                    RouteSector::Start(sector) => self.scores[&sector.sight.unwrap().node_id],
                    RouteSector::Intermediate(sector) => self.scores[&sector.sight.unwrap().node_id],
                    _ => 0,
                }
            })
            .sum()
    }
}