pub mod traits;
pub mod evolution;
pub mod backtesting;
pub mod trade_management;
pub mod ml;
pub mod manager;

pub use manager::{ConfigManager, AppConfig};
pub use evolution::EvolutionConfig;
pub use backtesting::BacktestingConfig;
pub use trade_management::TradeManagementConfig;
pub use ml::MLConfig;
