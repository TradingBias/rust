use crate::config::backtesting::BacktestingConfig;
use crate::config::evolution::EvolutionConfig;
use crate::config::trade_management::TradeManagementConfig;
use crate::engines::generation::evolution_engine::{
    EvolutionEngine,
    EvolutionConfig as EngineEvolutionConfig,
    ProgressCallback,
};
use crate::engines::generation::hall_of_fame::EliteStrategy;
use crate::engines::generation::semantic_mapper::SemanticMapper;
use crate::engines::generation::pareto::ObjectiveConfig;
use crate::engines::evaluation::Backtester;
use crate::data::IndicatorCache;
use crate::functions::registry::FunctionRegistry;
use crate::ui::state::StrategyDisplay;
use polars::prelude::*;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

/// Progress update from evolution thread
#[derive(Clone, Debug)]
pub struct ProgressUpdate {
    pub generation: usize,
    pub total_generations: usize,
    pub best_fitness: f64,
    pub hall_size: usize,
    pub status: String,
}

/// Result from evolution run
pub type EvolutionResult = Result<Vec<StrategyDisplay>, String>;

/// Progress callback that sends updates through channel
struct EvolutionProgressCallback {
    progress_tx: Sender<ProgressUpdate>,
    cancel_flag: Arc<Mutex<bool>>,
    total_generations: usize,
}

impl ProgressCallback for EvolutionProgressCallback {
    fn on_generation_start(&mut self, generation: usize) {
        // Send update when generation starts
        println!("🔄 Generation {} starting...", generation + 1);
        let _ = self.progress_tx.send(ProgressUpdate {
            generation,
            total_generations: self.total_generations,
            best_fitness: 0.0,
            hall_size: 0,
            status: format!("Generation {}/{} starting...", generation + 1, self.total_generations),
        });

        // Check for cancellation
        if self.cancel_flag.lock().map(|f| *f).unwrap_or(false) {
            // Cancellation will be handled by the engine check
        }
    }

    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hall_of_fame_size: usize) {
        println!("✓ Generation {} complete - Best: {:.2}", generation + 1, best_fitness);
        let _ = self.progress_tx.send(ProgressUpdate {
            generation: generation + 1,
            total_generations: self.total_generations,
            best_fitness,
            hall_size: hall_of_fame_size,
            status: format!("Generation {}/{} - Best: {:.2}", generation + 1, self.total_generations, best_fitness),
        });
    }

    fn on_strategy_evaluated(&mut self, strategy_num: usize, total: usize) {
        // Send granular progress updates every 10 strategies
        if strategy_num % 10 == 0 || strategy_num == total {
            let _ = self.progress_tx.send(ProgressUpdate {
                generation: 0, // Will be overridden by actual generation
                total_generations: self.total_generations,
                best_fitness: 0.0,
                hall_size: 0,
                status: format!("Evaluating strategies: {}/{}", strategy_num, total),
            });
        }
    }
}

pub struct EvolutionRunner {
    handle: Option<JoinHandle<EvolutionResult>>,
    progress_rx: Option<Receiver<ProgressUpdate>>,
    cancel_flag: Arc<Mutex<bool>>,
}

impl EvolutionRunner {
    /// Start evolution in background thread
    pub fn start(
        data: DataFrame,
        evolution_config: EvolutionConfig,
        backtesting_config: BacktestingConfig,
        trade_management_config: TradeManagementConfig,
        selected_indicators: Vec<String>,
        objective_configs: Vec<ObjectiveConfig>,
    ) -> Self {
        let (progress_tx, progress_rx) = channel();
        let cancel_flag = Arc::new(Mutex::new(false));
        let cancel_flag_clone = Arc::clone(&cancel_flag);

        // Spawn thread with increased stack size to handle deep AST recursion
        // Default is ~2MB, we use 16MB to safely handle recursive formula generation
        // and expression building with max_tree_depth up to 12
        let handle = thread::Builder::new()
            .stack_size(16 * 1024 * 1024) // 16MB stack
            .spawn(move || {
                Self::run_evolution(
                    data,
                    evolution_config,
                    backtesting_config,
                    trade_management_config,
                    selected_indicators,
                    objective_configs,
                    progress_tx,
                    cancel_flag_clone,
                )
            })
            .expect("Failed to spawn evolution thread");

        Self {
            handle: Some(handle),
            progress_rx: Some(progress_rx),
            cancel_flag,
        }
    }

    /// Poll for progress updates (non-blocking)
    pub fn poll_progress(&mut self) -> Option<ProgressUpdate> {
        if let Some(rx) = &self.progress_rx {
            rx.try_recv().ok()
        } else {
            None
        }
    }

    /// Check if evolution is complete and get results
    pub fn try_get_results(&mut self) -> Option<EvolutionResult> {
        if let Some(handle) = self.handle.take() {
            if handle.is_finished() {
                match handle.join() {
                    Ok(result) => Some(result),
                    Err(_) => Some(Err("Evolution thread panicked".to_string())),
                }
            } else {
                // Not finished yet, put handle back
                self.handle = Some(handle);
                None
            }
        } else {
            None
        }
    }

    /// Cancel the running evolution
    pub fn cancel(&mut self) {
        if let Ok(mut flag) = self.cancel_flag.lock() {
            *flag = true;
        }
    }

    /// Check if cancellation was requested
    fn is_cancelled(cancel_flag: &Arc<Mutex<bool>>) -> bool {
        cancel_flag.lock().map(|f| *f).unwrap_or(false)
    }

    /// Run the evolution (called in background thread)
    fn run_evolution(
        data: DataFrame,
        evolution_config: EvolutionConfig,
        backtesting_config: BacktestingConfig,
        _trade_management_config: TradeManagementConfig,
        _selected_indicators: Vec<String>,
        objective_configs: Vec<ObjectiveConfig>,
        progress_tx: Sender<ProgressUpdate>,
        cancel_flag: Arc<Mutex<bool>>,
    ) -> EvolutionResult {
        println!("🚀 Evolution thread started!");
        println!("  Population: {}", evolution_config.population_size);
        println!("  Generations: {}", evolution_config.num_generations);
        println!("  Data rows: {}", data.height());

        // Create components needed for evolution
        let registry = Arc::new(FunctionRegistry::new());
        let cache = Arc::new(IndicatorCache::new(1000));

        // Create backtester
        let backtester = Backtester::new(
            Arc::clone(&registry),
            Arc::clone(&cache),
            backtesting_config.initial_capital,
        );

        // Create semantic mapper
        let semantic_mapper = SemanticMapper::new(
            Arc::clone(&registry),
            evolution_config.max_tree_depth,
        );

        // Convert UI config to engine config
        let engine_config = EngineEvolutionConfig {
            population_size: evolution_config.population_size,
            generations: evolution_config.num_generations,
            genome_length: 100, // Default genome length
            gene_range: 0..1000, // Default gene range
            mutation_rate: evolution_config.mutation_rate,
            crossover_rate: evolution_config.crossover_rate,
            elitism_rate: evolution_config.elitism_count as f64 / evolution_config.population_size as f64,
            tournament_size: evolution_config.tournament_size,
            hall_of_fame_size: 10, // Keep top 10 strategies

            // Pareto multi-objective optimization (enabled by default)
            objective_configs: objective_configs.clone(),
            use_pareto: true,

            // Legacy single-objective fields (for backward compatibility)
            fitness_objectives: vec!["return_pct".to_string()],
            fitness_weights: vec![1.0],

            min_fitness_threshold: 0.0,
            seed: None, // Random seed
        };

        // Create evolution engine
        let mut engine = EvolutionEngine::new(engine_config, backtester, semantic_mapper);

        // Create progress callback
        let total_generations = evolution_config.num_generations;
        let callback = EvolutionProgressCallback {
            progress_tx: progress_tx.clone(),
            cancel_flag: cancel_flag.clone(),
            total_generations,
        };

        // Run evolution
        match engine.run(&data, callback) {
            Ok(elite_strategies) => {
                // Convert EliteStrategy to StrategyDisplay
                let displays: Vec<StrategyDisplay> = elite_strategies
                    .into_iter()
                    .enumerate()
                    .map(|(i, elite)| elite_to_display(elite, i + 1))
                    .collect();

                let _ = progress_tx.send(ProgressUpdate {
                    generation: total_generations,
                    total_generations,
                    best_fitness: displays.first().map(|d| d.fitness).unwrap_or(0.0),
                    hall_size: displays.len(),
                    status: format!("Complete! Found {} strategies", displays.len()),
                });

                Ok(displays)
            }
            Err(e) => {
                let _ = progress_tx.send(ProgressUpdate {
                    generation: 0,
                    total_generations,
                    best_fitness: 0.0,
                    hall_size: 0,
                    status: format!("Error: {}", e),
                });
                Err(format!("Evolution failed: {}", e))
            }
        }
    }
}

impl Drop for EvolutionRunner {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Helper function to convert EliteStrategy to StrategyDisplay
pub fn elite_to_display(elite: EliteStrategy, rank: usize) -> StrategyDisplay {
    // Extract metrics from the elite strategy
    let return_pct = elite.metrics.get("return_pct").copied().unwrap_or(0.0);
    let max_drawdown = elite.metrics.get("max_drawdown").copied().unwrap_or(0.0);
    let sharpe_ratio = elite.metrics.get("sharpe_ratio").copied().unwrap_or(0.0);
    let total_trades = elite.metrics.get("total_trades").copied().unwrap_or(0.0) as usize;
    let win_rate = elite.metrics.get("win_rate").copied().unwrap_or(0.0);

    StrategyDisplay {
        rank,
        fitness: elite.fitness,
        return_pct,
        total_trades,
        win_rate,
        max_drawdown,
        sharpe_ratio,
        formula: elite.ast.root.to_formula_short(60),
        formula_full: elite.ast.root.to_formula(),
        equity_curve: Vec::new(), // TODO: Get from backtesting results
        trades: Vec::new(),        // TODO: Get from backtesting results
    }
}
