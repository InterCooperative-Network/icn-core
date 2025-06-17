// icn-ccl/src/ast.rs
use serde::{Deserialize, Serialize};
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

impl TypeAnnotationNode {
    /// Returns true if two types are considered compatible.
    ///
    /// Currently `Mana` and `Integer` are treated as interchangeable
    /// since they share the same underlying WASM representation.
    pub fn compatible_with(&self, other: &Self) -> bool {
        self == other
            || matches!((self, other),
                (TypeAnnotationNode::Mana, TypeAnnotationNode::Integer)
                    | (TypeAnnotationNode::Integer, TypeAnnotationNode::Mana))
    }

    /// Returns true if this type behaves like an integer number.
    pub fn is_numeric(&self) -> bool {
        matches!(self, TypeAnnotationNode::Integer | TypeAnnotationNode::Mana)
    }
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
    If {
        condition: ExpressionNode,
        then_block: BlockNode,
        else_block: Option<BlockNode>,
    },
    // ... other statement types (loop, etc.)
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
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
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
