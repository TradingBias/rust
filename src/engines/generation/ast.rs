use crate::types::AstNode;

#[derive(Debug, Clone)]
pub enum StrategyAST {
    Rule {
        condition: Box<AstNode>,
        action: Box<AstNode>,
    },
}
