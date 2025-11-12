#[derive(Debug, Clone)]
pub struct LabelingConfig {
    pub profit_target_pct: f64,  // e.g., 0.02 = 2% profit target
    pub stop_loss_pct: f64,      // e.g., 0.01 = 1% stop loss
    pub time_limit_bars: usize,  // e.g., 10 bars maximum hold time
    pub use_atr_based: bool,     // Use ATR multiples instead of fixed percentages
    pub atr_profit_multiple: f64, // e.g., 2.0 = 2 * ATR for profit target
    pub atr_stop_multiple: f64,   // e.g., 1.0 = 1 * ATR for stop loss
}

impl Default for LabelingConfig {
    fn default() -> Self {
        Self {
            profit_target_pct: 0.02,
            stop_loss_pct: 0.01,
            time_limit_bars: 10,
            use_atr_based: false,
            atr_profit_multiple: 2.0,
            atr_stop_multiple: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Label {
    Profit = 1,   // Hit profit target
    Loss = -1,    // Hit stop loss
    Timeout = 0,  // Hit time limit
}

#[derive(Debug, Clone)]
pub struct LabeledSignal {
    pub signal_idx: usize,
    pub label: Label,
    pub bars_held: usize,
    pub return_pct: f64,
    pub hit_barrier: BarrierType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierType {
    Upper,    // Profit target
    Lower,    // Stop loss
    Vertical, // Time limit
}
