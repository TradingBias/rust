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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
