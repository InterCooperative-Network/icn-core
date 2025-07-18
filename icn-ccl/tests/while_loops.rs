use icn_ccl::compile_ccl_source_to_wasm;
use icn_ccl::{ast::StatementNode, parser::parse_ccl_source};

#[test]
fn parse_simple_while_loop() {
    let src = "fn test() -> Integer { let i = 0; while i < 5 { i = i + 1; } return i; }";
    let ast = parse_ccl_source(src).expect("parse");
    if let icn_ccl::ast::AstNode::Policy(items) = ast {
        if let icn_ccl::ast::PolicyStatementNode::FunctionDef(
            icn_ccl::ast::AstNode::FunctionDefinition { body, .. },
        ) = &items[0]
        {
            match &body.statements[1] {
                StatementNode::WhileLoop { .. } => {
                    // success
                }
                _ => panic!("expected while statement"),
            }
        } else {
            panic!("unexpected ast");
        }
    } else {
        panic!("unexpected root");
    }
}

#[test]
fn compile_simple_while_loop() {
    let src = r#"
        fn run() -> Integer {
            let count = 0;
            let target = 3;
            while count < target {
                count = count + 1;
            }
            return count;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_while_loop_with_break() {
    let src = r#"
        fn run() -> Integer {
            let sum = 0;
            let i = 0;
            while true {
                if i >= 5 {
                    break;
                }
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_while_loop_with_continue() {
    let src = r#"
        fn run() -> Integer {
            let sum = 0;
            let i = 0;
            while i < 10 {
                i = i + 1;
                if i % 2 == 0 {
                    continue;
                }
                sum = sum + i;
            }
            return sum;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_nested_while_loops() {
    let src = r#"
        fn run() -> Integer {
            let total = 0;
            let outer = 0;
            while outer < 3 {
                let inner = 0;
                while inner < 2 {
                    total = total + 1;
                    inner = inner + 1;
                }
                outer = outer + 1;
            }
            return total;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_while_with_complex_condition() {
    let src = r#"
        fn run() -> Integer {
            let x = 10;
            let y = 5;
            let count = 0;
            while x > 0 && y < 20 {
                x = x - 1;
                y = y + 2;
                count = count + 1;
            }
            return count;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_while_loop_in_governance_context() {
    let src = r#"
        fn count_eligible_voters(
            voter_count: Integer,
            min_reputation: Integer,
            base_reputation: Integer
        ) -> Integer {
            let eligible = 0;
            let current_voter = 0;
            
            while current_voter < voter_count {
                let voter_reputation = base_reputation + (current_voter * 5);
                if voter_reputation >= min_reputation {
                    eligible = eligible + 1;
                }
                current_voter = current_voter + 1;
            }
            
            return eligible;
        }

        fn run() -> Integer {
            return count_eligible_voters(10, 50, 30);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_while_loop_with_mana_calculations() {
    let src = r#"
        fn calculate_mana_distribution(members: Integer, total_mana: Mana) -> Mana {
            let distributed = 0;
            let current_member = 0;
            let base_allocation = total_mana / members;
            
            while current_member < members {
                let member_share = base_allocation;
                if current_member < 5 {
                    member_share = member_share + (member_share / 10);
                }
                distributed = distributed + member_share;
                current_member = current_member + 1;
            }
            
            return distributed;
        }

        fn run() -> Mana {
            return calculate_mana_distribution(8, 1000);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}