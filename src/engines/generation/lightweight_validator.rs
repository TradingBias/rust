use crate::engines::generation::ast::*;
use crate::functions::registry::FunctionRegistry;
use crate::error::TradebiasError;

pub struct LightweightValidator {
    registry: FunctionRegistry,
    max_depth: usize,
}

impl LightweightValidator {
    pub fn new(registry: FunctionRegistry, max_depth: usize) -> Self {
        Self { registry, max_depth }
    }

    /// Validate AST structure, types, and arity
    pub fn validate(&self, ast: &StrategyAST) -> Result<(), TradeBiasError> {
        match ast {
            StrategyAST::Rule { condition, action } => {
                self.validate_node(condition, 0)?;
                self.validate_node(action, 0)?;
                Ok(())
            }
        }
    }

    fn validate_node(&self, node: &AstNode, depth: usize) -> Result<(), TradebiasError> {
        // Depth check
        if depth > self.max_depth {
            return Err(TradeBiasError::Validation(format!(
                "AST depth {} exceeds maximum {}",
                depth, self.max_depth
            )));
        }

        match node {
            AstNode::Call { function, args } => {
                // Function exists?
                let func = self.registry.get_function(function).ok_or_else(|| {
                    TradeBiasError::Validation(format!("Unknown function: {}", function))
                })?;

                // Arity matches?
                if args.len() != func.arity() {
                    return Err(TradeBiasError::Validation(format!(
                        "Function {} expects {} args, got {}",
                        function,
                        func.arity(),
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
