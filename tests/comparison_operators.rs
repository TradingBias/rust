
use polars::prelude::*;
use polars::lazy::dsl;
use tradebias::functions::primitives::*;
use tradebias::functions::traits::Primitive;
use polars::df;

#[test]
fn test_series_greater_than() {
    let df = df! {
        "a" => &[1.0, 2.0, 3.0, 4.0],
        "b" => &[2.0, 2.0, 2.0, 2.0],
    }
    .unwrap();

    let lazy_df = df.lazy();
    let result_expr = GreaterThan
        .execute(&[dsl::col("a"), dsl::col("b")])
        .unwrap();
    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    let expected = Series::new("a".into(), &[false, false, true, true]);

    assert_eq!(result, expected.into());
}

#[test]
fn test_scalar_comparison() {
    let df = df! {
        "rsi" => &[30.0, 50.0, 70.0, 80.0],
    }
    .unwrap();

    let lazy_df = df.lazy();
    let result_expr = GreaterThanScalar
        .execute(&[dsl::col("rsi"), dsl::lit(70.0)])
        .unwrap();
    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    let expected = Series::new("rsi".into(), &[false, false, false, true]);

    assert_eq!(result, expected.into());
}

#[test]
fn test_cross_above() {
    let df = df! {
        "fast" => &[1.0, 2.0, 3.0, 4.0, 3.0],
        "slow" => &[2.0, 2.0, 2.0, 2.0, 2.0],
    }
    .unwrap();

    let lazy_df = df.lazy();
    let result_expr = CrossAbove
        .execute(&[dsl::col("fast"), dsl::col("slow")])
        .unwrap();
    let result_df = lazy_df.select(&[result_expr]).collect().unwrap();
    let result = result_df.get_columns()[0].clone();

    let expected = Series::new(
        "fast".into(),
        &[
            Some(false),
            Some(false),
            Some(true),
            Some(false),
            Some(false),
        ],
    );

    assert_eq!(result, expected.into());
}
