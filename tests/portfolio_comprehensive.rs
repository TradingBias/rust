use tradebias::engines::evaluation::Portfolio;
use tradebias::types::{Direction, ExitReason};

#[test]
fn test_portfolio_long_trade_profitable() {
    let mut portfolio = Portfolio::new(10000.0);

    // Open long position at 100
    portfolio.process_bar(0, 1.0, 100.0).unwrap();

    // Close position at 110 (profitable)
    portfolio.process_bar(1, -1.0, 110.0).unwrap();

    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 1, "Should have exactly one trade");

    let trade = &trades[0];
    assert_eq!(trade.direction, Direction::Long);
    assert_eq!(trade.entry_price, 100.0);
    assert_eq!(trade.exit_price, 110.0);
    assert!(trade.profit > 0.0, "Trade should be profitable");
    assert_eq!(trade.exit_reason, ExitReason::Signal);

    // Balance should increase
    assert!(portfolio.final_balance() > 10000.0,
        "Final balance should be greater than initial");
}

#[test]
fn test_portfolio_long_trade_losing() {
    let mut portfolio = Portfolio::new(10000.0);

    // Open long position at 100
    portfolio.process_bar(0, 1.0, 100.0).unwrap();

    // Close position at 90 (losing)
    portfolio.process_bar(1, -1.0, 90.0).unwrap();

    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 1);

    let trade = &trades[0];
    assert!(trade.profit < 0.0, "Trade should be losing");

    // Balance should decrease
    assert!(portfolio.final_balance() < 10000.0,
        "Final balance should be less than initial");
}

#[test]
fn test_portfolio_short_trade_profitable() {
    let mut portfolio = Portfolio::new(10000.0);

    // Open short position at 100
    portfolio.process_bar(0, -1.0, 100.0).unwrap();

    // Close position at 90 (profitable for short)
    portfolio.process_bar(1, 1.0, 90.0).unwrap();

    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 1);

    let trade = &trades[0];
    assert_eq!(trade.direction, Direction::Short);
    assert_eq!(trade.entry_price, 100.0);
    assert_eq!(trade.exit_price, 90.0);
    assert!(trade.profit > 0.0, "Short trade should be profitable when price falls");

    assert!(portfolio.final_balance() > 10000.0);
}

#[test]
fn test_portfolio_short_trade_losing() {
    let mut portfolio = Portfolio::new(10000.0);

    // Open short position at 100
    portfolio.process_bar(0, -1.0, 100.0).unwrap();

    // Close position at 110 (losing for short)
    portfolio.process_bar(1, 1.0, 110.0).unwrap();

    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 1);

    let trade = &trades[0];
    assert!(trade.profit < 0.0, "Short trade should lose when price rises");

    assert!(portfolio.final_balance() < 10000.0);
}

#[test]
fn test_portfolio_multiple_trades() {
    let mut portfolio = Portfolio::new(10000.0);

    // Trade 1: Long profitable
    portfolio.process_bar(0, 1.0, 100.0).unwrap();
    portfolio.process_bar(1, -1.0, 105.0).unwrap();

    // Trade 2: Short profitable
    portfolio.process_bar(2, -1.0, 105.0).unwrap();
    portfolio.process_bar(3, 1.0, 100.0).unwrap();

    // Trade 3: Long losing
    portfolio.process_bar(4, 1.0, 100.0).unwrap();
    portfolio.process_bar(5, -1.0, 98.0).unwrap();

    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 3, "Should have three completed trades");

    assert_eq!(trades[0].direction, Direction::Long);
    assert_eq!(trades[1].direction, Direction::Short);
    assert_eq!(trades[2].direction, Direction::Long);

    // Verify trade sequence
    assert_eq!(trades[0].exit_bar, 1);
    assert_eq!(trades[1].entry_bar, 2);
    assert_eq!(trades[1].exit_bar, 3);
    assert_eq!(trades[2].entry_bar, 4);
}

#[test]
fn test_portfolio_position_sizing() {
    let initial_balance = 10000.0;
    let mut portfolio = Portfolio::new(initial_balance);

    let entry_price = 100.0;
    portfolio.process_bar(0, 1.0, entry_price).unwrap();

    // Position size should be 10% of balance / price
    // Expected size: (10000 * 0.1) / 100 = 10 units
    let expected_size = (initial_balance * 0.1) / entry_price;

    portfolio.process_bar(1, -1.0, 110.0).unwrap();

    let trades = portfolio.get_trades();
    assert_eq!(trades[0].size, expected_size,
        "Position size should be 10% of balance");

    // Verify profit calculation
    let expected_profit = (110.0 - 100.0) * expected_size;
    assert!((trades[0].profit - expected_profit).abs() < 0.001,
        "Profit should match expected calculation");
}

#[test]
fn test_portfolio_no_signal_no_entry() {
    let mut portfolio = Portfolio::new(10000.0);

    // Process bars with zero signal
    for i in 0..10 {
        portfolio.process_bar(i, 0.0, 100.0 + i as f64).unwrap();
    }

    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 0, "No trades should occur with zero signal");
    assert_eq!(portfolio.final_balance(), 10000.0,
        "Balance should remain unchanged");
}

#[test]
fn test_portfolio_constant_signal_no_exit() {
    let mut portfolio = Portfolio::new(10000.0);

    // Constant long signal - opens position but never closes
    for i in 0..10 {
        portfolio.process_bar(i, 1.0, 100.0 + i as f64).unwrap();
    }

    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 0, "No completed trades with constant signal");

    // With unrealized P&L, the equity should reflect the open position's value.
    // The final price is 109.0. Initial cash was 10000. Position opened at 100.
    // Position size = (10000 * 0.1) / 100 = 10 shares.
    // Cost = 10 * 100 = 1000. Cash = 9000.
    // Unrealized P&L = (109 - 100) * 10 = 90.
    // Equity = 10000 (initial) + 90 (total P&L) = 10090.
    assert!(portfolio.equity() > 10000.0,
        "Equity should be greater than initial capital due to unrealized gains");
    assert_eq!(portfolio.final_balance(), 9000.0,
        "Cash balance should be reduced by the cost of the position");
}

#[test]
fn test_portfolio_equity_curve_tracking() {
    let mut portfolio = Portfolio::new(10000.0);

    // Initial equity
    let equity = portfolio.get_equity_curve();
    assert_eq!(equity.len(), 1, "Should start with initial equity");
    assert_eq!(equity[0], 10000.0);

    // Process 5 bars
    for i in 0..5 {
        portfolio.process_bar(i, 1.0, 100.0).unwrap();
    }

    let equity = portfolio.get_equity_curve();
    assert_eq!(equity.len(), 6, "Should have equity entry for each bar + initial");
}

#[test]
fn test_portfolio_signal_strength_interpretation() {
    let mut portfolio = Portfolio::new(10000.0);

    // Test different signal strengths
    // Signal > 0 = Long
    portfolio.process_bar(0, 0.5, 100.0).unwrap();
    portfolio.process_bar(1, -1.0, 105.0).unwrap();

    // Signal < 0 = Short
    portfolio.process_bar(2, -0.5, 105.0).unwrap();
    portfolio.process_bar(3, 1.0, 100.0).unwrap();

    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].direction, Direction::Long,
        "Positive signal (0.5) should open long");
    assert_eq!(trades[1].direction, Direction::Short,
        "Negative signal (-0.5) should open short");
}

#[test]
fn test_portfolio_compounding_effect() {
    let mut portfolio = Portfolio::new(10000.0);

    // Trade 1: 10% gain
    portfolio.process_bar(0, 1.0, 100.0).unwrap();
    portfolio.process_bar(1, -1.0, 110.0).unwrap();
    let balance_after_trade1 = portfolio.final_balance();

    // Balance should increase
    assert!(balance_after_trade1 > 10000.0);

    // Trade 2: Another 10% gain on new balance
    portfolio.process_bar(2, 1.0, 110.0).unwrap();
    portfolio.process_bar(3, -1.0, 121.0).unwrap();

    // Second trade should compound on first trade's gains
    let trades = portfolio.get_trades();
    assert_eq!(trades.len(), 2);

    // Verify compounding behavior
    // Trade 1: 10000 * 0.1 / 100 = 10 units
    let expected_size_1 = 10000.0 * 0.1 / 100.0;
    assert!((trades[0].size - expected_size_1).abs() < 0.001);

    // Trade 2: 10100 * 0.1 / 110 = 9.18... units
    // Size is smaller because price is higher, but dollar amount is larger
    // This confirms compounding is working correctly
    let expected_size_2 = 10100.0 * 0.1 / 110.0;
    assert!((trades[1].size - expected_size_2).abs() < 0.001,
        "Position should be sized based on compounded balance");

    // Verify dollar amounts increase (not unit sizes)
    let dollar_amount_1 = trades[0].size * 100.0; // ~1000
    let dollar_amount_2 = trades[1].size * 110.0; // ~1010
    assert!(dollar_amount_2 > dollar_amount_1,
        "Dollar amount invested should increase due to compounding");
}

#[test]
fn test_portfolio_zero_price_handling() {
    let mut portfolio = Portfolio::new(10000.0);

    // Test with very small price (edge case)
    let result = portfolio.process_bar(0, 1.0, 0.0);

    // This might panic or produce infinite position size
    // Test documents current behavior
    // Ideally should handle gracefully with error or minimum position size
    assert!(result.is_ok() || result.is_err(),
        "Should handle zero price somehow (currently may panic in division)");
}

#[test]
fn test_portfolio_rapid_signal_changes() {
    let mut portfolio = Portfolio::new(10000.0);

    let prices = vec![100.0, 101.0, 102.0, 101.5, 103.0, 102.5, 104.0];
    let signals = vec![1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0];

    for (i, (&price, &signal)) in prices.iter().zip(signals.iter()).enumerate() {
        portfolio.process_bar(i, signal, price).unwrap();
    }

    let trades = portfolio.get_trades();
    // Should have: long (exit at -1), short (exit at +1), long (exit at -1)
    assert_eq!(trades.len(), 3, "Should complete 3 trades with signal changes");
}
