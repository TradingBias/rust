use crate::config::backtesting::ValidationMethod;
use crate::config::trade_management::{StopLossConfig, TakeProfitConfig, PositionSizing};
use crate::data::DataPreview;
use crate::engines::generation::pareto::OptimizationDirection;
use polars::prelude::*;
use std::collections::{HashSet, HashMap};
use std::path::PathBuf;

/// Central application state for the UI
pub struct AppState {
    // Data Configuration
    pub data_file_path: Option<PathBuf>,
    pub loaded_data: Option<DataFrame>,
    pub data_preview: Option<DataPreview>,

    // Indicator Selection
    pub available_indicators: Vec<IndicatorInfo>,
    pub selected_indicators: HashSet<String>,

    // Optimization Metrics Configuration
    pub available_metrics: Vec<MetricInfo>,
    pub selected_metrics: HashMap<String, OptimizationDirection>, // metric_name -> direction

    // Trade Management Configuration
    pub initial_capital: f64,
    pub commission: f64,
    pub slippage: f64,
    pub stop_loss: StopLossConfig,
    pub take_profit: TakeProfitConfig,
    pub position_sizing: PositionSizing,
    pub max_positions: usize,

    // Evolution Configuration
    pub population_size: usize,
    pub num_generations: usize,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elitism_count: usize,
    pub max_tree_depth: usize,
    pub tournament_size: usize,

    // Backtesting Configuration
    pub validation_method: ValidationMethod,
    pub train_test_split: f64,
    pub num_folds: usize,

    // Execution State
    pub is_running: bool,
    pub current_generation: usize,
    pub progress_percentage: f32,
    pub status_message: String,

    // Results
    pub hall_of_fame: Vec<StrategyDisplay>,
    pub selected_strategy_idx: Option<usize>,

    // Sorting/Filtering
    pub sort_column: String,
    pub sort_ascending: bool,
    pub filter_min_trades: Option<usize>,
}

impl Default for AppState {
    fn default() -> Self {
        // Define available optimization metrics
        let available_metrics = vec![
            MetricInfo {
                name: "return_pct".to_string(),
                display_name: "Return %".to_string(),
                description: "Total return percentage".to_string(),
                default_direction: OptimizationDirection::Maximize,
            },
            MetricInfo {
                name: "sharpe_ratio".to_string(),
                display_name: "Sharpe Ratio".to_string(),
                description: "Risk-adjusted return metric".to_string(),
                default_direction: OptimizationDirection::Maximize,
            },
            MetricInfo {
                name: "max_drawdown".to_string(),
                display_name: "Max Drawdown %".to_string(),
                description: "Maximum peak-to-trough decline".to_string(),
                default_direction: OptimizationDirection::Minimize,
            },
            MetricInfo {
                name: "win_rate".to_string(),
                display_name: "Win Rate %".to_string(),
                description: "Percentage of winning trades".to_string(),
                default_direction: OptimizationDirection::Maximize,
            },
            MetricInfo {
                name: "profit_factor".to_string(),
                display_name: "Profit Factor".to_string(),
                description: "Gross profit / gross loss".to_string(),
                default_direction: OptimizationDirection::Maximize,
            },
            MetricInfo {
                name: "num_trades".to_string(),
                display_name: "Number of Trades".to_string(),
                description: "Total number of completed trades".to_string(),
                default_direction: OptimizationDirection::Maximize,
            },
        ];

        // Default selected metrics (return_pct, sharpe_ratio, max_drawdown)
        let mut selected_metrics = HashMap::new();
        selected_metrics.insert("return_pct".to_string(), OptimizationDirection::Maximize);
        selected_metrics.insert("sharpe_ratio".to_string(), OptimizationDirection::Maximize);
        selected_metrics.insert("max_drawdown".to_string(), OptimizationDirection::Minimize);

        Self {
            // Data Configuration
            data_file_path: None,
            loaded_data: None,
            data_preview: None,

            // Indicator Selection
            available_indicators: Vec::new(),
            selected_indicators: HashSet::new(),

            // Optimization Metrics Configuration
            available_metrics,
            selected_metrics,

            // Trade Management Configuration
            initial_capital: 10000.0,
            commission: 0.001,      // 0.1%
            slippage: 0.0005,       // 0.05%
            stop_loss: StopLossConfig::None,
            take_profit: TakeProfitConfig::None,
            position_sizing: PositionSizing::Fixed { size: 100.0 },
            max_positions: 1,

            // Evolution Configuration
            population_size: 500,
            num_generations: 100,
            mutation_rate: 0.15,
            crossover_rate: 0.85,
            elitism_count: 10,
            max_tree_depth: 12,
            tournament_size: 7,

            // Backtesting Configuration
            validation_method: ValidationMethod::Simple,
            train_test_split: 0.7,
            num_folds: 5,

            // Execution State
            is_running: false,
            current_generation: 0,
            progress_percentage: 0.0,
            status_message: "Ready".to_string(),

            // Results
            hall_of_fame: Vec::new(),
            selected_strategy_idx: None,

            // Sorting/Filtering
            sort_column: "fitness".to_string(),
            sort_ascending: false,
            filter_min_trades: None,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Display model for strategy in results table
#[derive(Clone)]
pub struct StrategyDisplay {
    pub rank: usize,
    pub fitness: f64,
    pub return_pct: f64,
    pub total_trades: usize,
    pub win_rate: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub formula: String,          // Short version for table
    pub formula_full: String,      // Full version
    pub equity_curve: Vec<f64>,
    pub trades: Vec<crate::types::Trade>,
}

/// Indicator information for selection
#[derive(Clone, Debug)]
pub struct IndicatorInfo {
    pub name: String,
    pub alias: String,
    pub category: IndicatorCategory,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndicatorCategory {
    Trend,
    Momentum,
    Volatility,
    Volume,
}

/// Metric information for optimization
#[derive(Clone, Debug)]
pub struct MetricInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub default_direction: OptimizationDirection,
}
