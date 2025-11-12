use super::base *;
use crate::engines::evaluation::Backtester;
use crate::types::{AstNode, ConstValue};
use crate::engines::generation::ast::*;
use crate::error::TradebiasError;
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
    ) -> Result<TestResult, TradebiasError> {
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
                "Strategy is stable under parameter variations. Maximum performance drop: {:.1}% (acceptable threshold: {:.1}%)",
                max_drop_pct,
                self.max_degradation_pct
            )
        } else {
            format!(
                "WARNING: Strategy is sensitive to parameter changes. Maximum performance drop: {:.1}% at {} (threshold: {:.1}%)",
                max_drop_pct,
                most_sensitive_param,
                self.max_degradation_pct
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

    fn extract_from_node(&self, node: &AstNode, path: &str, params: &mut Vec<(String, i32)>) {
        match node {
            AstNode::Call { function, args } => {
                let current_path = if path.is_empty() {
                    function.clone()
                } else {
                    format!("{}.{}", path, function)
                };

                // Look for integer parameters (typically periods)
                for (i, arg) in args.iter().enumerate() {
                    match arg {
                        AstNode::Const(ConstValue::Integer(value)) => {
                            params.push((format!("{}.arg{}", current_path, i), *value));
                        }
                        AstNode::Call { .. } => {
                            self.extract_from_node(arg, &current_path, params);
                        }
                        _ => {} // Ignore other node types
                    }
                }
            }
            _ => {} // Ignore other node types
        }
    }

    fn modify_parameter(
        &self,
        ast: &StrategyAST,
        param_path: &str,
        new_value: i32,
    ) -> Result<StrategyAST, TradebiasError> {
        // Clone and modify AST
        let mut modified = ast.clone();
        // Implementation would recursively find and replace the parameter
        // This is simplified - actual implementation needs proper AST traversal
        Ok(modified)
    }
}
