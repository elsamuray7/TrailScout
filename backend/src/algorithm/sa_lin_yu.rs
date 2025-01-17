use std::collections::HashMap;
use chrono::{DateTime, Utc};
use itertools::Itertools;
use pathfinding::prelude::*;
use rand::prelude::*;
use crate::algorithm::{_Algorithm, AlgorithmError, Area, compute_wait_and_service_time, EndSector, Route, RouteSector, ScoreMap, Sector, USER_PREF_MAX, UserPreferences};
use crate::data::graph::{Graph, Sight};
use std::time::Instant;
use crate::utils::dijkstra::run_ota_dijkstra_in_area;

/// Simulated Annealing internal user preference to score mapping
const USER_PREF_TO_SCORE: [usize; USER_PREF_MAX + 1] = [0, 1, 2, 4, 8, 16];

// Constant parameters
/// Initial temperature
const T_0: f64 = 0.7;
/// Multiplier for iterations on a temperature
const B: usize = 300;
/// Factor by which the temperature is cooled down
const ALPHA: f64 = 0.7;
/// Maximum allowed computation time
const MAX_TIME: u128 = 60_000;
/// Number of cooldowns that do not improve the result
const N_NON_IMPROVING: usize = 5;

/// Maximum number of sights to consider
const MAX_NUM_SIGHTS: usize = 100;

const SCORE_WEIGHT: f64 = 1.;
const DIST_WEIGHT: f64 = 1.;

/// Compute scores for tourist attractions based on user preferences for categories or specific
/// tourist attractions, respectively
fn compute_scores(sights: &Vec<&Sight>, user_prefs: UserPreferences) -> ScoreMap {
    let start = Instant::now();

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

    log::debug!("Computed scores in {} ms", start.elapsed().as_millis());

    scores
}

/// Build a distance map with distances from relevant nodes, i.e. the root node and all sight nodes
/// with a non-zero score, to all other nodes
fn build_distance_map<'a>(graph: &'a Graph,
                          area: &Area,
                          edge_radius: f64,
                          sights: &Vec<&'a Sight>,
                          root_id: usize,
                          scores: &ScoreMap) -> HashMap<usize, HashMap<usize, (usize, usize)>> {
    let successors = |node_id: usize|
        graph.get_outgoing_edges_in_area(node_id, area.lat, area.lon, edge_radius)
            .into_iter()
            .map(|edge| (edge.tgt, edge.dist))
            .collect::<Vec<(usize, usize)>>();

    let start = Instant::now();

    let mut distance_map = HashMap::with_capacity(sights.len());
    let mut sights_and_root = sights.iter().map(|&sight| sight.node_id)
        .filter(|sight_id| scores[sight_id].0 > 0).collect_vec();
    sights_and_root.push(root_id);

    for node_id in &sights_and_root {
        let dijkstra_result = dijkstra_all(
            node_id,
            |&node_id| successors(node_id));
        distance_map.insert(*node_id, dijkstra_result);
    }
    log::debug!("Pre-computed distances from {} relevant nodes in {} ms", sights_and_root.len(),
        start.elapsed().as_millis());

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

/// Insert the element at position `i` in `current_solution` at position `j` in `current_solution`
/// and remove it from its old position
fn determ_insert(current_solution: &mut Vec<&Sight>, i: usize, j: usize) {
    let elem = current_solution.remove(i);
    current_solution.insert(j, elem);
}

/// Select two indices `i` and `j` by random, insert the element at position `i` in
/// `current_solution` at position `j` in a new copy of `current_solution` and remove it from its
/// old position in the copy
fn insert<'a>(current_solution: &Vec<&'a Sight>) -> Vec<&'a Sight> {
    let mut rng = thread_rng();
    let size = current_solution.len();
    let i = rng.gen_range(0..size);
    let j = rng.gen_range(0..size);

    let mut result = current_solution.clone();
    determ_insert(&mut result, i, j);
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
    sights: Vec<&'a Sight>,
    root_id: usize,
    scores: ScoreMap,
    distance_map: HashMap<usize, HashMap<usize, (usize, usize)>>,
}

impl<'a> SimAnnealingLinYu<'a> {
    /// Unique string identifier of this algorithm implementation
    pub const ALGORITHM_NAME: &'static str = "DerAllerbesteste";

    /// Get the total score of `current_solution`.
    /// The total score is computed as the sum of the individual scores of all sights that can be
    /// included in the route without violating the time budget.
    fn get_total_score(&self, current_solution: &Vec<&'a Sight>) -> Result<usize, AlgorithmError> {
        let mut score = 0;
        let total_time_budget = self.end_time.signed_duration_since(self.start_time).num_seconds();
        let mut left_time_budget = total_time_budget;
        let mut curr_node_id = self.root_id;

        for &sight in current_solution {
            let curr_distance_map = &self.distance_map[&curr_node_id];
            let &(_, sight_travel_dist) = curr_distance_map.get(&sight.node_id)
                .ok_or_else(|| AlgorithmError::NoRouteFound { from: curr_node_id, to: sight.node_id })?;
            let sight_travel_time = (sight_travel_dist as f64 / self.walking_speed_mps) as i64 + 1;

            let sight_distance_map = &self.distance_map[&sight.node_id];
            let &(_, root_travel_dist) = sight_distance_map.get(&self.root_id)
                .ok_or_else(|| AlgorithmError::NoRouteFound { from: sight.node_id, to: self.root_id })?;
            let root_travel_time = (root_travel_dist as f64 / self.walking_speed_mps) as i64 + 1;

            let used_time_budget = total_time_budget - left_time_budget + sight_travel_time;
            match compute_wait_and_service_time(
                &self.start_time, &self.end_time, sight, used_time_budget, root_travel_time) {
                Some((wait_time, service_time)) => {
                    let sight_total_time = sight_travel_time + wait_time + service_time;
                    score += self.scores[&sight.node_id].0;
                    left_time_budget -= sight_total_time;
                    curr_node_id = sight.node_id;
                },
                None => break
            };
        }

        Ok(score)
    }

    /// Perform all possible swap moves on `best_solution` and output the indices to swap to get
    /// the maximum improvement on this solution
    fn perform_all_possible_swaps(&self, best_solution: &mut Vec<&'a Sight>) -> Result<Option<(usize, usize)>, AlgorithmError> {
        let len = best_solution.len();
        let mut best_score = self.get_total_score(best_solution)?;
        let mut best_swap = None;

        for i in 0..len-1 {
            for j in i+1..len {
                best_solution.swap(i, j);
                let new_score = self.get_total_score(best_solution)?;
                if new_score > best_score {
                    best_score = new_score;
                    best_swap = Some((i, j));
                }
                best_solution.swap(j, i);
            }
        }

        Ok(best_swap)
    }

    /// Perform all possible insertion moves on `best_solution` and output the indices to perform
    /// the insertion with the maximum improvement on this solution
    fn perform_all_possible_inserts(&self, best_solution: &mut Vec<&'a Sight>) -> Result<Option<(usize, usize)>, AlgorithmError> {
        let len = best_solution.len();
        let mut best_score = self.get_total_score(best_solution)?;
        let mut best_insert = None;

        for i in 0..len {
            for j in 0..len {
                if i == j {
                    continue;
                }
                determ_insert(best_solution, i, j);
                let new_score = self.get_total_score(best_solution)?;
                if new_score > best_score {
                    best_score = new_score;
                    best_insert = Some((i,j));
                }
                determ_insert(best_solution, j, i);
            }
        }

        Ok(best_insert)
    }

    /// Perform a local search on `best_solution` in order to further improve it
    fn local_search(&self, best_solution: &mut Vec<&'a Sight>) -> Result<(), AlgorithmError> {
        if let Some((i, j)) = self.perform_all_possible_swaps(best_solution)? {
            best_solution.swap(i, j);
        }
        if let Some((i, j)) = self.perform_all_possible_inserts(best_solution)? {
            determ_insert(best_solution, i, j);
        }
        Ok(())
    }

    /// Build a walking route from the best solution found so far
    fn build_route(&self, best_solution: Vec<&'a Sight>) -> Result<Route, AlgorithmError> {
        let mut route = Route::new();
        let total_time_budget = self.end_time.signed_duration_since(self.start_time).num_seconds();
        let mut left_time_budget = total_time_budget;
        let mut curr_node_id = self.root_id;

        for sight in best_solution {
            let curr_distance_map = &self.distance_map[&curr_node_id];
            let &(_, sight_travel_dist) = curr_distance_map.get(&sight.node_id)
                .ok_or_else(|| AlgorithmError::NoRouteFound { from: curr_node_id, to: sight.node_id })?;
            let sight_travel_time = (sight_travel_dist as f64 / self.walking_speed_mps) as i64 + 1;

            let sight_distance_map = &self.distance_map[&sight.node_id];
            let &(_, root_travel_dist) = sight_distance_map.get(&self.root_id)
                .ok_or_else(|| AlgorithmError::NoRouteFound { from: sight.node_id, to: self.root_id })?;
            let root_travel_time = (root_travel_dist as f64 / self.walking_speed_mps) as i64 + 1;

            let used_time_budget = total_time_budget - left_time_budget + sight_travel_time;
            match compute_wait_and_service_time(&self.start_time, &self.end_time, sight,
                                                used_time_budget, root_travel_time) {
                Some((wait_time, service_time)) => {
                    let sight_total_time = sight_travel_time + wait_time + service_time;
                    let path = build_path(&sight.node_id, curr_distance_map)
                        .into_iter().map(|node_id| self.graph.get_node(node_id)).collect_vec();
                    let sector = Sector::new(
                        &self.start_time, total_time_budget - left_time_budget,
                        sight_travel_time, wait_time, service_time, sight, path);
                    if route.is_empty() {
                        route.push(RouteSector::Start(sector));
                    } else {
                        route.push(RouteSector::Intermediate(sector));
                    }
                    left_time_budget -= sight_total_time;
                    curr_node_id = sight.node_id;
                }
                None => break
            };
        }
        let curr_distance_map = &self.distance_map[&curr_node_id];
        let &(_, root_travel_dist) = curr_distance_map.get(&self.root_id)
            .ok_or_else(|| AlgorithmError::NoRouteFound { from: curr_node_id, to: self.root_id })?;
        let root_travel_time = (root_travel_dist as f64 / self.walking_speed_mps) as i64 + 1;
        let path = build_path(&self.root_id, curr_distance_map)
            .into_iter().map(|node_id| self.graph.get_node(node_id)).collect_vec();
        let sector = EndSector::new(
            &self.start_time, total_time_budget - left_time_budget,
            root_travel_time, path);
        route.push(RouteSector::End(sector));

        log::debug!("Built walking route from best found solution");

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

        let time_budget = end_time.signed_duration_since(start_time).num_seconds() as f64;
        let edge_radius = walking_speed_mps * time_budget / std::f64::consts::PI / 2.0;
        let sights_radius = edge_radius.min(area.radius);
        let mut sights = graph.get_reachable_sights_in_area(area.lat, area.lon,
                                                        sights_radius, edge_radius);
        if sights.is_empty() {
            return Err(AlgorithmError::NoSightsFound);
        }

        let root_id = match graph.get_nearest_node_in_area(area.lat, area.lon, sights_radius) {
            Some(nearest_node) => nearest_node,
            None => return Err(AlgorithmError::NoNearestNodeFound)
        };

        let scores = compute_scores(&sights, user_prefs);

        if sights.len() > MAX_NUM_SIGHTS {
            // Keep best `MAX_NUM_SIGHTS` sights based on their score and distance to root
            let result_from_root = run_ota_dijkstra_in_area(graph, root_id,
                                                            area.lat, area.lon, edge_radius);
            let max_score = USER_PREF_TO_SCORE[USER_PREF_MAX] as f64;
            let max_dist = result_from_root.max_dist() as f64;
            sights.sort_unstable_by(|sight1, sight2| {
                let norm_score1 = scores[&sight1.node_id].0 as f64 / max_score;
                let norm_score2 = scores[&sight2.node_id].0 as f64 / max_score;
                // unwrap safety: get_reachable_sights_in_area ensures all sights are reachable
                let norm_dist1 = 1.0 - result_from_root.dist_to(sight1.node_id).unwrap() as f64 / max_dist;
                let norm_dist2 = 1.0 - result_from_root.dist_to(sight2.node_id).unwrap() as f64 / max_dist;
                let metric1 = (SCORE_WEIGHT * norm_score1 + DIST_WEIGHT * norm_dist1)
                    / (SCORE_WEIGHT + DIST_WEIGHT);
                let metric2 = (SCORE_WEIGHT * norm_score2 + DIST_WEIGHT * norm_dist2)
                    / (SCORE_WEIGHT + DIST_WEIGHT);
                metric2.total_cmp(&metric1)
            });
            sights.truncate(MAX_NUM_SIGHTS);
        }

        let distance_map = build_distance_map(
            graph, &area, edge_radius, &sights, root_id, &scores);

        Ok(Self {
            graph,
            start_time,
            end_time,
            walking_speed_mps,
            sights,
            root_id,
            scores,
            distance_map,
        })
    }

    fn compute_route(&self) -> Result<Route, AlgorithmError> {
        log::debug!("Starting route computation");
        let start = Instant::now();

        // Create a random initial route
        let mut rng = thread_rng();

        let mut randomized_sights = self.sights.iter()
            .filter(|sight| {
                let (score, category) = self.scores[&sight.node_id];
                score > 0 && sight.category == category
            })
            .map(|&sight| sight)
            .collect_vec();
        if randomized_sights.is_empty() {
            return Err(AlgorithmError::NoPreferencesProvided);
        }
        randomized_sights.shuffle(&mut rng);
        log::debug!("Computed randomized initial solution");

        log::debug!("Starting simulated annealing (T_0: {}, B: {}, ALPHA: {}, MAX_TIME: {}, N_NON_IMPROVING: {})",
            T_0, B, ALPHA, MAX_TIME, N_NON_IMPROVING);
        let sa_start = Instant::now();

        let mut t = T_0;
        let i_iter = randomized_sights.len() * B;
        let mut i = 0;

        let mut x = randomized_sights;
        let mut old_score = self.get_total_score(&x)?;
        log::debug!("Score of initial solution: {}", old_score);
        let mut x_best = x.clone();
        let mut f_best = old_score;
        let mut non_improving_count = 0;

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
            let new_score = self.get_total_score(&y)?;

            i += 1;

            let mut replace_solution = true;
            if old_score > new_score {
                let score_dif = old_score - new_score;
                let r = rng.gen::<f64>();
                let heur = std::f64::consts::E.powf(-(score_dif as f64) / t);
                if r >= heur {
                    replace_solution = false;
                } else {
                    log::trace!("Escaping from local optimum (new score: {new_score} <= old score: {old_score})");
                }
            }

            if replace_solution {
                old_score = new_score;
                x = y;

                if new_score > f_best {
                    log::trace!("Updating best score (new score: {} > old score: {})",
                        new_score, f_best);
                    f_best = new_score;
                    x_best = x.clone();
                    non_improving_count = 0;
                }
            }

            if i == i_iter {
                t *= ALPHA;
                log::trace!("Updated temperature: {}", t);
                i = 0;

                self.local_search(&mut x_best)?;
                f_best = self.get_total_score(&x_best)?;
                log::trace!("Performed local search on best solution (score: {})", f_best);

                let elapsed = sa_start.elapsed().as_millis();
                if elapsed > MAX_TIME {
                    log::debug!("Reached time limit (elapsed: {}, current temperature: {})",
                        elapsed, t);
                    break;
                }
                if non_improving_count == N_NON_IMPROVING {
                    log::debug!("Reached non-improving limit (non-improving: {}, current temperature: {})",
                        non_improving_count, t);
                    break;
                }
                non_improving_count += 1;
            }
        }

        let route = self.build_route(x_best)?;
        log::debug!("Finished simulated annealing. Computed walking route from node: {} including {} sights with total score: {}.",
             self.root_id, route.len() - 1, f_best);

        log::debug!("Finished route computation in {} ms", start.elapsed().as_millis());

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

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use once_cell::sync::Lazy;
    use crate::algorithm::{_Algorithm, Area, RouteSector, Sector, SightCategoryPref, USER_PREF_MAX, UserPreferences};
    use crate::algorithm::sa_lin_yu::{SimAnnealingLinYu, USER_PREF_TO_SCORE};
    use crate::algorithm::test::{END_TIME, START_TIME, WALKING_SPEED_MPS};
    use crate::data::graph::{Category, Graph};
    use crate::init_logging;
    use crate::utils::test_setup;

    #[test]
    fn test_sights_with_multiple_categories() {
        init_logging();

        let graph: &Lazy<Graph> = &test_setup::GRAPH;

        let start_time = DateTime::parse_from_rfc3339(START_TIME).unwrap()
            .with_timezone(&Utc);
        let end_time = DateTime::parse_from_rfc3339(END_TIME).unwrap()
            .with_timezone(&Utc);
        let algo = SimAnnealingLinYu::new(
            &graph,
            start_time,
            end_time,
            WALKING_SPEED_MPS,
            Area {
                lat: 53.064232700000005,
                lon: 8.793089,
                radius: 500.0,
            },
            UserPreferences {
                categories: vec![SightCategoryPref { category: Category::Activities, pref: 5 },
                                 SightCategoryPref { category: Category::Nightlife, pref: 3 }],
                sights: vec![],
            }).unwrap();

        let mut last = algo.sights.first().unwrap();
        for sight in &algo.sights[1..algo.sights.len()] {
            if sight.node_id == last.node_id
                && (sight.category == Category::Nightlife || sight.category == Category::Activities) {
                let (score, category) = algo.scores[&sight.node_id];
                assert_eq!(score, USER_PREF_TO_SCORE[USER_PREF_MAX], "Sight {} got smaller score", sight.node_id);
                assert_eq!(category, Category::Activities,
                           "Sight {} associated with category with smaller preference", sight.node_id);
                last = sight;
            }
        }

        let route = algo.compute_route()
            .expect("Error during route computation");
        let check_sector = |sector: &Sector| {
            let sight = sector.sight;
            let (_, category) = algo.scores[&sight.node_id];
            assert_eq!(category, sight.category,
                       "Sight {} in route associated with category with smaller preference",
                       sight.node_id);
        };
        for route_sector in &route {
            match route_sector {
                RouteSector::Start(sector) => check_sector(sector),
                RouteSector::Intermediate(sector) => check_sector(sector),
                _ => ()
            }
        }
    }
}