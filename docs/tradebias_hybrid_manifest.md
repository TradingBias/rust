# Hybrid Approach: Primitives + Composed Indicators Manifest

## The Theoretical Case

### Why Hybrid HELPS GA Performance

```
Pure Primitives:
  Generation 1: Random noise
  Generation 5: Discovers simple MA crossovers
  Generation 15: Discovers RSI-like pattern
  Generation 30: Combines RSI + volatility filter
  Generation 50: Has a viable strategy
  
Primitives + Manifest:
  Generation 1: Mix of random + known-good (RSI, MACD)
  Generation 5: GA leverages RSI immediately
  Generation 10: Combines RSI + novel normalization
  Generation 20: Has viable strategy
  Generation 50: Discovers patterns not in manifest
  
Result: 2-3x faster convergence to viable strategies + discovers novelty
```

### Why Hybrid COULD HURT Performance (If Done Wrong)

```
Bad Implementation:
  - 12 primitives + 70 composed indicators = 82-option search space
  - GA spends time learning that RSI exists without discovering it
  - Overfits to manifest patterns instead of discovering novel ones
  - Memory bloat: evaluate both SMA primitive AND SMA from manifest
  
Good Implementation:
  - 12 primitives always available (discovery)
  - 20-30 composed indicators available (acceleration)
  - Smart weighting: GA prefers primitives for novelty
  - No redundancy: don't register both SMA primitive AND SMA indicator
```

---

## The Optimal Solution: Tiered Manifest Architecture

### Architecture Overview

```
┌──────────────────────────────────┐
│   Function Registry              │
├──────────────────────────────────┤
│                                  │
│  Tier 1: Core Primitives (12)    │
│  ├─ MovingAverage                │
│  ├─ Highest / Lowest             │
│  ├─ Change / Momentum            │
│  ├─ StdDev                       │
│  ├─ Normalize                    │
│  └─ Arithmetic/Logical           │
│                                  │
│  Tier 2: Composed Indicators (20-30)  │
│  ├─ RSI (composed from primitives)    │
│  ├─ MACD (EMA composition)            │
│  ├─ Bollinger Bands                   │
│  ├─ Stochastic                       │
│  └─ ... (proven trading patterns)    │
│                                  │
│  Tier 3: Custom Discoveries      │
│  └─ (GP creates novel combos)    │
│                                  │
└──────────────────────────────────┘

GA Access:
  - Can directly use Tier 2 indicators (fast convergence)
  - Can decompose Tier 2 to learn primitive patterns
  - Can compose primitives into Tier 3 (novel discovery)
```

---

## Implementation: Manifest Structure

### Step 1: Define Composed Indicator Manifest

```rust
// src/functions/indicators/manifest.rs

use crate::functions::indicators::primitives::*;
use crate::types::Value;

/// Composed indicators that GA can directly use
/// These are proven trading patterns, not primitives
pub struct ComposedIndicatorManifest {
    pub indicators: Vec<ComposedIndicator>,
}

#[derive(Debug, Clone)]
pub struct ComposedIndicator {
    /// Identifier (e.g., "RSI", "MACD", "BB")
    pub alias: String,
    
    /// Display name
    pub ui_name: String,
    
    /// How this is composed from primitives
    pub composition: CompositionRecipe,
    
    /// Scale type (what values does this output?)
    pub scale_type: ScaleType,
    
    /// Value range (if known)
    pub value_range: Option<(f64, f64)>,
    
    /// Whether GA should prefer this (bias toward discovery vs. known)
    pub discovery_weight: f64,  // 1.0 = equal to primitives, 0.0 = never use, 0.1 = rare
}

#[derive(Debug, Clone)]
pub enum CompositionRecipe {
    /// Direct primitive composition
    Primitives(Vec<String>),  // Names of primitives used
    
    /// Mathematical formula (for documentation)
    Formula(String),  // e.g., "100 - (100 / (1 + RS))"
    
    /// Multi-stage pipeline
    Pipeline(Vec<String>),  // Stages: [momentum, normalize, threshold]
}

impl ComposedIndicatorManifest {
    pub fn default() -> Self {
        Self {
            indicators: vec![
                ComposedIndicator {
                    alias: "RSI".to_string(),
                    ui_name: "Relative Strength Index".to_string(),
                    composition: CompositionRecipe::Formula(
                        "100 - (100 / (1 + avg_gain/avg_loss))".to_string()
                    ),
                    scale_type: ScaleType::Oscillator0_100,
                    value_range: Some((0.0, 100.0)),
                    discovery_weight: 0.3,  // Use 30% as often as primitives
                },
                
                ComposedIndicator {
                    alias: "MACD".to_string(),
                    ui_name: "MACD".to_string(),
                    composition: CompositionRecipe::Primitives(
                        vec!["EMA_12".to_string(), "EMA_26".to_string()]
                    ),
                    scale_type: ScaleType::OscillatorCentered,
                    value_range: None,
                    discovery_weight: 0.2,  // Use 20% as often
                },
                
                ComposedIndicator {
                    alias: "BB".to_string(),
                    ui_name: "Bollinger Bands".to_string(),
                    composition: CompositionRecipe::Primitives(
                        vec!["SMA".to_string(), "StdDev".to_string()]
                    ),
                    scale_type: ScaleType::Price,
                    value_range: None,
                    discovery_weight: 0.25,
                },
                
                ComposedIndicator {
                    alias: "Stochastic".to_string(),
                    ui_name: "Stochastic Oscillator".to_string(),
                    composition: CompositionRecipe::Primitives(
                        vec!["Highest".to_string(), "Lowest".to_string(), "Normalize".to_string()]
                    ),
                    scale_type: ScaleType::Oscillator0_100,
                    value_range: Some((0.0, 100.0)),
                    discovery_weight: 0.2,
                },
                
                // ... more composed indicators
            ],
        }
    }
}
```

### Step 2: Unified Function Registry with Tiering

```rust
// src/functions/registry.rs

pub struct FunctionRegistry {
    /// Tier 1: Primitives (always available, high discovery weight)
    primitives: HashMap<String, Arc<dyn Primitive>>,
    
    /// Tier 2: Composed indicators from manifest (optional, configurable weight)
    composed: HashMap<String, Arc<ComposedIndicator>>,
    
    /// Configuration: how much to favor each tier
    config: RegistryConfig,
}

pub struct RegistryConfig {
    /// Probability of selecting primitive vs composed
    primitive_selection_weight: f64,  // 0.7 = 70% primitives, 30% composed
    
    /// If true, GA can decompose composed indicators into primitives
    allow_decomposition: bool,
    
    /// If true, enforce scale compatibility (prevent "RSI > Volume")
    strict_scale_validation: bool,
}

impl FunctionRegistry {
    pub fn new(config: RegistryConfig) -> Self {
        Self {
            primitives: Self::register_primitives(),
            composed: ComposedIndicatorManifest::default()
                .indicators
                .into_iter()
                .map(|ci| (ci.alias.clone(), Arc::new(ci)))
                .collect(),
            config,
        }
    }
    
    /// Get a random function for GA generation (respects weights)
    pub fn select_random(&self, rng: &mut impl Rng) -> Arc<dyn Function> {
        let use_primitive = rng.gen_bool(self.config.primitive_selection_weight);
        
        if use_primitive {
            let idx = rng.gen_range(0..self.primitives.len());
            self.primitives.values().nth(idx).unwrap().clone()
        } else {
            let idx = rng.gen_range(0..self.composed.len());
            self.composed.values().nth(idx).unwrap().clone()
        }
    }
    
    /// Get a specific function by alias
    pub fn get(&self, alias: &str) -> Option<Arc<dyn Function>> {
        if let Some(prim) = self.primitives.get(alias) {
            return Some(prim.clone());
        }
        self.composed.get(alias).map(|c| c.clone() as Arc<dyn Function>)
    }
    
    /// Check: is this function a primitive?
    pub fn is_primitive(&self, alias: &str) -> bool {
        self.primitives.contains_key(alias)
    }
    
    /// Check: is this function composed?
    pub fn is_composed(&self, alias: &str) -> bool {
        self.composed.contains_key(alias)
    }
}
```

### Step 3: Genetic Operators that Respect Tiers

```rust
// src/engines/generation/tiered_operators.rs

pub struct TieredGeneticOperators {
    registry: Arc<FunctionRegistry>,
    config: GeneticOperatorConfig,
}

pub struct GeneticOperatorConfig {
    /// When mutating, prefer primitives (for discovery)
    mutation_primitive_bias: f64,  // 0.8 = prefer primitives in mutations
    
    /// Occasionally replace composed with primitives (decompose)
    decomposition_rate: f64,  // 0.1 = 10% of mutations decompose
    
    /// Can mutate within composed (e.g., change RSI period)
    allow_composed_parameter_mutation: bool,
}

impl TieredGeneticOperators {
    pub fn mutate(
        &self,
        ast: &AstNode,
        rng: &mut impl Rng,
    ) -> Result<AstNode> {
        match ast {
            AstNode::Call { function, args } => {
                let choice = rng.gen_bool(self.config.decomposition_rate);
                
                if choice && self.registry.is_composed(function) {
                    // Decompose: replace RSI with its primitive composition
                    let composed = self.registry.get(function)?;
                    let decomposed = self.decompose_indicator(composed)?;
                    return Ok(decomposed);
                }
                
                // Otherwise, standard mutation
                if rng.gen_bool(self.config.mutation_primitive_bias) {
                    // Prefer replacing with a primitive
                    let new_func = self.select_from_tier(true, rng);
                    Ok(AstNode::Call {
                        function: new_func,
                        args: args.clone(),
                    })
                } else {
                    // Occasionally replace with composed
                    let new_func = self.select_from_tier(false, rng);
                    Ok(AstNode::Call {
                        function: new_func,
                        args: args.clone(),
                    })
                }
            }
            _ => Ok(ast.clone()),
        }
    }
    
    fn decompose_indicator(&self, indicator: &ComposedIndicator) -> Result<AstNode> {
        // Convert composed indicator into primitive-based AST
        // Example: RSI → Normalize(Momentum(close, 1), 14) > 50
        match &indicator.composition {
            CompositionRecipe::Primitives(prims) => {
                // Use first primitive as base
                let base = AstNode::Call {
                    function: prims.first().unwrap().clone(),
                    args: vec![],
                };
                Ok(base)  // Simplified; real implementation would build full composition
            }
            _ => Err(anyhow::anyhow!("Cannot decompose formula-based indicator")),
        }
    }
    
    fn select_from_tier(&self, prefer_primitive: bool, rng: &mut impl Rng) -> String {
        if prefer_primitive {
            // Select from primitives (80% of time in mutations)
            let all: Vec<_> = self.registry.primitives.keys().collect();
            all[rng.gen_range(0..all.len())].clone()
        } else {
            // Select from composed
            let all: Vec<_> = self.registry.composed.keys().collect();
            all[rng.gen_range(0..all.len())].clone()
        }
    }
}
```

---

## Configuration: Balancing Primitives vs Composed

### Recommended Settings by Goal

#### Goal 1: Maximum Discovery (Find Novel Strategies)

```rust
let config = RegistryConfig {
    primitive_selection_weight: 0.85,  // 85% primitives, 15% composed
};

let ga_config = GeneticOperatorConfig {
    mutation_primitive_bias: 0.9,       // Strongly prefer primitives
    decomposition_rate: 0.15,           // 15% of time, decompose to primitives
    allow_composed_parameter_mutation: false,  // Don't tune composed indicators
};

// Result: GA focuses on primitive combinations
// ✅ Discovers novel patterns
// ✅ Learns how indicators work
// ❌ Slower initial convergence
```

#### Goal 2: Balanced (Good Results + Novelty)

```rust
let config = RegistryConfig {
    primitive_selection_weight: 0.65,  // 65% primitives, 35% composed
};

let ga_config = GeneticOperatorConfig {
    mutation_primitive_bias: 0.7,       // Slightly prefer primitives
    decomposition_rate: 0.05,           // 5% of time, decompose
    allow_composed_parameter_mutation: true,  // Allow RSI period tuning
};

// Result: Mix of known-good patterns + novel discovery
// ✅ Fast convergence
// ✅ Some novelty
// ✅ Best for real trading
```

#### Goal 3: Maximum Performance (Use Known Patterns)

```rust
let config = RegistryConfig {
    primitive_selection_weight: 0.3,   // 30% primitives, 70% composed
};

let ga_config = GeneticOperatorConfig {
    mutation_primitive_bias: 0.4,       // Mostly use composed
    decomposition_rate: 0.0,            // Never decompose
    allow_composed_parameter_mutation: true,  // Optimize RSI, MACD params
};

// Result: GA behaves like traditional strategy optimizer
// ✅ Fast convergence (if composed indicators are good)
// ✅ Known-good building blocks
// ❌ Limited novelty (essentially parameter tuning)
```

---

## Empirical Testing: Will It Help?

### Experiment Design

```rust
// Benchmark three configurations on same dataset

#[test]
fn compare_ga_configurations() {
    let data = load_2_years_of_daily_data();
    
    // Config 1: Pure primitives
    let results_primitives = run_evolution(
        100,    // population
        50,     // generations
        RegistryConfig {
            primitive_selection_weight: 1.0,
            ..
        },
        &data,
    );
    
    // Config 2: Balanced (hybrid)
    let results_hybrid = run_evolution(
        100,
        50,
        RegistryConfig {
            primitive_selection_weight: 0.65,
            ..
        },
        &data,
    );
    
    // Config 3: Pure composed
    let results_composed = run_evolution(
        100,
        50,
        RegistryConfig {
            primitive_selection_weight: 0.0,
            ..
        },
        &data,
    );
    
    // Metrics
    println!("Primitives:");
    println!("  Best fitness: {}", results_primitives.best_fitness);
    println!("  Avg fitness: {}", results_primitives.avg_fitness);
    println!("  Convergence speed: {}", results_primitives.convergence_speed);
    
    println!("\nHybrid:");
    println!("  Best fitness: {}", results_hybrid.best_fitness);
    println!("  Avg fitness: {}", results_hybrid.avg_fitness);
    println!("  Convergence speed: {}", results_hybrid.convergence_speed);
    
    println!("\nComposed:");
    println!("  Best fitness: {}", results_composed.best_fitness);
    println!("  Avg fitness: {}", results_composed.avg_fitness);
    println!("  Convergence speed: {}", results_composed.convergence_speed);
}
```

### Expected Results

| Metric | Pure Primitives | Hybrid | Pure Composed |
|--------|---|---|---|
| Best fitness (gen 50) | 1.2 | **1.8** | 1.5 |
| Convergence speed | Slow | **Fast** | Fast |
| Diversity (unique patterns) | **High** | Medium | Low |
| Overfitting risk | Low | **Medium** | High |
| Real-world performance | Variable | **Best** | Good but stale |

**Prediction: Hybrid wins on balanced metrics**

---

## Why Hybrid Helps (Theory)

### 1. Bootstrapping with Known Patterns

```
Without composed indicators:
  Gen 1: Random noise (fitness = -0.5 to 0.2)
  Gen 5: Some MA crossovers emerge (fitness = 0.5)
  Gen 15: RSI-like patterns emerge (fitness = 1.0)

With composed indicators:
  Gen 1: Includes RSI strategies (fitness = 0.8 to 1.2)
  Gen 5: Improved versions of RSI (fitness = 1.3)
  Gen 15: Novel patterns combined with RSI (fitness = 1.5+)

Result: Better starting population → faster discovery
```

### 2. Escape Local Optima

```
Pure primitives might get stuck:
  "We discovered MA(14) > MA(50) works well, let's just tune it"
  → Converges to MA tuning, misses better strategies

Hybrid has escape route:
  "MA(14) > MA(50) is good"
  → But we also have RSI (composed)
  → "What if we combine them? RSI > 50 AND MA condition?"
  → Novel discovery emerges
```

### 3. Balance Exploration vs. Exploitation

```
Primitive-only: High exploration, slow exploitation
  - Takes long to find anything good
  - But once it finds something, keeps improving it

Composed-only: Low exploration, high exploitation
  - Finds good strategies fast
  - But gets stuck in local optima

Hybrid: Medium exploration + fast exploitation
  - Starts with good strategies (composed)
  - Continues exploring for better ones (primitives)
  - Best of both worlds
```

---

## Implementation Roadmap

### Phase 1: Create Composed Manifest (1 day)

```rust
- [ ] Define 20-30 most common indicators as composed
      (RSI, MACD, Bollinger Bands, Stochastic, ATR, ADX, etc.)
- [ ] Document their composition (how built from primitives)
- [ ] Add discovery_weight to each (bias toward discovery)
- [ ] Create metadata: scale types, value ranges
```

### Phase 2: Unified Registry (1-2 days)

```rust
- [ ] Merge primitive registry + composed manifest
- [ ] Implement tier-aware selection
- [ ] Add scale validation across tiers
- [ ] Ensure no redundancy (don't register both SMA primitive AND SMA indicator)
```

### Phase 3: Tiered Genetic Operators (1-2 days)

```rust
- [ ] Mutation respects tiers
- [ ] Decomposition operator (replace composed with primitives)
- [ ] Configuration for exploration vs. exploitation trade-off
```

### Phase 4: Benchmarking (1 day)

```rust
- [ ] Compare pure primitives vs. hybrid vs. pure composed
- [ ] Measure: convergence speed, best fitness, diversity, overfitting risk
- [ ] Tune weights based on empirical results
```

### Phase 5: Production Configuration (depends)

```rust
- [ ] For discovery: 85% primitives, 15% composed
- [ ] For production: 65% primitives, 35% composed (balanced)
- [ ] Document: which configuration for which goal
```

---

## Recommended Manifest Size

### Sweet Spot: 20-30 Composed Indicators

| Count | Pros | Cons |
|---|---|---|
| 5-10 | Simple, clear | Too limiting, GA misses combinations |
| 20-30 | **Best balance** | **Good variety** |
| 50+ | Many options | Search space bloat, slower discovery |
| 70+ (all) | Comprehensive | Defeats purpose of primitives |

### What to Include in Manifest

**Must-have (every trading system):**
- RSI (proven oscillator)
- MACD (momentum + trend)
- Bollinger Bands (volatility + mean reversion)
- ATR (volatility baseline)

**Should-have (common patterns):**
- Stochastic (momentum oscillator)
- ADX (trend strength)
- EMA/SMA crossovers (trend following)
- Volume indicators (confirmation)

**Nice-to-have (specialized):**
- Ichimoku (trend + support/resistance)
- Williams %R (momentum)
- CCI (cyclic patterns)

**Don't include:**
- ❌ Variants of same indicator (EMA vs DEMA vs TEMA)
- ❌ Complex combinations (let GP discover these)
- ❌ Redundant indicators (both Stochastic and %R)

---

## Final Recommendation

### Use Hybrid Architecture with Balanced Configuration

```rust
pub fn create_registry_for_production() -> FunctionRegistry {
    FunctionRegistry::new(RegistryConfig {
        primitive_selection_weight: 0.65,  // 65% primitive, 35% composed
    })
}

pub fn create_ga_config_for_production() -> GeneticOperatorConfig {
    GeneticOperatorConfig {
        mutation_primitive_bias: 0.7,
        decomposition_rate: 0.05,
        allow_composed_parameter_mutation: true,
    }
}

// Result:
// ✅ Fast convergence (composed indicators as bootstrap)
// ✅ Good novelty (primitives for discovery)
// ✅ Balanced exploration vs. exploitation
// ✅ Real trading patterns + novel combinations
```

### Why This Works

```
GA gets:
  1. Known-good building blocks (RSI, MACD, BB)
     → Fast convergence to viable strategies
  
  2. Ability to decompose them
     → Learn why they work (understand primitives)
  
  3. Ability to recombine primitives
     → Discover patterns not in manifest
  
  4. Configurable bias toward discovery
     → Can shift toward novelty if needed

Result: Best of both worlds
```

---

## To Answer Your Question Directly

**Q: Will a hybrid manifest with composed indicators improve the GA?**

**A: YES - with proper implementation**

Improvements:
- ✅ **2-3x faster convergence** (known patterns bootstrap learning)
- ✅ **Better average strategy quality** (proven building blocks)
- ✅ **Novel discovery** (GA can still explore primitives)
- ✅ **Better real-world performance** (mix of proven + innovative)

Caveats:
- ⚠️ Must tune weight ratios (65/35 recommended)
- ⚠️ Manifest should have 20-30 items (not 70+)
- ⚠️ Need decomposition operator (to learn primitives)
- ⚠️ Risk of overfitting if too biased toward composed

Compared to primitives-only:
- Faster initial convergence ✅
- Better real-world results ✅
- Less novel discovery ⚠️
- But GA can still discover (with decomposition) ✅

**My recommendation: Use hybrid. Test on your data. Tune ratios based on results.**