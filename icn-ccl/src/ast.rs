// icn-ccl/src/ast.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    // Top-level program node
    Program(Vec<TopLevelNode>),
    
    // Legacy support for simple policies
    Policy(Vec<PolicyStatementNode>),
    
    // New CCL 0.1 nodes
    ContractDeclaration {
        name: String,
        metadata: Vec<ContractMetaNode>,
        body: Vec<ContractBodyNode>,
    },
    
    RoleDeclaration {
        name: String,
        extends: Option<String>,
        body: Vec<RoleBodyNode>,
    },
    
    ProposalDeclaration {
        name: String,
        fields: Vec<ProposalFieldNode>,
    },
    
    FunctionDefinition {
        name: String,
        type_parameters: Vec<TypeParameterNode>,
        parameters: Vec<ParameterNode>,
        return_type: Option<TypeExprNode>,
        body: BlockNode,
    },
    
    StructDefinition {
        name: String,
        type_parameters: Vec<TypeParameterNode>,
        fields: Vec<FieldNode>,
    },
    
    EnumDefinition {
        name: String,
        type_parameters: Vec<TypeParameterNode>,
        variants: Vec<EnumVariantNode>,
    },
    
    StateDeclaration {
        name: String,
        type_expr: TypeExprNode,
        initial_value: Option<ExpressionNode>,
    },
    
    ConstDeclaration {
        name: String,
        type_expr: TypeExprNode,
        value: ExpressionNode,
    },
    
    ImportStatement {
        path: String,
        alias: Option<String>,
    },
    
    Block(BlockNode),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TopLevelNode {
    Import(ImportStatementNode),
    Contract(ContractDeclarationNode),
    Function(FunctionDeclarationNode),
    Struct(StructDeclarationNode),
    Enum(EnumDeclarationNode),
    Const(ConstDeclarationNode),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportStatementNode {
    pub path: String,
    pub alias: Option<String>,
}

// Type aliases for top-level nodes (reusing contract body node types)
pub type FunctionDeclarationNode = FunctionDefinitionNode;
pub type StructDeclarationNode = StructDefinitionNode;
pub type EnumDeclarationNode = EnumDefinitionNode;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractDeclarationNode {
    pub name: String,
    pub metadata: Vec<ContractMetaNode>,
    pub body: Vec<ContractBodyNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContractMetaNode {
    Scope(String),
    Version(String),
    Extends(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContractBodyNode {
    Role(RoleDeclarationNode),
    Proposal(ProposalDeclarationNode),
    Function(FunctionDefinitionNode),
    State(StateDeclarationNode),
    Struct(StructDefinitionNode),
    Enum(EnumDefinitionNode),
    Const(ConstDeclarationNode),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoleDeclarationNode {
    pub name: String,
    pub extends: Option<String>,
    pub body: Vec<RoleBodyNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RoleBodyNode {
    Can(Vec<String>),
    Requires(Vec<RequirementNode>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementNode {
    pub name: String,
    pub expr: ExpressionNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProposalDeclarationNode {
    pub name: String,
    pub fields: Vec<ProposalFieldNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProposalFieldNode {
    Description(String),
    Eligible(String),
    Duration(DurationExprNode),
    Quorum(u32), // percentage
    Threshold(ThresholdTypeNode),
    Execution(BlockNode),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThresholdTypeNode {
    Majority,
    Supermajority { numerator: u32, denominator: u32 },
    Consensus,
    Unanimous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DurationExprNode {
    pub value: i64,
    pub unit: DurationUnitNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DurationUnitNode {
    Days,
    Hours,
    Minutes,
    Seconds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDefinitionNode {
    pub name: String,
    pub type_parameters: Vec<TypeParameterNode>,
    pub parameters: Vec<ParameterNode>,
    pub return_type: Option<TypeExprNode>,
    pub body: BlockNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldNode {
    pub name: String,
    pub type_expr: TypeExprNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariantNode {
    pub name: String,
    pub type_expr: Option<TypeExprNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateDeclarationNode {
    pub name: String,
    pub type_expr: TypeExprNode,
    pub initial_value: Option<ExpressionNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConstDeclarationNode {
    pub name: String,
    pub type_expr: TypeExprNode,
    pub value: ExpressionNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructDefinitionNode {
    pub name: String,
    pub type_parameters: Vec<TypeParameterNode>,
    pub fields: Vec<FieldNode>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDefinitionNode {
    pub name: String,
    pub type_parameters: Vec<TypeParameterNode>,
    pub variants: Vec<EnumVariantNode>,
}

// Legacy policy statement nodes for backward compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyStatementNode {
    FunctionDef(AstNode), // Using AstNode::FunctionDefinition
    RuleDef(AstNode),     // Using AstNode::RuleDefinition
    StructDef(AstNode),
    Import { path: String, alias: String },
    ConstDef { name: String, value: ExpressionNode, type_ann: TypeAnnotationNode },
    MacroDef { name: String, params: Vec<String>, body: String },
    
    // Governance DSL statements
    EventDef { 
        name: String, 
        fields: Vec<(String, TypeAnnotationNode)> 
    },
    StateDef { 
        name: String, 
        type_ann: TypeAnnotationNode, 
        initial_value: Option<ExpressionNode> 
    },
    TriggerDef { 
        name: String, 
        condition: ExpressionNode, 
        action: ExpressionNode 
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterNode {
    pub name: String,
    pub type_expr: TypeExprNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypeParameterNode {
    pub name: String,
    pub bounds: Vec<String>, // Type constraints like "Clone + Debug"
}

// New unified type system for CCL 0.1
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeExprNode {
    // Basic types
    Integer,
    String,
    Boolean,
    Mana,
    Did,
    Timestamp,
    Duration,
    
    // Compound types
    Array(Box<TypeExprNode>),
    Map { key_type: Box<TypeExprNode>, value_type: Box<TypeExprNode> },
    Option(Box<TypeExprNode>),
    Result { ok_type: Box<TypeExprNode>, err_type: Box<TypeExprNode> },
    
    // Custom types
    Custom(String),
    
    // Generic types
    TypeParameter(String), // Reference to a type parameter like T
    GenericInstantiation { 
        base_type: String, 
        type_args: Vec<TypeExprNode> 
    }, // Like Vec<T> or Map<K, V>
}

// Legacy type annotation for backward compatibility
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotationNode {
    Mana,
    Bool,
    Did,
    String,
    Integer,
    Array(Box<TypeAnnotationNode>),
    Map {
        key_type: Box<TypeAnnotationNode>,
        value_type: Box<TypeAnnotationNode>,
    },
    Proposal,
    Vote,
    Option(Box<TypeAnnotationNode>),
    Result {
        ok_type: Box<TypeAnnotationNode>,
        err_type: Box<TypeAnnotationNode>,
    },
    Custom(String),
}

impl TypeExprNode {
    /// Convert to legacy TypeAnnotationNode for backward compatibility
    pub fn to_type_annotation(&self) -> TypeAnnotationNode {
        match self {
            TypeExprNode::Integer => TypeAnnotationNode::Integer,
            TypeExprNode::String => TypeAnnotationNode::String,
            TypeExprNode::Boolean => TypeAnnotationNode::Bool,
            TypeExprNode::Mana => TypeAnnotationNode::Mana,
            TypeExprNode::Did => TypeAnnotationNode::Did,
            TypeExprNode::Array(inner) => TypeAnnotationNode::Array(Box::new(inner.to_type_annotation())),
            TypeExprNode::Map { key_type, value_type } => TypeAnnotationNode::Map {
                key_type: Box::new(key_type.to_type_annotation()),
                value_type: Box::new(value_type.to_type_annotation()),
            },
            TypeExprNode::Option(inner) => TypeAnnotationNode::Option(Box::new(inner.to_type_annotation())),
            TypeExprNode::Result { ok_type, err_type } => TypeAnnotationNode::Result {
                ok_type: Box::new(ok_type.to_type_annotation()),
                err_type: Box::new(err_type.to_type_annotation()),
            },
            TypeExprNode::Custom(name) => TypeAnnotationNode::Custom(name.clone()),
            TypeExprNode::Timestamp => TypeAnnotationNode::Custom("Timestamp".to_string()),
            TypeExprNode::Duration => TypeAnnotationNode::Custom("Duration".to_string()),
            TypeExprNode::TypeParameter(name) => TypeAnnotationNode::Custom(name.clone()),
            TypeExprNode::GenericInstantiation { base_type, .. } => TypeAnnotationNode::Custom(base_type.clone()),
        }
    }
    
    /// Returns true if two types are considered compatible.
    pub fn compatible_with(&self, other: &Self) -> bool {
        self == other
            || matches!(
                (self, other),
                (TypeExprNode::Mana, TypeExprNode::Integer)
                    | (TypeExprNode::Integer, TypeExprNode::Mana)
            )
    }

    /// Returns true if this type behaves like an integer number.
    pub fn is_numeric(&self) -> bool {
        matches!(self, TypeExprNode::Integer | TypeExprNode::Mana)
    }
}

impl TypeAnnotationNode {
    /// Returns true if two types are considered compatible.
    pub fn compatible_with(&self, other: &Self) -> bool {
        self == other
            || matches!(
                (self, other),
                (TypeAnnotationNode::Mana, TypeAnnotationNode::Integer)
                    | (TypeAnnotationNode::Integer, TypeAnnotationNode::Mana)
            )
            || matches!(self, TypeAnnotationNode::Custom(t) if t == "Any")
            || matches!(other, TypeAnnotationNode::Custom(t) if t == "Any")
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
    // Updated statements for CCL 0.1
    Let {
        mutable: bool,
        name: String,
        type_expr: Option<TypeExprNode>,
        value: ExpressionNode,
    },
    Assignment {
        lvalue: LValueNode,
        value: ExpressionNode,
    },
    If {
        condition: ExpressionNode,
        then_block: BlockNode,
        else_ifs: Vec<(ExpressionNode, BlockNode)>,
        else_block: Option<BlockNode>,
    },
    While {
        condition: ExpressionNode,
        body: BlockNode,
    },
    For {
        iterator: String,
        iterable: ExpressionNode,
        body: BlockNode,
    },
    Match {
        expr: ExpressionNode,
        arms: Vec<MatchArmNode>,
    },
    Return(Option<ExpressionNode>),
    Break,
    Continue,
    Emit {
        event_name: String,
        fields: Vec<FieldInitNode>,
    },
    Require(ExpressionNode),
    ExpressionStatement(ExpressionNode),
    
    // Legacy statements for backward compatibility
    WhileLoop {
        condition: ExpressionNode,
        body: BlockNode,
    },
    ForLoop {
        iterator: String,
        iterable: ExpressionNode,
        body: BlockNode,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LValueNode {
    Identifier(String),
    MemberAccess {
        object: Box<ExpressionNode>,
        member: String,
    },
    IndexAccess {
        object: Box<ExpressionNode>,
        index: Box<ExpressionNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArmNode {
    pub pattern: PatternNode,
    pub guard: Option<ExpressionNode>, // Optional guard condition
    pub body: ExpressionNode, // Simplified: match arms return expressions
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternNode {
    Literal(LiteralNode),
    Variable(String),
    Wildcard,
    Struct {
        name: String,
        fields: Vec<StructFieldPattern>,
    },
    Enum {
        type_name: String,
        variant: String,
        inner: Option<Box<PatternNode>>,
    },
    Tuple(Vec<PatternNode>),
    Array(Vec<PatternNode>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructFieldPattern {
    pub name: String,
    pub pattern: Option<PatternNode>, // None means shorthand syntax
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldInitNode {
    pub name: String,
    pub value: ExpressionNode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpressionNode {
    // Literals
    Literal(LiteralNode),
    
    // Variables and functions
    Identifier(String),
    FunctionCall {
        name: String,
        args: Vec<ExpressionNode>,
    },
    MethodCall {
        object: Box<ExpressionNode>,
        method: String,
        args: Vec<ExpressionNode>,
    },
    
    // Binary operations
    BinaryOp {
        left: Box<ExpressionNode>,
        operator: BinaryOperator,
        right: Box<ExpressionNode>,
    },
    
    // Unary operations
    UnaryOp {
        operator: UnaryOperator,
        operand: Box<ExpressionNode>,
    },
    
    // Member access and indexing
    MemberAccess {
        object: Box<ExpressionNode>,
        member: String,
    },
    IndexAccess {
        object: Box<ExpressionNode>,
        index: Box<ExpressionNode>,
    },
    
    // Compound expressions
    ArrayLiteral(Vec<ExpressionNode>),
    MapLiteral(Vec<(ExpressionNode, ExpressionNode)>), // Key-value pairs
    StructLiteral {
        type_name: String,
        fields: Vec<FieldInitNode>,
    },
    EnumValue {
        enum_name: String,
        variant: String,
    },
    
    // Option and Result expressions
    Some(Box<ExpressionNode>),
    None,
    Ok(Box<ExpressionNode>),
    Err(Box<ExpressionNode>),
    
    // Special governance expressions
    Transfer {
        from: Box<ExpressionNode>,
        to: Box<ExpressionNode>,
        amount: Box<ExpressionNode>,
    },
    Mint {
        to: Box<ExpressionNode>,
        amount: Box<ExpressionNode>,
    },
    Burn {
        from: Box<ExpressionNode>,
        amount: Box<ExpressionNode>,
    },
    
    // Match expressions
    Match {
        expr: Box<ExpressionNode>,
        arms: Vec<MatchArmNode>,
    },
    
    // Legacy expressions
    IntegerLiteral(i64),
    StringLiteral(String),
    BooleanLiteral(bool),
    ArrayAccess {
        array: Box<ExpressionNode>,
        index: Box<ExpressionNode>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralNode {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Did(String),
    Timestamp(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
    Concat, // String concatenation
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not, // Logical negation
    Neg, // Arithmetic negation
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ActionNode {
    Allow,
    Deny,
    Charge(ExpressionNode),
}

/// Converts a Pest `Pair` into an AST node.
pub fn pair_to_ast(
    pair: pest::iterators::Pair<crate::parser::Rule>,
) -> Result<AstNode, crate::error::CclError> {
    use crate::error::CclError;
    use crate::parser::{self, Rule};
    
    match pair.as_rule() {
        Rule::program => {
            let mut items = Vec::new();
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::import_stmt => {
                        // Parse import statement
                        let import = parser::parse_import_statement(inner)?;
                        items.push(TopLevelNode::Import(import));
                    }
                    Rule::contract_decl => {
                        // Parse contract declaration
                        let contract = parser::parse_contract_declaration(inner)?;
                        items.push(TopLevelNode::Contract(contract));
                    }
                    Rule::fn_decl => {
                        // Parse standalone function declaration
                        let function = parser::parse_function_definition_new(inner)?;
                        items.push(TopLevelNode::Function(function));
                    }
                    Rule::struct_decl => {
                        // Parse standalone struct declaration
                        let struct_def = parser::parse_struct_declaration(inner)?;
                        items.push(TopLevelNode::Struct(struct_def));
                    }
                    Rule::enum_decl => {
                        // Parse standalone enum declaration
                        let enum_def = parser::parse_enum_declaration(inner)?;
                        items.push(TopLevelNode::Enum(enum_def));
                    }
                    // Rule::const_decl => {
                    //     // Parse standalone const declaration
                    //     let const_def = parser::parse_const_declaration(inner)?;
                    //     items.push(TopLevelNode::Const(const_def));
                    // }
                    Rule::EOI => (),
                    _ => {
                        return Err(CclError::ParsingError(format!(
                            "Unexpected rule in program: {:?}",
                            inner.as_rule()
                        )));
                    }
                }
            }
            Ok(AstNode::Program(items))
        }
        // Legacy support
        Rule::fn_decl => {
            let func = parser::parse_function_definition_new(pair)?;
            Ok(AstNode::FunctionDefinition {
                name: func.name,
                type_parameters: func.type_parameters,
                parameters: func.parameters,
                return_type: func.return_type,
                body: func.body,
            })
        }
        _ => {
            Err(CclError::ParsingError(format!(
                "Unsupported top-level rule: {:?}",
                pair.as_rule()
            )))
        }
    }
}
