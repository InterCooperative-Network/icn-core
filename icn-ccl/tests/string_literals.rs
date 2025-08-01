use icn_ccl::{
    ast::{AstNode, ExpressionNode, PolicyStatementNode, StatementNode},
    parser::parse_ccl_source,
};

#[test]
fn test_string_with_punctuation() {
    let src = r#"fn greet() -> String { return \"Hello, world!\"; }"#;
    let ast = parse_ccl_source(src).expect("parse");
    if let AstNode::Policy(items) = ast {
        if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition { body, .. }) =
            &items[0]
        {
            if let StatementNode::Return(Some(ExpressionNode::StringLiteral(val))) = &body.statements[0] {
                assert_eq!(val, "Hello, world!");
            } else {
                panic!("unexpected statement")
            }
        } else {
            panic!("unexpected ast")
        }
    } else {
        panic!("unexpected root")
    }
}

#[test]
fn test_string_with_newline_escape() {
    let src = r#"fn newline() -> String { return \"Line1\\nLine2\"; }"#;
    let ast = parse_ccl_source(src).expect("parse");
    if let AstNode::Policy(items) = ast {
        if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition { body, .. }) =
            &items[0]
        {
            if let StatementNode::Return(Some(ExpressionNode::StringLiteral(val))) = &body.statements[0] {
                assert_eq!(val, "Line1\nLine2");
            } else {
                panic!("unexpected statement")
            }
        } else {
            panic!("unexpected ast")
        }
    } else {
        panic!("unexpected root")
    }
}

#[test]
fn test_string_with_quote_escape() {
    let src = r#"fn quote() -> String { return \"She said \\\"hi\\\"\"; }"#;
    let ast = parse_ccl_source(src).expect("parse");
    if let AstNode::Policy(items) = ast {
        if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition { body, .. }) =
            &items[0]
        {
            if let StatementNode::Return(Some(ExpressionNode::StringLiteral(val))) = &body.statements[0] {
                assert_eq!(val, "She said \"hi\"");
            } else {
                panic!("unexpected statement")
            }
        } else {
            panic!("unexpected ast")
        }
    } else {
        panic!("unexpected root")
    }
}
