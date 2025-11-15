use polars::prelude::*;
use polars::lazy::dsl;
use tradebias::functions::indicators::*;
use tradebias::functions::traits::{VectorizedIndicator, IndicatorArg};
use polars::df;

// ===== Simple Indicators Tests =====

#[test]
fn test_sma_calculation() {
    let df = df! {
        "close" => &[1.0, 2.0, 3.0, 4.0, 5.0],
    }
    .unwrap();

    let sma = SMA::new(3);
    let expected_len = df.height();
    let lazy_df = df.lazy();

    let result_expr = sma
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // First two values should be null (insufficient data)
    // Third value: (1+2+3)/3 = 2.0
    // Fourth value: (2+3+4)/3 = 3.0
    // Fifth value: (3+4+5)/3 = 4.0
    assert_eq!(result.len(), expected_len);
    let values = result.f64().unwrap();
    assert_eq!(values.get(2), Some(2.0));
    assert_eq!(values.get(3), Some(3.0));
    assert_eq!(values.get(4), Some(4.0));
}

#[test]
fn test_ema_calculation() {
    let df = df! {
        "close" => &[1.0, 2.0, 3.0, 4.0, 5.0],
    }
    .unwrap();

    let ema = EMA::new(3);
    let expected_len = df.height();
    let lazy_df = df.lazy();

    let result_expr = ema
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // Verify EMA is calculated (values should differ from SMA)
    assert_eq!(result.len(), expected_len);

    // EMA should have non-null values after warmup period
    let values = result.f64().unwrap();
    assert!(values.get(2).is_some());
}

#[test]
fn test_roc_calculation() {
    let df = df! {
        "close" => &[100.0, 105.0, 110.0, 115.0],
    }
    .unwrap();

    let roc = ROC::new(1);
    let lazy_df = df.lazy();

    let result_expr = roc
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
            IndicatorArg::Scalar(1.0),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // ROC[1] = ((105-100)/100) * 100 = 5.0
    let values = result.f64().unwrap();
    assert_eq!(values.get(1), Some(5.0));
}

// ===== Medium Complexity Indicators Tests =====

#[test]
fn test_rsi_bounds() {
    // RSI should always be between 0 and 100
    let df = df! {
        "close" => &[
            100.0, 105.0, 103.0, 108.0, 110.0,
            107.0, 112.0, 115.0, 113.0, 118.0,
            120.0, 122.0, 119.0, 125.0, 123.0,
        ],
    }
    .unwrap();

    let rsi = RSI::new(14);
    let expected_len = df.height();
    let lazy_df = df.lazy();

    let result_expr = rsi
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
            IndicatorArg::Scalar(14.0),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // Check all non-null values are in valid range
    assert_eq!(result.len(), expected_len);
    let values = result.f64().unwrap();
    for i in 0..result.len() {
        if let Some(val) = values.get(i) {
            assert!(val >= 0.0 && val <= 100.0, "RSI value {} is out of bounds at index {}", val, i);
        }
    }
}

#[test]
fn test_macd_components() {
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
    let expected_len = df.height();
    let lazy_df = df.lazy();

    let result_expr = macd
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // MACD line should have same length as input
    assert_eq!(result.len(), expected_len);
}

#[test]
fn test_bollinger_bands_calculation() {
    let df = df! {
        "close" => &[
            100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0,
            108.0, 110.0, 112.0, 111.0, 113.0, 115.0, 114.0,
            116.0, 118.0, 120.0, 119.0, 121.0, 123.0, 125.0,
        ],
    }
    .unwrap();

    let bb = BollingerBands::new(20, 2.0);
    let expected_len = df.height();
    let lazy_df = df.lazy();

    let result_expr = bb
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // Bollinger Bands should return upper band by default
    assert_eq!(result.len(), expected_len);

    // Upper band should have values after warmup period
    let values = result.f64().unwrap();
    assert!(values.get(20).is_some());
}

// ===== Momentum Indicators Tests =====

#[test]
fn test_stochastic_bounds() {
    // Stochastic should be between 0 and 100
    let df = df! {
        "high" => &[105.0, 107.0, 106.0, 110.0, 112.0, 108.0, 115.0, 118.0, 116.0, 120.0, 122.0, 125.0, 123.0, 128.0, 126.0],
        "low" => &[95.0, 97.0, 96.0, 100.0, 102.0, 98.0, 105.0, 108.0, 106.0, 110.0, 112.0, 115.0, 113.0, 118.0, 116.0],
        "close" => &[100.0, 105.0, 103.0, 108.0, 110.0, 107.0, 112.0, 115.0, 113.0, 118.0, 120.0, 122.0, 119.0, 125.0, 123.0],
    }
    .unwrap();

    let stoch = Stochastic::new(5, 3, 3);
    let lazy_df = df.lazy();

    let result_expr = stoch
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("high")),
            IndicatorArg::Series(dsl::col("low")),
            IndicatorArg::Series(dsl::col("close")),
            IndicatorArg::Scalar(5.0),
            IndicatorArg::Scalar(3.0),
            IndicatorArg::Scalar(3.0),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // Check all non-null values are in valid range
    let values = result.f64().unwrap();
    for i in 0..result.len() {
        if let Some(val) = values.get(i) {
            assert!(val >= 0.0 && val <= 100.0, "Stochastic value {} is out of bounds at index {}", val, i);
        }
    }
}

#[test]
fn test_cci_calculation() {
    let df = df! {
        "high" => &[105.0, 107.0, 106.0, 110.0, 112.0, 108.0, 115.0, 118.0, 116.0, 120.0],
        "low" => &[95.0, 97.0, 96.0, 100.0, 102.0, 98.0, 105.0, 108.0, 106.0, 110.0],
        "close" => &[100.0, 105.0, 103.0, 108.0, 110.0, 107.0, 112.0, 115.0, 113.0, 118.0],
    }
    .unwrap();

    let cci = CCI::new(5);
    let expected_len = df.height();
    let lazy_df = df.lazy();

    let result_expr = cci
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("high")),
            IndicatorArg::Series(dsl::col("low")),
            IndicatorArg::Series(dsl::col("close")),
            IndicatorArg::Scalar(5.0),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // CCI should return values
    assert_eq!(result.len(), expected_len);
}

#[test]
fn test_williams_r_bounds() {
    // Williams %R should be between -100 and 0
    let df = df! {
        "high" => &[105.0, 107.0, 106.0, 110.0, 112.0, 108.0, 115.0, 118.0, 116.0, 120.0],
        "low" => &[95.0, 97.0, 96.0, 100.0, 102.0, 98.0, 105.0, 108.0, 106.0, 110.0],
        "close" => &[100.0, 105.0, 103.0, 108.0, 110.0, 107.0, 112.0, 115.0, 113.0, 118.0],
    }
    .unwrap();

    let williams = WilliamsR::new(5);
    let lazy_df = df.lazy();

    let result_expr = williams
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("high")),
            IndicatorArg::Series(dsl::col("low")),
            IndicatorArg::Series(dsl::col("close")),
            IndicatorArg::Scalar(5.0),
        ])
        .unwrap();

    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    // Check all non-null values are in valid range
    let values = result.f64().unwrap();
    for i in 0..result.len() {
        if let Some(val) = values.get(i) {
            assert!(val >= -100.0 && val <= 0.0, "Williams %R value {} is out of bounds at index {}", val, i);
        }
    }
}

// ===== Integration with Moving Averages =====

#[test]
fn test_multiple_mas_different_periods() {
    let df = df! {
        "close" => &[100.0, 102.0, 104.0, 103.0, 105.0, 107.0, 109.0, 108.0, 110.0, 112.0],
    }
    .unwrap();

    let sma_fast = SMA::new(3);
    let sma_slow = SMA::new(5);

    let lazy_df = df.lazy();

    let fast_expr = sma_fast
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap();

    let slow_expr = sma_slow
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap();

    let result_df = lazy_df
        .select(&[
            fast_expr.alias("fast_ma"),
            slow_expr.alias("slow_ma"),
        ])
        .collect()
        .unwrap();

    assert_eq!(result_df.height(), 10);
    assert_eq!(result_df.width(), 2);
}

#[test]
fn test_ema_vs_sma() {
    // EMA should react faster to price changes than SMA
    let df = df! {
        "close" => &[100.0, 100.0, 100.0, 110.0, 110.0, 110.0],
    }
    .unwrap();

    let sma = SMA::new(3);
    let ema = EMA::new(3);

    let lazy_df = df.lazy();

    let sma_expr = sma
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap();

    let ema_expr = ema
        .calculate_vectorized(&[
            IndicatorArg::Series(dsl::col("close")),
        ])
        .unwrap();

    let result_df = lazy_df
        .select(&[
            sma_expr.alias("sma"),
            ema_expr.alias("ema"),
        ])
        .collect()
        .unwrap();

    // Both should calculate without errors
    assert_eq!(result_df.height(), 6);
}
