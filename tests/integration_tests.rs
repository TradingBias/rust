use std::collections::HashMap;
use tradebias::engines::metrics::MetricsEngine;
use tradebias::types::{AstNode, Direction, ExitReason, StrategyResult, Trade, Value};

#[test]
fn test_metrics_engine() {
    let trades = vec![
        Trade {
            entry_bar: 0,
            exit_bar: 5,
            entry_price: 100.0,
            exit_price: 110.0,
            direction: Direction::Long,
            size: 1.0,
            profit: 10.0,
            exit_reason: ExitReason::TakeProfit,
            fees: 0.0,
        },
        Trade {
            entry_bar: 6,
            exit_bar: 10,
            entry_price: 110.0,
            exit_price: 105.0,
            direction: Direction::Short,
            size: 1.0,
            profit: 5.0,
            exit_reason: ExitReason::Signal,
            fees: 0.0,
        },
    ];

    let equity_curve = vec![10000.0, 10010.0, 10015.0];

    let result = StrategyResult {
        ast: AstNode::Const(Value::Bool(true)),
        metrics: HashMap::new(),
        trades,
        equity_curve,
        in_sample: true,
    };

    let engine = MetricsEngine::new(10000.0);
    let metrics = engine.calculate_all(&result);

    assert!(metrics.contains_key("return_pct"));
    assert!(metrics.contains_key("win_rate"));
    assert!(metrics.contains_key("max_drawdown_pct"));
    assert!(metrics.contains_key("sharpe_ratio"));
}
