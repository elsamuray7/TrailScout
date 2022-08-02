use std::collections::HashMap;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use pathfinding::prelude::*;
use rand::prelude::*;
use crate::algorithm::{_Algorithm, AlgorithmError, Area, Route, RouteSector, ScoreMap, Sector, UserPreferences};
use crate::data::graph::{Category, Graph, Node, Sight};
use std::time::Instant;

// Constant parameters
// Initial temperature
const T_0: f64 = 1.;
// Number of cooldowns that do not improve the result
const N_NON_IMPROVING: usize = 30;
// Factor by which the temperature is cooled down
const ALPHA: f64 = 0.97;
// Maximum allowed calculation time
const MAX_T: u128 = 5000;

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
                scores.insert(sight_id, category.pref);
            });
    }
    for sight in &user_prefs.sights {
        // TODO implement check whether SightPref really corresponds to sight
        scores.insert(sight.id, sight.pref);
    }
    log::debug!("Computed scores: {:?}", &scores);

    scores
}

/// Greedy implementation of the `Algorithm` trait.
///
/// The greedy algorithm tries to find the best route by including sights into the route based on
/// their score-distance ratio at that time until the time budget is used up.
pub struct SimAnnealingLinYu<'a> {
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

impl SimAnnealingLinYu<'_> {
    /// Unique string identifier of this algorithm implementation
    pub const ALGORITHM_NAME: &'static str = "DerAllerbesteste";

    fn calculate_score(&self, current_solution: &Vec<&Sight>, distance_map: &HashMap<usize, HashMap<&Node, (&Node, usize)>>) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0..100)
    }
}

impl<'a> _Algorithm<'a> for SimAnnealingLinYu<'a> {
    fn new(graph: &'a Graph,
           start_time: DateTime<Utc>,
           end_time: DateTime<Utc>,
           walking_speed_mps: f64,
           area: Area,
           user_prefs: UserPreferences) -> Result<Self, AlgorithmError> where Self: Sized {
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

    fn compute_route(&self) -> Route {
        let successors = |node: &Node|
            self.graph.get_outgoing_edges_in_area(node.id, self.area.lat, self.area.lon, self.area.radius)
                .into_iter()
                .map(|edge| (self.graph.get_node(edge.tgt), edge.dist))
                .collect::<Vec<(&Node, usize)>>();

        // Get the distances from the root and all sights to all other nodes
        let mut distance_map = HashMap::with_capacity(self.sights.len());
        let mut sights_and_root = self.sights.iter().map(|(&sight_id, _)| sight_id)
            .collect_vec();
        sights_and_root.push(self.root_id);
        for node_id in sights_and_root {
            let dijkstra_result = dijkstra_all(
                &self.graph.get_node(node_id),
                |node| successors(node));
            distance_map.insert(node_id, dijkstra_result);
        }

        // Create a random initial route
        let mut rng = thread_rng();
        let mut randomized_sights: Vec<_> = self.sights.iter()
            .map(|(_, &sight)| sight).collect();
        randomized_sights.shuffle(&mut rng);

        // // Enrich sight data with distance from previous sight or root, respectively
        // let mut initial_route = Vec::with_capacity(self.sights.len());
        // for (i, &sight) in randomized_sights.iter().enumerate() {
        //     if i == 0 {
        //         let distances = &distance_map[&self.root_id];
        //         let (_, dist) = distances[&self.graph.get_node(sight.node_id)];
        //         initial_route.push((sight, dist));
        //     } else {
        //         let distances = &distance_map[&randomized_sights[i - 1].node_id];
        //         let (_, dist) = distances[&self.graph.get_node(sight.node_id)];
        //         initial_route.push((sight, dist));
        //     }
        // }

        let i_iter = randomized_sights.len() * 5000;
        let start_time = Instant::now();

        let mut x = randomized_sights;
        let mut old_score = self.calculate_score(&x, &distance_map);
        let mut x_best = x.clone();
        let mut f_best = old_score;

        let mut t = T_0;
        let mut i = 0;

        loop {
            let p = rng.gen::<f64>();

            let y;
            if p <= 1./3. {
                y = swap(&x);
            } else if p <= 2./3. {
                y = insert(&x);
            } else {
                y = reverse(&x);
            }
            let new_score = self.calculate_score(&y, &distance_map);

            i += 1;

            if old_score > new_score {
                let score_dif = old_score - new_score;
                let r = rng.gen::<f64>();
                if r >= std::f64::consts::E.powf(-(score_dif as f64) / t) {
                    continue;

                }
            }
            old_score = new_score;
            x = y;

            if new_score > f_best {
                f_best = new_score;
                x_best = x.clone();
            }

            if i == i_iter {
                t *= ALPHA;
                i = 0;

                //TODO: PERFORM LOCAL SEARCH WHATEVER THAT MEANS

                let elapsed = start_time.elapsed().as_millis();
                if elapsed > MAX_T {
                    break;
                }
            }
        }

        let mut route = Route::new();
        let mut time_budget = (self.end_time.timestamp() - self.start_time.timestamp()) as usize;
        let mut curr_node_id = self.root_id;

        for sight in x_best {
            let sight_distance_map = &distance_map[&curr_node_id];
            let (_, dist) = sight_distance_map[&self.graph.get_node(sight.node_id)];
            let tb = (dist as f64 / self.walking_speed_mps) as usize + 1;
            let result_sight_to_root =
                dijkstra(&self.graph.get_node(sight.node_id),
                         |&node| successors(node),
                         |&node| node.id == self.root_id)
                    .expect("No route from sight to be included to end point");
            let back = (result_sight_to_root.1 as f64 / self.walking_speed_mps) as usize + 1;
            if time_budget >= (tb + back) {
                let sector = Sector::with_sight(tb, sight, build_path(
                    &self.graph.get_node(sight.node_id), sight_distance_map));
                if route.is_empty() {
                    route.push(RouteSector::Start(sector));
                } else {
                    route.push(RouteSector::Intermediate(sector));
                }
                curr_node_id = sight.node_id;
                time_budget -= tb;
            } else {
                let result_sight_to_root =
                    dijkstra(&self.graph.get_node(curr_node_id),
                             |&node| successors(node),
                             |&node| node.id == self.root_id)
                        .expect("No route from current sight to end point");
                let tb = (result_sight_to_root.1 as f64 / self.walking_speed_mps) as usize + 1;
                let sector = Sector::new(tb, result_sight_to_root.0);
                route.push(RouteSector::End(sector));
            }
        }

        route
    }
}

fn swap<'a>(current_solution: &Vec<&'a Sight>) -> Vec<&'a Sight> {
    let mut rng = thread_rng();
    let size = current_solution.len();
    let i = rng.gen_range(0..size);
    let j = rng.gen_range(0..size);

    let mut result = current_solution.clone();
    result.swap(i, j);
    result
}

fn insert<'a>(current_solution: &Vec<&'a Sight>) -> Vec<&'a Sight> {
    let mut rng = thread_rng();
    let size = current_solution.len();
    let i = rng.gen_range(0..size);
    let j = rng.gen_range(0..size);

    let mut result = current_solution.clone();
    if j < i {
        result.insert(j, current_solution[i]);
        result.remove(i + 1);
    } else if j > i {
        result.insert(j, current_solution[i]);
        result.remove(i);
    }
    result
}

fn reverse<'a>(current_solution: &Vec<&'a Sight>) -> Vec<&'a Sight> {
    let mut rng = thread_rng();
    let size = current_solution.len();
    let i = rng.gen_range(0..size);
    let j = rng.gen_range(0..size);

    let mut result = current_solution.clone();
    let partial_solution = &mut result[i..=j];
    partial_solution.reverse();
    result
}

#[cfg(test)]
mod example {
    #[test]
    fn test() {
        let mut myvec = vec![1, 2, 3, 4, 5, 6];
        let mut my_slice = &mut myvec[1..=4];
        my_slice.reverse();
        println!("{:?}", &myvec);
    }
}