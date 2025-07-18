// icn-ccl/src/optimizer.rs
use crate::ast::{
    ActionNode, AstNode, BinaryOperator, BlockNode, ExpressionNode, PolicyStatementNode,
    StatementNode, UnaryOperator,
};
use crate::error::CclError;

pub struct Optimizer {}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer {}
    }

    pub fn optimize(&self, ast: AstNode) -> Result<AstNode, CclError> {
        Ok(self.fold_ast(ast))
    }

    fn fold_ast(&self, ast: AstNode) -> AstNode {
        match ast {
            AstNode::Policy(items) => AstNode::Policy(
                items
                    .into_iter()
                    .map(|p| self.fold_policy_statement(p))
                    .collect(),
            ),
            AstNode::FunctionDefinition {
                name,
                parameters,
                return_type,
                body,
            } => AstNode::FunctionDefinition {
                name,
                parameters,
                return_type: return_type.clone(),
                body: self.fold_block(body),
            },
            AstNode::RuleDefinition {
                name,
                condition,
                action,
            } => AstNode::RuleDefinition {
                name,
                condition: self.fold_expr(condition),
                action: self.fold_action(action),
            },
            AstNode::Block(b) => AstNode::Block(self.fold_block(b)),
            AstNode::StructDefinition { .. } => ast,
        }
    }

    fn fold_policy_statement(&self, stmt: PolicyStatementNode) -> PolicyStatementNode {
        match stmt {
            PolicyStatementNode::FunctionDef(f) => {
                PolicyStatementNode::FunctionDef(self.fold_ast(f))
            }
            PolicyStatementNode::RuleDef(r) => PolicyStatementNode::RuleDef(self.fold_ast(r)),
            PolicyStatementNode::Import { path, alias } => {
                PolicyStatementNode::Import { path, alias }
            }
            PolicyStatementNode::StructDef(s) => PolicyStatementNode::StructDef(self.fold_ast(s)),
            PolicyStatementNode::ConstDef { name, value, type_ann } => {
                PolicyStatementNode::ConstDef { 
                    name, 
                    value: self.fold_expr(value), 
                    type_ann 
                }
            }
            PolicyStatementNode::MacroDef { name, params, body } => {
                PolicyStatementNode::MacroDef { name, params, body }
            }
            // Pass through governance DSL statements unchanged
            PolicyStatementNode::EventDef { name, fields } => {
                PolicyStatementNode::EventDef { name, fields }
            }
            PolicyStatementNode::StateDef { name, type_ann, initial_value } => {
                PolicyStatementNode::StateDef { 
                    name, 
                    type_ann, 
                    initial_value: initial_value.map(|v| self.fold_expr(v))
                }
            }
            PolicyStatementNode::TriggerDef { name, condition, action } => {
                PolicyStatementNode::TriggerDef { 
                    name, 
                    condition: self.fold_expr(condition), 
                    action: self.fold_expr(action) 
                }
            }
        }
    }

    fn fold_block(&self, block: BlockNode) -> BlockNode {
        BlockNode {
            statements: block
                .statements
                .into_iter()
                .map(|s| self.fold_statement(s))
                .collect(),
        }
    }

    fn fold_statement(&self, stmt: StatementNode) -> StatementNode {
        match stmt {
            StatementNode::Let { name, value } => StatementNode::Let {
                name,
                value: self.fold_expr(value),
            },
            StatementNode::ExpressionStatement(e) => {
                StatementNode::ExpressionStatement(self.fold_expr(e))
            }
            StatementNode::Return(e) => StatementNode::Return(self.fold_expr(e)),
            StatementNode::If {
                condition,
                then_block,
                else_block,
            } => StatementNode::If {
                condition: self.fold_expr(condition),
                then_block: self.fold_block(then_block),
                else_block: else_block.map(|b| self.fold_block(b)),
            },
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
            StatementNode::Break => StatementNode::Break,
            StatementNode::Continue => StatementNode::Continue,
        }
    }

    fn fold_action(&self, action: ActionNode) -> ActionNode {
        match action {
            ActionNode::Allow | ActionNode::Deny => action,
            ActionNode::Charge(e) => ActionNode::Charge(self.fold_expr(e)),
        }
    }

    fn fold_expr(&self, expr: ExpressionNode) -> ExpressionNode {
        match expr {
            ExpressionNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                let l = self.fold_expr(*left);
                let r = self.fold_expr(*right);
                if let (ExpressionNode::IntegerLiteral(li), ExpressionNode::IntegerLiteral(ri)) =
                    (&l, &r)
                {
                    return match operator {
                        BinaryOperator::Add => ExpressionNode::IntegerLiteral(li + ri),
                        BinaryOperator::Sub => ExpressionNode::IntegerLiteral(li - ri),
                        BinaryOperator::Mul => ExpressionNode::IntegerLiteral(li * ri),
                        BinaryOperator::Div => ExpressionNode::IntegerLiteral(li / ri),
                        BinaryOperator::Eq => ExpressionNode::BooleanLiteral(li == ri),
                        BinaryOperator::Neq => ExpressionNode::BooleanLiteral(li != ri),
                        BinaryOperator::Lt => ExpressionNode::BooleanLiteral(li < ri),
                        BinaryOperator::Lte => ExpressionNode::BooleanLiteral(li <= ri),
                        BinaryOperator::Gt => ExpressionNode::BooleanLiteral(li > ri),
                        BinaryOperator::Gte => ExpressionNode::BooleanLiteral(li >= ri),
                        _ => ExpressionNode::BinaryOp {
                            left: Box::new(l),
                            operator,
                            right: Box::new(r),
                        },
                    };
                }
                if let (ExpressionNode::BooleanLiteral(lb), ExpressionNode::BooleanLiteral(rb)) =
                    (&l, &r)
                {
                    return match operator {
                        BinaryOperator::And => ExpressionNode::BooleanLiteral(*lb && *rb),
                        BinaryOperator::Or => ExpressionNode::BooleanLiteral(*lb || *rb),
                        BinaryOperator::Eq => ExpressionNode::BooleanLiteral(lb == rb),
                        BinaryOperator::Neq => ExpressionNode::BooleanLiteral(lb != rb),
                        _ => ExpressionNode::BinaryOp {
                            left: Box::new(l),
                            operator,
                            right: Box::new(r),
                        },
                    };
                }
                ExpressionNode::BinaryOp {
                    left: Box::new(l),
                    operator,
                    right: Box::new(r),
                }
            }
            ExpressionNode::FunctionCall { name, arguments } => ExpressionNode::FunctionCall {
                name,
                arguments: arguments.into_iter().map(|a| self.fold_expr(a)).collect(),
            },
            ExpressionNode::UnaryOp { operator, operand } => {
                let folded_operand = self.fold_expr(*operand);
                match (&operator, &folded_operand) {
                    (UnaryOperator::Neg, ExpressionNode::IntegerLiteral(i)) => {
                        ExpressionNode::IntegerLiteral(-i)
                    }
                    (UnaryOperator::Not, ExpressionNode::BooleanLiteral(b)) => {
                        ExpressionNode::BooleanLiteral(!b)
                    }
                    _ => ExpressionNode::UnaryOp {
                        operator,
                        operand: Box::new(folded_operand),
                    },
                }
            }
            ExpressionNode::RequireProof(inner) => {
                ExpressionNode::RequireProof(Box::new(self.fold_expr(*inner)))
            }
            ExpressionNode::TryExpr { expr, catch_arm } => {
                ExpressionNode::TryExpr {
                    expr: Box::new(self.fold_expr(*expr)),
                    catch_arm: catch_arm.map(|c| Box::new(self.fold_expr(*c))),
                }
            }
            // Handle other cases explicitly
            ExpressionNode::IntegerLiteral(_) |
            ExpressionNode::BooleanLiteral(_) |
            ExpressionNode::StringLiteral(_) |
            ExpressionNode::Identifier(_) |
            ExpressionNode::NoneExpr => expr,
            ExpressionNode::SomeExpr(inner) => ExpressionNode::SomeExpr(Box::new(self.fold_expr(*inner))),
            ExpressionNode::OkExpr(inner) => ExpressionNode::OkExpr(Box::new(self.fold_expr(*inner))),
            ExpressionNode::ErrExpr(inner) => ExpressionNode::ErrExpr(Box::new(self.fold_expr(*inner))),
            ExpressionNode::ArrayLiteral(items) => ExpressionNode::ArrayLiteral(
                items.into_iter().map(|e| self.fold_expr(e)).collect()
            ),
            ExpressionNode::ArrayAccess { array, index } => ExpressionNode::ArrayAccess {
                array: Box::new(self.fold_expr(*array)),
                index: Box::new(self.fold_expr(*index)),
            },
            ExpressionNode::MapAccess { map, key } => ExpressionNode::MapAccess {
                map: Box::new(self.fold_expr(*map)),
                key: Box::new(self.fold_expr(*key)),
            },
            ExpressionNode::MapLiteral(entries) => ExpressionNode::MapLiteral(
                entries.into_iter()
                    .map(|(k, v)| (self.fold_expr(k), self.fold_expr(v)))
                    .collect()
            ),
            ExpressionNode::PanicExpr { message } => ExpressionNode::PanicExpr {
                message: Box::new(self.fold_expr(*message)),
            },
            // Pass through governance DSL expressions with optimization
            ExpressionNode::EventEmit { event_name, fields } => ExpressionNode::EventEmit {
                event_name,
                fields: fields.into_iter().map(|(name, expr)| (name, self.fold_expr(expr))).collect(),
            },
            ExpressionNode::StateRead { state_name } => ExpressionNode::StateRead { state_name },
            ExpressionNode::StateWrite { state_name, value } => ExpressionNode::StateWrite {
                state_name,
                value: Box::new(self.fold_expr(*value)),
            },
            ExpressionNode::TriggerAction { trigger_name, params } => ExpressionNode::TriggerAction {
                trigger_name,
                params: params.into_iter().map(|p| self.fold_expr(p)).collect(),
            },
            ExpressionNode::CrossContractCall { contract_address, function_name, params } => {
                ExpressionNode::CrossContractCall {
                    contract_address: Box::new(self.fold_expr(*contract_address)),
                    function_name,
                    params: params.into_iter().map(|p| self.fold_expr(p)).collect(),
                }
            },
            ExpressionNode::BreakExpr => ExpressionNode::BreakExpr,
            ExpressionNode::ContinueExpr => ExpressionNode::ContinueExpr,
            ExpressionNode::Match { value, arms } => ExpressionNode::Match {
                value: Box::new(self.fold_expr(*value)),
                arms: arms.into_iter().map(|(p, e)| (self.fold_expr(p), self.fold_expr(e))).collect(),
            },
        }
    }
}
