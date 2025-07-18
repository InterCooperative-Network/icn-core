use icn_ccl::compile_ccl_source_to_wasm;
use icn_ccl::{ast::ExpressionNode, parser::parse_ccl_source};

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
                icn_ccl::ast::StatementNode::Return(ExpressionNode::Match { .. }) => {
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
fn compile_match_integer_literal() {
    let src = r#"
        fn run() -> Integer {
            let value = 2;
            return match value {
                1 => 100,
                2 => 200,
                3 => 300,
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
                false => 0
            };
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_with_variable_patterns() {
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

        fn run() -> Integer {
            return classify_score(85);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_nested_match_expressions() {
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

        fn run() -> Integer {
            return complex_categorize(1, 20);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_in_governance_voting() {
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

        fn run() -> Integer {
            return calculate_vote_weight(2, 85, true);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_mana_allocation_policy() {
    let src = r#"
        fn determine_mana_allocation(
            member_tier: Integer,
            contribution_level: Integer
        ) -> Mana {
            let base_allocation = match member_tier {
                1 => 100,  // Founding member
                2 => 75,   // Core member
                3 => 50,   // Active member
                4 => 25,   // Regular member
                _ => 10    // New member
            };
            
            let contribution_multiplier = match contribution_level {
                3 => 150,  // 150% of base
                2 => 125,  // 125% of base
                1 => 100,  // 100% of base
                _ => 75    // 75% of base
            };
            
            return (base_allocation * contribution_multiplier) / 100;
        }

        fn run() -> Mana {
            return determine_mana_allocation(2, 3);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_with_complex_expressions() {
    let src = r#"
        fn process_proposal_status(
            status_code: Integer,
            votes_for: Integer,
            votes_against: Integer,
            total_voters: Integer
        ) -> Integer {
            let participation_rate = (votes_for + votes_against) * 100 / total_voters;
            
            return match status_code {
                1 => {  // Draft
                    if participation_rate > 0 {
                        2  // Move to discussion
                    } else {
                        1  // Stay in draft
                    }
                },
                2 => {  // Discussion
                    if participation_rate >= 30 {
                        3  // Move to voting
                    } else {
                        2  // Continue discussion
                    }
                },
                3 => {  // Voting
                    if votes_for > votes_against && participation_rate >= 50 {
                        4  // Approved
                    } else if participation_rate >= 50 {
                        5  // Rejected
                    } else {
                        3  // Continue voting
                    }
                },
                _ => 0  // Invalid status
            };
        }

        fn run() -> Integer {
            return process_proposal_status(3, 60, 30, 100);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_with_range_like_patterns() {
    let src = r#"
        fn categorize_age_group(age: Integer) -> Integer {
            return match age {
                0 => 0,   // Invalid
                1 => 1,   // Infant
                2 => 1,   // Infant
                3 => 1,   // Infant
                4 => 2,   // Child
                5 => 2,   // Child
                6 => 2,   // Child
                18 => 3,  // Adult
                65 => 4,  // Senior
                _ => 3    // Default to adult
            };
        }

        fn run() -> Integer {
            return categorize_age_group(25);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_identifier_patterns() {
    let src = r#"
        fn process_by_name(name_id: Integer) -> Integer {
            return match name_id {
                alice => 1,
                bob => 2,
                charlie => 3,
                _ => 0
            };
        }

        fn run() -> Integer {
            let alice = 10;
            let bob = 20;
            let charlie = 30;
            return process_by_name(bob);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}