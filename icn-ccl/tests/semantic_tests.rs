use icn_ccl::{parser::parse_ccl_source, semantic_analyzer::SemanticAnalyzer, CclError};

fn analyze_ok(src: &str) -> Result<(), CclError> {
    let ast = parse_ccl_source(src)?;
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast)
}

#[test]
fn test_return_type_mismatch() {
    let src = "fn bad() -> Bool { return 1; }";
    let res = analyze_ok(src);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}

#[test]
fn test_undefined_variable() {
    let src = "fn bad() -> Integer { return x; }";
    let res = analyze_ok(src);
    assert!(matches!(res, Err(CclError::SemanticError(_))));
}

#[test]
fn test_binary_type_error() {
    let src = "fn bad() -> Integer { let a = 1 + \"s\"; return 0; }";
    let res = analyze_ok(src);
    assert!(matches!(res, Err(CclError::TypeError(_))));
}

#[test]
fn test_valid_function() {
    let src = "fn good() -> Integer { let a = 1 + 2; return a; }";
    let res = analyze_ok(src);
    assert!(res.is_ok());
}

#[test]
fn test_function_with_params() {
    let src = "fn add(a: Integer, b: Integer) -> Integer { return a + b; }";
    let res = analyze_ok(src);
    assert!(res.is_ok());
}

#[test]
fn test_allow_rule() {
    let src = "rule r when true then allow";
    let res = analyze_ok(src);
    assert!(res.is_ok());
}

#[test]
fn test_deny_rule() {
    let src = "rule r when true then deny";
    let res = analyze_ok(src);
    assert!(res.is_ok());
}

#[test]
fn test_charge_rule() {
    let src = "rule r when true then charge 5";
    let res = analyze_ok(src);
    assert!(res.is_ok());
}

#[test]
fn test_option_result_match() {
    let src = r#"
        fn run() -> Integer {
            let v = Some(1);
            match v { 1 => 2, _ => 0 }
        }
    "#;
    let res = analyze_ok(src);
    assert!(res.is_ok());
}
