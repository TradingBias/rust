use tradebias::functions::registry::FunctionRegistry;

// Note: Indicator execution tests are disabled because indicators haven't implemented
// VectorizedIndicator trait yet. These tests verify registry population only.

#[test]
fn test_registry_has_all_momentum_indicators() {
    let registry = FunctionRegistry::new();
    let momentum_indicators = vec!["RSI", "Stochastic", "CCI", "WilliamsR", "Momentum", "AC", "AO", "RVI", "DeMarker"];

    for indicator in momentum_indicators {
        assert!(
            registry.get_indicator(indicator).is_some(),
            "{} should be registered",
            indicator
        );
    }
}

#[test]
fn test_registry_has_all_trend_indicators() {
    let registry = FunctionRegistry::new();
    let trend_indicators = vec!["SMA", "EMA", "MACD", "BB", "Envelopes", "SAR", "Bears", "Bulls", "DEMA", "TEMA", "TriX"];

    for indicator in trend_indicators {
        assert!(
            registry.get_indicator(indicator).is_some(),
            "{} should be registered",
            indicator
        );
    }
}

#[test]
fn test_registry_has_all_volatility_indicators() {
    let registry = FunctionRegistry::new();
    let volatility_indicators = vec!["ATR", "ADX", "StdDev"];

    for indicator in volatility_indicators {
        assert!(
            registry.get_indicator(indicator).is_some(),
            "{} should be registered",
            indicator
        );
    }
}

#[test]
fn test_registry_has_all_volume_indicators() {
    let registry = FunctionRegistry::new();
    let volume_indicators = vec!["OBV", "MFI", "Force", "Volumes", "Chaikin", "BWMFI"];

    for indicator in volume_indicators {
        assert!(
            registry.get_indicator(indicator).is_some(),
            "{} should be registered",
            indicator
        );
    }
}

#[test]
fn test_registry_has_all_primitives() {
    let registry = FunctionRegistry::new();
    let primitives = vec!["And", "Or", "Abs"];

    for primitive in primitives {
        assert!(
            registry.get_primitive(primitive).is_some(),
            "{} should be registered",
            primitive
        );
    }
}
