// icn-ccl/src/ast.rs
use serde::{Serialize, Deserialize};
// Potentially use types from icn_common like Did if they appear in AST
// use icn_common::Did;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    Policy(Vec<PolicyStatementNode>),
    FunctionDefinition {
        name: String,
        parameters: Vec<ParameterNode>,
        return_type: TypeAnnotationNode,
        body: BlockNode,
    },
    RuleDefinition {
        name: String,
        condition: ExpressionNode,
        action: ActionNode,
    },
    // ... other top-level nodes
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyStatementNode {
    FunctionDef(AstNode), // Using AstNode::FunctionDefinition
    RuleDef(AstNode),     // Using AstNode::RuleDefinition
    Import { path: String, alias: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterNode {
    pub name: String,
    pub type_ann: TypeAnnotationNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotationNode {
    Mana,
    Bool,
    Did, // Example, might be a string or a specific DID type
    String,
    Integer,
    Custom(String), // For user-defined types or imported ones
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BlockNode {
    pub statements: Vec<StatementNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatementNode {
    Let {
        name: String,
        value: ExpressionNode,
    },
    ExpressionStatement(ExpressionNode),
    Return(ExpressionNode),
    // ... other statement types (if, loop, etc.)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionNode {
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    StringLiteral(String),
    Identifier(String),
    FunctionCall {
        name: String,
        arguments: Vec<ExpressionNode>,
    },
    BinaryOp {
        left: Box<ExpressionNode>,
        operator: BinaryOperator,
        right: Box<ExpressionNode>,
    },
    // ... other expression types (unary op, member access, etc.)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionNode {
    Allow,
    Deny,
    Charge(ExpressionNode), // e.g., charge actor.mana(amount_expr)
    // ... other policy-specific actions
}

// Helper function to convert Pest pairs to AST (simplified example)
// pub fn pair_to_ast(pair: pest::iterators::Pair<Rule>) -> Result<AstNode, Error> {
//     match pair.as_rule() {
//         // ... conversion logic ...
//         _ => unimplemented!("Rule not implemented in AST conversion: {:?}", pair.as_rule()),
//     }
// } 