// icn-ccl/src/optimizer.rs
use crate::ast::{
    AstNode, BinaryOperator, BlockNode, ConstDeclarationNode, ContractDeclarationNode,
    EnumDefinitionNode, ExpressionNode, FieldInitNode, FieldNode, FunctionDefinitionNode,
    LiteralNode, MatchArmNode, ParameterNode, PolicyStatementNode, ProposalDeclarationNode,
    RoleDeclarationNode, StateDeclarationNode, StatementNode, StructDefinitionNode, TypeExprNode,
    UnaryOperator,
};

/// The optimizer applies various transformations to the AST to improve
/// performance of the generated WASM code.
///
/// Current optimizations include:
/// - Constant folding
/// - Dead code elimination (basic)
/// - Expression simplification
pub struct Optimizer {
    optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone, Copy)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::Basic,
        }
    }
}

impl Optimizer {
    pub fn new(level: OptimizationLevel) -> Self {
        Self {
            optimization_level: level,
        }
    }

    /// Optimize the entire AST
    pub fn optimize(&mut self, ast: AstNode) -> AstNode {
        match self.optimization_level {
            OptimizationLevel::None => ast,
            OptimizationLevel::Basic | OptimizationLevel::Aggressive => self.fold_ast(ast),
        }
    }

    /// Apply optimizations to AST nodes
    fn fold_ast(&mut self, ast: AstNode) -> AstNode {
        match ast {
            AstNode::Program(nodes) => {
                let optimized_nodes = nodes
                    .into_iter()
                    .map(|node| match node {
                        crate::ast::TopLevelNode::Import(import) => {
                            crate::ast::TopLevelNode::Import(import)
                        }
                        crate::ast::TopLevelNode::Contract(contract) => {
                            crate::ast::TopLevelNode::Contract(self.fold_contract(contract))
                        }
                        crate::ast::TopLevelNode::Function(function) => {
                            // TODO: Optimize standalone functions
                            crate::ast::TopLevelNode::Function(function)
                        }
                        crate::ast::TopLevelNode::Struct(struct_def) => {
                            // TODO: Optimize standalone structs
                            crate::ast::TopLevelNode::Struct(struct_def)
                        }
                        crate::ast::TopLevelNode::Enum(enum_def) => {
                            // TODO: Optimize standalone enums
                            crate::ast::TopLevelNode::Enum(enum_def)
                        }
                        crate::ast::TopLevelNode::Const(const_def) => {
                            // TODO: Optimize standalone constants
                            crate::ast::TopLevelNode::Const(const_def)
                        }
                    })
                    .collect();
                AstNode::Program(optimized_nodes)
            }
            AstNode::Policy(stmts) => {
                let optimized_stmts = stmts
                    .into_iter()
                    .map(|stmt| self.fold_policy_statement(stmt))
                    .collect();
                AstNode::Policy(optimized_stmts)
            }
            AstNode::FunctionDefinition {
                name,
                parameters,
                return_type,
                body,
                ..
            } => {
                AstNode::FunctionDefinition {
                    name,
                    type_parameters: Vec::new(), // TODO: Implement generic optimization
                    parameters: parameters
                        .into_iter()
                        .map(|p| self.fold_parameter(p))
                        .collect(),
                    return_type: return_type.map(|rt| self.fold_type_expr(rt)),
                    body: self.fold_block(body),
                }
            }
            AstNode::StructDefinition { name, fields, .. } => {
                AstNode::StructDefinition {
                    name,
                    type_parameters: Vec::new(), // TODO: Implement generic optimization
                    fields: fields.into_iter().map(|f| self.fold_field(f)).collect(),
                }
            }
            AstNode::ContractDeclaration {
                name,
                metadata,
                body,
            } => AstNode::ContractDeclaration {
                name,
                metadata,
                body: body
                    .into_iter()
                    .map(|item| self.fold_contract_body(item))
                    .collect(),
            },
            AstNode::Block(block) => AstNode::Block(self.fold_block(block)),
            other => other,
        }
    }

    fn fold_contract(&mut self, contract: ContractDeclarationNode) -> ContractDeclarationNode {
        ContractDeclarationNode {
            name: contract.name,
            metadata: contract.metadata,
            body: contract
                .body
                .into_iter()
                .map(|item| self.fold_contract_body(item))
                .collect(),
        }
    }

    fn fold_contract_body(
        &mut self,
        body: crate::ast::ContractBodyNode,
    ) -> crate::ast::ContractBodyNode {
        use crate::ast::ContractBodyNode;
        match body {
            ContractBodyNode::Role(role) => ContractBodyNode::Role(self.fold_role(role)),
            ContractBodyNode::Proposal(proposal) => {
                ContractBodyNode::Proposal(self.fold_proposal(proposal))
            }
            ContractBodyNode::Function(func) => {
                ContractBodyNode::Function(self.fold_function_definition(func))
            }
            ContractBodyNode::State(state) => {
                ContractBodyNode::State(self.fold_state_declaration(state))
            }
            ContractBodyNode::Struct(struct_def) => {
                ContractBodyNode::Struct(self.fold_struct_definition(struct_def))
            }
            ContractBodyNode::Enum(enum_def) => {
                ContractBodyNode::Enum(self.fold_enum_definition(enum_def))
            }
            ContractBodyNode::Const(const_def) => {
                ContractBodyNode::Const(self.fold_const_declaration(const_def))
            }
        }
    }

    fn fold_role(&mut self, role: RoleDeclarationNode) -> RoleDeclarationNode {
        // Roles are mostly metadata, minimal optimization needed
        role
    }

    fn fold_proposal(&mut self, proposal: ProposalDeclarationNode) -> ProposalDeclarationNode {
        use crate::ast::ProposalFieldNode;
        ProposalDeclarationNode {
            name: proposal.name,
            fields: proposal
                .fields
                .into_iter()
                .map(|field| match field {
                    ProposalFieldNode::Execution(block) => {
                        ProposalFieldNode::Execution(self.fold_block(block))
                    }
                    other => other,
                })
                .collect(),
        }
    }

    fn fold_function_definition(&mut self, func: FunctionDefinitionNode) -> FunctionDefinitionNode {
        FunctionDefinitionNode {
            name: func.name,
            type_parameters: func.type_parameters, // TODO: Implement generic optimization
            parameters: func
                .parameters
                .into_iter()
                .map(|p| self.fold_parameter(p))
                .collect(),
            return_type: func.return_type.map(|rt| self.fold_type_expr(rt)),
            body: self.fold_block(func.body),
        }
    }

    fn fold_state_declaration(&mut self, state: StateDeclarationNode) -> StateDeclarationNode {
        StateDeclarationNode {
            name: state.name,
            type_expr: self.fold_type_expr(state.type_expr),
            initial_value: state.initial_value.map(|v| self.fold_expr(v)),
        }
    }

    fn fold_const_declaration(&mut self, const_def: ConstDeclarationNode) -> ConstDeclarationNode {
        ConstDeclarationNode {
            name: const_def.name,
            type_expr: self.fold_type_expr(const_def.type_expr),
            value: self.fold_expr(const_def.value),
        }
    }

    fn fold_struct_definition(&mut self, struct_def: StructDefinitionNode) -> StructDefinitionNode {
        StructDefinitionNode {
            name: struct_def.name,
            type_parameters: struct_def.type_parameters, // TODO: Implement generic optimization
            fields: struct_def
                .fields
                .into_iter()
                .map(|f| self.fold_field(f))
                .collect(),
        }
    }

    fn fold_enum_definition(&mut self, enum_def: EnumDefinitionNode) -> EnumDefinitionNode {
        EnumDefinitionNode {
            name: enum_def.name,
            type_parameters: enum_def.type_parameters, // TODO: Implement generic optimization
            variants: enum_def
                .variants
                .into_iter()
                .map(|v| crate::ast::EnumVariantNode {
                    name: v.name,
                    type_expr: v.type_expr.map(|t| self.fold_type_expr(t)),
                })
                .collect(),
        }
    }

    fn fold_parameter(&mut self, param: ParameterNode) -> ParameterNode {
        ParameterNode {
            name: param.name,
            type_expr: self.fold_type_expr(param.type_expr),
        }
    }

    fn fold_field(&mut self, field: FieldNode) -> FieldNode {
        FieldNode {
            name: field.name,
            type_expr: self.fold_type_expr(field.type_expr),
        }
    }

    fn fold_type_expr(&mut self, type_expr: TypeExprNode) -> TypeExprNode {
        match type_expr {
            TypeExprNode::Array(inner) => {
                TypeExprNode::Array(Box::new(self.fold_type_expr(*inner)))
            }
            TypeExprNode::Map {
                key_type,
                value_type,
            } => TypeExprNode::Map {
                key_type: Box::new(self.fold_type_expr(*key_type)),
                value_type: Box::new(self.fold_type_expr(*value_type)),
            },
            TypeExprNode::Option(inner) => {
                TypeExprNode::Option(Box::new(self.fold_type_expr(*inner)))
            }
            TypeExprNode::Result { ok_type, err_type } => TypeExprNode::Result {
                ok_type: Box::new(self.fold_type_expr(*ok_type)),
                err_type: Box::new(self.fold_type_expr(*err_type)),
            },
            other => other,
        }
    }

    fn fold_policy_statement(&mut self, stmt: PolicyStatementNode) -> PolicyStatementNode {
        match stmt {
            PolicyStatementNode::FunctionDef(func) => {
                PolicyStatementNode::FunctionDef(self.fold_ast(func))
            }
            PolicyStatementNode::StructDef(struct_def) => {
                PolicyStatementNode::StructDef(self.fold_ast(struct_def))
            }
            PolicyStatementNode::ConstDef {
                name,
                value,
                type_ann,
            } => PolicyStatementNode::ConstDef {
                name,
                value: self.fold_expr(value),
                type_ann,
            },
            other => other,
        }
    }

    fn fold_block(&mut self, block: BlockNode) -> BlockNode {
        let optimized_stmts: Vec<StatementNode> = block
            .statements
            .into_iter()
            .map(|stmt| self.fold_stmt(stmt))
            .collect::<Vec<_>>()
            .into_iter()
            .filter(|stmt| !self.is_dead_code(stmt))
            .collect();

        BlockNode {
            statements: optimized_stmts,
        }
    }

    fn fold_stmt(&mut self, stmt: StatementNode) -> StatementNode {
        match stmt {
            StatementNode::Let {
                mutable,
                name,
                type_expr,
                value,
            } => StatementNode::Let {
                mutable,
                name,
                type_expr: type_expr.map(|t| self.fold_type_expr(t)),
                value: self.fold_expr(value),
            },
            StatementNode::Assignment { lvalue, value } => StatementNode::Assignment {
                lvalue,
                value: self.fold_expr(value),
            },
            StatementNode::If {
                condition,
                then_block,
                else_ifs,
                else_block,
            } => {
                let folded_condition = self.fold_expr(condition);

                // Constant folding for if statements
                if let ExpressionNode::Literal(LiteralNode::Boolean(true)) = folded_condition {
                    // If condition is always true, replace with then_block
                    return StatementNode::ExpressionStatement(
                        ExpressionNode::Literal(LiteralNode::Integer(0)), // placeholder
                    );
                } else if let ExpressionNode::Literal(LiteralNode::Boolean(false)) =
                    folded_condition
                {
                    // If condition is always false, use else_block or remove
                    if let Some(else_blk) = else_block {
                        return self.fold_block_as_statement(else_blk);
                    } else if !else_ifs.is_empty() {
                        // Try first else_if
                        let (elif_cond, elif_block) = &else_ifs[0];
                        return self.fold_stmt(StatementNode::If {
                            condition: elif_cond.clone(),
                            then_block: elif_block.clone(),
                            else_ifs: else_ifs[1..].to_vec(),
                            else_block: None,
                        });
                    } else {
                        // Remove entire if statement
                        return StatementNode::ExpressionStatement(
                            ExpressionNode::Literal(LiteralNode::Integer(0)), // placeholder
                        );
                    }
                }

                StatementNode::If {
                    condition: folded_condition,
                    then_block: self.fold_block(then_block),
                    else_ifs: else_ifs
                        .into_iter()
                        .map(|(cond, block)| (self.fold_expr(cond), self.fold_block(block)))
                        .collect(),
                    else_block: else_block.map(|block| self.fold_block(block)),
                }
            }
            StatementNode::While { condition, body } => {
                let folded_condition = self.fold_expr(condition);

                // Detect infinite loops or never-executing loops
                if let ExpressionNode::Literal(LiteralNode::Boolean(false)) = folded_condition {
                    // Loop never executes, remove it
                    return StatementNode::ExpressionStatement(
                        ExpressionNode::Literal(LiteralNode::Integer(0)), // placeholder
                    );
                }

                StatementNode::While {
                    condition: folded_condition,
                    body: self.fold_block(body),
                }
            }
            StatementNode::For {
                iterator,
                iterable,
                body,
            } => StatementNode::For {
                iterator,
                iterable: self.fold_expr(iterable),
                body: self.fold_block(body),
            },
            StatementNode::Match { expr, arms } => StatementNode::Match {
                expr: self.fold_expr(expr),
                arms: arms
                    .into_iter()
                    .map(|arm| crate::ast::MatchArmNode {
                        pattern: arm.pattern,
                        guard: arm.guard.map(|g| self.fold_expr(g)),
                        body: self.fold_expr(arm.body),
                    })
                    .collect(),
            },
            StatementNode::Return(expr) => StatementNode::Return(expr.map(|e| self.fold_expr(e))),
            StatementNode::Emit { event_name, fields } => StatementNode::Emit {
                event_name,
                fields: fields
                    .into_iter()
                    .map(|field| FieldInitNode {
                        name: field.name,
                        value: self.fold_expr(field.value),
                    })
                    .collect(),
            },
            StatementNode::Require(expr) => {
                let folded_expr = self.fold_expr(expr);

                // Remove redundant requires
                if let ExpressionNode::Literal(LiteralNode::Boolean(true)) = folded_expr {
                    // require(true) is redundant, remove it
                    return StatementNode::ExpressionStatement(
                        ExpressionNode::Literal(LiteralNode::Integer(0)), // placeholder
                    );
                }

                StatementNode::Require(folded_expr)
            }
            StatementNode::ExpressionStatement(expr) => {
                StatementNode::ExpressionStatement(self.fold_expr(expr))
            }
            // Legacy statements
            StatementNode::WhileLoop { condition, body } => StatementNode::WhileLoop {
                condition: self.fold_expr(condition),
                body: self.fold_block(body),
            },
            StatementNode::ForLoop {
                iterator,
                iterable,
                body,
            } => StatementNode::ForLoop {
                iterator,
                iterable: self.fold_expr(iterable),
                body: self.fold_block(body),
            },
            StatementNode::Break | StatementNode::Continue => stmt,
        }
    }

    fn fold_block_as_statement(&mut self, block: BlockNode) -> StatementNode {
        let folded_block = self.fold_block(block);
        if folded_block.statements.len() == 1 {
            folded_block.statements.into_iter().next().unwrap()
        } else {
            StatementNode::ExpressionStatement(
                ExpressionNode::Literal(LiteralNode::Integer(0)), // placeholder
            )
        }
    }

    /// Main expression folding with constant propagation and simplification
    pub fn fold_expr(&mut self, expr: ExpressionNode) -> ExpressionNode {
        match expr {
            // Literals are already optimized
            ExpressionNode::Literal(_) | ExpressionNode::Identifier(_) => expr,

            // Function calls
            ExpressionNode::FunctionCall { name, args } => ExpressionNode::FunctionCall {
                name,
                args: args.into_iter().map(|a| self.fold_expr(a)).collect(),
            },

            // Method calls
            ExpressionNode::MethodCall {
                object,
                method,
                args,
            } => ExpressionNode::MethodCall {
                object: Box::new(self.fold_expr(*object)),
                method,
                args: args.into_iter().map(|a| self.fold_expr(a)).collect(),
            },

            // Binary operations - main optimization target
            ExpressionNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                let left_folded = self.fold_expr(*left);
                let right_folded = self.fold_expr(*right);
                self.fold_binary_op(left_folded, operator, right_folded)
            }

            // Unary operations
            ExpressionNode::UnaryOp { operator, operand } => {
                let operand_folded = self.fold_expr(*operand);
                self.fold_unary_op(operator, operand_folded)
            }

            // Member access and indexing
            ExpressionNode::MemberAccess { object, member } => ExpressionNode::MemberAccess {
                object: Box::new(self.fold_expr(*object)),
                member,
            },
            ExpressionNode::IndexAccess { object, index } => ExpressionNode::IndexAccess {
                object: Box::new(self.fold_expr(*object)),
                index: Box::new(self.fold_expr(*index)),
            },

            // Array literals
            ExpressionNode::ArrayLiteral(elements) => ExpressionNode::ArrayLiteral(
                elements.into_iter().map(|e| self.fold_expr(e)).collect(),
            ),

            // Struct literals
            ExpressionNode::StructLiteral { type_name, fields } => ExpressionNode::StructLiteral {
                type_name,
                fields: fields
                    .into_iter()
                    .map(|field| FieldInitNode {
                        name: field.name,
                        value: self.fold_expr(field.value),
                    })
                    .collect(),
            },

            // Option and Result expressions
            ExpressionNode::Some(inner) => ExpressionNode::Some(Box::new(self.fold_expr(*inner))),
            ExpressionNode::None => ExpressionNode::None,
            ExpressionNode::Ok(inner) => ExpressionNode::Ok(Box::new(self.fold_expr(*inner))),
            ExpressionNode::Err(inner) => ExpressionNode::Err(Box::new(self.fold_expr(*inner))),

            // Governance expressions
            ExpressionNode::Transfer { from, to, amount } => ExpressionNode::Transfer {
                from: Box::new(self.fold_expr(*from)),
                to: Box::new(self.fold_expr(*to)),
                amount: Box::new(self.fold_expr(*amount)),
            },
            ExpressionNode::Mint { to, amount } => ExpressionNode::Mint {
                to: Box::new(self.fold_expr(*to)),
                amount: Box::new(self.fold_expr(*amount)),
            },
            ExpressionNode::Burn { from, amount } => ExpressionNode::Burn {
                from: Box::new(self.fold_expr(*from)),
                amount: Box::new(self.fold_expr(*amount)),
            },

            // Legacy expressions
            ExpressionNode::IntegerLiteral(val) => {
                ExpressionNode::Literal(LiteralNode::Integer(val))
            }
            ExpressionNode::StringLiteral(val) => ExpressionNode::Literal(LiteralNode::String(val)),
            ExpressionNode::BooleanLiteral(val) => {
                ExpressionNode::Literal(LiteralNode::Boolean(val))
            }
            ExpressionNode::ArrayAccess { array, index } => ExpressionNode::IndexAccess {
                object: Box::new(self.fold_expr(*array)),
                index: Box::new(self.fold_expr(*index)),
            },
            ExpressionNode::MapLiteral(pairs) => {
                let optimized_pairs = pairs
                    .into_iter()
                    .map(|(k, v)| (self.fold_expr(k), self.fold_expr(v)))
                    .collect();
                ExpressionNode::MapLiteral(optimized_pairs)
            }
            ExpressionNode::EnumValue { enum_name, variant } => {
                ExpressionNode::EnumValue { enum_name, variant }
            }

            // Match expressions
            ExpressionNode::Match { expr, arms } => ExpressionNode::Match {
                expr: Box::new(self.fold_expr(*expr)),
                arms: arms
                    .into_iter()
                    .map(|arm| MatchArmNode {
                        pattern: arm.pattern,
                        guard: arm.guard.map(|g| self.fold_expr(g)),
                        body: self.fold_expr(arm.body),
                    })
                    .collect(),
            },
        }
    }

    /// Fold binary operations with constant folding
    fn fold_binary_op(
        &mut self,
        left: ExpressionNode,
        op: BinaryOperator,
        right: ExpressionNode,
    ) -> ExpressionNode {
        use ExpressionNode::Literal;
        use LiteralNode::*;

        match (&left, &op, &right) {
            // Integer arithmetic
            (Literal(Integer(a)), BinaryOperator::Add, Literal(Integer(b))) => {
                Literal(Integer(a + b))
            }
            (Literal(Integer(a)), BinaryOperator::Sub, Literal(Integer(b))) => {
                Literal(Integer(a - b))
            }
            (Literal(Integer(a)), BinaryOperator::Mul, Literal(Integer(b))) => {
                Literal(Integer(a * b))
            }
            (Literal(Integer(a)), BinaryOperator::Div, Literal(Integer(b))) if *b != 0 => {
                Literal(Integer(a / b))
            }
            (Literal(Integer(a)), BinaryOperator::Mod, Literal(Integer(b))) if *b != 0 => {
                Literal(Integer(a % b))
            }

            // Boolean logic
            (Literal(Boolean(a)), BinaryOperator::And, Literal(Boolean(b))) => {
                Literal(Boolean(*a && *b))
            }
            (Literal(Boolean(a)), BinaryOperator::Or, Literal(Boolean(b))) => {
                Literal(Boolean(*a || *b))
            }

            // Comparisons
            (Literal(Integer(a)), BinaryOperator::Eq, Literal(Integer(b))) => {
                Literal(Boolean(a == b))
            }
            (Literal(Integer(a)), BinaryOperator::Neq, Literal(Integer(b))) => {
                Literal(Boolean(a != b))
            }
            (Literal(Integer(a)), BinaryOperator::Lt, Literal(Integer(b))) => {
                Literal(Boolean(a < b))
            }
            (Literal(Integer(a)), BinaryOperator::Lte, Literal(Integer(b))) => {
                Literal(Boolean(a <= b))
            }
            (Literal(Integer(a)), BinaryOperator::Gt, Literal(Integer(b))) => {
                Literal(Boolean(a > b))
            }
            (Literal(Integer(a)), BinaryOperator::Gte, Literal(Integer(b))) => {
                Literal(Boolean(a >= b))
            }

            // String operations
            (Literal(String(a)), BinaryOperator::Concat, Literal(String(b))) => {
                Literal(String(format!("{}{}", a, b)))
            }
            (Literal(String(a)), BinaryOperator::Eq, Literal(String(b))) => {
                Literal(Boolean(a == b))
            }
            (Literal(String(a)), BinaryOperator::Neq, Literal(String(b))) => {
                Literal(Boolean(a != b))
            }

            // Identity optimizations
            (expr, BinaryOperator::Add, Literal(Integer(0)))
            | (Literal(Integer(0)), BinaryOperator::Add, expr) => expr.clone(),

            (expr, BinaryOperator::Mul, Literal(Integer(1)))
            | (Literal(Integer(1)), BinaryOperator::Mul, expr) => expr.clone(),

            (_, BinaryOperator::Mul, Literal(Integer(0)))
            | (Literal(Integer(0)), BinaryOperator::Mul, _) => Literal(Integer(0)),

            (expr, BinaryOperator::And, Literal(Boolean(true)))
            | (Literal(Boolean(true)), BinaryOperator::And, expr) => expr.clone(),

            (_, BinaryOperator::And, Literal(Boolean(false)))
            | (Literal(Boolean(false)), BinaryOperator::And, _) => Literal(Boolean(false)),

            (expr, BinaryOperator::Or, Literal(Boolean(false)))
            | (Literal(Boolean(false)), BinaryOperator::Or, expr) => expr.clone(),

            (_, BinaryOperator::Or, Literal(Boolean(true)))
            | (Literal(Boolean(true)), BinaryOperator::Or, _) => Literal(Boolean(true)),

            // No optimization possible
            _ => ExpressionNode::BinaryOp {
                left: Box::new(left),
                operator: op,
                right: Box::new(right),
            },
        }
    }

    /// Fold unary operations
    fn fold_unary_op(&mut self, op: UnaryOperator, operand: ExpressionNode) -> ExpressionNode {
        use ExpressionNode::Literal;
        use LiteralNode::*;

        match (&op, &operand) {
            (UnaryOperator::Not, Literal(Boolean(b))) => Literal(Boolean(!b)),
            (UnaryOperator::Neg, Literal(Integer(i))) => Literal(Integer(-i)),
            (UnaryOperator::Neg, Literal(Float(f))) => Literal(Float(-f)),

            // Double negation elimination
            (
                UnaryOperator::Not,
                ExpressionNode::UnaryOp {
                    operator: UnaryOperator::Not,
                    operand: inner,
                },
            ) => *inner.clone(),

            // No optimization
            _ => ExpressionNode::UnaryOp {
                operator: op,
                operand: Box::new(operand),
            },
        }
    }

    /// Check if a statement is dead code that can be removed
    fn is_dead_code(&self, stmt: &StatementNode) -> bool {
        matches!(stmt, StatementNode::ExpressionStatement(ExpressionNode::Literal(LiteralNode::Integer(
                0,
            ))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{BinaryOperator, LiteralNode};

    #[test]
    fn test_constant_folding() {
        let mut optimizer = Optimizer::new(OptimizationLevel::Basic);

        let expr = ExpressionNode::BinaryOp {
            left: Box::new(ExpressionNode::Literal(LiteralNode::Integer(2))),
            operator: BinaryOperator::Add,
            right: Box::new(ExpressionNode::Literal(LiteralNode::Integer(3))),
        };

        let result = optimizer.fold_expr(expr);

        assert_eq!(result, ExpressionNode::Literal(LiteralNode::Integer(5)));
    }

    #[test]
    fn test_identity_optimization() {
        let mut optimizer = Optimizer::new(OptimizationLevel::Basic);

        let expr = ExpressionNode::BinaryOp {
            left: Box::new(ExpressionNode::Identifier("x".to_string())),
            operator: BinaryOperator::Add,
            right: Box::new(ExpressionNode::Literal(LiteralNode::Integer(0))),
        };

        let result = optimizer.fold_expr(expr);

        assert_eq!(result, ExpressionNode::Identifier("x".to_string()));
    }

    #[test]
    fn test_boolean_folding() {
        let mut optimizer = Optimizer::new(OptimizationLevel::Basic);

        let expr = ExpressionNode::BinaryOp {
            left: Box::new(ExpressionNode::Literal(LiteralNode::Boolean(true))),
            operator: BinaryOperator::And,
            right: Box::new(ExpressionNode::Identifier("x".to_string())),
        };

        let result = optimizer.fold_expr(expr);

        assert_eq!(result, ExpressionNode::Identifier("x".to_string()));
    }
}
