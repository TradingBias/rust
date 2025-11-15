use crate::config::backtesting::BacktestingConfig;
use crate::config::evolution::EvolutionConfig;
use crate::config::trade_management::TradeManagementConfig;
use crate::engines::generation::pareto::ObjectiveConfig;
use crate::ui::state::AppState;

pub struct ConfigBridge;

impl ConfigBridge {
    /// Convert AppState to BacktestingConfig
    pub fn to_backtesting_config(state: &AppState) -> BacktestingConfig {
        BacktestingConfig {
            validation_method: state.validation_method.clone(),
            train_test_split: state.train_test_split,
            num_folds: state.num_folds,
            initial_capital: state.initial_capital,
            commission: state.commission,
            slippage: state.slippage,
        }
    }

    /// Convert AppState to EvolutionConfig
    pub fn to_evolution_config(state: &AppState) -> EvolutionConfig {
        use crate::config::evolution::SelectionMethod;

        EvolutionConfig {
            population_size: state.population_size,
            num_generations: state.num_generations,
            mutation_rate: state.mutation_rate,
            crossover_rate: state.crossover_rate,
            selection_method: SelectionMethod::Tournament, // Default to tournament
            elitism_count: state.elitism_count,
            max_tree_depth: state.max_tree_depth,
            tournament_size: state.tournament_size,
        }
    }

    /// Convert AppState to TradeManagementConfig
    pub fn to_trade_management_config(state: &AppState) -> TradeManagementConfig {
        TradeManagementConfig {
            stop_loss: state.stop_loss.clone(),
            take_profit: state.take_profit.clone(),
            position_sizing: state.position_sizing.clone(),
            max_positions: state.max_positions,
        }
    }

    /// Convert selected metrics from AppState to ObjectiveConfig vector
    pub fn to_objective_configs(state: &AppState) -> Vec<ObjectiveConfig> {
        state.selected_metrics
            .iter()
            .map(|(metric_name, direction)| ObjectiveConfig {
                metric_name: metric_name.clone(),
                direction: *direction,
            })
            .collect()
    }
}
