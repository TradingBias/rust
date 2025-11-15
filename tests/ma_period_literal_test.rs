use tradebias::{
    data::IndicatorCache,
    engines::generation::{
        semantic_mapper::SemanticMapper,
        gene_consumer::GeneConsumer,
    },
    engines::evaluation::backtester::Backtester,
    functions::registry::FunctionRegistry,
    types::AstNode,
};
use polars::prelude::*;
use std::sync::Arc;

#[test]
fn test_bears_indicator_period() {
    // Create test data
    let df = df! {
        "open" => &[100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0,
                    110.0, 111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0],
        "high" => &[101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0, 109.0, 110.0,
                    111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0, 119.0, 120.0],
        "low" => &[99.0, 100.0, 101.0, 102.0, 103.0, 104.0, 105.0, 106.0, 107.0, 108.0,
                   109.0, 110.0, 111.0, 112.0, 113.0, 114.0, 115.0, 116.0, 117.0, 118.0],
        "close" => &[100.5, 101.5, 102.5, 103.5, 104.5, 105.5, 106.5, 107.5, 108.5, 109.5,
                     110.5, 111.5, 112.5, 113.5, 114.5, 115.5, 116.5, 117.5, 118.5, 119.5],
        "volume" => &[1000.0, 1100.0, 1200.0, 1300.0, 1400.0, 1500.0, 1600.0, 1700.0, 1800.0, 1900.0,
                      2000.0, 2100.0, 2200.0, 2300.0, 2400.0, 2500.0, 2600.0, 2700.0, 2800.0, 2900.0],
    }
    .unwrap();

    let registry = Arc::new(FunctionRegistry::new());
    let cache = Arc::new(IndicatorCache::new(100));

    // Generate many strategies to try to trigger the error
    for seed in 0..100 {
        let genome = create_test_genome_with_seed(seed);

        let mapper = SemanticMapper::new(registry.clone(), 5);
        let ast_result = mapper.create_strategy_ast(&genome);

        if let Ok(ast) = ast_result {
            println!("Seed {}: Testing strategy...", seed);
            print_ast_debug(&ast.root, 0);

            let backtester = Backtester::new(registry.clone(), cache.clone(), 10000.0);
            let result = backtester.run(&ast, &df);

            match result {
                Ok(_) => println!("  ✓ Success"),
                Err(e) => {
                    println!("  ✗ Error: {}", e);
                    if e.to_string().contains("MA period must be an integer literal") {
                        println!("\n=== REPRODUCED THE BUG ===");
                        println!("Seed: {}", seed);
                        println!("AST:");
                        print_ast_debug(&ast.root, 0);
                        panic!("MA period literal error reproduced!");
                    }
                }
            }
        }
    }
}

fn create_test_genome_with_seed(seed: u32) -> Vec<u32> {
    // Create a genome that increases the chances of generating Bears/Bulls
    let mut genome = Vec::new();
    let mut rng_state = seed;

    for _ in 0..200 {
        // Simple LCG random number generator
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        genome.push(rng_state);
    }

    genome
}

fn print_ast_debug(node: &AstNode, indent: usize) {
    let prefix = "  ".repeat(indent);
    match node {
        AstNode::Const(val) => println!("{}Const({:?})", prefix, val),
        AstNode::Call { function, args } => {
            println!("{}Call({})", prefix, function);
            for arg in args {
                print_ast_debug(arg, indent + 1);
            }
        }
        AstNode::Rule { condition, action } => {
            println!("{}Rule", prefix);
            println!("{}  Condition:", prefix);
            print_ast_debug(condition, indent + 2);
            println!("{}  Action:", prefix);
            print_ast_debug(action, indent + 2);
        }
    }
}
