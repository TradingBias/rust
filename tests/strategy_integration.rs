use polars::prelude::*;
use polars::lazy::dsl;
use tradebias::functions::indicators::*;
use tradebias::functions::primitives::*;
use tradebias::functions::traits::{VectorizedIndicator, IndicatorArg, Primitive};
use polars::df;

/// Test MA crossover strategy using indicators + comparison primitives
#[test]
fn test_ma_crossover_strategy() {
    // Create sample OHLCV data
    let df = df! {
        "timestamp" => &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        "close" => &[100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0, 108.0, 110.0, 112.0],
    }
    .unwrap();

    let fast_ma = SMA::new(3);
    let slow_ma = SMA::new(5);

    let lazy_df = df.lazy();

    // Calculate indicators
    let fast_expr = fast_ma
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("fast_ma");

    let slow_expr = slow_ma
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("slow_ma");

    // Add MAs to dataframe
    let df_with_mas = lazy_df
        .with_column(fast_expr)
        .with_column(slow_expr)
        .collect()
        .unwrap();

    // Generate signals using comparison operators
    let lazy_df2 = df_with_mas.lazy();

    let buy_signal = CrossAbove
        .execute(&[dsl::col("fast_ma"), dsl::col("slow_ma")])
        .unwrap()
        .alias("buy_signal");

    let sell_signal = CrossBelow
        .execute(&[dsl::col("fast_ma"), dsl::col("slow_ma")])
        .unwrap()
        .alias("sell_signal");

    let result = lazy_df2
        .with_column(buy_signal)
        .with_column(sell_signal)
        .collect()
        .unwrap();

    // Verify signals are boolean series
    assert_eq!(result.column("buy_signal").unwrap().dtype(), &DataType::Boolean);
    assert_eq!(result.column("sell_signal").unwrap().dtype(), &DataType::Boolean);

    // Verify we have all columns
    assert!(result.column("fast_ma").is_ok());
    assert!(result.column("slow_ma").is_ok());
    assert!(result.column("buy_signal").is_ok());
    assert!(result.column("sell_signal").is_ok());

    println!("MA Crossover Strategy:");
    println!("{}", result);
}

/// Test RSI threshold strategy
#[test]
fn test_rsi_threshold_strategy() {
    let df = df! {
        "close" => &[
            100.0, 105.0, 103.0, 108.0, 110.0,
            107.0, 112.0, 115.0, 113.0, 118.0,
            120.0, 122.0, 119.0, 125.0, 123.0,
        ],
    }
    .unwrap();

    let rsi = RSI::new(14);
    let lazy_df = df.lazy();

    // Calculate RSI
    let rsi_expr = rsi
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
            IndicatorArg::Scalar(14.0),
        ])
        .unwrap()
        .alias("rsi");

    let df_with_rsi = lazy_df.with_column(rsi_expr).collect().unwrap();

    // Generate signals: buy when RSI < 30, sell when RSI > 70
    let lazy_df2 = df_with_rsi.lazy();

    let oversold = LessThanScalar
        .execute(&[dsl::col("rsi"), dsl::lit(30.0)])
        .unwrap()
        .alias("oversold");

    let overbought = GreaterThanScalar
        .execute(&[dsl::col("rsi"), dsl::lit(70.0)])
        .unwrap()
        .alias("overbought");

    let result = lazy_df2
        .with_column(oversold)
        .with_column(overbought)
        .collect()
        .unwrap();

    // Verify signals work
    assert_eq!(result.column("oversold").unwrap().dtype(), &DataType::Boolean);
    assert_eq!(result.column("overbought").unwrap().dtype(), &DataType::Boolean);

    println!("RSI Threshold Strategy:");
    println!("{}", result);
}

/// Test combined strategy: MA crossover + RSI filter
#[test]
fn test_combined_ma_rsi_strategy() {
    let df = df! {
        "close" => &[
            100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0,
            108.0, 110.0, 112.0, 111.0, 113.0, 115.0, 114.0,
            116.0, 118.0, 120.0, 119.0, 121.0, 123.0,
        ],
    }
    .unwrap();

    let fast_ma = SMA::new(5);
    let slow_ma = SMA::new(10);
    let rsi = RSI::new(14);

    let lazy_df = df.lazy();

    // Calculate all indicators
    let fast_expr = fast_ma
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("fast_ma");

    let slow_expr = slow_ma
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("slow_ma");

    let rsi_expr = rsi
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
            IndicatorArg::Scalar(14.0),
        ])
        .unwrap()
        .alias("rsi");

    let df_with_indicators = lazy_df
        .with_column(fast_expr)
        .with_column(slow_expr)
        .with_column(rsi_expr)
        .collect()
        .unwrap();

    // Generate complex signals
    let lazy_df2 = df_with_indicators.lazy();

    let ma_cross = CrossAbove
        .execute(&[dsl::col("fast_ma"), dsl::col("slow_ma")])
        .unwrap()
        .alias("ma_cross");

    let rsi_ok = LessThanScalar
        .execute(&[dsl::col("rsi"), dsl::lit(70.0)])
        .unwrap()
        .alias("rsi_ok");

    // Buy signal: MA crossover AND RSI < 70
    let buy_signal = And
        .execute(&[dsl::col("ma_cross"), dsl::col("rsi_ok")])
        .unwrap()
        .alias("buy_signal");

    let result = lazy_df2
        .with_column(ma_cross)
        .with_column(rsi_ok)
        .with_column(buy_signal)
        .collect()
        .unwrap();

    // Verify all signals
    assert!(result.column("ma_cross").is_ok());
    assert!(result.column("rsi_ok").is_ok());
    assert!(result.column("buy_signal").is_ok());

    println!("Combined MA+RSI Strategy:");
    println!("{}", result);
}

/// Test MACD strategy
#[test]
fn test_macd_strategy() {
    let df = df! {
        "close" => &[
            100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0,
            108.0, 110.0, 112.0, 111.0, 113.0, 115.0, 114.0,
            116.0, 118.0, 120.0, 119.0, 121.0, 123.0, 125.0,
            124.0, 126.0, 128.0, 127.0, 129.0, 131.0, 130.0,
        ],
    }
    .unwrap();

    let macd = MACD::new(12, 26, 9);
    let lazy_df = df.lazy();

    // Calculate MACD
    let macd_expr = macd
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("macd");

    let df_with_macd = lazy_df.with_column(macd_expr).collect().unwrap();

    // Generate signals: MACD crosses above 0
    let lazy_df2 = df_with_macd.lazy();

    let macd_positive = GreaterThanScalar
        .execute(&[dsl::col("macd"), dsl::lit(0.0)])
        .unwrap()
        .alias("macd_positive");

    let macd_negative = LessThanScalar
        .execute(&[dsl::col("macd"), dsl::lit(0.0)])
        .unwrap()
        .alias("macd_negative");

    let result = lazy_df2
        .with_column(macd_positive)
        .with_column(macd_negative)
        .collect()
        .unwrap();

    // Verify signals
    assert_eq!(result.column("macd_positive").unwrap().dtype(), &DataType::Boolean);
    assert_eq!(result.column("macd_negative").unwrap().dtype(), &DataType::Boolean);

    println!("MACD Strategy:");
    println!("{}", result);
}

/// Test EMA crossover strategy (faster than SMA)
#[test]
fn test_ema_crossover_strategy() {
    let df = df! {
        "close" => &[
            100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0,
            108.0, 110.0, 112.0, 111.0, 113.0, 115.0,
        ],
    }
    .unwrap();

    let fast_ema = EMA::new(5);
    let slow_ema = EMA::new(10);

    let lazy_df = df.lazy();

    // Calculate EMAs
    let fast_expr = fast_ema
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("ema_fast");

    let slow_expr = slow_ema
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("ema_slow");

    let df_with_emas = lazy_df
        .with_column(fast_expr)
        .with_column(slow_expr)
        .collect()
        .unwrap();

    // Generate crossover signals
    let lazy_df2 = df_with_emas.lazy();

    let bullish = CrossAbove
        .execute(&[dsl::col("ema_fast"), dsl::col("ema_slow")])
        .unwrap()
        .alias("bullish_cross");

    let bearish = CrossBelow
        .execute(&[dsl::col("ema_fast"), dsl::col("ema_slow")])
        .unwrap()
        .alias("bearish_cross");

    let result = lazy_df2
        .with_column(bullish)
        .with_column(bearish)
        .collect()
        .unwrap();

    // Verify results
    assert!(result.column("ema_fast").is_ok());
    assert!(result.column("ema_slow").is_ok());
    assert!(result.column("bullish_cross").is_ok());
    assert!(result.column("bearish_cross").is_ok());

    println!("EMA Crossover Strategy:");
    println!("{}", result);
}

/// Test multi-timeframe logic (comparing different period indicators)
#[test]
fn test_multi_period_alignment() {
    let df = df! {
        "close" => &[
            100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0,
            108.0, 110.0, 112.0, 111.0, 113.0, 115.0, 114.0,
            116.0, 118.0, 120.0, 119.0, 121.0, 123.0,
        ],
    }
    .unwrap();

    let sma_short = SMA::new(5);
    let sma_medium = SMA::new(10);
    let sma_long = SMA::new(15);

    let lazy_df = df.lazy();

    // Calculate all SMAs
    let short_expr = sma_short
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("sma_5");

    let medium_expr = sma_medium
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("sma_10");

    let long_expr = sma_long
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap()
        .alias("sma_15");

    let df_with_smas = lazy_df
        .with_column(short_expr)
        .with_column(medium_expr)
        .with_column(long_expr)
        .collect()
        .unwrap();

    // Check trend alignment: sma_5 > sma_10 > sma_15 (strong uptrend)
    let lazy_df2 = df_with_smas.lazy();

    let cond1 = GreaterThan
        .execute(&[dsl::col("sma_5"), dsl::col("sma_10")])
        .unwrap()
        .alias("short_above_medium");

    let cond2 = GreaterThan
        .execute(&[dsl::col("sma_10"), dsl::col("sma_15")])
        .unwrap()
        .alias("medium_above_long");

    let aligned = And
        .execute(&[dsl::col("short_above_medium"), dsl::col("medium_above_long")])
        .unwrap()
        .alias("trend_aligned");

    let result = lazy_df2
        .with_column(cond1)
        .with_column(cond2)
        .with_column(aligned)
        .collect()
        .unwrap();

    // Verify we can detect trend alignment
    assert!(result.column("trend_aligned").is_ok());

    println!("Multi-Period Trend Alignment:");
    println!("{}", result);
}
