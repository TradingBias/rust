use crate::types::ScaleType;

pub struct IndicatorManifest {
    pub tier1: Vec<ComposedIndicator>,
    pub tier2: Vec<ComposedIndicator>,
    pub tier3: Vec<ComposedIndicator>,
}

pub struct ComposedIndicator {
    pub alias: String,
    pub ui_name: String,
    pub scale_type: ScaleType,
    pub value_range: Option<(f64, f64)>,
    pub composition: CompositionRecipe,
    pub discovery_weight: f64,
}

pub enum CompositionRecipe {
    Primitives(Vec<String>), // Using String for primitive aliases
    Formula(String),
}

// Default Tier 1 indicators
pub const TIER1_INDICATORS: &[&str] = &[
    "SMA", "EMA", "RSI", "MACD", "BB", "ATR", "Stochastic", "ADX", "OBV", "CCI",
];

// Default Tier 2 indicators
pub const TIER2_INDICATORS: &[&str] = &[
    "WilliamsR", "MFI", "ROC", "DeMarker", "StdDev", "Envelopes", "SAR", "Force",
    "Bears", "Bulls", "Momentum", "DEMA", "TEMA", "RVI", "TriX", "Volumes",
    "Chaikin", "BWMFI", "AC", "AO",
];

impl IndicatorManifest {
    pub fn default() -> Self {
        Self {
            tier1: Self::build_tier1(),
            tier2: Self::build_tier2(),
            tier3: Vec::new(),
        }
    }

    fn build_tier1() -> Vec<ComposedIndicator> {
        // In a real scenario, we would have full metadata for each.
        // Here we create simplified versions.
        TIER1_INDICATORS.iter().map(|&alias| ComposedIndicator {
            alias: alias.to_string(),
            ui_name: alias.to_string(), // Simplified
            scale_type: ScaleType::Price, // Simplified
            value_range: None,
            composition: CompositionRecipe::Formula(format!("{} built-in", alias)),
            discovery_weight: 1.0,
        }).collect()
    }

    fn build_tier2() -> Vec<ComposedIndicator> {
        TIER2_INDICATORS.iter().map(|&alias| ComposedIndicator {
            alias: alias.to_string(),
            ui_name: alias.to_string(),
            scale_type: ScaleType::Price,
            value_range: None,
            composition: CompositionRecipe::Formula(format!("{} built-in", alias)),
            discovery_weight: 0.5,
        }).collect()
    }

    pub fn add_to_tier3(&mut self, indicator: ComposedIndicator) {
        self.tier3.push(indicator);
    }

    pub fn remove_from_tier3(&mut self, alias: &str) {
        self.tier3.retain(|ind| ind.alias != alias);
    }

    pub fn get_all_available(&self) -> Vec<&ComposedIndicator> {
        self.tier1.iter()
            .chain(self.tier2.iter())
            .chain(self.tier3.iter())
            .collect()
    }
}
