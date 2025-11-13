use tradebias::engines::evaluation::Portfolio;

#[test]
fn test_unrealized_pnl_long_position() {
    let mut portfolio = Portfolio::new(10000.0);

    // With 10% of capital ($1000) at $50/share, we buy 20 shares.
    portfolio.open_position(0, 1.0, 50.0).unwrap();

    // Price increases to $55
    portfolio.calculate_unrealized_pnl(55.0);

    // Unrealized P&L should be (55-50) * 20 = $100
    assert_eq!(portfolio.unrealized_pnl, 100.0);
    assert_eq!(portfolio.total_pnl, 100.0);
}

#[test]
fn test_portfolio_value_with_open_positions() {
    let mut portfolio = Portfolio::new(10000.0);

    // Buy 20 shares at $50 (costs $1000)
    portfolio.open_position(0, 1.0, 50.0).unwrap();
    // Cash remaining: $9000

    // Price increases to $60
    portfolio.calculate_unrealized_pnl(60.0);

    // Position value: 20 * $60 = $1200
    // Total value: $9000 cash + $1200 position value = $10200
    assert_eq!(portfolio.total_value(), 10200.0);
}

#[test]
fn test_drawdown_with_unrealized_losses() {
    let mut portfolio = Portfolio::new(10000.0);

    // Buy 10 shares at $100 (costs $1000)
    portfolio.open_position(0, 1.0, 100.0).unwrap();

    // Price drops to $80
    portfolio.process_bar(1, 0.0, 80.0).unwrap();

    // Unrealized loss: (80-100) * 10 = -$200
    // Equity: $10000 - $200 = $9800
    // Drawdown: (10000 - 9800) / 10000 = 0.02
    assert_eq!(portfolio.unrealized_pnl, -200.0);
    assert_eq!(portfolio.current_drawdown, 0.02);
}

#[test]
fn test_close_position_updates_pnl() {
    let mut portfolio = Portfolio::new(10000.0);

    // Buy 10 shares at $100
    portfolio.open_position(0, 1.0, 100.0).unwrap();

    // Close position at $110
    portfolio.close_position(1, 110.0, tradebias::types::ExitReason::Signal).unwrap();

    // Realized P&L should be (110 - 100) * 10 = $100
    assert_eq!(portfolio.realized_pnl, 100.0);
    assert_eq!(portfolio.cash, 10100.0);
}
