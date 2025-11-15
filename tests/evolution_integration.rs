use tradebias::config::backtesting::{BacktestingConfig, ValidationMethod};
use tradebias::config::evolution::EvolutionConfig;
use tradebias::data::IndicatorCache;
use tradebias::engines::evaluation::Backtester;
use tradebias::engines::generation::evolution_engine::{
    EvolutionConfig as EngineEvolutionConfig, EvolutionEngine, ProgressCallback,
};
use tradebias::engines::generation::semantic_mapper::SemanticMapper;
use tradebias::functions::registry::FunctionRegistry;
use polars::prelude::*;
use std::sync::Arc;

/// Simple progress callback for testing
struct TestProgressCallback {
    last_generation: usize,
}

impl ProgressCallback for TestProgressCallback {
    fn on_generation_start(&mut self, _generation: usize) {}

    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hall_size: usize) {
        self.last_generation = generation;
        println!(
            "Generation {}: Best Fitness = {:.4}, Hall Size = {}",
            generation + 1,
            best_fitness,
            hall_size
        );
    }

    fn on_strategy_evaluated(&mut self, _strategy_num: usize, _total: usize) {}
}

/// Load test data from CSV
fn load_test_data() -> Result<DataFrame, Box<dyn std::error::Error>> {
    let df = CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some("tests/data/BTC_1day_sample.csv".into()))?
        .finish()?;
    Ok(df)
}

/// Create a minimal evolution config for fast testing
fn create_test_evolution_config() -> EvolutionConfig {
    EvolutionConfig {
        population_size: 20,
        num_generations: 5,
        mutation_rate: 0.15,
        crossover_rate: 0.85,
        selection_method: tradebias::config::evolution::SelectionMethod::Tournament,
        elitism_count: 2,
        max_tree_depth: 5,
        tournament_size: 3,
    }
}

/// Create backtesting config
fn create_test_backtesting_config() -> BacktestingConfig {
    BacktestingConfig {
        validation_method: ValidationMethod::Simple,
        train_test_split: 0.7,
        num_folds: 5,
        initial_capital: 10000.0,
        commission: 0.001,
        slippage: 0.001,
    }
}

#[test]
fn test_evolution_basic() {
    println!("\n=== Testing Evolution with Basic Config ===");

    let data = match load_test_data() {
        Ok(df) => df,
        Err(e) => {
            println!("⚠️  Skipping test - could not load test data: {}", e);
            return;
        }
    };

    println!("✓ Loaded {} rows of data", data.height());

    let evolution_config = create_test_evolution_config();
    let backtesting_config = create_test_backtesting_config();

    // Create components
    let registry = Arc::new(FunctionRegistry::new());
    let cache = Arc::new(IndicatorCache::new(1000));

    let backtester = Backtester::new(
        Arc::clone(&registry),
        Arc::clone(&cache),
        backtesting_config.initial_capital,
    );

    let semantic_mapper = SemanticMapper::new(
        Arc::clone(&registry),
        evolution_config.max_tree_depth,
    );

    // Create engine config
    let engine_config = EngineEvolutionConfig {
        population_size: evolution_config.population_size,
        generations: evolution_config.num_generations,
        genome_length: 100,
        gene_range: 0..1000,
        mutation_rate: evolution_config.mutation_rate,
        crossover_rate: evolution_config.crossover_rate,
        elitism_rate: evolution_config.elitism_count as f64 / evolution_config.population_size as f64,
        tournament_size: evolution_config.tournament_size,
        hall_of_fame_size: 5,
        fitness_objectives: vec!["return_pct".to_string()],
        fitness_weights: vec![1.0],
        min_fitness_threshold: 0.0,
        seed: Some(42), // Fixed seed for reproducibility
    };

    let mut engine = EvolutionEngine::new(engine_config, backtester, semantic_mapper);

    let callback = TestProgressCallback {
        last_generation: 0,
    };

    println!("\n🚀 Starting evolution...");
    let result = engine.run(&data, callback);

    match result {
        Ok(strategies) => {
            println!("\n✓ Evolution completed successfully!");
            println!("✓ Found {} elite strategies", strategies.len());

            for (i, strategy) in strategies.iter().enumerate() {
                println!(
                    "\n  Strategy #{}: Fitness = {:.4}",
                    i + 1,
                    strategy.fitness
                );
                println!("    Formula: {}", strategy.ast.root.to_formula_short(80));

                if let Some(return_pct) = strategy.metrics.get("return_pct") {
                    println!("    Return: {:.2}%", return_pct);
                }
            }

            assert!(
                !strategies.is_empty(),
                "Should have found at least one strategy"
            );
        }
        Err(e) => {
            panic!("Evolution failed: {}", e);
        }
    }
}

#[test]
fn test_evolution_with_different_depths() {
    println!("\n=== Testing Evolution with Different Max Depths ===");

    let data = match load_test_data() {
        Ok(df) => df,
        Err(e) => {
            println!("⚠️  Skipping test - could not load test data: {}", e);
            return;
        }
    };

    for max_depth in [3, 5, 7] {
        println!("\n--- Testing with max_depth = {} ---", max_depth);

        let mut evolution_config = create_test_evolution_config();
        evolution_config.max_tree_depth = max_depth;
        evolution_config.num_generations = 3; // Fewer generations for speed

        let backtesting_config = create_test_backtesting_config();

        let registry = Arc::new(FunctionRegistry::new());
        let cache = Arc::new(IndicatorCache::new(1000));

        let backtester = Backtester::new(
            Arc::clone(&registry),
            Arc::clone(&cache),
            backtesting_config.initial_capital,
        );

        let semantic_mapper = SemanticMapper::new(
            Arc::clone(&registry),
            evolution_config.max_tree_depth,
        );

        let engine_config = EngineEvolutionConfig {
            population_size: 10,
            generations: 3,
            genome_length: 100,
            gene_range: 0..1000,
            mutation_rate: 0.15,
            crossover_rate: 0.85,
            elitism_rate: 0.2,
            tournament_size: 3,
            hall_of_fame_size: 3,
            fitness_objectives: vec!["return_pct".to_string()],
            fitness_weights: vec![1.0],
            min_fitness_threshold: 0.0,
            seed: Some(42 + max_depth as u64), // Different seed for each depth
        };

        let mut engine = EvolutionEngine::new(engine_config, backtester, semantic_mapper);

        let callback = TestProgressCallback {
            last_generation: 0,
        };

        match engine.run(&data, callback) {
            Ok(strategies) => {
                println!("  ✓ Max depth {} succeeded, found {} strategies", max_depth, strategies.len());
            }
            Err(e) => {
                panic!("Evolution failed with max_depth {}: {}", max_depth, e);
            }
        }
    }

    println!("\n✓ All depth tests passed!");
}

#[test]
fn test_evolution_with_different_population_sizes() {
    println!("\n=== Testing Evolution with Different Population Sizes ===");

    let data = match load_test_data() {
        Ok(df) => df,
        Err(e) => {
            println!("⚠️  Skipping test - could not load test data: {}", e);
            return;
        }
    };

    for pop_size in [10, 20, 50] {
        println!("\n--- Testing with population_size = {} ---", pop_size);

        let mut evolution_config = create_test_evolution_config();
        evolution_config.population_size = pop_size;
        evolution_config.num_generations = 3;

        let backtesting_config = create_test_backtesting_config();

        let registry = Arc::new(FunctionRegistry::new());
        let cache = Arc::new(IndicatorCache::new(1000));

        let backtester = Backtester::new(
            Arc::clone(&registry),
            Arc::clone(&cache),
            backtesting_config.initial_capital,
        );

        let semantic_mapper = SemanticMapper::new(
            Arc::clone(&registry),
            evolution_config.max_tree_depth,
        );

        let engine_config = EngineEvolutionConfig {
            population_size: pop_size,
            generations: 3,
            genome_length: 100,
            gene_range: 0..1000,
            mutation_rate: 0.15,
            crossover_rate: 0.85,
            elitism_rate: 2.0 / pop_size as f64,
            tournament_size: 3,
            hall_of_fame_size: 3,
            fitness_objectives: vec!["return_pct".to_string()],
            fitness_weights: vec![1.0],
            min_fitness_threshold: 0.0,
            seed: Some(42 + pop_size as u64),
        };

        let mut engine = EvolutionEngine::new(engine_config, backtester, semantic_mapper);

        let callback = TestProgressCallback {
            last_generation: 0,
        };

        match engine.run(&data, callback) {
            Ok(strategies) => {
                println!("  ✓ Population size {} succeeded, found {} strategies", pop_size, strategies.len());
            }
            Err(e) => {
                panic!("Evolution failed with population_size {}: {}", pop_size, e);
            }
        }
    }

    println!("\n✓ All population size tests passed!");
}
