use icn_ccl::{ast::StatementNode, compile_ccl_source_to_wasm, parser::parse_ccl_source, CclError};

#[test]
fn parse_simple_for_loop() {
    let src = "fn test() -> Integer { let sum = 0; for item in [1, 2, 3] { sum = sum + item; } return sum; }";
    let ast = parse_ccl_source(src).expect("parse");
    if let icn_ccl::ast::AstNode::Policy(items) = ast {
        if let icn_ccl::ast::PolicyStatementNode::FunctionDef(
            icn_ccl::ast::AstNode::FunctionDefinition { body, .. },
        ) = &items[0]
        {
            match &body.statements[1] {
                StatementNode::ForLoop { .. } => {
                    // success
                }
                _ => panic!("expected for statement"),
            }
        } else {
            panic!("unexpected ast");
        }
    } else {
        panic!("unexpected root");
    }
}

#[test]
fn parse_for_loop_with_array_variable() {
    let src = "fn test() -> Integer { let numbers = [10, 20, 30]; let total = 0; for value in numbers { total = total + value; } return total; }";
    let ast = parse_ccl_source(src).expect("parse");
    // Basic parsing validation - detailed AST structure validation omitted for brevity
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_nested_for_loops() {
    let src = "fn test() -> Integer { let total = 0; for outer in [1, 2, 3] { for inner in [10, 20] { total = total + outer + inner; } } return total; }";
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_for_loop_with_conditional_logic() {
    let src = r#"
        fn test() -> Integer {
            let processed = 0;
            for score in [85, 90, 75, 95, 80] {
                if score >= 80 {
                    if score >= 90 {
                        processed = processed + 2;
                    } else {
                        processed = processed + 1;
                    }
                }
            }
            return processed;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

// Test that for loops fail compilation (expected since WASM backend not implemented)
#[test]
fn for_loop_compilation_not_yet_supported() {
    let src = r#"
        fn run() -> Integer {
            let sum = 0;
            for num in [1, 2, 3, 4, 5] {
                sum = sum + num;
            }
            return sum;
        }
    "#;
    let result = compile_ccl_source_to_wasm(src);
    match result {
        Err(CclError::WasmGenerationError(msg)) => {
            assert!(msg.contains("For loops not yet supported"));
        }
        _ => panic!("Expected WasmGenerationError for for loops"),
    }
}

#[test]
fn parse_for_loop_in_governance_context() {
    let src = r#"
        fn calculate_voting_power(reputation_scores: [Integer; 5]) -> Integer {
            let total_power = 0;
            for reputation in reputation_scores {
                let base_power = 10;
                let bonus_power = 0;
                
                if reputation >= 75 {
                    bonus_power = 20;
                } else if reputation >= 50 {
                    bonus_power = 10;
                }
                
                total_power = total_power + base_power + bonus_power;
            }
            return total_power;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_for_loop_with_mana_distribution() {
    let src = r#"
        fn distribute_mana_equally(member_ids: [Integer; 4], total_mana: Mana) -> Mana {
            let distributed = 0;
            let per_member = total_mana / 4;
            
            for member_id in member_ids {
                let allocation = per_member;
                
                // Special allocation for founding members (id < 10)
                if member_id < 10 {
                    allocation = allocation + (allocation / 20);
                }
                
                distributed = distributed + allocation;
            }
            
            return distributed;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_for_loop_boolean_array() {
    let src = r#"
        fn count_approvals(votes: [Bool; 3]) -> Integer {
            let approvals = 0;
            for vote in votes {
                if vote {
                    approvals = approvals + 1;
                }
            }
            return approvals;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}
