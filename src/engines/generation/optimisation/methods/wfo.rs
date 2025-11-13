use super::base::*;
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::engines::generation::optimisation::splitters::{
    base::DataSplitter,
    wfo::WalkForwardSplitter,
    types::WindowType,
};
use crate::error::TradebiasError;
use polars::prelude::*;
use std::collections::HashMap;

pub struct WalkForwardMethod {
    splitter: WalkForwardSplitter,
    backtester: Backtester,
}

impl WalkForwardMethod {
    pub fn new(
        in_sample_pct: f64,
        out_of_sample_pct: f64,
        n_folds: usize,
        window_type: WindowType,
        backtester: Backtester,
    ) -> Self {
        Self {
            splitter: WalkForwardSplitter::new(
                in_sample_pct,
                out_of_sample_pct,
                n_folds,
                window_type,
            ),
            backtester,
        }
    }
}

impl ValidationMethod for WalkForwardMethod {
    fn name(&self) -> &str {
        "Walk-Forward Optimization"
    }

    fn validate(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
    ) -> Result<AggregatedResult, TradebiasError> {
        // Split data into folds
        let splits = self.splitter.split(data)?;

        // Run backtest on each fold
        let mut fold_results = Vec::new();

        for split in splits {
            // In-sample backtest
            let is_result = self.backtester.run(ast, &split.in_sample)?;

            // Out-of-sample backtest
            let oos_result = self.backtester.run(ast, &split.out_of_sample)?;

            fold_results.push(ValidationResult {
                in_sample_result: is_result,
                out_of_sample_result: oos_result,
                fold_num: split.fold_num,
            });
        }

        // Aggregate metrics across folds
        let aggregate_metrics = self.aggregate_metrics(&fold_results);

        Ok(AggregatedResult {
            method: self.name().to_string(),
            folds: fold_results,
            aggregate_metrics,
        })
    }
}

impl WalkForwardMethod {
    fn aggregate_metrics(&self, folds: &[ValidationResult]) -> HashMap<String, f64> {
        let mut aggregated = HashMap::new();

        if folds.is_empty() {
            return aggregated;
        }

        // Get metric names from first fold
        let metric_names: Vec<String> = folds[0]
            .out_of_sample_result
            .metrics
            .keys()
            .cloned()
            .collect();

        // Calculate mean of each metric across OOS results
        for metric_name in metric_names {
            let values: Vec<f64> = folds
                .iter()
                .filter_map(|f| f.out_of_sample_result.metrics.get(&metric_name).copied())
                .collect();

            if !values.is_empty() {
                let mean = values.iter().sum::<f64>() / values.len() as f64;
                let std = calculate_std(&values, mean);

                aggregated.insert(format!("{}_mean", metric_name), mean);
                aggregated.insert(format!("{}_std", metric_name), std);
                aggregated.insert(format!("{}_min", metric_name), values.iter().copied().fold(f64::INFINITY, f64::min));
                aggregated.insert(format!("{}_max", metric_name), values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
            }
        }

        // Calculate consistency score (lower std = more consistent)
        if let Some(sharpe_std) = aggregated.get("sharpe_ratio_std") {
            let consistency = 1.0 / (1.0 + sharpe_std);
            aggregated.insert("consistency_score".to_string(), consistency);
        }

        aggregated
    }
}

fn calculate_std(values: &[f64], mean: f64) -> f64 {
    if values.len() <= 1 {
        return 0.0;
    }

    let variance = values
        .iter()
        .map(|v| (v - mean).powi(2))
        .sum::<f64>() / (values.len() - 1) as f64;

    variance.sqrt()
}
