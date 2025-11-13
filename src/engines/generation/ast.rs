use crate::types::AstNode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyMetadata {
    pub source: String,
    pub generation: usize,
    pub properties: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyAST {
    pub root: Box<AstNode>,
    pub metadata: StrategyMetadata,
}

impl StrategyAST {
    pub fn as_node(&self) -> &AstNode {
        &self.root
    }
}
