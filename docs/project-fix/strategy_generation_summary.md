# Strategy Generation System - How TradeBias Creates Trading Strategies

## Overview

TradeBias uses a **genetic programming (GP) approach** to automatically discover trading strategies. Instead of manually coding rules, the system evolves strategies over multiple generations, selecting those that perform best on historical data.

Think of it like biological evolution: random strategies are created, tested, and the best ones are kept and "bred" to create even better strategies. Over time, this process discovers complex trading rules that would be difficult to design manually.

---

## The Generation Pipeline

### 1. From DNA to Strategy

Every strategy starts as a **genome** - a simple list of random numbers (genes). This genome acts like DNA, encoding all the information needed to build a complete trading strategy.

**File**: `src/engines/generation/evolution_engine.rs`

The evolution engine creates a population of 500 random genomes (by default), each containing 100 genes with values between 0 and 1000. These numbers seem arbitrary, but they're actually instructions for building the strategy tree.

### 2. The Semantic Mapper - The Architect

The **semantic mapper** reads the genome and builds a strategy Abstract Syntax Tree (AST). This is where genes become decisions.

**File**: `src/engines/generation/semantic_mapper.rs`

The mapper consumes genes sequentially, using each gene to make a choice:
- **Gene 1**: Should I use an indicator or a data field? (gene value modulo 3)
- **Gene 2**: Which indicator? RSI, MACD, or Bollinger Bands? (gene value picks from list)
- **Gene 3**: What period should I use? 14, 20, or 50? (gene value selects from typical periods)
- **Gene 4**: What threshold? 70, 80, or 30? (gene value maps to appropriate range)

This process is deterministic - the same genome always produces the same strategy.

### 3. Building the Strategy Tree

Strategies are built recursively as trees. A strategy must answer: **"When should I buy or sell?"**

The answer is structured as:
```
IF [condition is true] THEN [action: go long (+1) or short (-1)]
```

**Conditions** are boolean expressions like:
- "RSI is greater than 70"
- "Close price crossed above the 50-period moving average"
- "MACD is less than 0 AND RSI is greater than 30"

**Actions** are simple: 1.0 for long positions, -1.0 for short positions.

The mapper builds this tree top-down, recursively generating sub-expressions until it reaches terminal nodes (actual data like Close price or constant values).

### 4. Depth Control - Preventing Monsters

Without limits, the mapper could create infinitely deep strategies. A depth limit (default: 12 levels) prevents this.

When the depth limit is reached, the mapper switches to "terminal mode" - only creating simple leaf nodes:
- Data accessors (Open, High, Low, Close, Volume)
- Constant values
- Simple comparisons

This ensures every strategy finishes in reasonable time and complexity.

---

## What Guides the Generation?

### Hard-Coded Wisdom

The system embeds trading knowledge through carefully chosen defaults:

**Common Technical Analysis Periods**
The mapper doesn't generate random periods like 73 or 142. Instead, it chooses from standard values traders actually use:
- Short-term: 5, 7, 9, 10, 12, 14
- Medium-term: 20, 21, 25, 30
- Long-term: 50, 100, 200

These are the periods you'll find in any trading textbook.

**Data Fields**
Only the standard OHLCV (Open, High, Low, Close, Volume) fields are used - no exotic or derived fields unless explicitly created by indicators.

**Mathematical Operations**
Simple operations only: Add, Subtract, Multiply, Divide. No complex functions that could create numerical instability.

### Indicator Metadata - Smart Parameter Selection

**File**: `src/utils/indicator_metadata.rs`

This is the system's "encyclopedia of indicators." For each indicator, it knows:

**1. Scale Type** - How to interpret values
- **Price-level** (SMA, EMA): Values near market price (e.g., 50,000 for BTC)
- **Bounded oscillators** (RSI, Stochastic): 0-100 range
- **Centered oscillators** (MACD): Oscillates around zero
- **Volatility measures** (ATR): Small decimal values
- **Volume-based** (OBV): Large integers

**2. Typical Periods**
- RSI: [9, 14, 21, 25] - not random periods
- MACD: (12, 26, 9) - the industry standard
- Moving averages: [10, 20, 50, 100, 200]

**3. Appropriate Thresholds**
- RSI: 20, 30, 70, 80 (overbought/oversold levels)
- MACD: -10, 0, 10 (zero-crossing is key)
- ATR: Small values scaled to market volatility

This metadata prevents nonsensical comparisons like "RSI > 1000000" or "use period 3 for a long-term trend."

### Type System - Preventing Invalid Combinations

**File**: `src/functions/traits.rs`

Every function declares what types it accepts and returns:
- **NumericSeries**: A time series of numbers (prices, indicator values)
- **BoolSeries**: A time series of true/false (trading signals)
- **Integer**: Scalar integers (periods, lookback windows)
- **Float**: Scalar decimals (thresholds, constants)

The semantic mapper only generates type-compatible combinations. You can't accidentally create:
- "RSI AND 50" (can't AND a number with a number, needs booleans)
- "IF 14.5 THEN..." (can't use a number as a condition, needs boolean)
- "SMA(Close, 45.7)" (period must be integer, not float)

### Function Registry - The Toolbox

**File**: `src/functions/registry.rs`

The registry contains 29 indicators and 23 primitive operations:

**Momentum Indicators** (9): RSI, Stochastic, CCI, Williams%R, Momentum, Awesome Oscillator, RVI, DeMarker, Accelerator

**Trend Indicators** (12): SMA, EMA, MACD, Bollinger Bands, Envelopes, Parabolic SAR, Bears Power, Bulls Power, DEMA, TEMA, TriX

**Volatility Indicators** (3): ATR, ADX, Standard Deviation

**Volume Indicators** (5): OBV, MFI, Force Index, Volumes, Chaikin Oscillator

**Primitives** (23): Logical operations (AND, OR), Math (Add, Subtract, Multiply, Divide), Comparisons (greater than, less than, equals), Cross detection (cross above, cross below)

Each function has metadata about its purpose, inputs, and outputs.

---

## Validation Layers - Quality Control

### 1. Lightweight Validation

**File**: `src/engines/generation/lightweight_validator.rs`

After a strategy is generated, it's checked for basic correctness:
- **Depth check**: Does it exceed the maximum depth?
- **Function existence**: Are all functions actually registered?
- **Arity matching**: Do all functions have the correct number of arguments?
- **Basic type checking**: Are input/output types compatible?

Invalid strategies are rejected immediately.

### 2. Diversity Validation

**File**: `src/engines/generation/diversity_validator.rs`

This prevents "degenerate" strategies that make no practical sense:

**Example**: `IF RSI(14) > RSI(15) THEN buy`

This compares RSI with period 14 to RSI with period 15. The periods are so similar (difference of 1) that the two RSI values will be nearly identical, making the condition meaningless.

The diversity validator rejects strategies where:
- The same indicator appears multiple times with near-identical parameters
- Parameters differ by less than a minimum threshold (default: 5)

### 3. Deduplication

The Hall of Fame (top 10 strategies) maintains a set of unique strategies. Each strategy is converted to a canonical string representation.

If a new strategy matches an existing one (even if generated differently), it's rejected. This prevents the population from filling up with copies of the same strategy.

---

## How Strategies are Evaluated

### Expression Building

**File**: `src/engines/evaluation/expression.rs`

The AST is converted into Polars expressions - a vectorized query language that executes on the entire dataset at once (no loops!).

Example transformation:
```
AST: Rule {
    condition: gt_scalar(RSI(Close, 14), 70),
    action: 1.0
}

Polars: when(rsi(col("close"), 14) > 70).then(1.0).otherwise(0.0)
```

This generates a column of trading signals: 1.0 (go long), -1.0 (go short), or 0.0 (do nothing).

### Backtesting

**File**: `src/engines/evaluation/backtester.rs`

The backtester simulates trading:

1. **Generate signals**: Run the Polars expression on historical data
2. **Simulate trades**: Walk through each bar, opening/closing positions based on signals
3. **Track portfolio**: Monitor cash, positions, realized P&L, unrealized P&L
4. **Calculate metrics**: Return percentage, number of trades, maximum drawdown

**Position Management Rules**:
- Only one position at a time (either long, short, or flat)
- Position size: 10% of available cash
- Entry: When signal changes from 0 to +1 (long) or -1 (short)
- Exit: When signal changes to opposite direction
- Final position: Closed at the end of the data

**File**: `src/engines/evaluation/portfolio.rs`

The portfolio tracks both realized (closed trade) P&L and unrealized (open position) P&L, giving accurate mark-to-market valuation.

### Fitness Calculation

After backtesting, each strategy gets a fitness score. By default:
```
fitness = return_percentage × 1.0
```

The configuration supports multi-objective fitness (e.g., maximize returns while minimizing drawdown), but currently only return percentage is used.

**File**: `src/config/evolution.rs`

---

## Evolution Process - Survival of the Fittest

### Selection

**Tournament Selection** (default):
- Pick 7 random strategies from the population
- Select the one with highest fitness
- Repeat to fill the next generation

This gives better strategies higher probability of being selected while still allowing weaker ones a chance.

### Elitism

The top 10% of strategies automatically survive to the next generation unchanged. This ensures good solutions are never lost.

### Crossover (85% probability)

Two parent genomes are combined:
- Pick a random crossover point (e.g., gene 47)
- Child 1 gets genes 0-47 from Parent 1, genes 48-99 from Parent 2
- Child 2 gets genes 0-47 from Parent 2, genes 48-99 from Parent 1

This mixes successful strategies, potentially combining good traits.

### Mutation (15% probability)

Random genes are changed to random values. This introduces new variations and prevents the population from getting stuck.

### Hall of Fame

The top 10 strategies ever seen are maintained throughout evolution. These are deduplicated and ranked by fitness.

At the end of evolution, these are the candidate strategies for deployment.

---

## Configuration Parameters

### Population and Generations

**File**: `src/config/evolution.rs`

- **Population size**: 500 (how many strategies per generation)
- **Generations**: 100 (how many iterations)
- **Total evaluations**: 500 × 100 = 50,000 strategies tested

### Genetic Operators

- **Mutation rate**: 15% (probability of random gene changes)
- **Crossover rate**: 85% (probability of combining parent strategies)
- **Elitism**: Top 10% survive unchanged
- **Tournament size**: 7 (selection pressure)

### Strategy Complexity

- **Genome length**: 100 genes
- **Gene range**: 0-1000
- **Max tree depth**: 12 levels
- **Hall of Fame size**: 10 strategies

### Risk Management

**File**: `src/config/trade_management.rs`

- **Position size**: 10% of cash per trade
- **Max positions**: 1 (no position pyramiding)
- **Initial capital**: Configurable via backtesting config

---

## Example Strategy Evolution

Let's walk through a concrete example:

### Generation 1 (Random)
Strategy might be: `IF Volume > 1000000 THEN go long`
- Too simple, doesn't consider price
- Fitness: -5% (loses money)

### Generation 20 (Early evolution)
Strategy: `IF RSI(Close, 14) > 70 THEN go short`
- Uses a real indicator
- Captures overbought conditions
- Fitness: 8% (profitable!)

### Generation 50 (Crossover improvements)
Strategy: `IF RSI(Close, 14) > 70 AND Close < SMA(Close, 50) THEN go short`
- Combines momentum (RSI) and trend (SMA)
- Only shorts when price is below moving average (trend confirmation)
- Fitness: 15% (better!)

### Generation 100 (Final)
Strategy: `IF (RSI(Close, 14) > 75 AND MACD(Close, 12, 26, 9) < 0) OR Close crossed_below EMA(Close, 20) THEN go short`
- Complex multi-indicator logic
- Uses crossover detection
- Multiple entry conditions
- Fitness: 22% (best found)

This demonstrates how complexity and performance increase over generations.

---

## Current Limitations

### 1. Simple Action Space
Strategies can only output:
- +1.0 (go long)
- -1.0 (go short)
- 0.0 (do nothing)

There's no ability to:
- Vary position size based on confidence
- Set stop-losses or take-profits in the strategy itself
- Close positions without opening opposite ones

### 2. Limited Semantic Validation
The system ensures type correctness but doesn't prevent logical nonsense:
- Comparing Volume to Close price (different scales)
- Using volatility indicators as trend signals
- Creating unreasonably complex conditions unlikely to generalize

### 3. Single Objective Optimization
Currently only return percentage is optimized. The system doesn't consider:
- Risk-adjusted returns (Sharpe ratio)
- Maximum drawdown limits
- Number of trades (could overtrade)
- Win rate or profit factor

### 4. No Walk-Forward Testing
All strategies are optimized on the full dataset (in-sample). There's no automatic out-of-sample validation, risking overfitting.

### 5. No Transaction Costs
Backtesting assumes zero slippage and zero commissions. Real-world performance will be lower, especially for high-frequency strategies.

### 6. Performance Bottlenecks
- No parallel strategy evaluation (strategies tested sequentially)
- Expression caching disabled due to stack overflow issues
- Large populations with complex indicators are slow

---

## Suggested Improvements to Guide Sensical Strategies

### 1. Semantic Constraints and Rules

**Add Type-Aware Comparison Constraints**
- Prevent comparing indicators of different scale types
- Example: Don't allow `ATR(14) > RSI(14)` (volatility vs oscillator)
- Implementation: Check indicator metadata before generating comparisons

**Logical Complexity Penalties**
- Penalize overly complex strategies (many nested ANDs/ORs)
- Add fitness penalty proportional to AST depth
- Formula: `adjusted_fitness = raw_fitness - (depth_penalty × max_depth)`

**Indicator Category Awareness**
- Encourage combining different categories (trend + momentum + volume)
- Penalize using multiple indicators from same category
- Example: Prefer `RSI AND SMA` over `RSI AND Stochastic`

### 2. Multi-Objective Fitness Function

**Risk-Adjusted Returns**
Instead of just return percentage, use:
```
fitness = (Sharpe Ratio × 0.5) + (Return % × 0.3) + (Win Rate × 0.2)
```

**Pareto Optimization**
- Optimize for multiple objectives simultaneously
- Maintain Pareto frontier of non-dominated solutions
- Example objectives: maximize returns, minimize drawdown, minimize trade count

**Trade Quality Metrics**
- Reward strategies with reasonable trade frequency (not too high, not too low)
- Penalize strategies with very few trades (likely overfitted)
- Target range: 20-100 trades per year

### 3. Validation and Filtering

**Cross-Validation During Evolution**
- Split data into train/validation sets
- Evaluate fitness on training, validate on hold-out
- Only promote strategies that perform well on both

**Reality Checks**
- Reject strategies with unrealistic returns (>100% on conservative data)
- Flag strategies that trade too frequently (>10 times per day)
- Warn on strategies with very high drawdowns (>50%)

**Statistical Significance Testing**
- Compare strategy returns to random entry/exit
- Only keep strategies that beat random with statistical significance
- Use permutation tests or Monte Carlo simulation

### 4. Guided Generation Heuristics

**Template-Based Initialization**
Instead of purely random genomes, seed population with known patterns:
- "Mean reversion template": Oscillator oversold → buy
- "Trend following template": Price > MA AND MA rising → buy
- "Breakout template": Price cross above resistance → buy

**Smart Mutation Operators**
Current mutation is random. Add intelligent mutations:
- **Parameter tuning**: Change RSI(14) to RSI(21) (similar strategy, different param)
- **Indicator substitution**: Replace RSI with Stochastic (same category)
- **Logic refinement**: Add one more condition to existing rule

**Macro-Mutation (Subtree Replacement)**
Occasionally replace entire subtrees rather than single genes:
- Replace `RSI(14) > 70` with `Stochastic(14,3,3) > 80`
- Helps escape local optima
- Maintains overall strategy structure

### 5. Domain Knowledge Injection

**Indicator Combination Rules**
Encode best practices:
- "Trend + Momentum": Always combine trend indicator with momentum
- "Confirmation requirement": Require 2+ indicators to agree before trading
- "Multi-timeframe": Use different periods (short + long) for confirmation

**Time-of-Day Awareness**
- Add time-based conditions (don't trade at market open/close)
- Prevent overnight positions if data shows gap risk
- Requires adding time-based primitives to the function registry

**Market Regime Detection**
- Generate different strategies for trending vs ranging markets
- Use volatility filters (only trade when ATR > threshold)
- Combine multiple strategies with regime switching

### 6. Incremental Complexity

**Start Simple, Grow Complex**
- Early generations: Max depth = 3 (simple strategies)
- Later generations: Gradually increase max depth to 12
- Prevents premature complexity and overfitting

**Complexity-Adjusted Fitness**
```
adjusted_fitness = raw_fitness / (1 + complexity_factor)
```
Where complexity = number of indicators + tree depth

This rewards parsimonious strategies following Occam's Razor.

### 7. Diversity Preservation

**Niching and Speciation**
- Maintain sub-populations of different strategy types
- "Trend following niche", "Mean reversion niche", "Breakout niche"
- Prevents population collapse to single strategy type

**Novelty Search**
- Reward strategies that are different from existing ones
- Measure behavioral diversity (not just genetic)
- Example: Count unique entry patterns, exit patterns

**Crowding Distance**
- Penalize strategies too similar to others in Hall of Fame
- Maintain diversity in final candidate set
- User gets variety of strategies to choose from

### 8. Robustness Testing

**Parameter Sensitivity Analysis**
- Automatically test strategy with slightly modified parameters
- If RSI(14) is in strategy, test RSI(13) and RSI(15)
- Reward strategies that are stable (performance doesn't collapse)

**Different Market Conditions**
- Test on bull markets, bear markets, sideways markets separately
- Penalize strategies that only work in one regime
- Use data from different time periods or instruments

**Monte Carlo Simulation**
- Randomize trade order (same trades, different sequence)
- Add noise to entry/exit prices (simulate slippage)
- Check if performance holds with realistic variations

### 9. Explainability and Interpretation

**Human-Readable Strategy Descriptions**
Auto-generate descriptions like:
```
"This strategy goes long when RSI is oversold and price crosses above
the 50-day moving average, indicating a potential trend reversal in
oversold conditions."
```

**Strategy Categorization**
Automatically classify strategies:
- Trend Following / Mean Reversion / Breakout
- Short-term / Medium-term / Long-term
- High frequency / Low frequency

**Feature Importance**
Identify which indicators contribute most to performance:
- Run ablation tests (remove one indicator at a time)
- Helps understand if strategy is robust or lucky

### 10. Advanced Genetic Operators

**Adaptive Mutation Rates**
- Increase mutation when population converges (lacks diversity)
- Decrease mutation when improvements are being made
- Self-adjusting based on fitness variance

**Co-evolution**
- Evolve strategies AND evaluation methods simultaneously
- Prevents overfitting to specific fitness function
- More robust to regime changes

**Hybrid with Local Search**
- Use genetic programming for global search
- Apply gradient-based optimization for parameter fine-tuning
- Best of both worlds: structure discovery + parameter optimization

---

## Implementation Priority

If I were to prioritize improvements for immediate impact:

### High Priority (Biggest Impact)
1. **Multi-objective fitness** (risk-adjusted returns)
2. **Walk-forward validation** (prevent overfitting)
3. **Semantic constraints** (prevent nonsense comparisons)
4. **Trade quality filters** (reasonable trade frequency)

### Medium Priority (Good ROI)
5. **Template-based initialization** (start with known patterns)
6. **Complexity penalties** (favor simpler strategies)
7. **Parameter sensitivity testing** (robustness check)
8. **Indicator category awareness** (diverse combinations)

### Lower Priority (Nice to Have)
9. **Niching and speciation** (strategy diversity)
10. **Explainability features** (understand why strategies work)
11. **Adaptive mutation** (self-tuning evolution)
12. **Time-based conditions** (intraday awareness)

---

## Conclusion

The TradeBias strategy generation system is sophisticated and well-architected, with strong type safety and sensible defaults. The genetic programming approach successfully explores a vast strategy space.

However, it currently optimizes for in-sample returns without sufficient constraints on complexity, robustness, or risk-adjusted performance. The suggested improvements would guide evolution toward strategies that are:

- **Sensible**: Semantically meaningful indicator combinations
- **Robust**: Perform well across different market conditions
- **Risk-aware**: Consider drawdown and volatility, not just returns
- **Generalizable**: Validated on out-of-sample data
- **Explainable**: Human-understandable trading logic

The foundation is solid; these enhancements would take it from generating "strategies that worked in the past" to generating "strategies likely to work in the future."
