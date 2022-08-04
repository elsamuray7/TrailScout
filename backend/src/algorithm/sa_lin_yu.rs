use std::collections::HashMap;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use pathfinding::prelude::*;
use rand::prelude::*;
use crate::algorithm::{_Algorithm, AlgorithmError, Area, Route, RouteSector, ScoreMap, Sector, UserPreferences};
use crate::data::graph::{Category, Graph, Node, Sight};
use std::time::Instant;

// Constant parameters
/// Initial temperature
const T_0: f64 = 1.;
/// Number of cooldowns that do not improve the result
const N_NON_IMPROVING: usize = 30;
/// Factor by which the temperature is cooled down
const ALPHA: f64 = 0.97;
/// Maximum allowed calculation time
const MAX_TIME: u128 = 5000;

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

/// Build a distance map with distances from relevant nodes, i.e. the root node and all sight nodes
/// with a non-zero score, to all other nodes
fn build_distance_map<'a>(graph: &'a Graph,
                          area: &Area,
                          sights: &HashMap<usize, &'a Sight>,
                          root_id: usize,
                          scores: &ScoreMap) -> HashMap<usize, HashMap<&'a Node, (&'a Node, usize)>> {
    let successors = |node: &Node|
        graph.get_outgoing_edges_in_area(node.id, area.lat, area.lon, area.radius)
            .into_iter()
            .map(|edge| (graph.get_node(edge.tgt), edge.dist))
            .collect::<Vec<(&Node, usize)>>();

    let mut distance_map = HashMap::with_capacity(sights.len());
    let mut sights_and_root = sights.iter().map(|(&sight_id, _)| sight_id)
        .filter(|sight_id| scores[sight_id] > 0).collect_vec();
    sights_and_root.push(root_id);

    let mut count = 0;
    let total = sights_and_root.len();
    for node_id in sights_and_root {
        count += 1;
        log::trace!("Pre-computing distances from node {} ({} / {})", node_id, count, total);
        let dijkstra_result = dijkstra_all(
            &graph.get_node(node_id),
            |node| successors(node));
        distance_map.insert(node_id, dijkstra_result);
    }
    log::debug!("Pre-computed distances from relevant nodes");

    distance_map
}

/// Select two indices by random and swap the elements of `current_solution` at these indices
fn swap<'a>(current_solution: &Vec<&'a Sight>) -> Vec<&'a Sight> {
    let mut rng = thread_rng();
    let size = current_solution.len();
    let i = rng.gen_range(0..size);
    let j = rng.gen_range(0..size);

    let mut result = current_solution.clone();
    result.swap(i, j);
    result
}

/// Select two indices `i` and `j` by random and insert the element at position `i` at position `j`
/// in `current_solution`
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

/// Select two indices by random and reverse the slice of `current_solution` between these two
/// indices
fn reverse<'a>(current_solution: &Vec<&'a Sight>) -> Vec<&'a Sight> {
    let mut rng = thread_rng();
    let size = current_solution.len();
    let i = rng.gen_range(0..size);
    let j = rng.gen_range(0..size);

    let mut result = current_solution.clone();
    let partial_solution;
    if j < i {
        partial_solution = &mut result[j..=i];
    } else {
        partial_solution = &mut result[i..=j];
    }
    partial_solution.reverse();
    result
}

/// Implementation of the `Algorithm` trait based on Lin and Yu's Simulated Annealing algorithm
/// (2012).
///
/// The simulated annealing algorithm tries to find the best route by generating a random initial
/// solution and improve it with the local operations `swap`, `insert` and `reverse`.
/// Therefore, we start with the initial temperature `T_0`, which represents the probability with
/// which we escape from local maxima.
/// The temperature will always cool down after a certain amount of iterations and the algorithm
/// stops and outputs the best solution found so far if it already ran more than `MAX_TIME` seconds.
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
    distance_map: HashMap<usize, HashMap<&'a Node, (&'a Node, usize)>>,
}

impl<'a> SimAnnealingLinYu<'a> {
    /// Unique string identifier of this algorithm implementation
    pub const ALGORITHM_NAME: &'static str = "DerAllerbesteste";

    /// Get the total score of `current_solution`.
    /// The total score is computed as the sum of the individual scores of all sights that can be
    /// included in the route without violating the time budget.
    fn get_total_score(&self, current_solution: &Vec<&'a Sight>) -> Result<usize, AlgorithmError> {
        // TODO compute scores according to issue #145
        let mut score = 0;
        let mut time_budget = (self.end_time.timestamp() - self.start_time.timestamp()) as usize;
        let mut curr_node_id = self.root_id;

        for &sight in current_solution {
            let curr_distance_map = &self.distance_map[&curr_node_id];
            let &(_, sight_travel_dist) = curr_distance_map.get(&self.graph.get_node(sight.node_id))
                .ok_or(AlgorithmError::NoRouteFound { from: curr_node_id, to: sight.node_id })?;
            let sight_travel_time = (sight_travel_dist as f64 / self.walking_speed_mps) as usize + 1;

            let sight_distance_map = &self.distance_map[&sight.node_id];
            let &(_, root_travel_dist) = sight_distance_map.get(&self.graph.get_node(self.root_id))
                .ok_or(AlgorithmError::NoRouteFound { from: sight.node_id, to: self.root_id })?;
            let root_travel_time = (root_travel_dist as f64 / self.walking_speed_mps) as usize + 1;

            if time_budget >= (sight_travel_time + root_travel_time) {
                score += self.scores[&sight.node_id];
                time_budget -= sight_travel_time;
                curr_node_id = sight.node_id;
            } else {
                break;
            }
        }

        Ok(score)
    }

    /// Build a walking route from the best solution found so far
    fn build_route(&self, best_solution: Vec<&'a Sight>) -> Result<Route, AlgorithmError> {
        let mut route = Route::new();
        let mut time_budget = (self.end_time.timestamp() - self.start_time.timestamp()) as usize;
        let mut curr_node_id = self.root_id;

        for sight in best_solution {
            let curr_distance_map = &self.distance_map[&curr_node_id];
            let &(_, sight_travel_dist) = curr_distance_map.get(&self.graph.get_node(sight.node_id))
                .ok_or(AlgorithmError::NoRouteFound { from: curr_node_id, to: sight.node_id })?;
            let sight_travel_time = (sight_travel_dist as f64 / self.walking_speed_mps) as usize + 1;

            let sight_distance_map = &self.distance_map[&sight.node_id];
            let &(_, root_travel_dist) = sight_distance_map.get(&self.graph.get_node(self.root_id))
                .ok_or(AlgorithmError::NoRouteFound { from: sight.node_id, to: self.root_id })?;
            let root_travel_time = (root_travel_dist as f64 / self.walking_speed_mps) as usize + 1;

            if time_budget >= (sight_travel_time + root_travel_time) {
                let sector =
                    Sector::with_sight(sight_travel_time, sight, build_path(
                        &self.graph.get_node(sight.node_id), curr_distance_map));
                if route.is_empty() {
                    route.push(RouteSector::Start(sector));
                } else {
                    route.push(RouteSector::Intermediate(sector));
                }
                curr_node_id = sight.node_id;
                time_budget -= sight_travel_time;
            } else {
                let curr_distance_map = &self.distance_map[&curr_node_id];
                let &(_, root_travel_dist) = curr_distance_map.get(&self.graph.get_node(self.root_id))
                    .ok_or(AlgorithmError::NoRouteFound { from: curr_node_id, to: self.root_id })?;
                let root_travel_time = (root_travel_dist as f64 / self.walking_speed_mps) as usize + 1;

                let sector = Sector::new(root_travel_time, build_path(
                    &self.graph.get_node(self.root_id), curr_distance_map));
                route.push(RouteSector::End(sector));
            }
        }
        log::debug!("Computed walking route");

        Ok(route)
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
        let distance_map = build_distance_map(graph, &area, &sights, root_id, &scores);
        Ok(Self {
            graph,
            start_time,
            end_time,
            walking_speed_mps,
            area,
            sights,
            root_id,
            scores,
            distance_map,
        })
    }

    fn compute_route(&self) -> Result<Route, AlgorithmError> {
        // Create a random initial route
        let mut rng = thread_rng();
        let mut randomized_sights = self.sights.iter()
            .filter(|(sight_id, _)| self.scores[*sight_id] > 0)
            .map(|(_, &sight)| sight).collect_vec();
        randomized_sights.shuffle(&mut rng);
        log::debug!("Computed randomized initial solution");

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

        log::debug!("Starting simulated annealing");

        let i_iter = randomized_sights.len() * 5000;
        let start_time = Instant::now();

        let mut x = randomized_sights;
        let mut old_score = self.get_total_score(&x)?;
        let mut x_best = x.clone();
        let mut f_best = old_score;

        let mut t = T_0;
        let mut i = 0;

        loop {
            let p = rng.gen::<f64>();
            log::trace!("Computed p-value: {}", p);

            let y;
            if p <= 1./3. {
                y = swap(&x);
            } else if p <= 2./3. {
                y = insert(&x);
            } else {
                y = reverse(&x);
            }
            let new_score = self.get_total_score(&y)?;
            log::trace!("Computed new solution from current solution (old score: {}, new score: {})",
                old_score, new_score);

            i += 1;
            log::trace!("Iteration {} / {}", i, i_iter);

            if old_score > new_score {
                let score_dif = old_score - new_score;
                let r = rng.gen::<f64>();
                let heur = std::f64::consts::E.powf(-(score_dif as f64) / t);
                if r >= heur {
                    log::trace!("Continue with next iteration (r-value: {} >= heuristic: {})",
                        r, heur);
                    continue;
                }
            }
            old_score = new_score;
            x = y;

            if new_score > f_best {
                f_best = new_score;
                x_best = x.clone();
                log::trace!("Updated best score (new score: {} > best score so far: {})",
                    new_score, f_best);
            }

            if i == i_iter {
                t *= ALPHA;
                log::trace!("Updated temperature: {}", t);
                i = 0;

                //TODO: PERFORM LOCAL SEARCH WHATEVER THAT MEANS

                let elapsed = start_time.elapsed().as_millis();
                if elapsed > MAX_TIME {
                    log::trace!("Reached time limit (elapsed: {} > limit: {})", elapsed, MAX_TIME);
                    break;
                }
            }
        }

        log::debug!("Finished simulated annealing");

        self.build_route(x_best)
    }
}