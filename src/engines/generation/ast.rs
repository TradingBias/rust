use crate::types::AstNode;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct StrategyMetadata {
    pub source: String,
    pub generation: usize,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct StrategyAST {
    pub root: Box<AstNode>,
    pub metadata: StrategyMetadata,
}

impl StrategyAST {
    pub fn as_node(&self) -> &AstNode {
        &self.root
    }
}
