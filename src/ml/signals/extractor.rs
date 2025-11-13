use super::types::*;
use crate::engines::evaluation::expression::ExpressionBuilder;
use crate::engines::generation::ast::StrategyAST;
use crate::types::AstNode;
use crate::error::{TradebiasError, Result};
use polars::prelude::*;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

pub struct SignalExtractor {
    expression_builder: ExpressionBuilder,
}

impl SignalExtractor {
    pub fn new(expression_builder: ExpressionBuilder) -> Self {
        Self { expression_builder }
    }

    /// Extract all signals from strategy AST
    pub fn extract(
        &self,
        ast: &StrategyAST,
        data: &DataFrame,
    ) -> Result<SignalDataset> {
        let condition = match ast.root.as_ref() {
            AstNode::Rule { condition, .. } => condition,
            _ => return Err(TradebiasError::Validation("StrategyAST root is not a Rule node".to_string())),
        };

        // Build condition expression
        let condition_expr = self.expression_builder.build(condition, data)?;

        // Evaluate condition over full dataset
        let condition_series = data
            .clone()
            .lazy()
            .select([condition_expr.alias("condition")])
            .collect()?
            .column("condition")?
            .bool()?
            .clone();

        // Find where condition is true
        let mut signals = Vec::new();

        for (idx, is_signal) in condition_series.into_iter().enumerate() {
            if let Some(true) = is_signal {
                let timestamp = self.get_timestamp_at(data, idx)?;
                let direction = self.get_signal_direction(ast)?;
                let indicator_values = self.extract_indicator_values(data, idx)?;

                signals.push(Signal {
                    timestamp,
                    bar_index: idx,
                    direction,
                    indicator_values,
                });
            }
        }

        Ok(SignalDataset {
            signals,
            market_data: data.clone(),
        })
    }

    fn get_timestamp_at(&self, data: &DataFrame, idx: usize) -> Result<DateTime<Utc>> {
        let timestamp_ms = data
            .column("timestamp")?
            .datetime()?
            .phys.get(idx)
            .ok_or_else(|| TradebiasError::Validation("Invalid timestamp at index".to_string()))?;

        let timestamp_s = timestamp_ms / 1000;
        DateTime::<Utc>::from_timestamp(timestamp_s, 0)
            .ok_or_else(|| TradebiasError::Validation("Invalid timestamp value".to_string()))
    }

    fn get_signal_direction(&self, ast: &StrategyAST) -> Result<SignalDirection> {
        // Extract action from AST
        match ast.root.as_ref() {
            AstNode::Rule { action, .. } => {
                match action.as_ref() {
                    AstNode::Call { function, .. } if function == "OpenLong" => Ok(SignalDirection::Long),
                    AstNode::Call { function, .. } if function == "OpenShort" => Ok(SignalDirection::Short),
                    _ => Err(TradebiasError::Validation("Unknown or unsupported action in AST".to_string())),
                }
            },
            _ => Err(TradebiasError::Validation("StrategyAST root is not a Rule node for signal direction".to_string())),
        }
    }

    fn extract_indicator_values(
        &self,
        data: &DataFrame,
        idx: usize,
    ) -> Result<HashMap<String, f64>> {
        let mut values = HashMap::new();

        // Extract common indicators at signal bar
        if let Ok(col) = data.column("close") {
            if let Some(val) = col.f64()?.get(idx) {
                values.insert("close".to_string(), val);
            }
        }

        // Add more indicators as needed...

        Ok(values)
    }
}
