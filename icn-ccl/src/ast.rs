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
    Block(BlockNode),
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
    Did, // Decentralized identifier type
    String,
    Integer,
    Array(Box<TypeAnnotationNode>), // Arrays of any type, e.g., Array<Integer>
    Proposal,                       // Governance proposal type
    Vote,                           // Vote type for governance
    Custom(String),                 // For user-defined types or imported ones
}

impl TypeAnnotationNode {
    /// Returns true if two types are considered compatible.
    ///
    /// Currently `Mana` and `Integer` are treated as interchangeable
    /// since they share the same underlying WASM representation.
    pub fn compatible_with(&self, other: &Self) -> bool {
        self == other
            || matches!(
                (self, other),
                (TypeAnnotationNode::Mana, TypeAnnotationNode::Integer)
                    | (TypeAnnotationNode::Integer, TypeAnnotationNode::Mana)
            )
    }

    /// Returns true if this type behaves like an integer number.
    pub fn is_numeric(&self) -> bool {
        matches!(self, TypeAnnotationNode::Integer | TypeAnnotationNode::Mana)
    }

    /// Returns true if this type can be stored/compared
    pub fn is_comparable(&self) -> bool {
        matches!(
            self,
            TypeAnnotationNode::Integer
                | TypeAnnotationNode::Mana
                | TypeAnnotationNode::Bool
                | TypeAnnotationNode::String
                | TypeAnnotationNode::Did
        )
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
    WhileLoop {
        condition: ExpressionNode,
        body: BlockNode,
    },
    // ... other statement types (loop, etc.)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionNode {
    IntegerLiteral(i64),
    BooleanLiteral(bool),
    StringLiteral(String),
    ArrayLiteral(Vec<ExpressionNode>), // [1, 2, 3] or ["a", "b", "c"]
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
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<ExpressionNode>,
    },
    ArrayAccess {
        array: Box<ExpressionNode>,
        index: Box<ExpressionNode>,
    },
    // ... other expression types (member access, etc.)
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
    Concat, // String concatenation: "hello" + " world"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not, // Logical negation: !true -> false
    Neg, // Arithmetic negation: -5 -> -5
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionNode {
    Allow,
    Deny,
    Charge(ExpressionNode), // e.g., charge actor.mana(amount_expr)
                            // ... other policy-specific actions
}

/// Converts a Pest `Pair` into an AST node.
pub fn pair_to_ast(
    pair: pest::iterators::Pair<crate::parser::Rule>,
) -> Result<AstNode, crate::error::CclError> {
    use crate::error::CclError;
    use crate::parser::{self, Rule};
    match pair.as_rule() {
        Rule::policy => {
            let mut items = Vec::new();
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::function_definition => {
                        items.push(PolicyStatementNode::FunctionDef(
                            parser::parse_function_definition(inner)?,
                        ));
                    }
                    Rule::policy_statement => {
                        let mut stmt_inner = inner.into_inner();
                        let stmt = stmt_inner.next().ok_or_else(|| {
                            CclError::ParsingError("Empty policy statement".to_string())
                        })?;
                        match stmt.as_rule() {
                            Rule::rule_definition => items.push(PolicyStatementNode::RuleDef(
                                parser::parse_rule_definition(stmt)?,
                            )),
                            Rule::import_statement => {
                                let mut i = stmt.into_inner();
                                let path_pair = i.next().ok_or_else(|| {
                                    CclError::ParsingError("Import missing path".to_string())
                                })?;
                                let alias_pair = i.next().ok_or_else(|| {
                                    CclError::ParsingError("Import missing alias".to_string())
                                })?;
                                let path = path_pair.as_str().trim_matches('"').to_string();
                                let alias = alias_pair.as_str().to_string();
                                items.push(PolicyStatementNode::Import { path, alias });
                            }
                            _ => {
                                return Err(CclError::ParsingError(format!(
                                    "Unexpected policy statement: {:?}",
                                    stmt.as_rule()
                                )));
                            }
                        }
                    }
                    Rule::EOI => (),
                    _ => {
                        return Err(CclError::ParsingError(format!(
                            "Unexpected rule in policy: {:?}",
                            inner.as_rule()
                        )));
                    }
                }
            }
            Ok(AstNode::Policy(items))
        }
        Rule::function_definition => parser::parse_function_definition(pair),
        Rule::rule_definition => parser::parse_rule_definition(pair),
        Rule::import_statement => {
            let mut i = pair.into_inner();
            let path_pair = i
                .next()
                .ok_or_else(|| CclError::ParsingError("Import missing path".to_string()))?;
            let alias_pair = i
                .next()
                .ok_or_else(|| CclError::ParsingError("Import missing alias".to_string()))?;
            let raw_path = path_pair.as_str().trim_matches('"');
            let path = crate::parser::unescape_string(raw_path)?;
            let alias = alias_pair.as_str().to_string();
            Ok(AstNode::Policy(vec![PolicyStatementNode::Import {
                path,
                alias,
            }]))
        }
        Rule::block => Ok(AstNode::Block(parser::parse_block(pair)?)),
        Rule::policy_statement => match parser::parse_policy_statement(pair)? {
            PolicyStatementNode::RuleDef(rule) => Ok(rule),
            PolicyStatementNode::Import { path, alias } => {
                Ok(AstNode::Policy(vec![PolicyStatementNode::Import {
                    path,
                    alias,
                }]))
            }
            PolicyStatementNode::FunctionDef(func) => Ok(func),
        },
        Rule::statement => Ok(AstNode::Block(BlockNode {
            statements: vec![parser::parse_statement(pair)?],
        })),
        _ => unreachable!(
            "pair_to_ast called with unsupported rule: {:?}",
            pair.as_rule()
        ),
    }
}
