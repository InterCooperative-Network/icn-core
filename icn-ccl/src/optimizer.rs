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
            ExpressionNode::UnaryOp { operator, expr } => {
                let folded = self.fold_expr(*expr);
                match (&operator, &folded) {
                    (UnaryOperator::Neg, ExpressionNode::IntegerLiteral(i)) => {
                        ExpressionNode::IntegerLiteral(-i)
                    }
                    (UnaryOperator::Not, ExpressionNode::BooleanLiteral(b)) => {
                        ExpressionNode::BooleanLiteral(!b)
                    }
                    _ => ExpressionNode::UnaryOp {
                        operator,
                        expr: Box::new(folded),
                    },
                }
            }
            e => e,
        }
    }
}
