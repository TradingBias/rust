pub mod ast;
pub mod semantic_mapper;
pub mod operators;
pub mod hall_of_fame;
pub mod evolution_engine;
pub mod progress;
pub mod gene_consumer;
pub mod diversity_validator;
pub mod lightweight_validator;
pub mod optimisation;
pub mod genome;
pub mod pareto;

pub use genome::Genome;
pub use ast::*;
pub use hall_of_fame::{HallOfFame, EliteStrategy};
pub use evolution_engine::{EvolutionEngine, EvolutionConfig, ProgressCallback};
pub use progress::{ConsoleProgressCallback, IpcProgressCallback};
pub use semantic_mapper::SemanticMapper;
pub use diversity_validator::DiversityValidator;
pub use lightweight_validator::LightweightValidator;
pub use pareto::{ObjectiveConfig, OptimizationDirection};
pub use optimisation::{
    methods::{
        base::{ValidationMethod, AggregatedResult, ValidationResult},
        wfo::WalkForwardMethod,
    },
    splitters::{
        base::DataSplitter,
        simple::SimpleSplitter,
        wfo::WalkForwardSplitter,
        types::{DataSplit, SplitConfig, WindowType},
    },
};
