use crate::engines::evaluation::Backtester;
use crate::engines::generation::{
    hall_of_fame::{EliteStrategy, HallOfFame, get_canonical_ast_string},
    operators::*,
    semantic_mapper::SemanticMapper,
    genome::Genome,
    ast::StrategyAST,
};
use crate::error::TradebiasError;
use polars::prelude::*;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashMap;

pub struct EvolutionConfig {
    pub population_size: usize,
    pub generations: usize,
    pub genome_length: usize,
    pub gene_range: std::ops::Range<u32>,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elitism_rate: f64,
    pub tournament_size: usize,
    pub hall_of_fame_size: usize,
    pub fitness_objectives: Vec<String>, // Metric names
    pub fitness_weights: Vec<f64>,       // Weights for multi-objective
    pub min_fitness_threshold: f64,
    pub seed: Option<u64>,
}

pub struct EvolutionEngine {
    config: EvolutionConfig,
    backtester: Backtester,
    semantic_mapper: SemanticMapper,
    hall_of_fame: HallOfFame,
    rng: StdRng,
}

pub trait ProgressCallback: Send {
    fn on_generation_start(&mut self, generation: usize);
    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hall_of_fame_size: usize);
    fn on_strategy_evaluated(&mut self, strategy_num: usize, total: usize);
}

impl EvolutionEngine {
    pub fn new(
        config: EvolutionConfig,
        backtester: Backtester,
        semantic_mapper: SemanticMapper,
    ) -> Self {
        let rng = match config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let hall_of_fame = HallOfFame::new(config.hall_of_fame_size);

        Self {
            config,
            backtester,
            semantic_mapper,
            hall_of_fame,
            rng,
        }
    }

    /// Run the evolution process
    pub fn run<C: ProgressCallback>(
        &mut self,
        data: &DataFrame,
        mut callback: C,
    ) -> Result<Vec<EliteStrategy>, TradebiasError> {
        // Initialize population
        let mut population = self.initialize_population();

        // Evolution loop
        for generation in 0..self.config.generations {
            callback.on_generation_start(generation);

            // Evaluate fitness for all individuals
            let evaluated = self.evaluate_population(&population, data, &mut callback)?;

            // Update Hall of Fame
            for (genome, fitness, ast, metrics) in &evaluated {
                let canonical_string = get_canonical_ast_string(ast);
                let elite = EliteStrategy {
                    ast: ast.clone(),
                    genome: genome.clone(),
                    fitness: *fitness,
                    metrics: metrics.clone(),
                    canonical_string,
                };
                self.hall_of_fame.try_.add(elite);
            }

            // Get best fitness for progress tracking
            let best_fitness = evaluated
                .iter()
                .map(|(_, f, _, _)| *f)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);

            callback.on_generation_complete(generation, best_fitness, self.hall_of_fame.len());

            // Check termination
            if generation == self.config.generations - 1 {
                break;
            }

            // Create next generation
            population = self.create_next_generation(&evaluated);
        }

        Ok(self.hall_of_fame.get_all().to_vec())
    }

    fn initialize_population(&mut self) -> Vec<Genome> {
        (0..self.config.population_size)
            .map(|_| {
                random_genome(
                    self.config.genome_length,
                    self.config.gene_range.clone(),
                    &mut self.rng,
                )
            })
            .collect()
    }

    fn evaluate_population<C: ProgressCallback>(
        &mut self,
        population: &[Genome],
        data: &DataFrame,
        callback: &mut C,
    ) -> Result<Vec<(Genome, f64, StrategyAST, HashMap<String, f64>)>, TradebiasError> {
        let mut results = Vec::new();

        for (i, genome) in population.iter().enumerate() {
            callback.on_strategy_evaluated(i + 1, population.len());

            // Generate AST from genome
            let ast = self.semantic_mapper.create_strategy_ast(genome)?;

            // Run backtest
            let backtest_result = self.backtester.run(ast.as_node(), data)?;

            // Calculate fitness
            let fitness = self.calculate_fitness(&backtest_result.metrics);

            results.push((genome.clone(), fitness, ast, backtest_result.metrics));
        }

        Ok(results)
    }

    fn calculate_fitness(&self, metrics: &HashMap<String, f64>) -> f64 {
        let mut fitness = 0.0;

        for (objective, weight) in self.config.fitness_objectives.iter().zip(&self.config.fitness_weights) {
            if let Some(&value) = metrics.get(objective) {
                fitness += weight * value;
            }
        }

        fitness
    }

    fn create_next_generation(
        &mut self,
        evaluated: &[(Genome, f64, StrategyAST, HashMap<String, f64>)],
    ) -> Vec<Genome> {
        let mut next_generation = Vec::new();
        let population_fitness: Vec<(Genome, f64)> = evaluated
            .iter()
            .map(|(g, f, _, _)| (g.clone(), *f))
            .collect();

        // Elitism: copy top performers
        let elite_count = (self.config.population_size as f64 * self.config.elitism_rate) as usize;
        let mut sorted = population_fitness.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (genome, _) in sorted.iter().take(elite_count) {
            next_generation.push(genome.clone());
        }

        // Generate offspring
        while next_generation.len() < self.config.population_size {
            if self.rng.gen::<f64>() < self.config.crossover_rate {
                // Crossover
                let parent1 = tournament_selection(
                    &population_fitness,
                    self.config.tournament_size,
                    &mut self.rng,
                );
                let parent2 = tournament_selection(
                    &population_fitness,
                    self.config.tournament_size,
                    &mut self.rng,
                );

                let (mut child1, mut child2) = crossover(&parent1, &parent2, &mut self.rng);

                // Apply mutation
                mutate(&mut child1, self.config.mutation_rate, self.config.gene_range.clone(), &mut self.rng);
                mutate(&mut child2, self.config.mutation_rate, self.config.gene_range.clone(), &mut self.rng);

                next_generation.push(child1);
                if next_generation.len() < self.config.population_size {
                    next_generation.push(child2);
                }
            } else {
                // Reproduction (copy)
                let parent = tournament_selection(
                    &population_fitness,
                    self.config.tournament_size,
                    &mut self.rng,
                );
                let mut child = parent;
                mutate(&mut child, self.config.mutation_rate, self.config.gene_range.clone(), &mut self.rng);
                next_generation.push(child);
            }
        }

        next_generation.truncate(self.config.population_size);
        next_generation
    }

    pub fn get_hall_of_fame(&self) -> &HallOfFame {
        &self.hall_of_fame
    }
}
