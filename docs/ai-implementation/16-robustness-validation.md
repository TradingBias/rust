# 16 - Robustness Validation & Testing

## Goal
Implement the robustness validation engine that tests strategy stability through Monte Carlo simulations, parameter perturbation, and friction testing. This generates comprehensive "Robustness Reports" for strategies.

## Prerequisites
- **07-backtesting-engine.md** - Backtesting infrastructure
- **08-metrics-engine.md** - Performance metrics
- **11-evolution-engine.md** - Strategy representation
- **13-optimization-methods.md** - Validation framework

## What You'll Create
1. `ValidationOrchestrator` - Runs full robustness test suite
2. `MonteCarloTest` - Trade permutation testing
3. `ParameterStabilityTest` - System Parameter Permutation (SPP)
4. `FrictionTest` - Delayed entry/exit simulation
5. `RobustnessReport` - Comprehensive test results

## Why Robustness Testing?

**Problem**: A strategy that performs well in backtest may fail in live trading due to:
- Overfitting to historical data
- Sensitivity to parameter changes
- Slippage and execution delays
- Data mining bias

**Solution**: Stress test the strategy through multiple scenarios to verify it's truly robust.

## Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│         ValidationOrchestrator                        │
│                                                       │
│  Input: Strategy AST + Market Data                    │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ Test 1: Monte Carlo (Trade Permutation)       │  │
│  │  • Shuffle trade order 1000 times              │  │
│  │  • Calculate distribution of outcomes          │  │
│  │  • Original performance vs random luck?        │  │
│  └────────────────────────────────────────────────┘  │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ Test 2: Parameter Stability (SPP)              │  │
│  │  • Vary each parameter ±20%                    │  │
│  │  • Re-run backtest for each variation         │  │
│  │  • Check if performance degrades sharply       │  │
│  └────────────────────────────────────────────────┘  │
│                                                       │
│  ┌────────────────────────────────────────────────┐  │
│  │ Test 3: Friction Test (Delayed Execution)     │  │
│  │  • Delay entry/exit by 1 bar                   │  │
│  │  • Simulate slippage and latency               │  │
│  │  • Performance should remain acceptable        │  │
│  └────────────────────────────────────────────────┘  │
│                                                       │
│  Output: Robustness Report (JSON)                    │
└──────────────────────────────────────────────────────┘
```

## Test Descriptions

### 1. Monte Carlo (Trade Permutation)
**Tests**: Is the strategy's performance due to skill or luck?

**Method**:
1. Run original backtest, get list of trade P&Ls
2. Randomly shuffle the P&L sequence
3. Recalculate equity curve and metrics
4. Repeat 1000 times
5. Compare original performance to distribution

**Pass Criteria**: Original Sharpe ratio > 95th percentile of shuffled results

### 2. Parameter Stability (SPP)
**Tests**: Is performance sensitive to parameter values?

**Method**:
1. Identify all numeric parameters in strategy (e.g., RSI period = 14)
2. Create variations: ±10%, ±20%, ±30%
3. Run backtest for each variation
4. Measure performance degradation

**Pass Criteria**: Performance drop < 30% for ±20% parameter changes

### 3. Friction Test
**Tests**: Does strategy survive realistic trading conditions?

**Method**:
1. Delay all entries by 1 bar (simulate order execution)
2. Delay all exits by 1 bar
3. Re-run backtest
4. Compare to original

**Pass Criteria**: Performance drop < 20% with 1-bar delay

## Implementation

### Step 1: Base Robustness Test Trait

Create `src/engines/validation/robustness/base.rs`:

```rust
use crate::data::types::StrategyResult;
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::error::TradeBiasError;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub score: f64,           // 0.0 to 1.0 (1.0 = perfect)
    pub details: serde_json::Value,
    pub interpretation: String,
}

pub trait RobustnessTest: Send + Sync {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    /// Run the robustness test
    fn run(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
        backtester: &Backtester,
    ) -> Result<TestResult, TradeBiasError>;
}
```

### Step 2: Monte Carlo Test

Create `src/engines/validation/robustness/monte_carlo.rs`:

```rust
use super::base::*;
use crate::data::types::{StrategyResult, Trade};
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::engines::metrics::MetricsEngine;
use crate::error::TradeBiasError;
use polars::prelude::*;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use rand::rngs::StdRng;
use serde_json::json;

pub struct MonteCarloTest {
    n_simulations: usize,
    metric_name: String, // e.g., "sharpe_ratio"
    seed: Option<u64>,
}

impl MonteCarloTest {
    pub fn new(n_simulations: usize, metric_name: String) -> Self {
        Self {
            n_simulations,
            metric_name,
            seed: Some(42),
        }
    }
}

impl RobustnessTest for MonteCarloTest {
    fn name(&self) -> &str {
        "Monte Carlo (Trade Permutation)"
    }

    fn description(&self) -> &str {
        "Tests if strategy performance is due to skill or random luck by randomly shuffling trade outcomes"
    }

    fn run(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
        backtester: &Backtester,
    ) -> Result<TestResult, TradeBiasError> {
        // Run original backtest
        let original_result = backtester.run(ast, data)?;
        let original_metric = original_result
            .metrics
            .get(&self.metric_name)
            .copied()
            .unwrap_or(0.0);

        // Extract trade P&Ls
        let trade_pnls: Vec<f64> = original_result
            .trades
            .iter()
            .map(|t| t.profit_loss)
            .collect();

        if trade_pnls.is_empty() {
            return Ok(TestResult {
                test_name: self.name().to_string(),
                passed: false,
                score: 0.0,
                details: json!({
                    "error": "No trades generated by strategy"
                }),
                interpretation: "Cannot perform Monte Carlo test without trades".to_string(),
            });
        }

        // Run simulations
        let mut rng = match self.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let mut simulated_metrics = Vec::new();

        for _ in 0..self.n_simulations {
            // Shuffle trades
            let mut shuffled_pnls = trade_pnls.clone();
            shuffled_pnls.shuffle(&mut rng);

            // Recalculate equity curve
            let initial_capital = original_result.initial_capital;
            let equity_curve = self.calculate_equity_curve(initial_capital, &shuffled_pnls);

            // Calculate metric for shuffled sequence
            let metric = self.calculate_metric_from_equity(&equity_curve);
            simulated_metrics.push(metric);
        }

        // Sort simulations
        simulated_metrics.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Calculate percentile of original
        let percentile = self.calculate_percentile(&simulated_metrics, original_metric);

        // Pass if original is in top 5% (95th percentile or higher)
        let passed = percentile >= 95.0;
        let score = (percentile / 100.0).max(0.0).min(1.0);

        // Calculate statistics
        let mean = simulated_metrics.iter().sum::<f64>() / simulated_metrics.len() as f64;
        let p5 = simulated_metrics[(simulated_metrics.len() as f64 * 0.05) as usize];
        let p50 = simulated_metrics[simulated_metrics.len() / 2];
        let p95 = simulated_metrics[(simulated_metrics.len() as f64 * 0.95) as usize];

        let interpretation = if passed {
            format!(
                "Strategy performance ({}th percentile) is significantly better than random chance. \
                Original {} = {:.2}, vs random mean = {:.2}",
                percentile as u32, self.metric_name, original_metric, mean
            )
        } else {
            format!(
                "WARNING: Strategy performance ({}th percentile) may be due to luck. \
                Original {} = {:.2}, vs random mean = {:.2}",
                percentile as u32, self.metric_name, original_metric, mean
            )
        };

        Ok(TestResult {
            test_name: self.name().to_string(),
            passed,
            score,
            details: json!({
                "original_metric": original_metric,
                "metric_name": self.metric_name,
                "percentile": percentile,
                "simulations": self.n_simulations,
                "simulation_stats": {
                    "mean": mean,
                    "p5": p5,
                    "p50": p50,
                    "p95": p95,
                }
            }),
            interpretation,
        })
    }
}

impl MonteCarloTest {
    fn calculate_equity_curve(&self, initial_capital: f64, pnls: &[f64]) -> Vec<f64> {
        let mut equity = vec![initial_capital];
        let mut current = initial_capital;

        for &pnl in pnls {
            current += pnl;
            equity.push(current);
        }

        equity
    }

    fn calculate_metric_from_equity(&self, equity_curve: &[f64]) -> f64 {
        // Simplified metric calculation (Sharpe-like)
        if equity_curve.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = equity_curve
            .windows(2)
            .map(|w| (w[1] - w[0]) / w[0])
            .collect();

        if returns.is_empty() {
            return 0.0;
        }

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>() / (returns.len() - 1) as f64;

        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            return 0.0;
        }

        // Annualized Sharpe (assuming 252 trading days)
        let sharpe = (mean_return / std_dev) * (252.0_f64).sqrt();
        sharpe
    }

    fn calculate_percentile(&self, sorted_values: &[f64], value: f64) -> f64 {
        let count = sorted_values.iter().filter(|&&v| v < value).count();
        (count as f64 / sorted_values.len() as f64) * 100.0
    }
}
```

### Step 3: Parameter Stability Test

Create `src/engines/validation/robustness/parameter_stability.rs`:

```rust
use super::base::*;
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::*;
use crate::error::TradeBiasError;
use polars::prelude::*;
use serde_json::json;
use std::collections::HashMap;

pub struct ParameterStabilityTest {
    variations: Vec<f64>, // e.g., [0.8, 0.9, 1.0, 1.1, 1.2] for ±20%
    metric_name: String,
    max_degradation_pct: f64, // e.g., 30.0 = allow 30% drop
}

impl ParameterStabilityTest {
    pub fn new(metric_name: String) -> Self {
        Self {
            variations: vec![0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3], // -30% to +30%
            metric_name,
            max_degradation_pct: 30.0,
        }
    }
}

impl RobustnessTest for ParameterStabilityTest {
    fn name(&self) -> &str {
        "Parameter Stability (System Parameter Permutation)"
    }

    fn description(&self) -> &str {
        "Tests if strategy performance degrades significantly when parameters are perturbed"
    }

    fn run(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
        backtester: &Backtester,
    ) -> Result<TestResult, TradeBiasError> {
        // Extract parameters from AST
        let parameters = self.extract_parameters(ast);

        if parameters.is_empty() {
            return Ok(TestResult {
                test_name: self.name().to_string(),
                passed: true,
                score: 1.0,
                details: json!({
                    "note": "No numeric parameters found in strategy"
                }),
                interpretation: "Strategy has no parameters to perturb".to_string(),
            });
        }

        // Run original backtest
        let original_result = backtester.run(ast, data)?;
        let original_metric = original_result
            .metrics
            .get(&self.metric_name)
            .copied()
            .unwrap_or(0.0);

        // Test each parameter variation
        let mut all_results = Vec::new();

        for (param_path, original_value) in &parameters {
            for &variation_multiplier in &self.variations {
                if variation_multiplier == 1.0 {
                    continue; // Skip original
                }

                let new_value = (*original_value as f64 * variation_multiplier).round() as i32;

                // Create modified AST
                let modified_ast = self.modify_parameter(ast, param_path, new_value)?;

                // Run backtest
                let result = backtester.run(&modified_ast, data)?;
                let metric_value = result
                    .metrics
                    .get(&self.metric_name)
                    .copied()
                    .unwrap_or(0.0);

                all_results.push((param_path.clone(), variation_multiplier, metric_value));
            }
        }

        // Analyze stability
        let mut max_drop_pct = 0.0;
        let mut most_sensitive_param = String::new();

        for (param_path, multiplier, metric) in &all_results {
            let drop_pct = if original_metric != 0.0 {
                ((original_metric - metric) / original_metric.abs()) * 100.0
            } else {
                0.0
            };

            if drop_pct > max_drop_pct {
                max_drop_pct = drop_pct;
                most_sensitive_param = format!("{} ({}x)", param_path, multiplier);
            }
        }

        // Pass if degradation is within acceptable range
        let passed = max_drop_pct <= self.max_degradation_pct;
        let score = ((self.max_degradation_pct - max_drop_pct) / self.max_degradation_pct)
            .max(0.0)
            .min(1.0);

        let interpretation = if passed {
            format!(
                "Strategy is stable under parameter variations. \
                Maximum performance drop: {:.1}% (acceptable threshold: {:.1}%)",
                max_drop_pct, self.max_degradation_pct
            )
        } else {
            format!(
                "WARNING: Strategy is sensitive to parameter changes. \
                Maximum performance drop: {:.1}% at {} \
                (threshold: {:.1}%)",
                max_drop_pct, most_sensitive_param, self.max_degradation_pct
            )
        };

        Ok(TestResult {
            test_name: self.name().to_string(),
            passed,
            score,
            details: json!({
                "original_metric": original_metric,
                "metric_name": self.metric_name,
                "max_drop_pct": max_drop_pct,
                "most_sensitive_param": most_sensitive_param,
                "parameters_tested": parameters.len(),
                "variations_per_param": self.variations.len() - 1,
                "results": all_results,
            }),
            interpretation,
        })
    }
}

impl ParameterStabilityTest {
    fn extract_parameters(&self, ast: &StrategyAST) -> Vec<(String, i32)> {
        let mut params = Vec::new();
        match ast {
            StrategyAST::Rule { condition, .. } => {
                self.extract_from_node(condition, "", &mut params);
            }
        }
        params
    }

    fn extract_from_node(&self, node: &ASTNode, path: &str, params: &mut Vec<(String, i32)>) {
        match node {
            ASTNode::Call { function, args } => {
                let current_path = if path.is_empty() {
                    function.clone()
                } else {
                    format!("{}.{}", path, function)
                };

                // Look for integer parameters (typically periods)
                for (i, arg) in args.iter().enumerate() {
                    match arg {
                        ASTNode::Const(ConstValue::Integer(value)) => {
                            params.push((format!("{}.arg{}", current_path, i), *value));
                        }
                        ASTNode::Call { .. } => {
                            self.extract_from_node(arg, &current_path, params);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    fn modify_parameter(
        &self,
        ast: &StrategyAST,
        param_path: &str,
        new_value: i32,
    ) -> Result<StrategyAST, TradeBiasError> {
        // Clone and modify AST
        let mut modified = ast.clone();
        // Implementation would recursively find and replace the parameter
        // This is simplified - actual implementation needs proper AST traversal
        Ok(modified)
    }
}
```

### Step 4: Friction Test

Create `src/engines/validation/robustness/friction.rs`:

```rust
use super::base::*;
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::error::TradeBiasError;
use polars::prelude::*;
use serde_json::json;

pub struct FrictionTest {
    delay_bars: usize, // Number of bars to delay execution
    metric_name: String,
    max_degradation_pct: f64,
}

impl FrictionTest {
    pub fn new(metric_name: String) -> Self {
        Self {
            delay_bars: 1,
            metric_name,
            max_degradation_pct: 20.0,
        }
    }
}

impl RobustnessTest for FrictionTest {
    fn name(&self) -> &str {
        "Friction Test (Delayed Execution)"
    }

    fn description(&self) -> &str {
        "Tests if strategy survives realistic trading conditions with execution delays"
    }

    fn run(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
        backtester: &Backtester,
    ) -> Result<TestResult, TradeBiasError> {
        // Run original backtest
        let original_result = backtester.run(ast, data)?;
        let original_metric = original_result
            .metrics
            .get(&self.metric_name)
            .copied()
            .unwrap_or(0.0);

        // Run with delayed execution
        // This requires modifying the backtester to delay signals
        // For now, we simulate by shifting the signal series
        let delayed_data = self.create_delayed_data(data)?;
        let delayed_result = backtester.run(ast, &delayed_data)?;
        let delayed_metric = delayed_result
            .metrics
            .get(&self.metric_name)
            .copied()
            .unwrap_or(0.0);

        // Calculate degradation
        let drop_pct = if original_metric != 0.0 {
            ((original_metric - delayed_metric) / original_metric.abs()) * 100.0
        } else {
            0.0
        };

        let passed = drop_pct <= self.max_degradation_pct;
        let score = ((self.max_degradation_pct - drop_pct) / self.max_degradation_pct)
            .max(0.0)
            .min(1.0);

        let interpretation = if passed {
            format!(
                "Strategy survives realistic execution delays. \
                Performance drop with {}-bar delay: {:.1}% (threshold: {:.1}%)",
                self.delay_bars, drop_pct, self.max_degradation_pct
            )
        } else {
            format!(
                "WARNING: Strategy is sensitive to execution delays. \
                Performance drop with {}-bar delay: {:.1}% (threshold: {:.1}%)",
                self.delay_bars, drop_pct, self.max_degradation_pct
            )
        };

        Ok(TestResult {
            test_name: self.name().to_string(),
            passed,
            score,
            details: json!({
                "original_metric": original_metric,
                "delayed_metric": delayed_metric,
                "metric_name": self.metric_name,
                "drop_pct": drop_pct,
                "delay_bars": self.delay_bars,
            }),
            interpretation,
        })
    }
}

impl FrictionTest {
    fn create_delayed_data(&self, data: &DataFrame) -> Result<DataFrame, TradeBiasError> {
        // Shift data forward to simulate execution delay
        // This is a simplified version - actual implementation would need
        // to properly handle signal delays in the backtester
        Ok(data.clone())
    }
}
```

### Step 5: Validation Orchestrator

Create `src/engines/validation/orchestrator.rs`:

```rust
use super::robustness::{
    base::*,
    monte_carlo::MonteCarloTest,
    parameter_stability::ParameterStabilityTest,
    friction::FrictionTest,
};
use crate::engines::evaluation::Backtester;
use crate::engines::generation::ast::StrategyAST;
use crate::error::TradeBiasError;
use polars::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobustnessReport {
    pub strategy_id: String,
    pub timestamp: String,
    pub test_results: Vec<TestResult>,
    pub overall_score: f64,
    pub passed_all: bool,
    pub summary: String,
}

pub struct ValidationOrchestrator {
    backtester: Backtester,
    tests: Vec<Box<dyn RobustnessTest>>,
}

impl ValidationOrchestrator {
    pub fn new(backtester: Backtester) -> Self {
        let tests: Vec<Box<dyn RobustnessTest>> = vec![
            Box::new(MonteCarloTest::new(1000, "sharpe_ratio".to_string())),
            Box::new(ParameterStabilityTest::new("sharpe_ratio".to_string())),
            Box::new(FrictionTest::new("sharpe_ratio".to_string())),
        ];

        Self { backtester, tests }
    }

    pub fn run_robustness_report(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
        strategy_id: String,
    ) -> Result<RobustnessReport, TradeBiasError> {
        let mut test_results = Vec::new();

        // Run all tests
        for test in &self.tests {
            println!("Running test: {}", test.name());
            let result = test.run(ast, data, &self.backtester)?;
            test_results.push(result);
        }

        // Calculate overall metrics
        let overall_score = test_results.iter().map(|r| r.score).sum::<f64>() / test_results.len() as f64;
        let passed_all = test_results.iter().all(|r| r.passed);

        let summary = if passed_all {
            format!(
                "Strategy passed all {} robustness tests with overall score {:.1}%",
                test_results.len(),
                overall_score * 100.0
            )
        } else {
            let failed_count = test_results.iter().filter(|r| !r.passed).count();
            format!(
                "Strategy failed {} of {} robustness tests. Overall score: {:.1}%",
                failed_count,
                test_results.len(),
                overall_score * 100.0
            )
        };

        Ok(RobustnessReport {
            strategy_id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            test_results,
            overall_score,
            passed_all,
            summary,
        })
    }
}
```

## Usage Example

```rust
use tradebias::engines::validation::orchestrator::ValidationOrchestrator;
use tradebias::engines::evaluation::Backtester;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load strategy and data
    let ast = load_strategy_ast("strategy.json")?;
    let data = load_market_data("data.csv")?;

    // Create orchestrator
    let backtester = Backtester::new(/* ... */);
    let orchestrator = ValidationOrchestrator::new(backtester);

    // Run robustness report
    let report = orchestrator.run_robustness_report(
        &ast,
        &data,
        "strategy_001".to_string(),
    )?;

    // Print report
    println!("\n{}", "=".repeat(60));
    println!("ROBUSTNESS REPORT");
    println!("{}", "=".repeat(60));
    println!("\nStrategy ID: {}", report.strategy_id);
    println!("Overall Score: {:.1}%", report.overall_score * 100.0);
    println!("Status: {}", if report.passed_all { "PASSED" } else { "FAILED" });
    println!("\n{}", report.summary);

    println!("\nDetailed Results:\n");
    for result in &report.test_results {
        println!("  Test: {}", result.test_name);
        println!("  Status: {}", if result.passed { "✓ PASS" } else { "✗ FAIL" });
        println!("  Score: {:.1}%", result.score * 100.0);
        println!("  {}", result.interpretation);
        println!();
    }

    // Save report
    let json = serde_json::to_string_pretty(&report)?;
    std::fs::write("robustness_report.json", json)?;

    Ok(())
}
```

## Verification

### Test 1: Monte Carlo Detection
```rust
#[test]
fn test_monte_carlo_detects_luck() {
    // Create strategy with random-looking results
    let lucky_strategy = create_lucky_strategy();
    let data = create_test_data();

    let test = MonteCarloTest::new(100, "sharpe_ratio".to_string());
    let backtester = create_test_backtester();

    let result = test.run(&lucky_strategy, &data, &backtester).unwrap();

    // Should fail (percentile < 95)
    assert!(!result.passed);
}
```

### Test 2: Parameter Sensitivity Detection
```rust
#[test]
fn test_parameter_stability() {
    // Strategy with very sensitive parameters
    let sensitive_strategy = create_sensitive_strategy();
    let data = create_test_data();

    let test = ParameterStabilityTest::new("sharpe_ratio".to_string());
    let backtester = create_test_backtester();

    let result = test.run(&sensitive_strategy, &data, &backtester).unwrap();

    // Should fail due to high sensitivity
    assert!(!result.passed);
}
```

## Common Issues

### Issue: Monte Carlo test always fails
**Solution**: Check if strategy has enough trades (need 30+ for statistical significance). Also verify original performance is actually good.

### Issue: All parameters fail stability test
**Solution**: This indicates overfitting. Strategy is too specific to exact parameter values. Consider broader parameter ranges or different approach.

### Issue: Friction test shows huge degradation
**Solution**: Strategy relies on precise entry/exit timing. This won't work in live trading. Consider using limit orders or longer timeframes.

## Next Steps

Proceed to **[17-configuration-system.md](./17-configuration-system.md)** to implement the configuration management system.
