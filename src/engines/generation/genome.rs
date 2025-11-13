/// Genome representation for genetic programming
///
/// A genome is a sequence of integers that deterministically maps to a strategy AST.
/// Each gene (u32 value) is consumed sequentially by the SemanticMapper to make
/// decisions about strategy structure:
/// - Which indicator/primitive to use
/// - What parameter values to apply
/// - How to combine expressions
///
/// # Why use Genome instead of AST directly?
///
/// Genetic algorithms work best on simple, linear structures:
/// - **Crossover**: Swapping genome segments is trivial (array slicing)
/// - **Mutation**: Changing individual genes is straightforward
/// - **No invalid states**: Any genome can be mapped to a valid AST
///
/// In contrast, tree-based operations on ASTs often produce invalid strategies.
///
/// # Conversion
///
/// Use `SemanticMapper::create_strategy_ast()` to convert Genome -> StrategyAST
///
/// # Example
///
/// ```
/// let genome = vec![42, 17, 88, 3, 45, 12, 99];
/// // Via SemanticMapper, this might map to:
/// // if RSI(Close, 14) > 70 then OpenLong
/// ```
pub type Genome = Vec<u32>;
