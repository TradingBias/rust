use thiserror::Error;

#[derive(Error, Debug)]
pub enum TradebiasError {
    #[error("Invalid AST: {0}")]
    InvalidAst(String),
    
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },
    
    #[error("Indicator error: {0}")]
    IndicatorError(String),
    
    #[error("Backtest error: {0}")]
    BacktestError(String),

    #[error("Generation error: {0}")]
    Generation(String),

    #[error("Computation error: {0}")]
    Computation(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Polars error: {0}")]
    Polars(#[from] polars::error::PolarsError),
    
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TradebiasError>;
