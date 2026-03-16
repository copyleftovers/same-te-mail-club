// This module is an internal algorithm with no public API consumers that need custom hashers.
#![allow(clippy::implicit_hasher)]

use std::collections::HashMap;
use uuid::Uuid;

/// Input to the assignment algorithm.
pub struct AssignmentInput {
    /// Confirmed participant IDs.
    pub participants: Vec<Uuid>,
    /// Social weight between two participants. Higher = stronger existing connection.
    /// Keyed by (min(a,b), max(a,b)) for canonical ordering.
    pub social_weights: HashMap<(Uuid, Uuid), u32>,
}

/// A single cohort's assignment cycle.
pub struct Cycle {
    /// Ordered list of participant IDs forming the cycle.
    /// participants[0] sends to participants[1], ..., participants[N-1] sends to participants[0].
    pub participants: Vec<Uuid>,
    /// Total social weight score (lower is better).
    pub score: u32,
}

/// Full assignment result.
pub struct AssignmentResult {
    pub cohorts: Vec<Cycle>,
}

/// Compute canonical key for a pair of participants.
/// Always (min, max) for consistent `HashMap` lookups.
#[must_use]
pub fn weight_key(a: Uuid, b: Uuid) -> (Uuid, Uuid) {
    if a < b { (a, b) } else { (b, a) }
}

/// Build the social weight matrix from DB data.
///
/// Called by `admin/assignments.rs` before running the algorithm.
///
/// `group_memberships`: `(user_id, group_id, group_weight)` — from `known_group_members` joined
/// with `known_groups`.
///
/// `past_pairings`: `(sender, recipient)` — queried from `assignments JOIN seasons WHERE phase IN
/// ('complete', 'cancelled')`.
#[must_use]
pub fn build_weight_matrix(
    group_memberships: &[(Uuid, Uuid, u32)],
    past_pairings: &[(Uuid, Uuid)],
) -> HashMap<(Uuid, Uuid), u32> {
    let mut weights: HashMap<(Uuid, Uuid), u32> = HashMap::new();

    // Index memberships by group_id for pairwise comparison.
    let mut groups: HashMap<Uuid, Vec<(Uuid, u32)>> = HashMap::new();
    for &(user_id, group_id, group_weight) in group_memberships {
        groups
            .entry(group_id)
            .or_default()
            .push((user_id, group_weight));
    }

    // For each group, add weight for every pair of members.
    for members in groups.values() {
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                let key = weight_key(members[i].0, members[j].0);
                // Use the group weight (same for all members of a group).
                *weights.entry(key).or_insert(0) += members[i].1;
            }
        }
    }

    // +1 for each past pairing (either direction).
    for &(sender, recipient) in past_pairings {
        let key = weight_key(sender, recipient);
        *weights.entry(key).or_insert(0) += 1;
    }

    weights
}

/// Split N participants into cohorts of 3–15.
///
/// For N <= 15: single cohort.
/// For N > 15: find the partition minimizing max deviation from mean group size.
/// All groups must be in [3, 15] range.
#[must_use]
pub fn split_cohorts(participants: &[Uuid]) -> Vec<Vec<Uuid>> {
    let n = participants.len();

    if n <= 15 {
        return vec![participants.to_vec()];
    }

    // Find the best number of groups and their sizes.
    let best_partition = find_best_partition(n);

    let mut result = Vec::new();
    let mut offset = 0;
    for size in best_partition {
        result.push(participants[offset..offset + size].to_vec());
        offset += size;
    }

    result
}

/// Find the best partition of `n` into groups.
///
/// Prefers groups in [11, 15] (target range). Falls back to [3, 15] only when
/// the target range yields no valid partition.
/// "Best" = fewest groups first (maximizes group size), then minimizes max deviation.
fn find_best_partition(n: usize) -> Vec<usize> {
    // Try target range [11, 15] first.
    if let Some(partition) = find_partition_in_range(n, 11, 15) {
        return partition;
    }
    // Fall back to [3, 15].
    find_partition_in_range(n, 3, 15)
        .expect("no valid partition found — caller should ensure N >= 3")
}

/// Find the most balanced partition of `n` into groups of size [`min_size`, `max_size`].
///
/// Among all valid group counts, pick the one with fewest groups (largest groups).
/// Ties broken by minimizing max deviation from mean.
fn find_partition_in_range(n: usize, min_size: usize, max_size: usize) -> Option<Vec<usize>> {
    let min_groups = n.div_ceil(max_size);
    let max_groups = n / min_size;

    let mut best: Option<Vec<usize>> = None;
    let mut best_k = usize::MAX;
    let mut best_max_dev: u64 = u64::MAX;

    for k in min_groups..=max_groups {
        let base_size = n / k;
        let remainder = n % k;
        let largest = base_size + usize::from(remainder > 0);

        if base_size < min_size || largest > max_size {
            continue;
        }

        let mut sizes = Vec::with_capacity(k);
        // Compute max deviation as integer: at most 1 (since groups differ by at most 1).
        // Use remainder count as tie-breaker — fewer remainder groups = more balanced.
        let max_dev = u64::from(remainder > 0) * 1000 + remainder as u64;

        for i in 0..k {
            let size = if i < remainder {
                base_size + 1
            } else {
                base_size
            };
            sizes.push(size);
        }

        // Prefer fewer groups (larger cohorts), then lower deviation.
        let better = k < best_k || (k == best_k && max_dev < best_max_dev);
        if best.is_none() || better {
            best_k = k;
            best_max_dev = max_dev;
            best = Some(sizes);
        }
    }

    best
}

/// Look up the social weight between two participants.
fn edge_weight(a: Uuid, b: Uuid, social_weights: &HashMap<(Uuid, Uuid), u32>) -> u32 {
    social_weights.get(&weight_key(a, b)).copied().unwrap_or(0)
}

/// Score a complete cycle: sum of social weights of all edges.
fn score_cycle(cycle: &[Uuid], social_weights: &HashMap<(Uuid, Uuid), u32>) -> u32 {
    let n = cycle.len();
    let mut total: u32 = 0;
    for i in 0..n {
        let next = (i + 1) % n;
        total += edge_weight(cycle[i], cycle[next], social_weights);
    }
    total
}

/// Generate a Hamiltonian cycle for a cohort that minimizes total social weight.
///
/// Backtracking DFS with greedy heuristic. Multiple random restarts, keep best.
///
/// # Panics
///
/// Panics if `participants` has fewer than 3 elements.
#[must_use]
pub fn generate_cycle(
    participants: &[Uuid],
    social_weights: &HashMap<(Uuid, Uuid), u32>,
    attempts: usize,
) -> Cycle {
    use rand::seq::SliceRandom;

    let n = participants.len();
    assert!(n >= 3, "need at least 3 participants for a cycle");

    let mut best_cycle: Option<Vec<Uuid>> = None;
    let mut best_score = u32::MAX;

    let mut rng = rand::rng();

    for _ in 0..attempts {
        let mut order: Vec<Uuid> = participants.to_vec();
        order.shuffle(&mut rng);

        if let Some(cycle) = try_build_cycle(&order, social_weights) {
            let s = score_cycle(&cycle, social_weights);
            if s < best_score {
                best_score = s;
                best_cycle = Some(cycle);
            }
        }
    }

    // If no randomized attempt works, just use the original order (always valid as a cycle).
    let cycle = best_cycle.unwrap_or_else(|| participants.to_vec());
    let score = score_cycle(&cycle, social_weights);

    Cycle {
        participants: cycle,
        score,
    }
}

/// Try to build a Hamiltonian cycle starting from `start_order[0]`.
///
/// Greedy DFS: at each step, pick the unvisited neighbor with the lowest
/// social weight to the current node. Backtrack if stuck.
fn try_build_cycle(
    start_order: &[Uuid],
    social_weights: &HashMap<(Uuid, Uuid), u32>,
) -> Option<Vec<Uuid>> {
    let n = start_order.len();
    let mut path: Vec<Uuid> = vec![start_order[0]];
    let mut visited = vec![false; n];
    visited[0] = true;

    if backtrack(&mut path, &mut visited, start_order, social_weights, n) {
        Some(path)
    } else {
        None
    }
}

fn backtrack(
    path: &mut Vec<Uuid>,
    visited: &mut [bool],
    participants: &[Uuid],
    social_weights: &HashMap<(Uuid, Uuid), u32>,
    n: usize,
) -> bool {
    if path.len() == n {
        // Complete cycle — last node connects back to first.
        return true;
    }

    let current = *path.last().expect("path is non-empty");

    // Build candidate list sorted by edge weight (ascending — greedy).
    let mut candidates: Vec<(u32, usize)> = Vec::new();
    for (i, visited_flag) in visited.iter().enumerate() {
        if !visited_flag {
            let w = edge_weight(current, participants[i], social_weights);
            candidates.push((w, i));
        }
    }
    candidates.sort_unstable();

    for (_, idx) in candidates {
        visited[idx] = true;
        path.push(participants[idx]);

        if backtrack(path, visited, participants, social_weights, n) {
            return true;
        }

        path.pop();
        visited[idx] = false;
    }

    false
}

/// Top-level: split into cohorts, generate cycle for each.
#[must_use]
pub fn generate_assignments(input: &AssignmentInput) -> AssignmentResult {
    let cohorts = split_cohorts(&input.participants);
    let cycles = cohorts
        .iter()
        .map(|cohort| generate_cycle(cohort, &input.social_weights, 100))
        .collect();
    AssignmentResult { cohorts: cycles }
}

/// Validate that a set of assignments forms valid cycles.
///
/// Every participant sends to exactly one, receives from exactly one.
/// Each cohort forms a single connected loop.
///
/// # Errors
///
/// Returns `Err` with a description if the topology is invalid.
pub fn validate_cycles(result: &AssignmentResult) -> Result<(), String> {
    let mut all_senders: HashMap<Uuid, Uuid> = HashMap::new();
    let mut all_receivers: HashMap<Uuid, Uuid> = HashMap::new();

    for (cohort_idx, cycle) in result.cohorts.iter().enumerate() {
        let n = cycle.participants.len();
        if n < 3 {
            return Err(format!("cohort {cohort_idx} has fewer than 3 participants"));
        }

        // Check cycle forms a single loop: each participant sends to the next.
        let mut seen = std::collections::HashSet::new();
        for i in 0..n {
            let sender = cycle.participants[i];
            let recipient = cycle.participants[(i + 1) % n];

            if !seen.insert(sender) {
                return Err(format!(
                    "cohort {cohort_idx}: duplicate participant {sender}"
                ));
            }

            if let Some(existing) = all_senders.insert(sender, recipient) {
                return Err(format!(
                    "participant {sender} sends to both {existing} and {recipient}"
                ));
            }

            if let Some(existing) = all_receivers.insert(recipient, sender) {
                return Err(format!(
                    "participant {recipient} receives from both {existing} and {sender}"
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_uuids(n: usize) -> Vec<Uuid> {
        (0..n).map(|_| Uuid::new_v4()).collect()
    }

    // ── split_cohorts ────────────────────────────────────────────────────────

    #[test]
    fn split_3_participants_single_cohort() {
        let ids = make_uuids(3);
        let cohorts = split_cohorts(&ids);
        assert_eq!(cohorts.len(), 1);
        assert_eq!(cohorts[0].len(), 3);
    }

    #[test]
    fn split_11_participants_single_cohort() {
        let ids = make_uuids(11);
        let cohorts = split_cohorts(&ids);
        assert_eq!(cohorts.len(), 1);
        assert_eq!(cohorts[0].len(), 11);
    }

    #[test]
    fn split_15_participants_single_cohort() {
        let ids = make_uuids(15);
        let cohorts = split_cohorts(&ids);
        assert_eq!(cohorts.len(), 1);
        assert_eq!(cohorts[0].len(), 15);
    }

    #[test]
    fn split_16_participants_two_cohorts() {
        let ids = make_uuids(16);
        let cohorts = split_cohorts(&ids);
        assert_eq!(cohorts.len(), 2);
        let total: usize = cohorts.iter().map(Vec::len).sum();
        assert_eq!(total, 16);
        for c in &cohorts {
            assert!(c.len() >= 3);
            assert!(c.len() <= 15);
        }
    }

    #[test]
    fn split_25_participants_two_cohorts_balanced() {
        let ids = make_uuids(25);
        let cohorts = split_cohorts(&ids);
        assert_eq!(cohorts.len(), 2);
        let total: usize = cohorts.iter().map(Vec::len).sum();
        assert_eq!(total, 25);
        // Should be 13+12 or 12+13, not 15+10.
        let mut sizes: Vec<usize> = cohorts.iter().map(Vec::len).collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![12, 13]);
    }

    #[test]
    fn split_30_participants_two_cohorts_of_15() {
        let ids = make_uuids(30);
        let cohorts = split_cohorts(&ids);
        assert_eq!(cohorts.len(), 2);
        for c in &cohorts {
            assert_eq!(c.len(), 15);
        }
    }

    #[test]
    fn split_31_participants() {
        let ids = make_uuids(31);
        let cohorts = split_cohorts(&ids);
        let total: usize = cohorts.iter().map(Vec::len).sum();
        assert_eq!(total, 31);
        for c in &cohorts {
            assert!(c.len() >= 3, "cohort too small: {}", c.len());
            assert!(c.len() <= 15, "cohort too large: {}", c.len());
        }
    }

    // ── generate_cycle ───────────────────────────────────────────────────────

    #[test]
    fn cycle_3_participants() {
        let ids = make_uuids(3);
        let weights = HashMap::new();
        let cycle = generate_cycle(&ids, &weights, 10);
        assert_eq!(cycle.participants.len(), 3);
        // All participants present.
        for id in &ids {
            assert!(cycle.participants.contains(id));
        }
    }

    #[test]
    fn cycle_15_participants() {
        let ids = make_uuids(15);
        let weights = HashMap::new();
        let cycle = generate_cycle(&ids, &weights, 10);
        assert_eq!(cycle.participants.len(), 15);
        for id in &ids {
            assert!(cycle.participants.contains(id));
        }
    }

    #[test]
    fn cycle_score_with_weights() {
        let ids = make_uuids(3);
        let mut weights = HashMap::new();
        weights.insert(weight_key(ids[0], ids[1]), 10);
        weights.insert(weight_key(ids[1], ids[2]), 5);
        weights.insert(weight_key(ids[0], ids[2]), 1);

        let cycle = generate_cycle(&ids, &weights, 100);
        // With 3 participants, all edges must be used. Total = 10 + 5 + 1 = 16.
        assert_eq!(cycle.score, 16);
    }

    // ── validate_cycles ──────────────────────────────────────────────────────

    #[test]
    fn validate_valid_single_cycle() {
        let ids = make_uuids(5);
        let result = AssignmentResult {
            cohorts: vec![Cycle {
                participants: ids,
                score: 0,
            }],
        };
        assert!(validate_cycles(&result).is_ok());
    }

    #[test]
    fn validate_valid_two_cohorts() {
        let ids = make_uuids(6);
        let result = AssignmentResult {
            cohorts: vec![
                Cycle {
                    participants: ids[0..3].to_vec(),
                    score: 0,
                },
                Cycle {
                    participants: ids[3..6].to_vec(),
                    score: 0,
                },
            ],
        };
        assert!(validate_cycles(&result).is_ok());
    }

    #[test]
    fn validate_duplicate_participant_across_cohorts() {
        let ids = make_uuids(5);
        let result = AssignmentResult {
            cohorts: vec![
                Cycle {
                    participants: ids[0..3].to_vec(),
                    score: 0,
                },
                Cycle {
                    // ids[2] is in both cohorts
                    participants: vec![ids[2], ids[3], ids[4]],
                    score: 0,
                },
            ],
        };
        assert!(validate_cycles(&result).is_err());
    }

    #[test]
    fn validate_too_small_cohort() {
        let ids = make_uuids(2);
        let result = AssignmentResult {
            cohorts: vec![Cycle {
                participants: ids,
                score: 0,
            }],
        };
        assert!(validate_cycles(&result).is_err());
    }

    // ── generate_assignments (end-to-end) ────────────────────────────────────

    #[test]
    fn full_generation_3_participants() {
        let ids = make_uuids(3);
        let input = AssignmentInput {
            participants: ids.clone(),
            social_weights: HashMap::new(),
        };
        let result = generate_assignments(&input);
        assert_eq!(result.cohorts.len(), 1);
        assert_eq!(result.cohorts[0].participants.len(), 3);
        assert!(validate_cycles(&result).is_ok());
    }

    #[test]
    fn full_generation_25_participants() {
        let ids = make_uuids(25);
        let input = AssignmentInput {
            participants: ids.clone(),
            social_weights: HashMap::new(),
        };
        let result = generate_assignments(&input);
        assert_eq!(result.cohorts.len(), 2);
        let total: usize = result.cohorts.iter().map(|c| c.participants.len()).sum();
        assert_eq!(total, 25);
        assert!(validate_cycles(&result).is_ok());
    }

    #[test]
    fn full_generation_30_participants() {
        let ids = make_uuids(30);
        let input = AssignmentInput {
            participants: ids,
            social_weights: HashMap::new(),
        };
        let result = generate_assignments(&input);
        assert!(validate_cycles(&result).is_ok());
        let total: usize = result.cohorts.iter().map(|c| c.participants.len()).sum();
        assert_eq!(total, 30);
    }

    // ── weight_key ───────────────────────────────────────────────────────────

    #[test]
    fn weight_key_canonical() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        assert_eq!(weight_key(a, b), weight_key(b, a));
    }

    // ── build_weight_matrix ──────────────────────────────────────────────────

    #[test]
    fn build_weights_from_groups() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let group_id = Uuid::new_v4();

        let memberships = vec![(a, group_id, 3), (b, group_id, 3), (c, group_id, 3)];

        let weights = build_weight_matrix(&memberships, &[]);
        // a-b, a-c, b-c should all have weight 3.
        assert_eq!(weights[&weight_key(a, b)], 3);
        assert_eq!(weights[&weight_key(a, c)], 3);
        assert_eq!(weights[&weight_key(b, c)], 3);
    }

    #[test]
    fn build_weights_from_past_pairings() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        let pairings = vec![(a, b), (b, a)];
        let weights = build_weight_matrix(&[], &pairings);
        assert_eq!(weights[&weight_key(a, b)], 2);
    }

    #[test]
    fn build_weights_combined() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let group_id = Uuid::new_v4();

        let memberships = vec![(a, group_id, 5), (b, group_id, 5)];
        let pairings = vec![(a, b)];
        let weights = build_weight_matrix(&memberships, &pairings);
        // 5 from group + 1 from pairing = 6
        assert_eq!(weights[&weight_key(a, b)], 6);
    }

    // ── Social weight influence on cycle ─────────────────────────────────────

    #[test]
    fn social_weights_influence_cycle_order() {
        // With 4+ participants, the algorithm should avoid placing high-weight
        // pairs adjacent in the cycle when alternatives exist.
        let ids = make_uuids(5);
        let mut weights = HashMap::new();
        // Make ids[0] and ids[1] very expensive to pair.
        weights.insert(weight_key(ids[0], ids[1]), 100);

        let cycle = generate_cycle(&ids, &weights, 100);

        // Check adjacency: ids[0] and ids[1] should ideally NOT be adjacent.
        let n = cycle.participants.len();
        let pos_0 = cycle
            .participants
            .iter()
            .position(|&x| x == ids[0])
            .unwrap();
        let pos_1 = cycle
            .participants
            .iter()
            .position(|&x| x == ids[1])
            .unwrap();
        let diff = pos_0.abs_diff(pos_1);
        let adjacent = diff == 1 || diff == n - 1;

        // With 5 participants, the algorithm has enough flexibility to avoid adjacency.
        assert!(
            !adjacent,
            "high-weight pair should be non-adjacent with enough alternatives"
        );
    }
}
