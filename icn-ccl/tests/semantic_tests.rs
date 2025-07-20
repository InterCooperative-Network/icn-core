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

#[test]
fn test_for_loop_semantics() {
    let src = r#"
        fn run() -> Integer {
            let nums = [1,2,3];
            let total = 0;
            for n in nums {
                let total = total + n;
            }
            return total;
        }
    "#;
    let res = analyze_ok(src);
    assert!(res.is_ok());
}

#[test]
fn test_standalone_struct_usage() {
    let src = r#"
        struct Point { x: Integer, y: Integer }
        fn run() -> Integer {
            let p = Point { x: 1, y: 2 };
            return p.x;
        }
    "#;
    let res = analyze_ok(src);
    assert!(res.is_ok());
}

#[test]
fn test_struct_member_not_found() {
    let src = "struct P { x: Integer } fn bad() -> Integer { let p = P { x: 1 }; return p.y; }";
    let res = analyze_ok(src);
    assert!(matches!(res, Err(CclError::SemanticError(_))));
}

#[test]
fn test_const_declaration() {
    let src = "const ANSWER: Integer = 42; fn run() -> Integer { return ANSWER; }";
    let res = analyze_ok(src);
    assert!(res.is_ok());
}

#[test]
fn test_match_pattern_type_error() {
    let src = "fn run() -> Integer { let v = 1; match v { true => 0, _ => 1 } }";
    let res = analyze_ok(src);
    assert!(matches!(res, Err(CclError::TypeMismatchError { .. })));
}

#[test]
fn test_enum_definition_usage() {
    let src = "enum Color { Red, Blue } fn run() -> Integer { let c = Color::Red; match c { Color::Red => 1, _ => 0 } }";
    let res = analyze_ok(src);
    assert!(res.is_ok());
}
