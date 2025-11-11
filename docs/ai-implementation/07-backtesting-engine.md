# 07 - Backtesting Engine

## Goal
Implement the backtesting engine for strategy evaluation.

## Prerequisites
- **[06-registry-and-cache.md](./06-registry-and-cache.md)** completed
- Registry and cache working

## What You'll Create
1. Expression Builder (`src/engines/evaluation/expression.rs`)
2. Portfolio Simulator (`src/engines/evaluation/portfolio.rs`)
3. Backtester (`src/engines/evaluation/backtester.rs`)

## Key Concepts

The backtesting engine:
1. Converts AST to Polars expressions
2. Executes vectorized calculations
3. Simulates trades based on signals
4. Calculates performance metrics

## Quick Implementation

### Step 1: Expression Builder

```rust
// src/engines/evaluation/expression.rs
use crate::types::AstNode;
use crate::functions::registry::FunctionRegistry;
use crate::error::*;
use polars::prelude::*;

pub struct ExpressionBuilder {
    registry: Arc<FunctionRegistry>,
    cache: Arc<IndicatorCache>,
}

impl ExpressionBuilder {
    pub fn new(registry: Arc<FunctionRegistry>, cache: Arc<IndicatorCache>) -> Self {
        Self { registry, cache }
    }

    pub fn build(&self, ast: &AstNode, df: &DataFrame) -> Result<Expr> {
        match ast {
            AstNode::Const(value) => self.build_const(value),
            AstNode::Call { function, args } => self.build_call(function, args, df),
            AstNode::Rule { condition, action } => self.build_rule(condition, action, df),
        }
    }

    fn build_call(&self, function: &str, args: &[Box<AstNode>], df: &DataFrame) -> Result<Expr> {
        // Check cache first
        let cache_key = self.create_cache_key(function, args, df);
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(col(&cached.name()));
        }

        // Get indicator/primitive from registry
        if let Some(indicator) = self.registry.get_indicator(function) {
            self.build_indicator_call(indicator, args, df)
        } else if let Some(primitive) = self.registry.get_primitive(function) {
            self.build_primitive_call(primitive, args, df)
        } else {
            Err(TradeBiasError::IndicatorNotFound(function.to_string()))
        }
    }

    // ... implementation details
}
```

### Step 2: Portfolio Simulator

```rust
// src/engines/evaluation/portfolio.rs
use crate::types::*;
use crate::error::*;

pub struct Portfolio {
    initial_balance: f64,
    balance: f64,
    position: Option<Position>,
    trades: Vec<Trade>,
    equity_curve: Vec<f64>,
}

struct Position {
    direction: Direction,
    entry_bar: usize,
    entry_price: f64,
    size: f64,
}

impl Portfolio {
    pub fn new(initial_balance: f64) -> Self {
        Self {
            initial_balance,
            balance: initial_balance,
            position: None,
            trades: Vec::new(),
            equity_curve: vec![initial_balance],
        }
    }

    pub fn process_bar(&mut self, bar: usize, signal: f64, price: f64) -> Result<()> {
        if self.position.is_none() && signal != 0.0 {
            // Open position
            self.open_position(bar, signal, price)?;
        } else if self.position.is_some() {
            // Check exit conditions
            self.check_exit(bar, signal, price)?;
        }

        self.update_equity(price);
        Ok(())
    }

    fn open_position(&mut self, bar: usize, signal: f64, price: f64) -> Result<()> {
        let direction = if signal > 0.0 { Direction::Long } else { Direction::Short };
        let size = self.balance * 0.1 / price; // 10% of balance

        self.position = Some(Position {
            direction,
            entry_bar: bar,
            entry_price: price,
            size,
        });

        Ok(())
    }

    fn check_exit(&mut self, bar: usize, signal: f64, price: f64) -> Result<()> {
        if let Some(pos) = &self.position {
            let should_exit = match pos.direction {
                Direction::Long => signal < 0.0,
                Direction::Short => signal > 0.0,
            };

            if should_exit {
                self.close_position(bar, price, ExitReason::Signal)?;
            }
        }

        Ok(())
    }

    fn close_position(&mut self, bar: usize, price: f64, reason: ExitReason) -> Result<()> {
        if let Some(pos) = self.position.take() {
            let profit = match pos.direction {
                Direction::Long => (price - pos.entry_price) * pos.size,
                Direction::Short => (pos.entry_price - price) * pos.size,
            };

            self.balance += profit;

            self.trades.push(Trade {
                entry_bar: pos.entry_bar,
                exit_bar: bar,
                entry_price: pos.entry_price,
                exit_price: price,
                direction: pos.direction,
                size: pos.size,
                profit,
                exit_reason: reason,
                fees: 0.0,
            });
        }

        Ok(())
    }

    fn update_equity(&mut self, _price: f64) {
        self.equity_curve.push(self.balance);
    }

    pub fn get_trades(&self) -> &[Trade] {
        &self.trades
    }

    pub fn get_equity_curve(&self) -> &[f64] {
        &self.equity_curve
    }

    pub fn final_balance(&self) -> f64 {
        self.balance
    }
}
```

### Step 3: Backtester

```rust
// src/engines/evaluation/backtester.rs
use crate::types::*;
use crate::engines::evaluation::{ExpressionBuilder, Portfolio};
use crate::error::*;
use polars::prelude::*;

pub struct Backtester {
    expression_builder: Arc<ExpressionBuilder>,
    initial_balance: f64,
}

impl Backtester {
    pub fn new(
        registry: Arc<FunctionRegistry>,
        cache: Arc<IndicatorCache>,
        initial_balance: f64,
    ) -> Self {
        Self {
            expression_builder: Arc::new(ExpressionBuilder::new(registry, cache)),
            initial_balance,
        }
    }

    pub fn run(&self, ast: &AstNode, data: &DataFrame) -> Result<StrategyResult> {
        // Build expression from AST
        let signal_expr = self.expression_builder.build(ast, data)?;

        // Execute to get signal series
        let signals = data
            .clone()
            .lazy()
            .with_column(signal_expr.alias("signal"))
            .collect()?;

        let signal_series = signals.column("signal")?;
        let close_series = data.column("close")?;

        // Simulate trading
        let mut portfolio = Portfolio::new(self.initial_balance);

        for i in 0..signal_series.len() {
            let signal = signal_series.f64()?.get(i).unwrap_or(0.0);
            let price = close_series.f64()?.get(i).unwrap_or(0.0);

            portfolio.process_bar(i, signal, price)?;
        }

        // Calculate metrics
        let metrics = self.calculate_metrics(&portfolio)?;

        Ok(StrategyResult {
            ast: ast.clone(),
            metrics,
            trades: portfolio.get_trades().to_vec(),
            equity_curve: portfolio.get_equity_curve().to_vec(),
            in_sample: true,
        })
    }

    fn calculate_metrics(&self, portfolio: &Portfolio) -> Result<HashMap<String, f64>> {
        let mut metrics = HashMap::new();

        let final_balance = portfolio.final_balance();
        let return_pct = (final_balance - self.initial_balance) / self.initial_balance * 100.0;

        metrics.insert("return_pct".to_string(), return_pct);
        metrics.insert("num_trades".to_string(), portfolio.get_trades().len() as f64);
        metrics.insert("final_balance".to_string(), final_balance);

        Ok(metrics)
    }
}
```

## Verification

```rust
#[test]
fn test_backtester() {
    // Create sample data
    let df = df! {
        "close" => &[100.0, 101.0, 102.0, 101.5, 103.0],
    }.unwrap();

    // Simple strategy: buy when close > 101
    let ast = AstNode::Call {
        function: "GreaterThan".to_string(),
        args: vec![
            Box::new(AstNode::Const(Value::String("close".to_string()))),
            Box::new(AstNode::Const(Value::Float(101.0))),
        ],
    };

    let registry = Arc::new(FunctionRegistry::new());
    let cache = Arc::new(IndicatorCache::new(100));
    let backtester = Backtester::new(registry, cache, 10000.0);

    let result = backtester.run(&ast, &df).unwrap();

    assert!(result.trades.len() > 0);
    assert!(result.metrics.contains_key("return_pct"));
}
```

## Next Steps

Proceed to **[08-metrics-engine.md](./08-metrics-engine.md)** to implement comprehensive performance metrics.
