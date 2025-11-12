# Phase 3: Additional Features

**Goal**: Complete all remaining features for 100% implementation coverage

**Priority**: MEDIUM - Enhances functionality but not critical for core operation

**Prerequisites**: Phases 1 and 2 must be complete

## Task Breakdown

### 1. Tier 2 Indicators Implementation (20 indicators)

**Status**: Not Started
**Spec**: `docs/ai-implementation/05-indicators-tier2.md`

All Tier 2 indicators follow the same pattern as Tier 1. Each implements either `VectorizedIndicator` or `StatefulIndicator` trait.

#### 1.1 Momentum Indicators (7 indicators)
**File**: `src/functions/momentum.rs`

Add these indicators using the vectorized pattern:

```rust
// 1. Williams %R
pub struct WilliamsR;

impl Indicator for WilliamsR {
    fn alias(&self) -> &'static str { "WilliamsR" }
    fn ui_name(&self) -> &'static str { "Williams Percent Range" }
    fn scale_type(&self) -> ScaleType { ScaleType::Oscillator0_100 }
    fn value_range(&self) -> Option<(f64, f64)> { Some((-100.0, 0.0)) }
    fn arity(&self) -> usize { 4 }
    fn input_types(&self) -> Vec<DataType> {
        vec![
            DataType::NumericSeries,  // high
            DataType::NumericSeries,  // low
            DataType::NumericSeries,  // close
            DataType::Integer,        // period
        ]
    }
    fn calculation_mode(&self) -> CalculationMode { CalculationMode::Vectorized }
    fn generate_mql5(&self, args: &[String]) -> String {
        format!("iWPR(_Symbol, _Period, {}, {})", args[3], "{}")
    }
}

impl VectorizedIndicator for WilliamsR {
    fn calculate_vectorized(&self, args: &[IndicatorArg]) -> Result<Expr> {
        let high = args[0].as_series()?;
        let low = args[1].as_series()?;
        let close = args[2].as_series()?;
        let period = args[3].as_scalar()? as usize;

        let hh = high.clone().rolling_max(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        });

        let ll = low.clone().rolling_min(RollingOptionsFixedWindow {
            window_size: period,
            min_periods: period,
            ..Default::default()
        });

        Ok((lit(-100.0) * (hh.clone() - close) / (hh - ll)))
    }
}

// 2. MFI (Money Flow Index)
pub struct MFI;
// Implementation following vectorized pattern
// Formula: 100 - (100 / (1 + Money Flow Ratio))

// 3. ROC (Rate of Change)
pub struct ROC;
// Implementation: ((close - close[n]) / close[n]) * 100

// 4. DeMarker
pub struct DeMarker;
// Implementation: SMA(DeMax, period) / (SMA(DeMax, period) + SMA(DeMin, period))

// 5. RVI (Relative Vigor Index)
pub struct RVI;
// Implementation: SMA((close - open), period) / SMA((high - low), period)

// 6. Force Index
pub struct Force;
// Implementation: volume * (close - close[1])

// 7. TriX (Triple Exponential Average)
pub struct TriX;
// Implementation: EMA(EMA(EMA(close, period)))
```

**Implementation Guide**:
- Follow the pattern from `docs/ai-implementation/05-indicators-tier2.md`
- Use vectorized calculations with Polars Expr
- Provide proper metadata (alias, ui_name, scale_type, value_range)
- Implement generate_mql5() for MetaTrader compatibility

#### 1.2 Trend Indicators (6 indicators)
**File**: `src/functions/trend.rs`

```rust
// 1. DEMA (Double Exponential Moving Average)
pub struct DEMA;
// Implementation: 2*EMA(close, period) - EMA(EMA(close, period), period)

// 2. TEMA (Triple Exponential Moving Average)
pub struct TEMA;
// Implementation: 3*EMA - 3*EMA(EMA) + EMA(EMA(EMA))

// 3. Envelopes
pub struct Envelopes;
// Implementation: MA ± (MA * deviation%)

// 4. SAR (Parabolic SAR) - STATEFUL
pub struct SAR;
// This requires stateful implementation due to complex conditional logic
impl StatefulIndicator for SAR {
    // Track: long/short position, extreme point, acceleration factor
}

// 5. Bulls Power
pub struct Bulls;
// Implementation: High - EMA(close, period)

// 6. Bears Power
pub struct Bears;
// Implementation: Low - EMA(close, period)
```

**Note**: SAR is stateful and requires careful implementation with state tracking.

#### 1.3 Volatility Indicators (2 indicators)
**File**: `src/functions/volatility.rs`

```rust
// 1. StdDev (Standard Deviation)
pub struct StdDev;
// Already exists as primitive - may need to add as indicator wrapper

// 2. Chaikin Volatility
pub struct Chaikin;
// Implementation: ROC(EMA(high - low, period), period)
```

#### 1.4 Volume Indicators (5 indicators)
**File**: `src/functions/volume.rs`

```rust
// 1. Volumes
pub struct Volumes;
// Implementation: Simple volume aggregation/analysis

// 2. BWMFI (Bill Williams Market Facilitation Index)
pub struct BWMFI;
// Implementation: (high - low) / volume

// 3. AC (Acceleration/Deceleration)
pub struct AC;
// Implementation: AO - SMA(AO, 5)

// 4. AO (Awesome Oscillator)
pub struct AO;
// Implementation: SMA(median_price, 5) - SMA(median_price, 34)

// 5. Momentum (Volume-based)
pub struct VolumeMomentum;
// Implementation: volume - volume[n]
```

#### 1.5 Register All Tier 2 Indicators
**File**: `src/functions/registry.rs`

Add all 20 new indicators to the registry:

```rust
impl FunctionRegistry {
    pub fn with_tier2_indicators(mut self) -> Self {
        // Momentum
        self.register_indicator(Arc::new(WilliamsR));
        self.register_indicator(Arc::new(MFI));
        self.register_indicator(Arc::new(ROC));
        self.register_indicator(Arc::new(DeMarker));
        self.register_indicator(Arc::new(RVI));
        self.register_indicator(Arc::new(Force));
        self.register_indicator(Arc::new(TriX));

        // Trend
        self.register_indicator(Arc::new(DEMA));
        self.register_indicator(Arc::new(TEMA));
        self.register_indicator(Arc::new(Envelopes));
        self.register_indicator(Arc::new(SAR));
        self.register_indicator(Arc::new(Bulls));
        self.register_indicator(Arc::new(Bears));

        // Volatility
        self.register_indicator(Arc::new(StdDev));
        self.register_indicator(Arc::new(Chaikin));

        // Volume
        self.register_indicator(Arc::new(Volumes));
        self.register_indicator(Arc::new(BWMFI));
        self.register_indicator(Arc::new(AC));
        self.register_indicator(Arc::new(AO));
        self.register_indicator(Arc::new(VolumeMomentum));

        self
    }
}
```

### 2. Data Connectors Implementation

**Status**: Partially Implemented (module structure only)
**Spec**: `docs/ai-implementation/18-data-connectors.md`

#### 2.1 Create Base Connector Trait
**File**: `src/data/connectors/base.rs` (if not exists)

```rust
use crate::error::TradebiasError;
use polars::prelude::*;
use async_trait::async_trait;

#[async_trait]
pub trait DataConnector: Send + Sync {
    /// Connector name
    fn name(&self) -> &'static str;

    /// Fetch OHLCV data for a symbol and timeframe
    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        start: i64,  // Unix timestamp
        end: i64,
    ) -> Result<DataFrame, TradebiasError>;

    /// Check if connector is available/authenticated
    async fn health_check(&self) -> Result<(), TradebiasError>;
}

#[derive(Debug, Clone, Copy)]
pub enum Timeframe {
    M1,
    M5,
    M15,
    M30,
    H1,
    H4,
    D1,
    W1,
    MN1,
}
```

#### 2.2 Implement CSV Connector
**File**: `src/data/connectors/csv.rs`

```rust
use super::base::{DataConnector, Timeframe};
use crate::error::TradebiasError;
use async_trait::async_trait;
use polars::prelude::*;
use std::path::PathBuf;

pub struct CsvConnector {
    base_path: PathBuf,
}

impl CsvConnector {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
}

#[async_trait]
impl DataConnector for CsvConnector {
    fn name(&self) -> &'static str {
        "CSV"
    }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        start: i64,
        end: i64,
    ) -> Result<DataFrame, TradebiasError> {
        let file_path = self.base_path.join(format!("{}_{:?}.csv", symbol, timeframe));

        let df = CsvReader::from_path(file_path)
            .map_err(|e| TradebiasError::Data(format!("CSV read error: {}", e)))?
            .has_header(true)
            .finish()
            .map_err(|e| TradebiasError::Data(format!("CSV parse error: {}", e)))?;

        // Filter by timestamp range
        // Assumes 'time' column exists
        Ok(df)
    }

    async fn health_check(&self) -> Result<(), TradebiasError> {
        if !self.base_path.exists() {
            return Err(TradebiasError::Data(
                format!("CSV base path does not exist: {:?}", self.base_path)
            ));
        }
        Ok(())
    }
}
```

#### 2.3 Implement MT5 Connector Stub
**File**: `src/data/connectors/mt5.rs`

```rust
use super::base::{DataConnector, Timeframe};
use crate::error::TradebiasError;
use async_trait::async_trait;
use polars::prelude::*;

pub struct MT5Connector {
    // Connection details
    host: String,
    port: u16,
}

#[async_trait]
impl DataConnector for MT5Connector {
    fn name(&self) -> &'static str {
        "MetaTrader 5"
    }

    async fn fetch_ohlcv(
        &self,
        symbol: &str,
        timeframe: Timeframe,
        start: i64,
        end: i64,
    ) -> Result<DataFrame, TradebiasError> {
        // TODO: Implement MT5 API integration
        Err(TradebiasError::Data("MT5 connector not implemented".to_string()))
    }

    async fn health_check(&self) -> Result<(), TradebiasError> {
        // TODO: Ping MT5 server
        Err(TradebiasError::Data("MT5 connector not implemented".to_string()))
    }
}
```

#### 2.4 Update Data Module
**File**: `src/data/mod.rs`

```rust
pub mod connectors;

pub use connectors::base::{DataConnector, Timeframe};
pub use connectors::csv::CsvConnector;
pub use connectors::mt5::MT5Connector;
```

#### 2.5 Add Data Error Variant
**File**: `src/error.rs`

```rust
// Add to TradebiasError enum:
Data(String),

// Add to Display impl:
TradebiasError::Data(msg) => write!(f, "Data error: {}", msg),
```

#### 2.6 Add async-trait Dependency
**File**: `Cargo.toml`

```toml
[dependencies]
async-trait = "0.1"
tokio = { version = "1.0", features = ["full"] }
```

### 3. ML Signal and Filtering Engines

**Status**: Partially Implemented (directory structure only)
**Spec**: `docs/ai-implementation/19-calibration-signal-engines.md`

#### 3.1 Implement Signal Generator Base
**File**: `src/ml/signals/base.rs`

```rust
use crate::error::TradebiasError;
use polars::prelude::*;

pub trait SignalGenerator: Send + Sync {
    fn name(&self) -> &'static str;

    fn generate_signals(
        &self,
        predictions: &DataFrame,
        threshold: f64,
    ) -> Result<DataFrame, TradebiasError>;
}
```

#### 3.2 Implement Meta-Labeling Signal
**File**: `src/ml/signals/meta_labeling.rs`

```rust
use super::base::SignalGenerator;
use crate::error::TradebiasError;
use polars::prelude::*;

pub struct MetaLabelingSignal {
    primary_model_threshold: f64,
}

impl SignalGenerator for MetaLabelingSignal {
    fn name(&self) -> &'static str {
        "Meta-Labeling"
    }

    fn generate_signals(
        &self,
        predictions: &DataFrame,
        threshold: f64,
    ) -> Result<DataFrame, TradebiasError> {
        // Use meta-model predictions to filter primary model signals
        // Only take trades where meta-model predicts high probability of success
        todo!()
    }
}
```

#### 3.3 Implement Filtering System Base
**File**: `src/ml/filtering/base.rs`

```rust
use crate::error::TradebiasError;
use polars::prelude::*;

pub trait SignalFilter: Send + Sync {
    fn name(&self) -> &'static str;

    fn filter_signals(
        &self,
        signals: &DataFrame,
        market_data: &DataFrame,
    ) -> Result<DataFrame, TradebiasError>;
}
```

#### 3.4 Implement Volatility Filter
**File**: `src/ml/filtering/volatility.rs`

```rust
use super::base::SignalFilter;
use crate::error::TradebiasError;
use polars::prelude::*;

pub struct VolatilityFilter {
    min_volatility: f64,
    max_volatility: f64,
}

impl SignalFilter for VolatilityFilter {
    fn name(&self) -> &'static str {
        "Volatility Filter"
    }

    fn filter_signals(
        &self,
        signals: &DataFrame,
        market_data: &DataFrame,
    ) -> Result<DataFrame, TradebiasError> {
        // Filter out signals in extreme volatility regimes
        todo!()
    }
}
```

#### 3.5 Update ML Module Structure
**File**: `src/ml/signals/mod.rs`

```rust
pub mod base;
pub mod meta_labeling;

pub use base::SignalGenerator;
pub use meta_labeling::MetaLabelingSignal;
```

**File**: `src/ml/filtering/mod.rs`

```rust
pub mod base;
pub mod volatility;

pub use base::SignalFilter;
pub use volatility::VolatilityFilter;
```

**File**: `src/ml/mod.rs`

```rust
pub mod features;
pub mod labeling;
pub mod models;
pub mod signals;
pub mod filtering;
```

## Implementation Order

1. **Tier 2 Indicators** (Tasks 1.1-1.5) - Extends functionality
2. **Data Connectors** (Tasks 2.1-2.6) - Essential for data input
3. **Signal/Filtering Engines** (Tasks 3.1-3.5) - ML pipeline completion

## Success Criteria

After Phase 3 completion:
- [ ] All 20 Tier 2 indicators implemented and registered
- [ ] CSV data connector fully functional
- [ ] MT5 connector stub in place for future implementation
- [ ] Signal generation and filtering framework implemented
- [ ] All modules in `docs/ai-implementation` marked "Implemented"
- [ ] `cargo build` still succeeds with all features

## Notes

- Tier 2 indicators can be implemented in parallel by different developers
- Data connectors should start with CSV (simplest) before attempting MT5
- ML signal engines may need refinement based on actual model outputs
- Consider adding comprehensive tests after all features are complete
- Some implementations (MT5, complex signals) may be left as stubs initially

## Future Enhancements (Post Phase 3)

- Additional data connectors (Binance, Interactive Brokers, etc.)
- More sophisticated signal filters
- Real-time data streaming support
- Advanced meta-labeling techniques
- Ensemble model support
