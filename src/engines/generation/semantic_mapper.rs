use crate::engines::generation::{
    gene_consumer::GeneConsumer,
    ast::{StrategyAST, StrategyMetadata},
};
use crate::functions::registry::FunctionRegistry;
use crate::types::{AstNode, DataType, Value as ConstValue};
use crate::error::TradebiasError;
use crate::functions::strategy::StrategyFunction;
use crate::utils::indicator_metadata::MetadataRegistry;
use std::sync::Arc;

pub struct SemanticMapper {
    registry: Arc<FunctionRegistry>,
    metadata: MetadataRegistry,
    max_depth: usize,
}

impl SemanticMapper {
    pub fn new(registry: Arc<FunctionRegistry>, max_depth: usize) -> Self {
        Self {
            registry,
            metadata: MetadataRegistry::new(),
            max_depth,
        }
    }

    /// Main entry point: Create complete strategy AST from genome
    pub fn create_strategy_ast(&self, genome: &[u32]) -> Result<StrategyAST, TradebiasError> {
        let mut consumer = GeneConsumer::new(genome);

        // Build condition (must return BoolSeries)
        let condition = self.build_expression(DataType::BoolSeries, &mut consumer, 0)?;

        // Build action (simple for now, can be extended)
        // 1.0 = Long signal, -1.0 = Short signal
        let action_choice = consumer.choose(2);
        let action = if action_choice == 0 {
            AstNode::Const(ConstValue::Float(1.0)) // Long
        } else {
            AstNode::Const(ConstValue::Float(-1.0)) // Short
        };

        let root = AstNode::Rule {
            condition: Box::new(condition),
            action: Box::new(action),
        };

        Ok(StrategyAST {
            root: Box::new(root),
            metadata: StrategyMetadata::default(),
        })
    }

    /// Recursively build expression of desired type
    fn build_expression(
        &self,
        desired_type: DataType,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<AstNode, TradebiasError> {
        // Depth limit to prevent infinite recursion
        if depth >= self.max_depth {
            return self.build_terminal(desired_type, consumer);
        }

        match desired_type {
            DataType::BoolSeries => self.build_bool_series(consumer, depth),
            DataType::NumericSeries => self.build_numeric_series(consumer, depth),
            DataType::Integer => self.build_integer(consumer),
            DataType::Float => self.build_float(consumer),
        }
    }

    fn build_bool_series(
        &self,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<AstNode, TradebiasError> {
        // Get functions that return BoolSeries
        let functions = self.registry.get_by_output_type(DataType::BoolSeries);

        if functions.is_empty() {
            return Err(TradebiasError::Generation(
                "No functions return BoolSeries".to_string(),
            ));
        }

        // Choose function
        let func_idx = consumer.choose(functions.len());
        let func = &functions[func_idx];

        // Build arguments
        let args = self.build_arguments(func, consumer, depth + 1)?;

        Ok(AstNode::Call {
            function: func.name().to_string(),
            args,
        })
    }

    fn build_numeric_series(
        &self,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<AstNode, TradebiasError> {
        // Choice: indicator, primitive data accessor, or math operation
        let choice = consumer.choose(3);

        match choice {
            0 => self.build_indicator(consumer, depth),
            1 => self.build_data_accessor(consumer),
            _ => self.build_math_operation(consumer, depth),
        }
    }

    fn build_indicator(
        &self,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<AstNode, TradebiasError> {
        let indicators = self.registry.get_indicators();

        if indicators.is_empty() {
            return self.build_data_accessor(consumer);
        }

        let func_idx = consumer.choose(indicators.len());
        let func = &indicators[func_idx];

        // Build arguments with smart parameter generation
        let args = self.build_indicator_arguments(&StrategyFunction::Indicator(func.clone()), consumer, depth + 1)?;

        Ok(AstNode::Call {
            function: func.alias().to_string(),
            args,
        })
    }

    fn build_indicator_arguments(
        &self,
        func: &StrategyFunction,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<Vec<Box<AstNode>>, TradebiasError> {
        let input_types = func.input_types();
        let mut args = Vec::new();

        let indicator = func.as_indicator().ok_or_else(|| {
            TradebiasError::Generation("Expected an indicator".to_string())
        })?;

        for arg_type in input_types {
            match arg_type {
                DataType::Integer => {
                    // Smart period generation using metadata
                    if let Some(meta) = self.metadata.get(indicator.alias()) {
                        if let Some(periods) = &meta.typical_periods {
                            let period = periods[consumer.choose(periods.len())];
                            args.push(Box::new(AstNode::Const(ConstValue::Integer(period as i64))));
                        } else {
                            args.push(Box::new(self.build_integer(consumer)?));
                        }
                    } else {
                        args.push(Box::new(self.build_integer(consumer)?));
                    }
                }
                _ => {
                    args.push(Box::new(self.build_expression(arg_type, consumer, depth)?));
                }
            }
        }

        Ok(args)
    }

    fn build_arguments(
        &self,
        func: &StrategyFunction,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<Vec<Box<AstNode>>, TradebiasError> {
        func.input_types()
            .into_iter()
            .map(|arg_type| self.build_expression(arg_type, consumer, depth).map(Box::new))
            .collect()
    }

    fn build_data_accessor(&self, consumer: &mut GeneConsumer) -> Result<AstNode, TradebiasError> {
        let accessors = ["Open", "High", "Low", "Close", "Volume"];
        let choice = consumer.choose(accessors.len());

        Ok(AstNode::Call {
            function: accessors[choice].to_string(),
            args: vec![],
        })
    }

    fn build_math_operation(
        &self,
        consumer: &mut GeneConsumer,
        depth: usize,
    ) -> Result<AstNode, TradebiasError> {
        let operations = ["Add", "Subtract", "Multiply", "Divide"];
        let choice = consumer.choose(operations.len());

        // Must call build_expression to ensure depth checking happens
        let arg1 = self.build_expression(DataType::NumericSeries, consumer, depth + 1)?;
        let arg2 = self.build_expression(DataType::NumericSeries, consumer, depth + 1)?;

        Ok(AstNode::Call {
            function: operations[choice].to_string(),
            args: vec![Box::new(arg1), Box::new(arg2)],
        })
    }

    fn build_integer(&self, consumer: &mut GeneConsumer) -> Result<AstNode, TradebiasError> {
        // Common indicator periods
        let periods = [5, 7, 9, 10, 12, 14, 20, 21, 25, 30, 50, 100, 200];
        let value = periods[consumer.choose(periods.len())];

        Ok(AstNode::Const(ConstValue::Integer(value)))
    }

    fn build_float(&self, consumer: &mut GeneConsumer) -> Result<AstNode, TradebiasError> {
        let value = consumer.float_range(0.0, 100.0);
        Ok(AstNode::Const(ConstValue::Float(value)))
    }

    fn build_terminal(
        &self,
        desired_type: DataType,
        consumer: &mut GeneConsumer,
    ) -> Result<AstNode, TradebiasError> {
        match desired_type {
            DataType::NumericSeries => self.build_data_accessor(consumer),
            DataType::Integer => self.build_integer(consumer),
            DataType::Float => self.build_float(consumer),
            DataType::BoolSeries => {
                // When we hit max depth and need a BoolSeries, create a simple comparison
                // This prevents the "Cannot build terminal for type BoolSeries" error
                let comparisons = ["gt_scalar", "lt_scalar", "gte_scalar", "lte_scalar"];
                let choice = consumer.choose(comparisons.len());

                // Get a numeric series (data accessor)
                let series = self.build_data_accessor(consumer)?;

                // Get a scalar threshold value
                let threshold = self.build_float(consumer)?;

                Ok(AstNode::Call {
                    function: comparisons[choice].to_string(),
                    args: vec![Box::new(series), Box::new(threshold)],
                })
            }
        }
    }
}
