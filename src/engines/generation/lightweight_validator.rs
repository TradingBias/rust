use crate::engines::generation::ast::*;
use crate::functions::registry::FunctionRegistry;
use crate::error::TradebiasError;
use crate::types::AstNode;

pub struct LightweightValidator {
    registry: FunctionRegistry,
    max_depth: usize,
}

impl LightweightValidator {
    pub fn new(registry: FunctionRegistry, max_depth: usize) -> Self {
        Self { registry, max_depth }
    }

    /// Validate AST structure, types, and arity
    pub fn validate(&self, ast: &StrategyAST) -> Result<(), TradebiasError> {
        self.validate_node(ast.as_node(), 0)
    }

    fn validate_node(&self, node: &AstNode, depth: usize) -> Result<(), TradebiasError> {
        // Depth check
        if depth > self.max_depth {
            return Err(TradebiasError::Validation(format!(
                "AST depth {} exceeds maximum {}",
                depth, self.max_depth
            )));
        }

        match node {
            AstNode::Rule { condition, action } => {
                self.validate_node(condition, depth + 1)?;
                self.validate_node(action, depth + 1)?;
                Ok(())
            }
            AstNode::Call { function, args } => {
                // Function exists?
                let func = self.registry.get_function(function).ok_or_else(|| {
                    TradebiasError::Validation(format!("Unknown function: {}", function))
                })?;

                // Arity matches?
                let arity = match func {
                    crate::functions::strategy::StrategyFunction::Indicator(ref i) => i.arity(),
                    crate::functions::strategy::StrategyFunction::Primitive(ref p) => p.arity(),
                };
                if args.len() != arity {
                    return Err(TradebiasError::Validation(format!(
                        "Function {} expects {} args, got {}",
                        function,
                        arity,
                        args.len()
                    )));
                }

                // Type compatibility (simplified check)
                let input_types = func.input_types();
                for (i, arg) in args.iter().enumerate() {
                    if i < input_types.len() {
                        // TODO: Deep type checking
                        self.validate_node(arg, depth + 1)?;
                    }
                }

                Ok(())
            }
            AstNode::Const(_) => Ok(()), // Constants are always valid
        }
    }
}
