/// Pareto optimization utilities for multi-objective evolution
/// Implements NSGA-II style fast non-dominated sorting and crowding distance

use std::collections::HashMap;

/// Defines whether a metric should be maximized or minimized
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationDirection {
    Maximize,
    Minimize,
}

/// Configuration for a single objective in multi-objective optimization
#[derive(Debug, Clone)]
pub struct ObjectiveConfig {
    pub metric_name: String,
    pub direction: OptimizationDirection,
}

/// Individual with multiple objective values
#[derive(Debug, Clone)]
pub struct MultiObjectiveIndividual<T> {
    pub data: T,
    pub objectives: Vec<f64>,
    pub rank: usize,           // Pareto rank (0 = best frontier)
    pub crowding_distance: f64, // Diversity measure
}

impl<T> MultiObjectiveIndividual<T> {
    pub fn new(data: T, objectives: Vec<f64>) -> Self {
        Self {
            data,
            objectives,
            rank: 0,
            crowding_distance: 0.0,
        }
    }
}

/// Check if individual A dominates individual B
/// A dominates B if A is no worse than B in all objectives and strictly better in at least one
pub fn dominates(
    a_objectives: &[f64],
    b_objectives: &[f64],
    directions: &[OptimizationDirection],
) -> bool {
    if a_objectives.len() != b_objectives.len() || a_objectives.len() != directions.len() {
        return false;
    }

    let mut at_least_one_better = false;

    for i in 0..a_objectives.len() {
        let a_val = a_objectives[i];
        let b_val = b_objectives[i];

        let (a_better, b_better) = match directions[i] {
            OptimizationDirection::Maximize => (a_val > b_val, b_val > a_val),
            OptimizationDirection::Minimize => (a_val < b_val, b_val < a_val),
        };

        if b_better {
            // B is better in this objective, so A does not dominate B
            return false;
        }

        if a_better {
            at_least_one_better = true;
        }
    }

    at_least_one_better
}

/// Fast non-dominated sorting (NSGA-II algorithm)
/// Returns individuals grouped by Pareto front (0 = best, 1 = second best, etc.)
pub fn fast_non_dominated_sort<T: Clone>(
    individuals: &mut Vec<MultiObjectiveIndividual<T>>,
    directions: &[OptimizationDirection],
) -> Vec<Vec<usize>> {
    let n = individuals.len();

    // For each individual, track:
    // - domination_count: how many individuals dominate it
    // - dominated_solutions: indices of individuals it dominates
    let mut domination_count = vec![0; n];
    let mut dominated_solutions: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut fronts: Vec<Vec<usize>> = Vec::new();

    // First front (non-dominated individuals)
    let mut first_front = Vec::new();

    // Compare all pairs
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }

            if dominates(&individuals[i].objectives, &individuals[j].objectives, directions) {
                // i dominates j
                dominated_solutions[i].push(j);
            } else if dominates(&individuals[j].objectives, &individuals[i].objectives, directions) {
                // j dominates i
                domination_count[i] += 1;
            }
        }

        if domination_count[i] == 0 {
            // This individual is non-dominated
            individuals[i].rank = 0;
            first_front.push(i);
        }
    }

    fronts.push(first_front);

    // Generate subsequent fronts
    let mut front_index = 0;
    while front_index < fronts.len() && !fronts[front_index].is_empty() {
        let mut next_front = Vec::new();

        for &i in &fronts[front_index] {
            for &j in &dominated_solutions[i] {
                domination_count[j] -= 1;
                if domination_count[j] == 0 {
                    individuals[j].rank = front_index + 1;
                    next_front.push(j);
                }
            }
        }

        if !next_front.is_empty() {
            fronts.push(next_front);
        }
        front_index += 1;
    }

    fronts
}

/// Calculate crowding distance for individuals in a front
/// Crowding distance measures how close an individual is to its neighbors
/// Higher values indicate more diversity (isolated individuals)
pub fn calculate_crowding_distance<T>(
    individuals: &mut [MultiObjectiveIndividual<T>],
    front_indices: &[usize],
) {
    let front_size = front_indices.len();

    if front_size <= 2 {
        // Boundary solutions have infinite crowding distance
        for &idx in front_indices {
            individuals[idx].crowding_distance = f64::INFINITY;
        }
        return;
    }

    let num_objectives = individuals[front_indices[0]].objectives.len();

    // Initialize crowding distances
    for &idx in front_indices {
        individuals[idx].crowding_distance = 0.0;
    }

    // For each objective
    for obj in 0..num_objectives {
        // Sort front by this objective
        let mut sorted_indices: Vec<usize> = front_indices.to_vec();
        sorted_indices.sort_by(|&a, &b| {
            individuals[a].objectives[obj]
                .partial_cmp(&individuals[b].objectives[obj])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Boundary points have infinite distance
        individuals[sorted_indices[0]].crowding_distance = f64::INFINITY;
        individuals[sorted_indices[front_size - 1]].crowding_distance = f64::INFINITY;

        // Find the range for normalization
        let min_val = individuals[sorted_indices[0]].objectives[obj];
        let max_val = individuals[sorted_indices[front_size - 1]].objectives[obj];
        let range = max_val - min_val;

        if range.abs() < 1e-10 {
            // All values are the same for this objective
            continue;
        }

        // Calculate crowding distance for interior points
        for i in 1..(front_size - 1) {
            let idx = sorted_indices[i];
            let prev_val = individuals[sorted_indices[i - 1]].objectives[obj];
            let next_val = individuals[sorted_indices[i + 1]].objectives[obj];

            individuals[idx].crowding_distance += (next_val - prev_val) / range;
        }
    }
}

/// Extract objective values from metrics based on configuration
pub fn extract_objectives(
    metrics: &HashMap<String, f64>,
    objective_configs: &[ObjectiveConfig],
) -> Vec<f64> {
    objective_configs
        .iter()
        .map(|config| {
            metrics.get(&config.metric_name).copied().unwrap_or(0.0)
        })
        .collect()
}

/// Compare two individuals for selection (crowded comparison operator)
/// Returns true if individual A should be preferred over individual B
pub fn crowded_comparison<T>(
    a: &MultiObjectiveIndividual<T>,
    b: &MultiObjectiveIndividual<T>,
) -> bool {
    // Prefer lower rank (better Pareto front)
    if a.rank < b.rank {
        return true;
    }
    if a.rank > b.rank {
        return false;
    }

    // Same rank: prefer higher crowding distance (more diverse)
    a.crowding_distance > b.crowding_distance
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dominance_maximize() {
        let directions = vec![OptimizationDirection::Maximize, OptimizationDirection::Maximize];

        // A is better in both objectives
        assert!(dominates(&[10.0, 20.0], &[5.0, 10.0], &directions));

        // A is better in one, equal in other
        assert!(dominates(&[10.0, 20.0], &[10.0, 10.0], &directions));

        // A is better in one, worse in other - no dominance
        assert!(!dominates(&[10.0, 5.0], &[5.0, 10.0], &directions));

        // Equal in both - no dominance
        assert!(!dominates(&[10.0, 20.0], &[10.0, 20.0], &directions));
    }

    #[test]
    fn test_dominance_mixed() {
        let directions = vec![OptimizationDirection::Maximize, OptimizationDirection::Minimize];

        // A has higher first objective (good) and lower second objective (good)
        assert!(dominates(&[10.0, 5.0], &[5.0, 10.0], &directions));

        // A has higher first (good) but higher second (bad) - no dominance
        assert!(!dominates(&[10.0, 15.0], &[5.0, 10.0], &directions));
    }

    #[test]
    fn test_fast_non_dominated_sort() {
        let directions = vec![OptimizationDirection::Maximize, OptimizationDirection::Maximize];

        let mut individuals = vec![
            MultiObjectiveIndividual::new(0, vec![1.0, 5.0]),  // Front 0
            MultiObjectiveIndividual::new(1, vec![3.0, 3.0]),  // Front 0
            MultiObjectiveIndividual::new(2, vec![5.0, 1.0]),  // Front 0
            MultiObjectiveIndividual::new(3, vec![2.0, 2.0]),  // Front 1
            MultiObjectiveIndividual::new(4, vec![1.0, 1.0]),  // Front 2
        ];

        let fronts = fast_non_dominated_sort(&mut individuals, &directions);

        assert_eq!(fronts.len(), 3);
        assert_eq!(fronts[0].len(), 3); // First front has 3 individuals
        assert_eq!(individuals[0].rank, 0);
        assert_eq!(individuals[1].rank, 0);
        assert_eq!(individuals[2].rank, 0);
        assert_eq!(individuals[3].rank, 1);
        assert_eq!(individuals[4].rank, 2);
    }

    #[test]
    fn test_crowding_distance() {
        let directions = vec![OptimizationDirection::Maximize, OptimizationDirection::Maximize];

        let mut individuals = vec![
            MultiObjectiveIndividual::new(0, vec![1.0, 5.0]),
            MultiObjectiveIndividual::new(1, vec![3.0, 3.0]),
            MultiObjectiveIndividual::new(2, vec![5.0, 1.0]),
        ];

        let fronts = fast_non_dominated_sort(&mut individuals, &directions);
        calculate_crowding_distance(&mut individuals, &fronts[0]);

        // Boundary individuals should have infinite crowding distance
        assert!(individuals[fronts[0][0]].crowding_distance.is_infinite() ||
                individuals[fronts[0][2]].crowding_distance.is_infinite());
    }
}
