use icn_ccl::{
    ast::{
        AstNode, BinaryOperator, BlockNode, ExpressionNode, ParameterNode, PolicyStatementNode,
        StatementNode, TypeAnnotationNode, UnaryOperator,
    },
    parser::parse_ccl_source,
};

#[test]
fn arithmetic_mixed_precedence() {
    let src = "fn run() -> Integer { return 1 + 2 * 3 - 4 / 2; }";
    let ast = parse_ccl_source(src).expect("parse");
    if let AstNode::Policy(items) = ast {
        if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition { body, .. }) =
            &items[0]
        {
            if let StatementNode::Return(expr) = &body.statements[0] {
                let expected = ExpressionNode::BinaryOp {
                    left: Box::new(ExpressionNode::BinaryOp {
                        left: Box::new(ExpressionNode::IntegerLiteral(1)),
                        operator: BinaryOperator::Add,
                        right: Box::new(ExpressionNode::BinaryOp {
                            left: Box::new(ExpressionNode::IntegerLiteral(2)),
                            operator: BinaryOperator::Mul,
                            right: Box::new(ExpressionNode::IntegerLiteral(3)),
                        }),
                    }),
                    operator: BinaryOperator::Sub,
                    right: Box::new(ExpressionNode::BinaryOp {
                        left: Box::new(ExpressionNode::IntegerLiteral(4)),
                        operator: BinaryOperator::Div,
                        right: Box::new(ExpressionNode::IntegerLiteral(2)),
                    }),
                };
                assert_eq!(expr, &expected);
            } else {
                panic!("expected return statement")
            }
        } else {
            panic!("unexpected ast")
        }
    } else {
        panic!("unexpected root")
    }
}

#[test]
fn nested_gte_comparisons() {
    let src = "fn run(a: Integer, b: Integer, c: Integer) -> Bool { return a >= b >= c; }";
    let ast = parse_ccl_source(src).expect("parse");
    if let AstNode::Policy(items) = ast {
        if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition { body, .. }) =
            &items[0]
        {
            if let StatementNode::Return(expr) = &body.statements[0] {
                let expected = ExpressionNode::BinaryOp {
                    left: Box::new(ExpressionNode::BinaryOp {
                        left: Box::new(ExpressionNode::Identifier("a".into())),
                        operator: BinaryOperator::Gte,
                        right: Box::new(ExpressionNode::Identifier("b".into())),
                    }),
                    operator: BinaryOperator::Gte,
                    right: Box::new(ExpressionNode::Identifier("c".into())),
                };
                assert_eq!(expr, &expected);
            } else {
                panic!("expected return")
            }
        } else {
            panic!("unexpected ast")
        }
    } else {
        panic!("unexpected root")
    }
}

#[test]
fn unary_logic_combination() {
    let src =
        "fn run(x: Bool, y: Integer, z: Integer) -> Bool { return !x || -y * 2 > 3 && z != 0; }";
    let ast = parse_ccl_source(src).expect("parse");
    if let AstNode::Policy(items) = ast {
        if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition { body, .. }) =
            &items[0]
        {
            if let StatementNode::Return(expr) = &body.statements[0] {
                let expected = ExpressionNode::BinaryOp {
                    left: Box::new(ExpressionNode::UnaryOp {
                        operator: UnaryOperator::Not,
                        operand: Box::new(ExpressionNode::Identifier("x".into())),
                    }),
                    operator: BinaryOperator::Or,
                    right: Box::new(ExpressionNode::BinaryOp {
                        left: Box::new(ExpressionNode::BinaryOp {
                            left: Box::new(ExpressionNode::BinaryOp {
                                left: Box::new(ExpressionNode::UnaryOp {
                                    operator: UnaryOperator::Neg,
                                    operand: Box::new(ExpressionNode::Identifier("y".into())),
                                }),
                                operator: BinaryOperator::Mul,
                                right: Box::new(ExpressionNode::IntegerLiteral(2)),
                            }),
                            operator: BinaryOperator::Gt,
                            right: Box::new(ExpressionNode::IntegerLiteral(3)),
                        }),
                        operator: BinaryOperator::And,
                        right: Box::new(ExpressionNode::BinaryOp {
                            left: Box::new(ExpressionNode::Identifier("z".into())),
                            operator: BinaryOperator::Neq,
                            right: Box::new(ExpressionNode::IntegerLiteral(0)),
                        }),
                    }),
                };
                assert_eq!(expr, &expected);
            } else {
                panic!("expected return")
            }
        } else {
            panic!("unexpected ast")
        }
    } else {
        panic!("unexpected root")
    }
}
