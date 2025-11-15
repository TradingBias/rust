# Strategy Generation Summary

This project employs a genetic algorithm, specifically a Grammatical Evolution approach, to generate trading strategies. The core process is orchestrated by the `EvolutionEngine`.

## Data Flow and Key Components:

1.  **Initiation (`EvolutionRunner`):** The strategy generation process is typically initiated by the `EvolutionRunner` service, often in response to a user interface action. This service is responsible for assembling the necessary components and configurations required for the evolutionary process.

2.  **Genotype (`Genome`):** A trading strategy is represented genetically as a `Genome`, which is essentially a simple vector of 32-bit unsigned integers (`Vec<u32>`). This linear, numerical format is well-suited for manipulation by genetic operators.

3.  **Evolution Loop (`EvolutionEngine`):** The `EvolutionEngine` manages the main genetic algorithm loop.
    *   **Initial Population:** It begins by creating an initial population of random `Genome`s.
    *   **Iterative Generations:** For a predefined number of generations, the engine performs the following steps:
        a.  **Translation (`SemanticMapper`):** Each `Genome` from the population is translated into a `StrategyAST` (Abstract Syntax Tree) by the `SemanticMapper`. This mapper interprets the integers within the genome sequentially to make decisions, such as selecting indicators, determining parameter values, and constructing a valid strategy tree.
        b.  **Evaluation (`Backtester`):** The newly generated `StrategyAST` is then executed against historical market data using the `Backtester`. This step simulates the strategy's performance and gathers relevant metrics.
        c.  **Fitness Calculation:** A fitness score is derived from the performance metrics obtained during backtesting. This score quantifies the quality of the strategy.
        d.  **Selection, Crossover, & Mutation:** Based on their fitness scores, the fittest individuals are selected to become parents for the next generation. Genetic operators like `crossover` (where parts of two parent genomes are swapped) and `mutate` (where random changes are introduced into a genome) are applied to these parents to create new offspring, forming the next generation's population.

4.  **Phenotype (`AstNode`):** The functional representation of a strategy is a tree composed of `AstNode` enums. These nodes represent various elements of a strategy, such as conditional rules (`IF...THEN`), function calls (e.g., `RSI(14)` for a Relative Strength Index indicator with a period of 14), and constant values.

5.  **Result:** Throughout the evolutionary process, the best-performing strategies discovered are stored in a `HallOfFame`. At the conclusion of the process, these top strategies are returned as the output of the generation engine.

This architectural design effectively separates the concerns of genetic manipulation (handled by the `EvolutionEngine` and `operators.rs` through simple vector operations) from the intricate process of constructing valid and meaningful trading strategies (managed by the `SemanticMapper`).

## Key Files and Components:

*   **`src/engines/generation/evolution_engine.rs`**: The central orchestrator of the strategy generation process, containing the main genetic algorithm loop.
*   **`src/engines/generation/genome.rs`**: Defines the genetic representation of a strategy as a `Vec<u32>`.
*   **`src/engines/generation/semantic_mapper.rs`**: Translates the genotype (`Genome`) into the phenotype (`StrategyAST`).
*   **`src/types.rs`**: Defines the structure of a strategy's Abstract Syntax Tree (`AstNode`).
*   **`src/ui/services/evolution_runner.rs`**: Integrates the generation engine into the application, handling configuration and execution.
*   **`src/engines/generation/operators.rs`**: Provides genetic operators like crossover, mutation, and selection.