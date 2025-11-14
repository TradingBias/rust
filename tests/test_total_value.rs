use tradebias::engines::evaluation::Portfolio;

#[test]
fn test_total_value_long_position() {
    let mut portfolio = Portfolio::new(10000.0);

    // Open long position: Buy at $100
    portfolio.process_bar(0, 1.0, 100.0).unwrap();

    // Position size = (10000 * 0.1) / 100 = 10 shares
    // Cash spent = 10 * 100 = 1000
    // Cash remaining = 9000

    // Price increases to $110
    portfolio.process_bar(1, 0.0, 110.0).unwrap();

    // Position value = 10 * 110 = 1100
    // Total value should = 9000 (cash) + 1100 (position) = 10100
    // Unrealized P&L = (110 - 100) * 10 = 100
    // Equity = 10000 + 100 = 10100

    let total_value = portfolio.total_value();
    let equity = portfolio.equity();

    println!("Long position:");
    println!("  Cash: {}", portfolio.cash);
    println!("  Position value: {}", portfolio.current_position_value);
    println!("  Total value: {}", total_value);
    println!("  Unrealized P&L: {}", portfolio.unrealized_pnl);
    println!("  Equity: {}", equity);

    // Both methods should give the same result
    assert_eq!(equity, 10100.0, "Equity should be 10100");
    assert_eq!(total_value, equity, "Total value should equal equity");
}

#[test]
fn test_total_value_short_position() {
    let mut portfolio = Portfolio::new(10000.0);

    // Open short position: Short at $100
    portfolio.process_bar(0, -1.0, 100.0).unwrap();

    // Position size = (10000 * 0.1) / 100 = 10 shares
    // Cash received from short = 10 * 100 = 1000
    // Cash after short = 11000

    // Price decreases to $90 (profitable for short)
    portfolio.process_bar(1, 0.0, 90.0).unwrap();

    // Current liability = 10 * 90 = 900 (we owe 10 shares worth $90 each)
    // Total value should = 11000 (cash) - 900 (liability) = 10100
    // Unrealized P&L = (100 - 90) * 10 = 100
    // Equity = 10000 + 100 = 10100

    let total_value = portfolio.total_value();
    let equity = portfolio.equity();

    println!("\nShort position:");
    println!("  Cash: {}", portfolio.cash);
    println!("  Position value: {}", portfolio.current_position_value);
    println!("  Total value: {}", total_value);
    println!("  Unrealized P&L: {}", portfolio.unrealized_pnl);
    println!("  Equity: {}", equity);

    // Both methods should give the same result
    assert_eq!(equity, 10100.0, "Equity should be 10100");
    assert_eq!(total_value, equity, "Total value should equal equity");
}

#[test]
fn test_total_value_short_position_losing() {
    let mut portfolio = Portfolio::new(10000.0);

    // Open short position: Short at $100
    portfolio.process_bar(0, -1.0, 100.0).unwrap();

    // Price increases to $110 (losing for short)
    portfolio.process_bar(1, 0.0, 110.0).unwrap();

    // Current liability = 10 * 110 = 1100 (we owe 10 shares worth $110 each)
    // Total value should = 11000 (cash) - 1100 (liability) = 9900
    // Unrealized P&L = (100 - 110) * 10 = -100
    // Equity = 10000 + (-100) = 9900

    let total_value = portfolio.total_value();
    let equity = portfolio.equity();

    println!("\nShort position (losing):");
    println!("  Cash: {}", portfolio.cash);
    println!("  Position value: {}", portfolio.current_position_value);
    println!("  Total value: {}", total_value);
    println!("  Unrealized P&L: {}", portfolio.unrealized_pnl);
    println!("  Equity: {}", equity);

    // Both methods should give the same result
    assert_eq!(equity, 9900.0, "Equity should be 9900");
    assert_eq!(total_value, equity, "Total value should equal equity");
}
