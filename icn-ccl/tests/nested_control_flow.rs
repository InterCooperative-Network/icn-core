use icn_ccl::compile_ccl_source_to_wasm;

#[test]
fn compile_while_loop_with_nested_if() {
    let src = r#"
        fn run() -> Integer {
            let sum = 0;
            let i = 0;
            
            while i < 10 {
                if i % 2 == 0 {
                    if i % 4 == 0 {
                        sum = sum + i * 2;
                    } else {
                        sum = sum + i;
                    }
                }
                i = i + 1;
            }
            
            return sum;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_nested_if_statements_complex() {
    let src = r#"
        fn run() -> Integer {
            let result = 0;
            let category = 2;
            let value = 15;
            
            if category == 1 {
                if value > 10 {
                    if value > 20 {
                        result = 100;
                    } else {
                        result = 50;
                    }
                } else {
                    result = 10;
                }
            } else if category == 2 {
                if value % 2 == 0 {
                    result = value * 2;
                } else {
                    result = value * 3;
                }
            } else {
                result = 0;
            }
            
            return result;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_with_nested_conditions() {
    let src = r#"
        fn run() -> Integer {
            let operation_type = 2;
            let value = 6;
            let result = 0;
            
            result = match operation_type {
                1 => {
                    if value > 5 {
                        value * 2
                    } else {
                        value
                    }
                },
                2 => {
                    if value % 2 == 0 {
                        if value > 4 {
                            value * 3
                        } else {
                            value + 10
                        }
                    } else {
                        value + 1
                    }
                },
                _ => 0
            };
            
            return result;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_while_loop_with_multiple_conditions() {
    let src = r#"
        fn run() -> Integer {
            let processed = 0;
            let i = 0;
            let threshold = 15;
            
            while i < 20 {
                if i < 5 {
                    i = i + 1;
                } else if i > threshold {
                    processed = processed + i;
                    i = i + 2;
                } else {
                    if i % 3 == 0 {
                        processed = processed + (i * 2);
                    } else {
                        processed = processed + i;
                    }
                    i = i + 1;
                }
            }
            
            return processed;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_governance_vote_processing_complex() {
    let src = r#"
        fn process_voting_logic(
            voter_tier: Integer,
            proposal_type: Integer,
            voter_reputation: Integer
        ) -> Integer {
            let total_weight = 0;
            
            let base_weight = match voter_tier {
                1 => 50,  // Founding member
                2 => 40,  // Core member
                3 => 30,  // Active member
                _ => 20   // Regular member
            };
            
            if proposal_type == 1 {  // Regular proposal
                if voter_reputation >= 75 {
                    total_weight = base_weight + 20;
                } else if voter_reputation >= 50 {
                    total_weight = base_weight + 10;
                } else {
                    total_weight = base_weight;
                }
            } else if proposal_type == 2 {  // Constitutional change
                if voter_tier <= 2 {  // Only founding and core members get bonus
                    if voter_reputation >= 80 {
                        total_weight = base_weight * 150 / 100;
                    } else {
                        total_weight = base_weight * 125 / 100;
                    }
                } else {
                    total_weight = base_weight;
                }
            } else {
                total_weight = base_weight / 2;
            }
            
            return total_weight;
        }

        fn run() -> Integer {
            return process_voting_logic(1, 2, 85);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_nested_mana_calculations() {
    let src = r#"
        fn calculate_mana_distribution_tiered(
            member_tier: Integer,
            activity_score: Integer,
            base_pool: Mana
        ) -> Mana {
            let allocation = 0;
            
            let tier_allocation = match member_tier {
                1 => base_pool / 4,   // 25% for tier 1
                2 => base_pool / 6,   // ~17% for tier 2
                3 => base_pool / 8,   // ~12% for tier 3
                _ => base_pool / 10   // 10% for others
            };
            
            if activity_score >= 80 {
                if activity_score >= 95 {
                    allocation = tier_allocation * 150 / 100;  // 150% bonus
                } else {
                    allocation = tier_allocation * 125 / 100;  // 125% bonus
                }
            } else if activity_score >= 60 {
                allocation = tier_allocation * 110 / 100;  // 110% bonus
            } else if activity_score >= 40 {
                allocation = tier_allocation;  // No bonus
            } else {
                allocation = tier_allocation * 80 / 100;  // 20% reduction
            }
            
            return allocation;
        }

        fn run() -> Mana {
            return calculate_mana_distribution_tiered(1, 90, 1000);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_deeply_nested_decision_tree() {
    let src = r#"
        fn complex_decision_tree(
            category: Integer,
            subcategory: Integer,
            value1: Integer,
            value2: Integer
        ) -> Integer {
            let result = 0;
            
            result = match category {
                1 => {
                    if value1 > 50 {
                        if subcategory == 1 {
                            if value2 > value1 {
                                value1 + value2
                            } else {
                                value1 * 2
                            }
                        } else {
                            value1 + subcategory
                        }
                    } else {
                        value1
                    }
                },
                2 => {
                    if subcategory > 1 {
                        let temp_result = value1 * subcategory;
                        if temp_result > 100 {
                            temp_result / 2
                        } else {
                            temp_result + value2
                        }
                    } else {
                        value2
                    }
                },
                _ => 0
            };
            
            return result;
        }

        fn run() -> Integer {
            return complex_decision_tree(1, 2, 60, 30);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}