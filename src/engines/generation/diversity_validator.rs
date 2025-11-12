use crate::engines::generation::ast::*;
use std::collections::{HashMap, HashSet};

/// Validates that indicator parameters are diverse
pub struct DiversityValidator {
    min_param_difference: i32,
}

impl DiversityValidator {
    pub fn new(min_param_difference: i32) -> Self {
        Self { min_param_difference }
    }

    /// Check if AST has diverse indicator parameters
    pub fn validate(&self, ast: &StrategyAST) -> bool {
        let mut indicator_params: HashMap<String, Vec<i32>> = HashMap::new();

        match ast {
            StrategyAST::Rule { condition, action } => {
                self.collect_indicator_params(condition, &mut indicator_params);
                self.collect_indicator_params(action, &mut indicator_params);
            }
        }

        // Check each indicator type
        for (_indicator, params) in indicator_params.iter() {
            if !self.are_params_diverse(params) {
                return false;
            }
        }

        true
    }

    fn collect_indicator_params(
        &self,
        node: &ASTNode,
        collector: &mut HashMap<String, Vec<i32>>,
    ) {
        match node {
            ASTNode::Call { function, args } => {
                // If this is an indicator call with integer params
                if args.len() > 0 {
                    if let Some(ASTNode::Const(ConstValue::Integer(period))) = args.get(1) {
                        collector
                            .entry(function.clone())
                            .or_insert_with(Vec::new)
                            .push(*period);
                    }
                }

                // Recurse into arguments
                for arg in args {
                    self.collect_indicator_params(arg, collector);
                }
            }
            ASTNode::Const(_) => {}
        }
    }

    fn are_params_diverse(&self, params: &[i32]) -> bool {
        if params.len() <= 1 {
            return true; // Single param is always diverse
        }

        // Check all pairs
        for i in 0..params.len() {
            for j in (i + 1)..params.len() {
                let diff = (params[i] - params[j]).abs();
                if diff < self.min_param_difference {
                    return false; // Too similar
                }
            }
        }

        true
    }
}
