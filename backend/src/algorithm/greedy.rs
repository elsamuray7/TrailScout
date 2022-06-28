use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use crate::data::graph::{Category, Graph, Node, Sight};
use itertools::Itertools;
use pathfinding::prelude::*;
use crate::algorithm::{Algorithm, Area, Route, RouteSector, ScoreMap, Sector, UserPreferences};

/// Compute scores for tourist attractions based on user preferences for categories or specific
/// tourist attractions, respectively
///
/// TODO map user preference number to algorithm internal score number
fn compute_scores(sights: &HashMap<usize, &Sight>, user_prefs: UserPreferences) -> ScoreMap {
    let mut scores: ScoreMap = sights.iter()
        .map(|(&sight_id, _)| (sight_id, 0_usize))
        .collect();
    for category in &user_prefs.categories {
        let _category_enum = category.name.parse::<Category>()
            .unwrap_or(Category::Other);
        sights.iter()
            .filter(|(_, sight)| matches!(&sight.category, _category_enum))
            .for_each(|(&sight_id, _)| {
                scores.insert(sight_id, category.pref);
            });
    }
    for sight in &user_prefs.sights {
        // TODO implement check whether SightPref really corresponds to sight
        scores.insert(sight.id, sight.pref);
    }
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
         let successors = |node: &Node|
             self.graph.get_outgoing_edges_in_area(node.id, self.area.lat, self.area.lon, self.area.radius)
                 .into_iter()
                 .map(|edge| (self.graph.get_node(edge.tgt), edge.dist))
                 .collect::<Vec<(&Node, usize)>>();

         let mut route: Route = vec![];
         let mut time_budget_left = (self.end_time.timestamp() - self.start_time.timestamp()) as usize;
         let mut sights_left: HashSet<_> = self.sights.keys().map(usize::to_owned).collect();
         let mut curr_node_id = self.root_id;
         loop {
             // calculate distances from curr_node to all sight nodes
             let result_to_sights: HashMap<&Node, (&Node, usize)> =
                 dijkstra_all(&self.graph.get_node(curr_node_id),
                              |&node| successors(node));

             // sort sight nodes by their distance to curr_node
             let sorted_dist_vec: Vec<_> = result_to_sights.values()
                 .filter(|(node, _)| sights_left.contains(&node.id))
                 .sorted_unstable_by(|(node1, dist1), (node2, dist2)| {
                     let score1 = self.scores[&node1.id];
                     let score2 = self.scores[&node2.id];

                     log::debug!("Comparing nodes {} and {}", node1.id, node2.id);
                     log::debug!("Node1: score: {}, distance to current position: {}", score1, dist1);
                     log::debug!("Node2: score: {}, distance to current position: {}", score2, dist2);

                     (score1 / dist1.max(&1)).cmp(&(score2 / dist2.max(&2)))
                 })
                 .collect();
             log::debug!("Sorted sights:\n{:?}", &sorted_dist_vec);

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

                         log::debug!("Checking sight {}: secs to include sight: {}, left time budget: {}",
                             sight_node.id, secs_total, time_budget_left);

                         if secs_total <= time_budget_left {
                             log::debug!("Adding sight to route");

                             // add sector containing sight and all intermediate nodes to route
                             let sector_nodes = build_path(&sight_node, &result_to_sights);
                             log::debug!("Appending sector to route:\n{:?}", &sector_nodes);

                             let sector = Sector::with_sight(secs_needed_to_sight as usize,
                                                             self.sights[&sight_node.id],
                                                             sector_nodes);
                             route.push(if curr_node_id == self.root_id {
                                 RouteSector::Start(sector)
                             } else {
                                 RouteSector::Intermediate(sector)
                             });

                             time_budget_left -= secs_total;
                             sights_left.remove(&sight_node.id);
                             curr_node_id = sight_node.id;
                             break;
                        }
                    }
                    None => continue // No path from sight to root found. Continue.
                };
            }

            // check whether any sight has been included in route and if not, go back to root
            if route.len() == len_route_before {
                log::debug!("Traveling back to root");

                let result_to_root =
                    dijkstra(&self.graph.get_node(curr_node_id),
                             |&node| successors(node),
                             |&node| node.id == self.root_id)
                        .expect("No path from last visited sight to root");

                let (sector_nodes, dist_to_root) = result_to_root;
                let secs_to_root = (dist_to_root as f64 / self.walking_speed_mps) as usize;
                log::debug!("Appending sector to route:\n{:?}", &sector_nodes);

                route.push(RouteSector::End(Sector::new(secs_to_root, sector_nodes)));
                break;
            }
        }

        route
    }

    fn map_node_to_sight(&self, node: &Node) -> Option<&Sight> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use crate::algorithm::{Algorithm, Area, SightCategoryPref, UserPreferences};
    use crate::algorithm::greedy::GreedyAlgorithm;
    use crate::data::graph::Graph;

    #[test]
    fn test_greedy() {
        let graph = Graph::parse_from_file("./osm_graphs/bremen-latest.fmi").unwrap();

        let algo = GreedyAlgorithm::new(&graph,
                                        DateTime::parse_from_rfc3339("1996-12-19T10:39:57-08:00").unwrap().with_timezone(&Utc),
                                        DateTime::parse_from_rfc3339("1996-12-19T20:39:57-08:00").unwrap().with_timezone(&Utc),
                                        7.0 / 3.6,
                                        Area {
                                            lat: 53.14519850000001,
                                            lon: 8.8384274,
                                            radius: 5.0,
                                        },
                                        UserPreferences {
                                            categories: vec![SightCategoryPref{ name: "Restaurants".to_string(), pref: 5 }],
                                            sights: vec![],
                                        });
        let route = algo.compute_route();

        println!("Computed travel route:\n{:#?}", &route);
    }
}