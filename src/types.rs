use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Value scale information
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScaleType {
    Price,              // Follows price (SMA, BB)
    Oscillator0_100,    // 0-100 bounded (RSI, Stochastic)
    OscillatorCentered, // Zero-centered (MACD, Momentum)
    Volatility,         // Small decimals (ATR, StdDev)
    Volume,             // Large integers (OBV)
    Ratio,              // Ratios (Williams %R)
}

/// Data type for expressions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    NumericSeries,  // Polars Series<f64>
    BoolSeries,     // Polars Series<bool>
    Integer,        // Scalar i32
    Float,          // Scalar f64
}

/// Abstract Syntax Tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AstNode {
    Const(Value),
    Call {
        function: String,
        args: Vec<Box<AstNode>>,
    },
    Rule {
        condition: Box<AstNode>,
        action: Box<AstNode>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
}

/// Trade record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub entry_bar: usize,
    pub exit_bar: usize,
    pub entry_price: f64,
    pub exit_price: f64,
    pub direction: Direction,
    pub size: f64,
    pub profit: f64,
    pub exit_reason: ExitReason,
    pub fees: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExitReason {
    StopLoss,
    TakeProfit,
    Signal,
    EndOfData,
}

/// Complete strategy evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResult {
    pub ast: AstNode,
    pub metrics: HashMap<String, f64>,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<f64>,
    pub in_sample: bool,
}

// AST Pretty Printer Implementation
impl AstNode {
    /// Format AstNode as a human-readable formula string
    pub fn to_formula(&self) -> String {
        match self {
            AstNode::Const(value) => match value {
                Value::Integer(i) => i.to_string(),
                Value::Float(f) => {
                    // Format with appropriate precision
                    if f.fract() == 0.0 {
                        format!("{:.0}", f)
                    } else if f.abs() < 0.01 {
                        format!("{:.4}", f)
                    } else {
                        format!("{:.2}", f)
                    }
                }
                Value::String(s) => format!("\"{}\"", s),
                Value::Bool(b) => b.to_string(),
            },
            AstNode::Call { function, args } => {
                let formatted_args: Vec<String> = args.iter()
                    .map(|arg| arg.to_formula())
                    .collect();
                format!("{}({})", function, formatted_args.join(", "))
            }
            AstNode::Rule { condition, action } => {
                format!("IF {} THEN {}", condition.to_formula(), action.to_formula())
            }
        }
    }

    /// Format for table display (truncated to max_len characters)
    pub fn to_formula_short(&self, max_len: usize) -> String {
        let full = self.to_formula();
        if full.len() > max_len {
            format!("{}...", &full[..max_len.saturating_sub(3)])
        } else {
            full
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_to_formula_constant() {
        let ast = AstNode::Const(Value::Float(3.14));
        assert_eq!(ast.to_formula(), "3.14");
    }

    #[test]
    fn test_ast_to_formula_call() {
        let ast = AstNode::Call {
            function: "RSI".to_string(),
            args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
        };
        assert_eq!(ast.to_formula(), "RSI(14)");
    }

    #[test]
    fn test_ast_to_formula_nested() {
        let ast = AstNode::Call {
            function: "Greater".to_string(),
            args: vec![
                Box::new(AstNode::Call {
                    function: "RSI".to_string(),
                    args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
                }),
                Box::new(AstNode::Const(Value::Integer(30))),
            ],
        };
        assert_eq!(ast.to_formula(), "Greater(RSI(14), 30)");
    }

    #[test]
    fn test_ast_to_formula_truncated() {
        let long_ast = AstNode::Call {
            function: "And".to_string(),
            args: vec![
                Box::new(AstNode::Call {
                    function: "Greater".to_string(),
                    args: vec![
                        Box::new(AstNode::Call {
                            function: "RSI".to_string(),
                            args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
                        }),
                        Box::new(AstNode::Const(Value::Integer(30))),
                    ],
                }),
                Box::new(AstNode::Call {
                    function: "Less".to_string(),
                    args: vec![
                        Box::new(AstNode::Call {
                            function: "RSI".to_string(),
                            args: vec![Box::new(AstNode::Const(Value::Integer(14)))],
                        }),
                        Box::new(AstNode::Const(Value::Integer(70))),
                    ],
                }),
            ],
        };

        let short = long_ast.to_formula_short(30);
        assert!(short.len() <= 30);
        assert!(short.ends_with("..."));
    }
}
