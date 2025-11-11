use crate::{
    error::{Result, TradebiasError},
    types::{Direction, ExitReason, Trade},
};

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
            self.open_position(bar, signal, price)?;
        } else if self.position.is_some() {
            self.check_exit(bar, signal, price)?;
        }

        self.update_equity(price);
        Ok(())
    }

    fn open_position(&mut self, bar: usize, signal: f64, price: f64) -> Result<()> {
        let direction = if signal > 0.0 {
            Direction::Long
        } else {
            Direction::Short
        };
        let size = self.balance * 0.1 / price;

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
