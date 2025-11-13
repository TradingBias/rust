use crate::{
    error::Result,
    types::{Direction, ExitReason, Trade},
};

pub struct Portfolio {
    pub initial_capital: f64,
    pub cash: f64,
    pub position: Option<Position>,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<f64>,

    // P&L and Drawdown Tracking
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_pnl: f64,
    pub current_position_value: f64,
    pub peak_equity: f64,
    pub max_drawdown: f64,
    pub current_drawdown: f64,
}

pub struct Position {
    pub direction: Direction,
    pub entry_bar: usize,
    pub entry_price: f64,
    pub size: f64,
}

impl Portfolio {
    pub fn new(initial_capital: f64) -> Self {
        Self {
            initial_capital,
            cash: initial_capital,
            position: None,
            trades: Vec::new(),
            equity_curve: vec![initial_capital],
            realized_pnl: 0.0,
            unrealized_pnl: 0.0,
            total_pnl: 0.0,
            current_position_value: 0.0,
            peak_equity: initial_capital,
            max_drawdown: 0.0,
            current_drawdown: 0.0,
        }
    }

    pub fn process_bar(&mut self, bar: usize, signal: f64, price: f64) -> Result<()> {
        if self.position.is_none() && signal != 0.0 {
            self.open_position(bar, signal, price)?;
        } else if self.position.is_some() {
            self.check_exit(bar, signal, price)?;
        }

        // Calculate unrealized P&L with the current price
        self.calculate_unrealized_pnl(price);

        // Update drawdown with the current equity
        self.update_drawdown();

        // Update the equity curve
        self.equity_curve.push(self.equity());

        Ok(())
    }

    pub fn open_position(&mut self, bar: usize, signal: f64, price: f64) -> Result<()> {
        let direction = if signal > 0.0 {
            Direction::Long
        } else {
            Direction::Short
        };
        let quantity = (self.cash * 0.1) / price;

        match direction {
            Direction::Long => self.cash -= quantity * price,
            Direction::Short => self.cash += quantity * price, // Add proceeds from short sale
        }

        self.position = Some(Position {
            direction,
            entry_bar: bar,
            entry_price: price,
            size: quantity,
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

    pub fn close_position(&mut self, bar: usize, price: f64, reason: ExitReason) -> Result<()> {
        if let Some(pos) = self.position.take() {
            let profit = match pos.direction {
                Direction::Long => (price - pos.entry_price) * pos.size,
                Direction::Short => (pos.entry_price - price) * pos.size,
            };

            match pos.direction {
                Direction::Long => self.cash += price * pos.size,
                Direction::Short => self.cash -= price * pos.size, // Deduct cost to buy back shares
            }
            self.realized_pnl += profit;

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

    pub fn get_trades(&self) -> &[Trade] {
        &self.trades
    }

    pub fn get_equity_curve(&self) -> &[f64] {
        &self.equity_curve
    }

    pub fn final_balance(&self) -> f64 {
        self.cash
    }

    // P&L and Drawdown Calculation Methods

    /// Calculate unrealized P&L for the open position.
    pub fn calculate_unrealized_pnl(&mut self, current_price: f64) {
        if let Some(position) = &self.position {
            let entry_value = position.size * position.entry_price;
            let current_value = position.size * current_price;

            let pnl = match position.direction {
                Direction::Long => current_value - entry_value,
                Direction::Short => entry_value - current_value,
            };

            self.unrealized_pnl = pnl;
            self.current_position_value = current_value;
        } else {
            self.unrealized_pnl = 0.0;
            self.current_position_value = 0.0;
        }

        self.total_pnl = self.realized_pnl + self.unrealized_pnl;
    }

    /// Get total portfolio value (cash + position at current price).
    pub fn total_value(&self) -> f64 {
        self.cash + self.current_position_value
    }

    /// Get equity (initial capital + total P&L).
    pub fn equity(&self) -> f64 {
        self.initial_capital + self.total_pnl
    }

    /// Update drawdown based on the current equity.
    pub fn update_drawdown(&mut self) {
        let current_equity = self.equity();

        if current_equity > self.peak_equity {
            self.peak_equity = current_equity;
        }

        if self.peak_equity > 0.0 {
            self.current_drawdown = (self.peak_equity - current_equity) / self.peak_equity;

            if self.current_drawdown > self.max_drawdown {
                self.max_drawdown = self.current_drawdown;
            }
        }
    }
}
