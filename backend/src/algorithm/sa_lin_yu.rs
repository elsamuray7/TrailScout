use std::collections::HashMap;
use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use itertools::Itertools;
use opening_hours_syntax::rules::RuleKind;
use pathfinding::prelude::*;
use rand::prelude::*;
use crate::algorithm::{_Algorithm, AlgorithmError, Area, Route, RouteSector, ScoreMap, Sector, USER_PREF_MAX, UserPreferences};
use crate::data::graph::{Category, Graph, Sight};
use std::time::Instant;

/// Simulated Annealing internal user preference to score mapping
const USER_PREF_TO_SCORE: [usize; USER_PREF_MAX + 1] = [0, 1, 2, 4, 8, 16];

// Constant parameters
/// Initial temperature
const T_0: f64 = 0.7;
/// Multiplier for iterations on a temperature
const B: usize = 100;
/// Factor by which the temperature is cooled down
const ALPHA: f64 = 0.7;
/// Maximum allowed computation time
const MAX_TIME: u128 = 60_000;
/// Number of cooldowns that do not improve the result
const N_NON_IMPROVING: usize = 5;

/// Maximum number of iterations on a temperature
const MAX_ITER_PER_TEMP: usize = 100 * B;

/// Compute scores for tourist attractions based on user preferences for categories or specific
/// tourist attractions, respectively
fn compute_scores(sights: &Vec<&Sight>, user_prefs: UserPreferences) -> Result<ScoreMap, AlgorithmError> {
    let start = Instant::now();

    let mut scores: ScoreMap = sights.iter()
        .map(|sight| (sight.node_id, (0_usize, sight.category))).collect();

    for category in &user_prefs.categories {
        let category_score = USER_PREF_TO_SCORE[category.get_valid_pref()];
        let category_enum = category.name.parse::<Category>().ok()
            .ok_or_else(|| AlgorithmError::UnknownCategory { unknown_name: category.name.clone() })?;
        sights.iter()
            .filter(|sight| sight.category == category_enum)
            .for_each(|sight| {
                let (prev_score, prev_category) = scores.get_mut(
                    &sight.node_id).unwrap();
                if category_score > *prev_score {
                    *prev_score = category_score;
                    *prev_category = category_enum;
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

    Ok(scores)
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

    let mut count = 0;
    let total = sights_and_root.len();
    for node_id in sights_and_root {
        count += 1;
        log::trace!("Pre-computing distances from node {} ({} / {})", node_id, count, total);
        let dijkstra_result = dijkstra_all(
            &node_id,
            |&node_id| successors(node_id));
        distance_map.insert(node_id, dijkstra_result);
    }
    log::debug!("Pre-computed distances from {} relevant nodes in {} ms", count,
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
    area: Area,
    sights: Vec<&'a Sight>,
    root_id: usize,
    scores: ScoreMap,
    distance_map: HashMap<usize, HashMap<usize, (usize, usize)>>,
}

impl<'a> SimAnnealingLinYu<'a> {
    /// Unique string identifier of this algorithm implementation
    pub const ALGORITHM_NAME: &'static str = "DerAllerbesteste";

    /// Compute wait and service time for given sight based on the already used time budget
    fn compute_wait_and_service_time(&self, sight: &Sight, used_time_budget: i64) -> Result<Option<(i64, i64)>, AlgorithmError> {
        let time_window = sight.opening_hours();

        // Determine current time (after given used time budget)
        let curr_time: NaiveDateTime = (self.start_time + Duration::seconds(used_time_budget))
            .naive_utc();

        // Determine the sights current open state
        // TODO handle DateLimitExceeded error properly
        let curr_state = time_window.state(curr_time)
            .expect("Failed to determine open state");

        // Initialize closure for computing the sights service time
        let latest_time = Utc::now().date().and_hms(23, 59, 59)
            .naive_utc();
        let secs_to_latest_time = latest_time.signed_duration_since(curr_time)
            .num_seconds();
        // Service time cannot exceed tomorrow midnight
        let max_service_time = sight.duration_of_stay_secs().min(secs_to_latest_time);
        let compute_service_time = |close_time: NaiveDateTime| {
            let possible_service_time = close_time.signed_duration_since(curr_time)
                .num_seconds();
            max_service_time.min(possible_service_time)
        };

        // Determine wait and service time based on the sights current open state
        let result = match curr_state {
            RuleKind::Open => {
                match time_window.next_change(curr_time) {
                    Ok(close_time) => {
                        if !time_window.is_closed(close_time) {
                            return Err(AlgorithmError::BadTimeWindow);
                        }
                        Some((0, compute_service_time(close_time)))
                    }
                    _ => Some((0, max_service_time))
                }
            }
            RuleKind::Closed => {
                match time_window.next_change(curr_time) {
                    Ok(open_time) => {
                        if !time_window.is_open(open_time) {
                            return Err(AlgorithmError::BadTimeWindow);
                        }
                        let wait_time = open_time.signed_duration_since(curr_time).num_seconds();
                        match time_window.next_change(curr_time) {
                            Ok(close_time) => {
                                if !time_window.is_closed(close_time) {
                                    return Err(AlgorithmError::BadTimeWindow);
                                }
                                Some((wait_time, compute_service_time(close_time)))
                            }
                            _ => Some((wait_time, max_service_time))
                        }
                    }
                    _ => None // The time window will not open again before midnight
                }
            }
            RuleKind::Unknown => {
                return Err(AlgorithmError::BadTimeWindow);
            }
        };

        Ok(result)
    }

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

            let (wait_time, service_time) = match self.compute_wait_and_service_time(
                sight, total_time_budget - left_time_budget)? {
                Some(result) => result,
                None => break
            };

            let sight_total_time = sight_travel_time + wait_time + service_time;
            if left_time_budget >= (sight_total_time + root_travel_time) {
                score += self.scores[&sight.node_id].0;
                left_time_budget -= sight_total_time;
                curr_node_id = sight.node_id;
            } else {
                break;
            }
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

            let (wait_time, service_time) = match self.compute_wait_and_service_time(
                sight, total_time_budget - left_time_budget)? {
                Some(result) => result,
                None => break
            };

            let sight_total_time = sight_travel_time + wait_time + service_time;
            if left_time_budget >= (sight_total_time + root_travel_time) {
                let path = build_path(&sight.node_id, curr_distance_map)
                    .into_iter().map(|node_id| self.graph.get_node(node_id)).collect_vec();
                let sector =
                    Sector::with_sight(sight_travel_time, wait_time, service_time,
                                       sight, path);
                if route.is_empty() {
                    route.push(RouteSector::Start(sector));
                } else {
                    route.push(RouteSector::Intermediate(sector));
                }
                left_time_budget -= sight_total_time;
                curr_node_id = sight.node_id;
            } else {
                let curr_distance_map = &self.distance_map[&curr_node_id];
                let &(_, root_travel_dist) = curr_distance_map.get(&self.root_id)
                    .ok_or_else(|| AlgorithmError::NoRouteFound { from: curr_node_id, to: self.root_id })?;
                let root_travel_time = (root_travel_dist as f64 / self.walking_speed_mps) as i64 + 1;

                let path = build_path(&self.root_id, curr_distance_map)
                    .into_iter().map(|node_id| self.graph.get_node(node_id)).collect_vec();
                let sector = Sector::new(root_travel_time, path);
                route.push(RouteSector::End(sector));

                break;
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

        let time_budget = end_time.signed_duration_since(start_time).num_seconds() as f64;
        let edge_radius = walking_speed_mps * time_budget / std::f64::consts::PI / 2.0;
        let sights_radius = edge_radius.min(area.radius);
        let sights = graph.get_reachable_sights_in_area(area.lat, area.lon,
                                                        sights_radius, edge_radius);
        if sights.is_empty() {
            return Err(AlgorithmError::NoSightsFound);
        }

        let root_id = graph.get_nearest_node(area.lat, area.lon);
        let scores = compute_scores(&sights, user_prefs)?;
        let distance_map = build_distance_map(
            graph, &area, edge_radius, &sights, root_id, &scores);

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
        let i_iter = (randomized_sights.len() * B).min(MAX_ITER_PER_TEMP);
        let mut i = 0;

        let mut x = randomized_sights;
        let mut old_score = self.get_total_score(&x)?;
        log::debug!("Score of initial solution: {}", old_score);
        let mut x_best = x.clone();
        let mut f_best = old_score;
        let mut non_improving_count = 0;

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

            let mut replace_solution = true;
            if old_score > new_score {
                let score_dif = old_score - new_score;
                let r = rng.gen::<f64>();
                let heur = std::f64::consts::E.powf(-(score_dif as f64) / t);
                if r >= heur {
                    log::trace!("Continue with next iteration (r-value: {} >= heuristic: {})",
                        r, heur);
                    replace_solution = false;
                }
            }

            if replace_solution {
                old_score = new_score;
                x = y;

                if new_score > f_best {
                    log::trace!("Updating best score (new score: {} > best score so far: {})",
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

    fn get_collected_score(&self, _: &Route) -> usize {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Utc};
    use once_cell::sync::Lazy;
    use crate::algorithm::{_Algorithm, Area, RouteSector, Sector, SightCategoryPref, UserPreferences};
    use crate::algorithm::sa_lin_yu::{SimAnnealingLinYu, USER_PREF_TO_SCORE};
    use crate::data::graph::{Category, Graph};
    use crate::init_logging;
    use crate::utils::test_setup;

    #[test]
    fn test_sights_with_multiple_categories() {
        init_logging();

        let graph: &Lazy<Graph> = &test_setup::GRAPH;

        let start_time = DateTime::parse_from_rfc3339("2022-07-01T10:00:00+01:00")
            .unwrap().with_timezone(&Utc);
        let end_time = DateTime::parse_from_rfc3339("2022-07-01T13:00:00+01:00")
            .unwrap().with_timezone(&Utc);
        let algo = SimAnnealingLinYu::new(
            &graph,
            start_time,
            end_time,
            7.0 / 3.6,
            Area {
                lat: 53.064232700000005,
                lon: 8.793089,
                radius: 500.0,
            },
            UserPreferences {
                categories: vec![SightCategoryPref { name: "Nightlife".to_string(), pref: 3 },
                                 SightCategoryPref { name: "Other".to_string(), pref: 5 }],
                sights: vec![],
            }).unwrap();

        let last = algo.sights.first().unwrap();
        for sight in &algo.sights[1..algo.sights.len()] {
            if sight.node_id == last.node_id
                && (sight.category == Category::Nightlife || sight.category == Category::Other) {
                let (score, category) = algo.scores[&sight.node_id];
                assert_eq!(score, USER_PREF_TO_SCORE[5], "Sight got smaller score");
                assert_eq!(category, Category::Other,
                           "Sight associated with category with smaller preference")
            }
        }

        let route = algo.compute_route()
            .expect("Error during route computation");
        let check_sector = |sector: Sector| {
            let sight = sector.sight.unwrap();
            let (_, category) = algo.scores[&sight.node_id];
            assert_eq!(category, sight.category,
                       "Sight in route associated with category with smaller preference");
        };
        for route_sector in route {
            match route_sector {
                RouteSector::Start(sector) => check_sector(sector),
                RouteSector::Intermediate(sector) => check_sector(sector),
                _ => ()
            }
        }
    }
}