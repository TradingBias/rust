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
use std::env;
use std::sync::Arc;

/// CLI progress callback with formatted output
struct CliProgressCallback {
    start_time: std::time::Instant,
}

impl ProgressCallback for CliProgressCallback {
    fn on_generation_start(&mut self, generation: usize) {
        print!("\r🔄 Generation {} starting...", generation + 1);
        std::io::Write::flush(&mut std::io::stdout()).unwrap();
    }

    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hall_size: usize) {
        let elapsed = self.start_time.elapsed();
        println!(
            "\r✓ Generation {}: Best = {:.4}, Hall = {}, Time = {:.2}s",
            generation + 1,
            best_fitness,
            hall_size,
            elapsed.as_secs_f64()
        );
    }

    fn on_strategy_evaluated(&mut self, _strategy_num: usize, _total: usize) {}
}

fn main() {
    println!("=== TradeBias Evolution Test Utility ===\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    let data_path = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("tests/data/BTC_1day_sample.csv");
    let population_size = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(50);
    let num_generations = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(10);
    let max_tree_depth = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(7);

    println!("Configuration:");
    println!("  Data file: {}", data_path);
    println!("  Population size: {}", population_size);
    println!("  Generations: {}", num_generations);
    println!("  Max tree depth: {}", max_tree_depth);
    println!();

    // Load data
    println!("📊 Loading data...");
    let data = match CsvReadOptions::default()
        .with_has_header(true)
        .try_into_reader_with_file_path(Some(data_path.into()))
    {
        Ok(reader) => match reader.finish() {
            Ok(df) => df,
            Err(e) => {
                eprintln!("❌ Error reading CSV: {}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("❌ Error opening file: {}", e);
            std::process::exit(1);
        }
    };

    println!("✓ Loaded {} rows of data", data.height());
    println!("✓ Columns: {:?}\n", data.get_column_names());

    // Create configs
    let evolution_config = EvolutionConfig {
        population_size,
        num_generations,
        mutation_rate: 0.15,
        crossover_rate: 0.85,
        selection_method: tradebias::config::evolution::SelectionMethod::Tournament,
        elitism_count: (population_size as f64 * 0.1) as usize,
        max_tree_depth,
        tournament_size: 7,
    };

    let backtesting_config = BacktestingConfig {
        validation_method: ValidationMethod::Simple,
        train_test_split: 0.7,
        num_folds: 5,
        initial_capital: 10000.0,
        commission: 0.001,
        slippage: 0.001,
    };

    // Create components
    println!("🔧 Initializing evolution engine...");
    let registry = Arc::new(FunctionRegistry::new());
    let cache = Arc::new(IndicatorCache::new(1000));

    println!("  ✓ Function registry loaded");
    println!("  ✓ Indicator cache initialized");

    let backtester = Backtester::new(
        Arc::clone(&registry),
        Arc::clone(&cache),
        backtesting_config.initial_capital,
    );

    let semantic_mapper = SemanticMapper::new(Arc::clone(&registry), max_tree_depth);

    // Create engine config
    let engine_config = EngineEvolutionConfig {
        population_size,
        generations: num_generations,
        genome_length: 100,
        gene_range: 0..1000,
        mutation_rate: evolution_config.mutation_rate,
        crossover_rate: evolution_config.crossover_rate,
        elitism_rate: evolution_config.elitism_count as f64 / population_size as f64,
        tournament_size: evolution_config.tournament_size,
        hall_of_fame_size: 10,
        fitness_objectives: vec!["return_pct".to_string()],
        fitness_weights: vec![1.0],
        min_fitness_threshold: 0.0,
        seed: None,
    };

    let mut engine = EvolutionEngine::new(engine_config, backtester, semantic_mapper);

    println!("✓ Evolution engine ready\n");

    // Run evolution
    println!("🚀 Starting evolution...\n");

    let callback = CliProgressCallback {
        start_time: std::time::Instant::now(),
    };

    match engine.run(&data, callback) {
        Ok(strategies) => {
            println!("\n");
            println!("═══════════════════════════════════════════════");
            println!("✓ Evolution completed successfully!");
            println!("═══════════════════════════════════════════════\n");

            println!("📈 Hall of Fame ({} strategies):\n", strategies.len());

            for (i, strategy) in strategies.iter().enumerate() {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("Rank #{}", i + 1);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("  Fitness:     {:.6}", strategy.fitness);

                if let Some(return_pct) = strategy.metrics.get("return_pct") {
                    println!("  Return:      {:.2}%", return_pct);
                }

                if let Some(num_trades) = strategy.metrics.get("num_trades") {
                    println!("  Trades:      {}", num_trades);
                }

                if let Some(sharpe) = strategy.metrics.get("sharpe_ratio") {
                    println!("  Sharpe:      {:.4}", sharpe);
                }

                if let Some(drawdown) = strategy.metrics.get("max_drawdown") {
                    println!("  Max DD:      {:.2}%", drawdown);
                }

                println!("\n  Formula (short):");
                println!("    {}", strategy.ast.root.to_formula_short(70));

                println!("\n  Formula (full):");
                let full_formula = strategy.ast.root.to_formula();
                for line in full_formula.lines() {
                    println!("    {}", line);
                }

                println!();
            }

            println!("═══════════════════════════════════════════════\n");
        }
        Err(e) => {
            eprintln!("\n❌ Evolution failed: {}", e);
            std::process::exit(1);
        }
    }
}
