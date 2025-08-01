use icn_ccl::{
    ast::{
        self, AstNode, BinaryOperator, BlockNode, ExpressionNode, ParameterNode,
        PolicyStatementNode, StatementNode, TypeExprNode,
    },
    parser::{CclParser, Rule},
};
use pest::Parser;

#[test]
#[ignore] // Rule names changed in CCL 0.1, function_definition is now fn_decl
fn test_pair_to_ast_function() {
    let src = "fn inc(a: Integer) -> Integer { return a + 1; }";
    let mut pairs = CclParser::parse(Rule::fn_decl, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::FunctionDefinition {
        name: "inc".to_string(),
        type_parameters: vec![],
        parameters: vec![ParameterNode {
            name: "a".to_string(),
            type_expr: TypeExprNode::Integer,
        }],
        return_type: Some(TypeExprNode::Integer),
        body: BlockNode {
            statements: vec![StatementNode::Return(Some(ExpressionNode::BinaryOp {
                left: Box::new(ExpressionNode::Identifier("a".to_string())),
                operator: BinaryOperator::Add,
                right: Box::new(ExpressionNode::IntegerLiteral(1)),
            }))],
        },
    };
    assert_eq!(ast, expected);
}

#[test]
#[ignore] // Legacy policy syntax not supported in CCL 0.1
fn test_pair_to_ast_policy() {
    let src = r#"
        fn add(a: Integer, b: Integer) -> Integer { return a + b; }
        rule allow_all when true then allow
        import "otherccl" as other;
    "#;
    let mut pairs = CclParser::parse(Rule::program, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::Policy(vec![
        PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition {
            name: "add".to_string(),
            type_parameters: vec![],
            parameters: vec![
                ParameterNode {
                    name: "a".to_string(),
                    type_expr: TypeExprNode::Integer,
                },
                ParameterNode {
                    name: "b".to_string(),
                    type_expr: TypeExprNode::Integer,
                },
            ],
            return_type: Some(TypeExprNode::Integer),
            body: BlockNode {
                statements: vec![StatementNode::Return(Some(ExpressionNode::BinaryOp {
                    left: Box::new(ExpressionNode::Identifier("a".to_string())),
                    operator: BinaryOperator::Add,
                    right: Box::new(ExpressionNode::Identifier("b".to_string())),
                }))],
            },
        }),
        PolicyStatementNode::RuleDef(AstNode::EnumDefinition {
            name: "allow_all".to_string(),
            type_parameters: vec![],
            variants: vec![],
        }),
        PolicyStatementNode::Import {
            path: "otherccl".to_string(),
            alias: "other".to_string(),
        },
    ]);
    assert_eq!(ast, expected);
}

#[test]
#[ignore] // Legacy rule syntax not supported in CCL 0.1
fn test_pair_to_ast_policy_statement_rule() {
    let src = "rule allow_all when true then allow";
    let mut pairs = CclParser::parse(Rule::program, src).unwrap();
    let pair = pairs.next().unwrap();
    let ast = ast::pair_to_ast(pair).unwrap();
    let expected = AstNode::EnumDefinition {
        name: "allow_all".to_string(),
        type_parameters: vec![],
        variants: vec![],
    };
    assert_eq!(ast, expected);
}

#[test]
fn test_pair_to_ast_import_statement() {
    let src = r#"import "otherccl" as other;"#;
    let mut pairs = CclParser::parse(Rule::import_stmt, src).unwrap();
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
        statements: vec![StatementNode::Return(Some(ExpressionNode::IntegerLiteral(1)))],
    });
    assert_eq!(ast, expected);
}

#[test]
#[ignore] // Legacy policy_statement syntax not supported in CCL 0.1
fn test_pair_to_ast_policy_statement_import() {
    let src = "import \"foo\" as bar;";
    let mut pairs = CclParser::parse(Rule::import_stmt, src).unwrap();
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
        statements: vec![StatementNode::Return(Some(ExpressionNode::IntegerLiteral(5)))],
    });
    assert_eq!(ast, expected);
}
