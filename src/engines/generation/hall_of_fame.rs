use crate::engines::generation::ast::StrategyAST;
use crate::engines::generation::pareto::{ObjectiveConfig, MultiObjectiveIndividual};
use crate::engines::generation::pareto;

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct EliteStrategy {
    pub ast: StrategyAST,
    pub genome: Vec<u32>,
    pub fitness: f64,                  // Legacy single-objective fitness
    pub metrics: HashMap<String, f64>,
    pub canonical_string: String,       // For deduplication
    pub pareto_rank: usize,            // Pareto frontier rank (0 = best)
    pub crowding_distance: f64,        // Diversity measure
}

pub struct HallOfFame {
    strategies: Vec<EliteStrategy>,
    max_size: usize,
    seen_signatures: HashSet<String>,
    objective_configs: Vec<ObjectiveConfig>, // Multi-objective optimization config
    use_pareto: bool,                        // Whether to use Pareto optimization
}

impl HallOfFame {
    pub fn new(max_size: usize) -> Self {
        Self {
            strategies: Vec::new(),
            max_size,
            seen_signatures: HashSet::new(),
            objective_configs: Vec::new(),
            use_pareto: false,
        }
    }

    /// Create a new HallOfFame with Pareto optimization
    pub fn new_with_pareto(max_size: usize, objective_configs: Vec<ObjectiveConfig>) -> Self {
        Self {
            strategies: Vec::new(),
            max_size,
            seen_signatures: HashSet::new(),
            objective_configs,
            use_pareto: true,
        }
    }

    /// Attempt to add a strategy to the Hall of Fame
    pub fn try_add(&mut self, mut strategy: EliteStrategy) -> bool {
        // Deduplication check
        if self.seen_signatures.contains(&strategy.canonical_string) {
            return false; // Duplicate, reject
        }

        // If not using Pareto, initialize with default values
        if !self.use_pareto {
            strategy.pareto_rank = 0;
            strategy.crowding_distance = 0.0;
        }

        // Add to collection
        self.strategies.push(strategy.clone());
        self.seen_signatures.insert(strategy.canonical_string.clone());

        // Sort and trim based on optimization mode
        if self.use_pareto {
            self.sort_and_trim_pareto();
        } else {
            self.sort_and_trim_single();
        }

        true
    }

    /// Sort and trim using single-objective fitness
    fn sort_and_trim_single(&mut self) {
        // Sort by fitness (descending)
        self.strategies.sort_by(|a, b| {
            b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Trim to max size
        if self.strategies.len() > self.max_size {
            if let Some(removed) = self.strategies.pop() {
                self.seen_signatures.remove(&removed.canonical_string);
            }
        }
    }

    /// Sort and trim using Pareto dominance
    fn sort_and_trim_pareto(&mut self) {
        // Convert to MultiObjectiveIndividual
        let mut individuals: Vec<MultiObjectiveIndividual<usize>> = self.strategies
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let objectives = pareto::extract_objectives(&s.metrics, &self.objective_configs);
                MultiObjectiveIndividual::new(i, objectives)
            })
            .collect();

        // Extract optimization directions
        let directions: Vec<_> = self.objective_configs
            .iter()
            .map(|c| c.direction)
            .collect();

        // Perform fast non-dominated sorting
        let fronts = pareto::fast_non_dominated_sort(&mut individuals, &directions);

        // Calculate crowding distance for each front
        for front in &fronts {
            pareto::calculate_crowding_distance(&mut individuals, front);
        }

        // Update pareto_rank and crowding_distance in strategies
        for individual in &individuals {
            self.strategies[individual.data].pareto_rank = individual.rank;
            self.strategies[individual.data].crowding_distance = individual.crowding_distance;
        }

        // Sort by rank, then by crowding distance (descending)
        self.strategies.sort_by(|a, b| {
            match a.pareto_rank.cmp(&b.pareto_rank) {
                std::cmp::Ordering::Equal => {
                    // Higher crowding distance is better (more diverse)
                    b.crowding_distance.partial_cmp(&a.crowding_distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                other => other,
            }
        });

        // Trim to max size
        while self.strategies.len() > self.max_size {
            if let Some(removed) = self.strategies.pop() {
                self.seen_signatures.remove(&removed.canonical_string);
            }
        }
    }

    /// Get all elite strategies
    pub fn get_all(&self) -> &[EliteStrategy] {
        &self.strategies
    }

    /// Get top N strategies
    pub fn get_top_n(&self, n: usize) -> &[EliteStrategy] {
        &self.strategies[..n.min(self.strategies.len())]
    }

    /// Filter by minimum fitness threshold
    pub fn filter_by_threshold(&self, min_fitness: f64) -> Vec<EliteStrategy> {
        self.strategies
            .iter()
            .filter(|s| s.fitness >= min_fitness)
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.strategies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strategies.is_empty()
    }
}

/// Generate canonical string for deduplication
pub fn get_canonical_ast_string(ast: &StrategyAST) -> String {
    serde_json::to_string(ast).unwrap_or_else(|_| String::new())
}
