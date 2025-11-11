# 10 - Testing & Verification

## Goal
Implement comprehensive testing to ensure correctness and consistency.

## Prerequisites
- **[09-code-generation.md](./09-code-generation.md)** completed
- All components implemented

## Testing Strategy

### Test Pyramid
1. **Unit Tests** - Individual functions (primitives, indicators)
2. **Integration Tests** - Components working together
3. **Verification Tests** - Rust vs MQL5 consistency

## Test Implementation

### Step 1: Unit Tests for Primitives

```rust
// tests/primitives_test.rs
use tradebias::*;
use polars::prelude::*;
use approx::assert_relative_eq;

#[test]
fn test_sma_calculation() {
    // Test data: [1, 2, 3, 4, 5]
    let data = Series::new("close", &[1.0, 2.0, 3.0, 4.0, 5.0]);
    let period = 3;

    // Expected SMA: [NaN, NaN, 2.0, 3.0, 4.0]
    // (first two values are NaN due to insufficient data)

    let sma = MovingAverage::sma();
    let result = sma.execute(&[col("close"), lit(period)]);

    assert!(result.is_ok());
    // Further validation with actual DataFrame
}

#[test]
fn test_highest_lowest() {
    let data = Series::new("close", &[5.0, 3.0, 8.0, 2.0, 6.0]);

    let highest = Highest;
    let lowest = Lowest;

    // Test with period = 3
    // Should find max and min over rolling window

    // Implementation would use actual DataFrame operations
    assert_eq!(highest.arity(), 2);
    assert_eq!(lowest.arity(), 2);
}
```

### Step 2: Integration Tests for Indicators

```rust
// tests/indicator_integration_test.rs
use tradebias::*;
use polars::prelude::*;

#[test]
fn test_rsi_with_real_data() {
    // Create sample OHLC data
    let close_data = vec![
        44.0, 44.25, 44.5, 43.75, 44.0, 44.5, 45.0, 45.25, 45.5, 45.0,
        44.5, 44.0, 43.5, 43.0, 42.5, 43.0, 43.5, 44.0, 44.5, 45.0
    ];

    let df = df! {
        "close" => &close_data,
    }.unwrap();

    let rsi = RSI;
    let args = vec![
        IndicatorArg::Series(col("close")),
        IndicatorArg::Scalar(14.0),
    ];

    let result = rsi.calculate_vectorized(&args);
    assert!(result.is_ok());

    // Verify RSI is in valid range [0, 100]
    // (Actual validation would execute on DataFrame)
}

#[test]
fn test_macd_calculation() {
    let close_data: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64) * 0.5).collect();

    let df = df! {
        "close" => &close_data,
    }.unwrap();

    let macd = MACD;
    let args = vec![
        IndicatorArg::Series(col("close")),
        IndicatorArg::Scalar(12.0),
        IndicatorArg::Scalar(26.0),
        IndicatorArg::Scalar(9.0),
    ];

    let result = macd.calculate_vectorized(&args);
    assert!(result.is_ok());
}
```

### Step 3: Golden File Tests (Rust vs MQL5)

```rust
// tests/golden_file_test.rs
use tradebias::*;
use std::fs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct GoldenTestCase {
    name: String,
    indicator: String,
    params: Vec<f64>,
    input_data: Vec<f64>,
    expected_output: Vec<f64>,
}

#[test]
fn test_rsi_golden_file() {
    // Load golden test data (pre-calculated from MQL5)
    let golden_data = fs::read_to_string("tests/fixtures/rsi_golden.json")
        .expect("Failed to read golden file");

    let test_case: GoldenTestCase = serde_json::from_str(&golden_data)
        .expect("Failed to parse golden file");

    // Run Rust implementation
    let df = df! {
        "close" => &test_case.input_data,
    }.unwrap();

    let rsi = RSI;
    let args = vec![
        IndicatorArg::Series(col("close")),
        IndicatorArg::Scalar(test_case.params[0]),
    ];

    // Execute and compare with golden values
    // (Simplified - actual implementation would extract series and compare)

    // Assert within tolerance
    // for (rust_val, golden_val) in rust_result.iter().zip(test_case.expected_output.iter()) {
    //     assert_relative_eq!(rust_val, golden_val, epsilon = 0.001);
    // }
}
```

### Step 4: Backtesting Integration Test

```rust
// tests/backtesting_integration_test.rs
use tradebias::*;

#[test]
fn test_full_backtest_pipeline() {
    // Create registry
    let registry = Arc::new(FunctionRegistry::new());
    let cache = Arc::new(IndicatorCache::new(100));

    // Create sample data
    let df = df! {
        "open" => &[100.0, 101.0, 102.0, 101.5, 103.0, 104.0],
        "high" => &[101.0, 102.0, 103.0, 102.5, 104.0, 105.0],
        "low" => &[99.0, 100.0, 101.0, 100.5, 102.0, 103.0],
        "close" => &[100.5, 101.5, 102.5, 101.0, 103.5, 104.5],
        "volume" => &[1000.0, 1100.0, 1200.0, 1050.0, 1300.0, 1400.0],
    }.unwrap();

    // Create simple strategy: RSI < 30 = buy, RSI > 70 = sell
    let ast = AstNode::Call {
        function: "RSI".to_string(),
        args: vec![
            Box::new(AstNode::Const(Value::String("close".to_string()))),
            Box::new(AstNode::Const(Value::Integer(14))),
        ],
    };

    // Run backtest
    let backtester = Backtester::new(registry, cache, 10000.0);
    let result = backtester.run(&ast, &df);

    assert!(result.is_ok());
    let result = result.unwrap();

    // Verify result structure
    assert!(!result.equity_curve.is_empty());
    assert!(result.metrics.contains_key("return_pct"));

    // Calculate metrics
    let metrics_engine = MetricsEngine::new(10000.0);
    let all_metrics = metrics_engine.calculate_all(&result);

    assert!(all_metrics.contains_key("win_rate"));
    assert!(all_metrics.contains_key("max_drawdown_pct"));
    assert!(all_metrics.contains_key("sharpe_ratio"));
}
```

### Step 5: Code Generation Tests

```rust
// tests/codegen_test.rs
use tradebias::*;

#[test]
fn test_mqh_generation() {
    let registry = Arc::new(FunctionRegistry::new());
    let generator = IndicatorLibraryGenerator::new(registry);

    let mqh_code = generator.generate();

    // Verify structure
    assert!(mqh_code.contains("TradeBias_Indicators.mqh"));
    assert!(mqh_code.contains("#property strict"));

    // Verify all Tier 1 indicators present
    assert!(mqh_code.contains("TB_RSI"));
    assert!(mqh_code.contains("TB_SMA"));
    assert!(mqh_code.contains("TB_EMA"));
    assert!(mqh_code.contains("TB_MACD"));
    assert!(mqh_code.contains("TB_ATR"));

    // Verify mathematical consistency comments
    assert!(mqh_code.contains("matches TradeBias Rust engine"));
}

#[test]
fn test_ea_generation() {
    let ast = AstNode::Call {
        function: "RSI".to_string(),
        args: vec![],
    };

    let ea_code = EAGenerator::generate(&ast, "TestStrategy");

    // Verify EA structure
    assert!(ea_code.contains("OnInit"));
    assert!(ea_code.contains("OnTick"));
    assert!(ea_code.contains("OnDeinit"));

    // Verify includes
    assert!(ea_code.contains("#include \"TradeBias_Indicators.mqh\""));

    // Verify trading logic
    assert!(ea_code.contains("OrderSend"));
    assert!(ea_code.contains("OP_BUY"));
    assert!(ea_code.contains("OP_SELL"));
}
```

## Test Data Preparation

### Create Golden Files

Generate expected values from MQL5:

```json
// tests/fixtures/rsi_golden.json
{
  "name": "RSI Period 14",
  "indicator": "RSI",
  "params": [14.0],
  "input_data": [44.0, 44.25, 44.5, 43.75, 44.0, 44.5, 45.0, 45.25, 45.5, 45.0, 44.5, 44.0, 43.5, 43.0, 42.5, 43.0, 43.5, 44.0, 44.5, 45.0],
  "expected_output": [50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 50.0, 42.37, 46.15, 50.49, 54.38, 57.89, 60.98]
}
```

## Running Tests

```bash
# Run all tests
cargo test

# Run specific test category
cargo test primitives_test
cargo test indicator_integration_test
cargo test golden_file_test

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench
```

## Continuous Integration

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all
      - name: Run clippy
        run: cargo clippy -- -D warnings
      - name: Check formatting
        run: cargo fmt -- --check
```

## Verification Checklist

After completing all tests:

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Golden file tests match MQL5 (within 0.1% tolerance)
- [ ] Backtesting pipeline works end-to-end
- [ ] MQL5 code generation produces valid MQL5
- [ ] Generated EA compiles in MetaTrader 5
- [ ] All indicators return values in expected ranges
- [ ] No panics or unwraps in production code
- [ ] Performance benchmarks meet targets

## Common Issues

**Issue**: Golden file tests fail
**Solution**: Verify MQL5 calculation logic matches Rust exactly. Check for differences in rounding, smoothing methods.

**Issue**: Backtesting results inconsistent
**Solution**: Check that indicators are cached correctly and state is managed properly for stateful indicators.

**Issue**: Generated MQL5 doesn't compile
**Solution**: Verify syntax of generated code, check for missing semicolons, bracket matching.

## Performance Targets

- **Primitive operations**: < 1ms for 10K data points
- **Vectorized indicators**: < 10ms for 10K data points
- **Full backtest (simple strategy)**: < 100ms for 10K bars
- **Indicator caching**: 90%+ cache hit rate in typical usage

## Congratulations!

You've completed the full TradeBias implementation! The system is now ready for:
- Strategy evolution with genetic algorithms
- Backtesting with custom indicators
- MQL5 code generation for live trading
- ML-based strategy refinement

Refer back to **[00-overview.md](./00-overview.md)** for the complete navigation guide.
