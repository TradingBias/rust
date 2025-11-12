use super::traits::{ConfigSection, ConfigManifest, FieldManifest};
use crate::error::TradebiasError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionConfig {
    pub population_size: usize,
    pub num_generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub selection_method: SelectionMethod,
    pub elitism_count: usize,
    pub max_tree_depth: usize,
    pub tournament_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectionMethod {
    Tournament,
    Roulette,
    Rank,
}

impl Default for EvolutionConfig {
    fn default() -> Self {
        Self {
            population_size: 500,
            num_generations: 100,
            mutation_rate: 0.15,
            crossover_rate: 0.85,
            selection_method: SelectionMethod::Tournament,
            elitism_count: 10,
            max_tree_depth: 12,
            tournament_size: 7,
        }
    }
}

impl ConfigSection for EvolutionConfig {
    fn section_name() -> &'static str {
        "evolution"
    }

    fn validate(&self) -> Result<(), TradebiasError> {
        if self.population_size < 10 {
            return Err(TradebiasError::Configuration(
                "Population size must be at least 10".to_string()
            ));
        }
        if self.mutation_rate < 0.0 || self.mutation_rate > 1.0 {
            return Err(TradebiasError::Configuration(
                "Mutation rate must be between 0 and 1".to_string()
            ));
        }
        if self.crossover_rate < 0.0 || self.crossover_rate > 1.0 {
            return Err(TradebiasError::Configuration(
                "Crossover rate must be between 0 and 1".to_string()
            ));
        }
        Ok(())
    }

    fn to_manifest(&self) -> ConfigManifest {
        ConfigManifest {
            section: "Evolution".to_string(),
            fields: vec![
                FieldManifest {
                    name: "population_size".to_string(),
                    field_type: "integer".to_string(),
                    default: serde_json::json!(500),
                    min: Some(10.0),
                    max: Some(10000.0),
                    description: "Number of strategies in population".to_string(),
                },
                // ... add all other fields
            ],
        }
    }
}
