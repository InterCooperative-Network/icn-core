use icn_ccl::compile_ccl_source_to_wasm;
use icn_ccl::{ast::ExpressionNode, parser::parse_ccl_source, CclError};

#[test]
fn parse_match_with_multiple_arms() {
    let src = "fn test(x: Integer) -> Integer { return match x { 1 => 10, 2 => 20, _ => 0 }; }";
    let ast = parse_ccl_source(src).expect("parse");
    if let icn_ccl::ast::AstNode::Policy(items) = ast {
        if let icn_ccl::ast::PolicyStatementNode::FunctionDef(
            icn_ccl::ast::AstNode::FunctionDefinition { body, .. },
        ) = &items[0]
        {
            match &body.statements[0] {
                icn_ccl::ast::StatementNode::Return(Some(ExpressionNode::Match { .. })) => {
                    // success
                }
                _ => panic!("expected match expression in return"),
            }
        } else {
            panic!("unexpected ast");
        }
    } else {
        panic!("unexpected root");
    }
}

#[test]
fn compile_simple_match_integer() {
    let src = r#"
        fn run() -> Integer {
            let value = 2;
            return match value {
                1 => 100,
                _ => 0
            };
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_boolean_values() {
    let src = r#"
        fn run() -> Integer {
            let flag = true;
            return match flag {
                true => 1,
                _ => 0
            };
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

// Note: Advanced matching with multiple arms is limited in current WASM backend
// The backend only executes the first arm, so these tests focus on parsing
#[test]
fn parse_match_with_variable_patterns() {
    let src = r#"
        fn classify_score(score: Integer) -> Integer {
            return match score {
                100 => 5,  // Perfect
                90 => 4,   // Excellent
                80 => 3,   // Good
                70 => 2,   // Fair
                _ => 1     // Needs improvement
            };
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_nested_match_expressions() {
    let src = r#"
        fn complex_categorize(type_id: Integer, sub_id: Integer) -> Integer {
            return match type_id {
                1 => match sub_id {
                    10 => 110,
                    20 => 120,
                    _ => 100
                },
                2 => match sub_id {
                    10 => 210,
                    20 => 220,
                    _ => 200
                },
                _ => 0
            };
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn parse_match_in_governance_voting() {
    let src = r#"
        fn calculate_vote_weight(
            vote_type: Integer,
            voter_reputation: Integer,
            is_core_member: Bool
        ) -> Integer {
            let base_weight = match vote_type {
                1 => 10,  // Regular proposal
                2 => 20,  // Policy change
                3 => 30,  // Constitutional amendment
                _ => 5    // Other
            };
            
            let reputation_bonus = match voter_reputation {
                100 => 15,
                90 => 12,
                80 => 10,
                70 => 8,
                _ => 5
            };
            
            let member_bonus = match is_core_member {
                true => 20,
                false => 0
            };
            
            return base_weight + reputation_bonus + member_bonus;
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}

#[test]
fn compile_simple_governance_match() {
    let src = r#"
        fn classify_proposal_type(type_id: Integer) -> Integer {
            return match type_id {
                1 => 10,  // Regular proposal
                _ => 5    // Default
            };
        }

        fn run() -> Integer {
            return classify_proposal_type(1);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_mana_allocation_simple() {
    let src = r#"
        fn determine_base_allocation(member_tier: Integer) -> Mana {
            return match member_tier {
                1 => 100,  // Founding member
                _ => 25    // Default allocation
            };
        }

        fn run() -> Mana {
            return determine_base_allocation(1);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn parse_match_identifier_patterns() {
    let src = r#"
        fn process_by_name(name_id: Integer) -> Integer {
            return match name_id {
                alice => 1,
                bob => 2,
                charlie => 3,
                _ => 0
            };
        }
    "#;
    let ast = parse_ccl_source(src).expect("parse");
    assert!(matches!(ast, icn_ccl::ast::AstNode::Policy(_)));
}
