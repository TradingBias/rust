use crate::error::TradebiasError;
use serde::{Deserialize, Serialize};

/// Trait for configuration sections
pub trait ConfigSection: Serialize + for<'de> Deserialize<'de> + Default + Clone {
    fn section_name() -> &'static str;
    fn validate(&self) -> Result<(), TradebiasError>;
    fn to_manifest(&self) -> ConfigManifest;
}

/// Configuration manifest for UI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigManifest {
    pub section: String,
    pub fields: Vec<FieldManifest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldManifest {
    pub name: String,
    pub field_type: String,
    pub default: serde_json::Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub description: String,
}
