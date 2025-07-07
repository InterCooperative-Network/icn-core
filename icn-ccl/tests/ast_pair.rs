use icn_ccl::{
    ast::{
        self, ActionNode, AstNode, BinaryOperator, BlockNode, ExpressionNode, ParameterNode,
        PolicyStatementNode, StatementNode, TypeAnnotationNode,
    },
    parser::{CclParser, Rule},
};
use pest::Parser;

#[test]
fn test_pair_to_ast_function() {
    let src = "fn inc(a: Integer) -> Integer { return a + 1; }";
    let mut pairs = CclParser::parse(Rule::function_definition, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::FunctionDefinition {
        name: "inc".to_string(),
        parameters: vec![ParameterNode {
            name: "a".to_string(),
            type_ann: TypeAnnotationNode::Integer,
        }],
        return_type: TypeAnnotationNode::Integer,
        body: BlockNode {
            statements: vec![StatementNode::Return(ExpressionNode::BinaryOp {
                left: Box::new(ExpressionNode::Identifier("a".to_string())),
                operator: BinaryOperator::Add,
                right: Box::new(ExpressionNode::IntegerLiteral(1)),
            })],
        },
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_pair_to_ast_policy() {
    let src = r#"
        fn add(a: Integer, b: Integer) -> Integer { return a + b; }
        rule allow_all when true then allow
        import "otherccl" as other;
    "#;
    let mut pairs = CclParser::parse(Rule::policy, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::Policy(vec![
        PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition {
            name: "add".to_string(),
            parameters: vec![
                ParameterNode {
                    name: "a".to_string(),
                    type_ann: TypeAnnotationNode::Integer,
                },
                ParameterNode {
                    name: "b".to_string(),
                    type_ann: TypeAnnotationNode::Integer,
                },
            ],
            return_type: TypeAnnotationNode::Integer,
            body: BlockNode {
                statements: vec![StatementNode::Return(ExpressionNode::BinaryOp {
                    left: Box::new(ExpressionNode::Identifier("a".to_string())),
                    operator: BinaryOperator::Add,
                    right: Box::new(ExpressionNode::Identifier("b".to_string())),
                })],
            },
        }),
        PolicyStatementNode::RuleDef(AstNode::RuleDefinition {
            name: "allow_all".to_string(),
            condition: ExpressionNode::BooleanLiteral(true),
            action: ActionNode::Allow,
        }),
        PolicyStatementNode::Import {
            path: "otherccl".to_string(),
            alias: "other".to_string(),
        },
    ]);
    assert_eq!(ast, expected);
}

#[test]
fn test_pair_to_ast_policy_statement_rule() {
    let src = "rule allow_all when true then allow";
    let mut pairs = CclParser::parse(Rule::policy_statement, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::RuleDefinition {
        name: "allow_all".to_string(),
        condition: ExpressionNode::BooleanLiteral(true),
        action: ActionNode::Allow,
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_pair_to_ast_import_statement() {
    let src = r#"import "otherccl" as other;"#;
    let mut pairs = CclParser::parse(Rule::import_statement, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::Policy(vec![PolicyStatementNode::Import {
        path: "otherccl".to_string(),
        alias: "other".to_string(),
    }]);
    assert_eq!(ast, expected);
}

#[test]
fn test_pair_to_ast_block() {
    let src = "{ return 1; }";
    let mut pairs = CclParser::parse(Rule::block, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::Block(BlockNode {
        statements: vec![StatementNode::Return(ExpressionNode::IntegerLiteral(1))],
    });
    assert_eq!(ast, expected);
}

#[test]
fn test_pair_to_ast_policy_statement_import() {
    let src = "import \"foo\" as bar;";
    let mut pairs = CclParser::parse(Rule::policy_statement, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::Policy(vec![PolicyStatementNode::Import {
        path: "foo".to_string(),
        alias: "bar".to_string(),
    }]);
    assert_eq!(ast, expected);
}

#[test]
fn test_pair_to_ast_statement_return() {
    let src = "return 5;";
    let mut pairs = CclParser::parse(Rule::statement, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::Block(BlockNode {
        statements: vec![StatementNode::Return(ExpressionNode::IntegerLiteral(5))],
    });
    assert_eq!(ast, expected);
}
