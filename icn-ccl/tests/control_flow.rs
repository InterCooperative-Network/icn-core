use icn_ccl::compile_ccl_source_to_wasm;
use icn_ccl::{ast::StatementNode, parser::parse_ccl_source};

#[test]
fn compile_simple_if() {
    let src = r#"
        fn check_value(x: Integer) -> Integer {
            if x > 10 {
                return 100;
            }
            return x;
        }

        fn run() -> Integer {
            return check_value(15);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_if_else() {
    let src = r#"
        fn calculate_discount(amount: Mana, is_premium: Bool) -> Mana {
            if is_premium {
                return amount * 80 / 100;
            } else {
                return amount * 95 / 100;
            }
        }

        fn run() -> Mana {
            return calculate_discount(1000, true);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_else_if_chain() {
    let src = r#"
        fn classify(x: Integer) -> Integer {
            if x < 0 {
                return 0;
            } else if x == 0 {
                return 1;
            } else {
                return 2;
            }
        }

        fn run() -> Integer {
            return classify(5);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_nested_ifs() {
    let src = r#"
        fn complex_decision(value: Integer, threshold1: Integer, threshold2: Integer) -> Integer {
            if value >= threshold1 {
                if value >= threshold2 {
                    return 3;
                } else {
                    return 2;
                }
            } else {
                return 1;
            }
        }

        fn run() -> Integer {
            return complex_decision(75, 50, 80);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_if_with_locals() {
    let src = r#"
        fn process_with_locals(input: Integer) -> Integer {
            let threshold = 25;
            let multiplier = 3;

            if input > threshold {
                let bonus = input * multiplier;
                return bonus;
            }

            return input;
        }

        fn run() -> Integer {
            return process_with_locals(30);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_governance_conditions() {
    let src = r#"
        fn evaluate_proposal_vote(
            voter_mana: Mana,
            voter_reputation: Integer,
            proposal_type: Integer,
            is_core_member: Bool
        ) -> Integer {
            let base_voting_power = voter_mana / 100;
            let reputation_bonus = voter_reputation / 10;

            if is_core_member {
                if proposal_type > 2 {
                    return base_voting_power + reputation_bonus + 50;
                } else {
                    return base_voting_power + reputation_bonus + 25;
                }
            } else {
                if voter_reputation >= 75 {
                    return base_voting_power + reputation_bonus;
                } else {
                    return base_voting_power;
                }
            }
        }

        fn run() -> Integer {
            return evaluate_proposal_vote(1000, 80, 3, !false);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_comparison_operators() {
    let src = r#"
        fn compare(a: Integer, b: Integer) -> Integer {
            if a < b {
                return 1;
            }
            if a <= b {
                return 1;
            }
            return 0;
        }

        fn run() -> Integer {
            return compare(1, 2);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn parse_else_if_chain() {
    let src = "fn test(x: Integer) -> Integer { if x < 0 { return 0; } else if x == 0 { return 1; } else { return 2; } }";
    let ast = parse_ccl_source(src).expect("parse");
    if let icn_ccl::ast::AstNode::Policy(items) = ast {
        if let icn_ccl::ast::PolicyStatementNode::FunctionDef(
            icn_ccl::ast::AstNode::FunctionDefinition { body, .. },
        ) = &items[0]
        {
            match &body.statements[0] {
                StatementNode::If {
                    else_block: Some(b),
                    ..
                } => {
                    assert_eq!(b.statements.len(), 1);
                    if let StatementNode::If { .. } = b.statements[0].clone() {
                        // success
                    } else {
                        panic!("expected nested if in else block");
                    }
                }
                _ => panic!("expected if"),
            }
        } else {
            panic!("unexpected ast");
        }
    } else {
        panic!("unexpected root");
    }
}
