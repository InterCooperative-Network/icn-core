use icn_ccl::{
    ast::{AstNode, ExpressionNode, PolicyStatementNode, StatementNode},
    compile_ccl_source_to_wasm,
    parser::parse_ccl_source,
    semantic_analyzer::SemanticAnalyzer,
    CclError,
};

#[test]
fn parse_array_literal_and_access() {
    let src = "fn test() -> Integer { let a = [1, 2, 3]; return a[1]; }";
    let ast = parse_ccl_source(src).expect("parse");
    if let AstNode::Policy(items) = ast {
        if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition { body, .. }) =
            &items[0]
        {
            match &body.statements[0] {
                StatementNode::Let { value, .. } => {
                    assert!(matches!(value, ExpressionNode::ArrayLiteral(_)));
                }
                _ => panic!("expected let"),
            }
            match &body.statements[1] {
                StatementNode::Return(ExpressionNode::ArrayAccess { .. }) => {}
                _ => panic!("expected array access"),
            }
        } else {
            panic!("unexpected ast");
        }
    } else {
        panic!("unexpected root");
    }
}

#[test]
fn compile_array_indexing() {
    let src = r#"
        fn run() -> Integer {
            let nums = [10, 20, 30];
            return nums[2];
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn semantic_error_mixed_array_types() {
    let src = "fn bad() -> Integer { let a = [1, true]; return 0; }";
    let res = compile_ccl_source_to_wasm(src);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}

#[test]
fn semantic_error_non_numeric_index() {
    let src = "fn bad() -> Integer { let a = [1,2]; return a[true]; }";
    let ast = parse_ccl_source(src).expect("parse");
    let mut analyzer = SemanticAnalyzer::new();
    let res = analyzer.analyze(&ast);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}
