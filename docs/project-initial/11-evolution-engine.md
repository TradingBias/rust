# 11 - Evolution Engine & Genetic Programming

## Goal
Implement the main genetic programming loop that evolves trading strategies through selection, crossover, mutation, and elitism. This is the core of the strategy generation system.

## Prerequisites
- **01-architecture.md** - Project structure
- **02-type-system.md** - Core types and error handling
- **03-primitives.md** - Building blocks for strategies
- **04-06** - Indicators and registry
- **07-backtesting-engine.md** - Fitness evaluation

## What You'll Create
1. `EvolutionEngine` - Main GP loop orchestrator
2. `HallOfFame` - Elite strategy preservation with deduplication
3. Genetic operators: Selection, Crossover, Mutation
4. Progress tracking and callbacks
5. Multi-objective fitness evaluation

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│              Evolution Engine                       │
│                                                     │
│  ┌──────────────────────────────────────────────┐  │
│  │  1. Initialize Population                    │  │
│  │     └─> Semantic Mapper (see doc 12)         │  │
│  └──────────────────────────────────────────────┘  │
│                      ↓                              │
│  ┌──────────────────────────────────────────────┐  │
│  │  2. Evaluate Fitness (each strategy)         │  │
│  │     ├─> Backtester (doc 07)                  │  │
│  │     ├─> MetricsEngine (doc 08)               │  │
│  │     └─> Multi-objective scoring               │  │
│  └──────────────────────────────────────────────┘  │
│                      ↓                              │
│  ┌──────────────────────────────────────────────┐  │
│  │  3. Selection                                 │  │
│  │     ├─> Tournament selection                  │  │
│  │     └─> Roulette wheel selection             │  │
│  └──────────────────────────────────────────────┘  │
│                      ↓                              │
│  ┌──────────────────────────────────────────────┐  │
│  │  4. Genetic Operators                         │  │
│  │     ├─> Crossover (subtree swap)             │  │
│  │     ├─> Mutation (node replacement)          │  │
│  │     └─> Reproduction (copy elite)            │  │
│  └──────────────────────────────────────────────┘  │
│                      ↓                              │
│  ┌──────────────────────────────────────────────┐  │
│  │  5. Hall of Fame Update                       │  │
│  │     ├─> Deduplication (canonical AST)        │  │
│  │     ├─> Threshold filtering                  │  │
│  │     └─> Top-N preservation                   │  │
│  └──────────────────────────────────────────────┘  │
│                      ↓                              │
│  ┌──────────────────────────────────────────────┐  │
│  │  6. Progress Callback                         │  │
│  │     └─> UI updates via IPC                   │  │
│  └──────────────────────────────────────────────┘  │
│                      ↓                              │
│         Loop back to step 2 for next generation    │
└─────────────────────────────────────────────────────┘
```

## Key Concepts

### 1. Genome Representation
Each strategy is encoded as a **genome** (list of integers) that deterministically maps to an AST:

```rust
pub type Genome = Vec<u32>;

// Example genome
let genome = vec![42, 17, 88, 3, 45, 12, 99, ...];
// Maps to: if RSI(Close, 14) > 70 then OpenLong
```

### 2. Fitness Scoring
Multi-objective fitness combines multiple metrics:

```rust
pub struct FitnessScore {
    pub primary: f64,      // Main objective (e.g., Sharpe ratio)
    pub secondary: Vec<f64>, // Additional objectives
    pub raw_metrics: HashMap<String, f64>,
}

// Example: Maximize Sharpe, minimize drawdown
fitness = 0.7 * sharpe_ratio - 0.3 * max_drawdown_pct
```

### 3. Hall of Fame
Maintains best strategies across all generations with automatic deduplication:

```rust
pub struct HallOfFame {
    strategies: Vec<EliteStrategy>,
    max_size: usize,
    seen_signatures: HashSet<String>, // For dedup
}
```

## Implementation

### Step 1: Hall of Fame with Deduplication

Create `src/engines/generation/hall_of_fame.rs`:

```rust
use crate::data::types::StrategyResult;
use crate::engines::generation::ast::StrategyAST;
use crate::utils::ast_converter::AstConverter;
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct EliteStrategy {
    pub ast: StrategyAST,
    pub genome: Vec<u32>,
    pub fitness: f64,
    pub metrics: HashMap<String, f64>,
    pub canonical_string: String, // For deduplication
}

pub struct HallOfFame {
    strategies: Vec<EliteStrategy>,
    max_size: usize,
    seen_signatures: HashSet<String>,
}

impl HallOfFame {
    pub fn new(max_size: usize) -> Self {
        Self {
            strategies: Vec::new(),
            max_size,
            seen_signatures: HashSet::new(),
        }
    }

    /// Attempt to add a strategy to the Hall of Fame
    pub fn try_add(&mut self, strategy: EliteStrategy) -> bool {
        // Deduplication check
        if self.seen_signatures.contains(&strategy.canonical_string) {
            return false; // Duplicate, reject
        }

        // Add to collection
        self.strategies.push(strategy.clone());
        self.seen_signatures.insert(strategy.canonical_string.clone());

        // Sort by fitness (descending)
        self.strategies.sort_by(|a, b| {
            b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Trim to max size
        if self.strategies.len() > self.max_size {
            if let Some(removed) = self.strategies.pop() {
                self.seen_signatures.remove(&removed.canonical_string);
            }
        }

        true
    }

    /// Get all elite strategies
    pub fn get_all(&self) -> &[EliteStrategy] {
        &self.strategies
    }

    /// Get top N strategies
    pub fn get_top_n(&self, n: usize) -> &[EliteStrategy] {
        &self.strategies[..n.min(self.strategies.len())]
    }

    /// Filter by minimum fitness threshold
    pub fn filter_by_threshold(&self, min_fitness: f64) -> Vec<EliteStrategy> {
        self.strategies
            .iter()
            .filter(|s| s.fitness >= min_fitness)
            .cloned()
            .collect()
    }

    pub fn len(&self) -> usize {
        self.strategies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.strategies.is_empty()
    }
}

/// Generate canonical string for deduplication
pub fn get_canonical_ast_string(ast: &StrategyAST) -> String {
    AstConverter::ast_to_canonical_json(ast)
}
```

### Step 2: Genetic Operators

Create `src/engines/generation/operators.rs`:

```rust
use crate::engines::generation::ast::StrategyAST;
use crate::engines::generation::genome::Genome;
use rand::Rng;

/// Tournament selection: pick best of K random candidates
pub fn tournament_selection<R: Rng>(
    population: &[(Genome, f64)],
    tournament_size: usize,
    rng: &mut R,
) -> Genome {
    let mut best_idx = rng.gen_range(0..population.len());
    let mut best_fitness = population[best_idx].1;

    for _ in 1..tournament_size {
        let idx = rng.gen_range(0..population.len());
        if population[idx].1 > best_fitness {
            best_idx = idx;
            best_fitness = population[idx].1;
        }
    }

    population[best_idx].0.clone()
}

/// Roulette wheel selection: probability proportional to fitness
pub fn roulette_selection<R: Rng>(
    population: &[(Genome, f64)],
    rng: &mut R,
) -> Genome {
    // Normalize fitness to probabilities
    let total_fitness: f64 = population.iter().map(|(_, f)| f.max(0.0)).sum();

    if total_fitness <= 0.0 {
        // All negative fitness, pick random
        return population[rng.gen_range(0..population.len())].0.clone();
    }

    let mut spin = rng.gen::<f64>() * total_fitness;

    for (genome, fitness) in population {
        spin -= fitness.max(0.0);
        if spin <= 0.0 {
            return genome.clone();
        }
    }

    // Fallback
    population[population.len() - 1].0.clone()
}

/// Single-point crossover: swap genome segments
pub fn crossover<R: Rng>(
    parent1: &Genome,
    parent2: &Genome,
    rng: &mut R,
) -> (Genome, Genome) {
    let len = parent1.len().min(parent2.len());
    if len <= 1 {
        return (parent1.clone(), parent2.clone());
    }

    let point = rng.gen_range(1..len);

    let mut child1 = parent1.clone();
    let mut child2 = parent2.clone();

    child1[point..].copy_from_slice(&parent2[point..]);
    child2[point..].copy_from_slice(&parent1[point..]);

    (child1, child2)
}

/// Mutation: randomly modify genes
pub fn mutate<R: Rng>(
    genome: &mut Genome,
    mutation_rate: f64,
    gene_range: std::ops::Range<u32>,
    rng: &mut R,
) {
    for gene in genome.iter_mut() {
        if rng.gen::<f64>() < mutation_rate {
            *gene = rng.gen_range(gene_range.clone());
        }
    }
}

/// Generate random genome
pub fn random_genome<R: Rng>(
    length: usize,
    gene_range: std::ops::Range<u32>,
    rng: &mut R,
) -> Genome {
    (0..length)
        .map(|_| rng.gen_range(gene_range.clone()))
        .collect()
}
```

### Step 3: Evolution Engine

Create `src/engines/generation/evolution_engine.rs`:

```rust
use crate::data::types::StrategyResult;
use crate::engines::evaluation::Backtester;
use crate::engines::generation::{
    hall_of_fame::{EliteStrategy, HallOfFame, get_canonical_ast_string},
    operators::*,
    semantic_mapper::SemanticMapper,
    genome::Genome,
};
use crate::error::TradeBiasError;
use polars::prelude::*;
use rand::SeedableRng;
use rand::rngs::StdRng;
use std::collections::HashMap;

pub struct EvolutionConfig {
    pub population_size: usize,
    pub generations: usize,
    pub genome_length: usize,
    pub gene_range: std::ops::Range<u32>,
    pub mutation_rate: f64,
    pub crossover_rate: f64,
    pub elitism_rate: f64,
    pub tournament_size: usize,
    pub hall_of_fame_size: usize,
    pub fitness_objectives: Vec<String>, // Metric names
    pub fitness_weights: Vec<f64>,       // Weights for multi-objective
    pub min_fitness_threshold: f64,
    pub seed: Option<u64>,
}

pub struct EvolutionEngine {
    config: EvolutionConfig,
    backtester: Backtester,
    semantic_mapper: SemanticMapper,
    hall_of_fame: HallOfFame,
    rng: StdRng,
}

pub trait ProgressCallback: Send {
    fn on_generation_start(&mut self, generation: usize);
    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hall_of_fame_size: usize);
    fn on_strategy_evaluated(&mut self, strategy_num: usize, total: usize);
}

impl EvolutionEngine {
    pub fn new(
        config: EvolutionConfig,
        backtester: Backtester,
        semantic_mapper: SemanticMapper,
    ) -> Self {
        let rng = match config.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_entropy(),
        };

        let hall_of_fame = HallOfFame::new(config.hall_of_fame_size);

        Self {
            config,
            backtester,
            semantic_mapper,
            hall_of_fame,
            rng,
        }
    }

    /// Run the evolution process
    pub fn run<C: ProgressCallback>(
        &mut self,
        data: &DataFrame,
        mut callback: C,
    ) -> Result<Vec<EliteStrategy>, TradeBiasError> {
        // Initialize population
        let mut population = self.initialize_population();

        // Evolution loop
        for generation in 0..self.config.generations {
            callback.on_generation_start(generation);

            // Evaluate fitness for all individuals
            let evaluated = self.evaluate_population(&population, data, &mut callback)?;

            // Update Hall of Fame
            for (genome, fitness, ast, metrics) in &evaluated {
                let canonical_string = get_canonical_ast_string(ast);
                let elite = EliteStrategy {
                    ast: ast.clone(),
                    genome: genome.clone(),
                    fitness: *fitness,
                    metrics: metrics.clone(),
                    canonical_string,
                };
                self.hall_of_fame.try_add(elite);
            }

            // Get best fitness for progress tracking
            let best_fitness = evaluated
                .iter()
                .map(|(_, f, _, _)| *f)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);

            callback.on_generation_complete(generation, best_fitness, self.hall_of_fame.len());

            // Check termination
            if generation == self.config.generations - 1 {
                break;
            }

            // Create next generation
            population = self.create_next_generation(&evaluated);
        }

        Ok(self.hall_of_fame.get_all().to_vec())
    }

    fn initialize_population(&mut self) -> Vec<Genome> {
        (0..self.config.population_size)
            .map(|_| {
                random_genome(
                    self.config.genome_length,
                    self.config.gene_range.clone(),
                    &mut self.rng,
                )
            })
            .collect()
    }

    fn evaluate_population<C: ProgressCallback>(
        &mut self,
        population: &[Genome],
        data: &DataFrame,
        callback: &mut C,
    ) -> Result<Vec<(Genome, f64, StrategyAST, HashMap<String, f64>)>, TradeBiasError> {
        let mut results = Vec::new();

        for (i, genome) in population.iter().enumerate() {
            callback.on_strategy_evaluated(i + 1, population.len());

            // Generate AST from genome
            let ast = self.semantic_mapper.create_strategy_ast(genome)?;

            // Run backtest
            let backtest_result = self.backtester.run(&ast, data)?;

            // Calculate fitness
            let fitness = self.calculate_fitness(&backtest_result.metrics);

            results.push((genome.clone(), fitness, ast, backtest_result.metrics));
        }

        Ok(results)
    }

    fn calculate_fitness(&self, metrics: &HashMap<String, f64>) -> f64 {
        let mut fitness = 0.0;

        for (objective, weight) in self.config.fitness_objectives.iter().zip(&self.config.fitness_weights) {
            if let Some(&value) = metrics.get(objective) {
                fitness += weight * value;
            }
        }

        fitness
    }

    fn create_next_generation(
        &mut self,
        evaluated: &[(Genome, f64, StrategyAST, HashMap<String, f64>)],
    ) -> Vec<Genome> {
        let mut next_generation = Vec::new();
        let population_fitness: Vec<(Genome, f64)> = evaluated
            .iter()
            .map(|(g, f, _, _)| (g.clone(), *f))
            .collect();

        // Elitism: copy top performers
        let elite_count = (self.config.population_size as f64 * self.config.elitism_rate) as usize;
        let mut sorted = population_fitness.clone();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (genome, _) in sorted.iter().take(elite_count) {
            next_generation.push(genome.clone());
        }

        // Generate offspring
        while next_generation.len() < self.config.population_size {
            if self.rng.gen::<f64>() < self.config.crossover_rate {
                // Crossover
                let parent1 = tournament_selection(
                    &population_fitness,
                    self.config.tournament_size,
                    &mut self.rng,
                );
                let parent2 = tournament_selection(
                    &population_fitness,
                    self.config.tournament_size,
                    &mut self.rng,
                );

                let (mut child1, mut child2) = crossover(&parent1, &parent2, &mut self.rng);

                // Apply mutation
                mutate(&mut child1, self.config.mutation_rate, self.config.gene_range.clone(), &mut self.rng);
                mutate(&mut child2, self.config.mutation_rate, self.config.gene_range.clone(), &mut self.rng);

                next_generation.push(child1);
                if next_generation.len() < self.config.population_size {
                    next_generation.push(child2);
                }
            } else {
                // Reproduction (copy)
                let parent = tournament_selection(
                    &population_fitness,
                    self.config.tournament_size,
                    &mut self.rng,
                );
                let mut child = parent;
                mutate(&mut child, self.config.mutation_rate, self.config.gene_range.clone(), &mut self.rng);
                next_generation.push(child);
            }
        }

        next_generation.truncate(self.config.population_size);
        next_generation
    }

    pub fn get_hall_of_fame(&self) -> &HallOfFame {
        &self.hall_of_fame
    }
}
```

### Step 4: Progress Callback Implementation

Create example callback in `src/engines/generation/progress.rs`:

```rust
use super::evolution_engine::ProgressCallback;

pub struct ConsoleProgressCallback;

impl ProgressCallback for ConsoleProgressCallback {
    fn on_generation_start(&mut self, generation: usize) {
        println!("Generation {} starting...", generation + 1);
    }

    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hof_size: usize) {
        println!(
            "Generation {} complete. Best fitness: {:.4}, Hall of Fame size: {}",
            generation + 1, best_fitness, hof_size
        );
    }

    fn on_strategy_evaluated(&mut self, strategy_num: usize, total: usize) {
        if strategy_num % 10 == 0 || strategy_num == total {
            println!("  Evaluated {}/{} strategies", strategy_num, total);
        }
    }
}

// For IPC communication with UI
pub struct IpcProgressCallback {
    sender: std::sync::mpsc::Sender<ProgressMessage>,
}

pub enum ProgressMessage {
    GenerationStart(usize),
    GenerationComplete { generation: usize, best_fitness: f64, hof_size: usize },
    StrategyEvaluated { current: usize, total: usize },
}

impl IpcProgressCallback {
    pub fn new(sender: std::sync::mpsc::Sender<ProgressMessage>) -> Self {
        Self { sender }
    }
}

impl ProgressCallback for IpcProgressCallback {
    fn on_generation_start(&mut self, generation: usize) {
        let _ = self.sender.send(ProgressMessage::GenerationStart(generation));
    }

    fn on_generation_complete(&mut self, generation: usize, best_fitness: f64, hof_size: usize) {
        let _ = self.sender.send(ProgressMessage::GenerationComplete {
            generation,
            best_fitness,
            hof_size,
        });
    }

    fn on_strategy_evaluated(&mut self, strategy_num: usize, total: usize) {
        let _ = self.sender.send(ProgressMessage::StrategyEvaluated {
            current: strategy_num,
            total,
        });
    }
}
```

### Step 5: Update Module Exports

Update `src/engines/generation/mod.rs`:

```rust
pub mod ast;
pub mod genome;
pub mod semantic_mapper;
pub mod evaluator;
pub mod operators;
pub mod hall_of_fame;
pub mod evolution_engine;
pub mod progress;

pub use ast::*;
pub use genome::*;
pub use hall_of_fame::{HallOfFame, EliteStrategy};
pub use evolution_engine::{EvolutionEngine, EvolutionConfig, ProgressCallback};
pub use progress::{ConsoleProgressCallback, IpcProgressCallback};
```

## Usage Example

```rust
use tradebias::engines::generation::{EvolutionEngine, EvolutionConfig, ConsoleProgressCallback};
use tradebias::engines::evaluation::Backtester;
use tradebias::engines::generation::semantic_mapper::SemanticMapper;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let data = load_market_data("data.csv")?;

    // Configure evolution
    let config = EvolutionConfig {
        population_size: 100,
        generations: 50,
        genome_length: 100,
        gene_range: 0..1000,
        mutation_rate: 0.1,
        crossover_rate: 0.7,
        elitism_rate: 0.1,
        tournament_size: 3,
        hall_of_fame_size: 20,
        fitness_objectives: vec!["sharpe_ratio".to_string(), "max_drawdown_pct".to_string()],
        fitness_weights: vec![0.7, -0.3], // Maximize Sharpe, minimize drawdown
        min_fitness_threshold: 0.0,
        seed: Some(42),
    };

    // Create components
    let backtester = Backtester::new(/* ... */);
    let semantic_mapper = SemanticMapper::new(/* ... */);

    // Run evolution
    let mut engine = EvolutionEngine::new(config, backtester, semantic_mapper);
    let callback = ConsoleProgressCallback;

    let hall_of_fame = engine.run(&data, callback)?;

    // Print results
    for (i, strategy) in hall_of_fame.iter().take(5).enumerate() {
        println!("Rank {}: Fitness = {:.4}", i + 1, strategy.fitness);
        println!("  Sharpe: {:.2}", strategy.metrics.get("sharpe_ratio").unwrap_or(&0.0));
        println!("  Drawdown: {:.2}%", strategy.metrics.get("max_drawdown_pct").unwrap_or(&0.0));
    }

    Ok(())
}
```

## Verification

### Test 1: Hall of Fame Deduplication
```rust
#[test]
fn test_hall_of_fame_dedup() {
    let mut hof = HallOfFame::new(10);

    let strategy1 = create_dummy_strategy("RSI > 70", 1.5);
    let strategy2 = create_dummy_strategy("RSI > 70", 1.8); // Same AST, higher fitness

    assert!(hof.try_add(strategy1));
    assert!(!hof.try_add(strategy2)); // Should be rejected (duplicate)
    assert_eq!(hof.len(), 1);
}
```

### Test 2: Genetic Operators
```rust
#[test]
fn test_crossover() {
    let parent1 = vec![1, 2, 3, 4, 5];
    let parent2 = vec![6, 7, 8, 9, 10];
    let mut rng = StdRng::seed_from_u64(42);

    let (child1, child2) = crossover(&parent1, &parent2, &mut rng);

    // Children should be different from parents
    assert_ne!(child1, parent1);
    assert_ne!(child2, parent2);

    // Children should contain genes from both parents
    assert!(child1.iter().any(|g| parent1.contains(g)));
    assert!(child1.iter().any(|g| parent2.contains(g)));
}
```

### Test 3: Full Evolution Run
```rust
#[test]
fn test_evolution_run() {
    let data = load_test_data();
    let config = EvolutionConfig {
        population_size: 20,
        generations: 5,
        // ... minimal config
    };

    let backtester = create_test_backtester();
    let semantic_mapper = create_test_semantic_mapper();

    let mut engine = EvolutionEngine::new(config, backtester, semantic_mapper);
    let callback = ConsoleProgressCallback;

    let result = engine.run(&data, callback);
    assert!(result.is_ok());

    let hof = result.unwrap();
    assert!(!hof.is_empty());
    assert!(hof[0].fitness >= hof[hof.len() - 1].fitness); // Sorted by fitness
}
```

## Performance Considerations

1. **Parallelization**: Evaluate population in parallel using Rayon:
   ```rust
   use rayon::prelude::*;

   let results: Vec<_> = population
       .par_iter()
       .map(|genome| evaluate_strategy(genome, data))
       .collect();
   ```

2. **Caching**: Cache indicator calculations across strategies (see doc 06)

3. **Early Stopping**: Terminate if no improvement for N generations

4. **Adaptive Parameters**: Adjust mutation rate based on diversity

## Common Issues

### Issue: Hall of Fame not filling up
**Solution**: Check `min_fitness_threshold` - it may be too high. Set to negative value or 0.0 initially.

### Issue: All strategies have same fitness
**Solution**: Check fitness calculation - ensure proper metric normalization and weights.

### Issue: Memory usage growing over time
**Solution**: Clear old population data after each generation. Don't keep full history unless needed.

## Next Steps

Proceed to **[12-semantic-generation.md](./12-semantic-generation.md)** to implement the semantic mapper that generates type-valid strategies from genomes.
