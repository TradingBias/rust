use crate::engines::generation::ast::StrategyAST;

use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct EliteStrategy {
    pub ast: StrategyAST,
    pub genome: Vec<u32>,
    pub fitness: f64,
    pub metrics: HashMap<String, f64>,
    pub canonical_string: String, // For deduplication
}

pub struct HallOfFame {
    strategies: Vec<EliteStrategy>,
    max_size: usize,
    seen_signatures: HashSet<String>,
}

impl HallOfFame {
    pub fn new(max_size: usize) -> Self {
        Self {
            strategies: Vec::new(),
            max_size,
            seen_signatures: HashSet::new(),
        }
    }

    /// Attempt to add a strategy to the Hall of Fame
    pub fn try_add(&mut self, strategy: EliteStrategy) -> bool {
        // Deduplication check
        if self.seen_signatures.contains(&strategy.canonical_string) {
            return false; // Duplicate, reject
        }

        // Add to collection
        self.strategies.push(strategy.clone());
        self.seen_signatures.insert(strategy.canonical_string.clone());

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

        true
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
