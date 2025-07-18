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
fn compile_for_loop_with_nested_match() {
    let src = r#"
        fn run() -> Integer {
            let total_score = 0;
            
            for value in [1, 2, 3, 4, 5] {
                let score = match value {
                    1 => 10,
                    2 => 20,
                    3 => {
                        match value + 1 {
                            4 => 35,
                            _ => 30
                        }
                    },
                    _ => 5
                };
                total_score = total_score + score;
            }
            
            return total_score;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_match_with_nested_loops() {
    let src = r#"
        fn run() -> Integer {
            let operation_type = 2;
            let result = 0;
            
            result = match operation_type {
                1 => {
                    let sum = 0;
                    for num in [1, 2, 3] {
                        sum = sum + num;
                    }
                    sum
                },
                2 => {
                    let product = 1;
                    let i = 1;
                    while i <= 3 {
                        product = product * i;
                        i = i + 1;
                    }
                    product
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
fn compile_nested_for_loops_with_conditions() {
    let src = r#"
        fn run() -> Integer {
            let matrix_sum = 0;
            
            for row in [1, 2, 3] {
                for col in [4, 5, 6] {
                    let cell_value = row * col;
                    
                    if cell_value > 8 {
                        if cell_value % 2 == 0 {
                            matrix_sum = matrix_sum + cell_value;
                        } else {
                            matrix_sum = matrix_sum + (cell_value / 2);
                        }
                    }
                }
            }
            
            return matrix_sum;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_while_loop_with_break_and_continue() {
    let src = r#"
        fn run() -> Integer {
            let processed = 0;
            let i = 0;
            
            while i < 20 {
                i = i + 1;
                
                if i < 5 {
                    continue;
                }
                
                if i > 15 {
                    break;
                }
                
                if i % 3 == 0 {
                    for multiplier in [1, 2] {
                        processed = processed + (i * multiplier);
                    }
                } else {
                    processed = processed + i;
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
        fn process_complex_voting(
            voter_ids: [Integer; 4],
            vote_types: [Integer; 4],
            proposal_priority: Integer
        ) -> Integer {
            let total_weight = 0;
            
            for voter_id in voter_ids {
                let base_weight = match voter_id {
                    1 => 50,  // Founding member
                    2 => 40,  // Core member
                    3 => 30,  // Active member
                    _ => 20   // Regular member
                };
                
                for vote_type in vote_types {
                    if vote_type == 1 {  // Approve
                        let weight_multiplier = match proposal_priority {
                            3 => {  // Critical
                                if voter_id <= 2 {
                                    150  // 150% weight for critical proposals
                                } else {
                                    125  // 125% weight for others
                                }
                            },
                            2 => 110,  // Important: 110% weight
                            1 => 100,  // Normal: 100% weight
                            _ => 90    // Low priority: 90% weight
                        };
                        
                        total_weight = total_weight + (base_weight * weight_multiplier / 100);
                    }
                }
            }
            
            return total_weight;
        }

        fn run() -> Integer {
            return process_complex_voting([1, 2, 3, 4], [1, 1, 0, 1], 3);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_nested_control_flow_with_mana() {
    let src = r#"
        fn calculate_mana_distribution_complex(
            member_tiers: [Integer; 3],
            activity_scores: [Integer; 3],
            total_pool: Mana
        ) -> Mana {
            let allocated = 0;
            let member_index = 0;
            
            for tier in member_tiers {
                let base_allocation = match tier {
                    1 => total_pool / 4,   // 25% for tier 1
                    2 => total_pool / 6,   // ~17% for tier 2
                    3 => total_pool / 8,   // ~12% for tier 3
                    _ => total_pool / 10   // 10% for others
                };
                
                let activity_score = activity_scores[member_index];
                let final_allocation = base_allocation;
                
                if activity_score >= 80 {
                    let bonus_iterations = activity_score / 20;
                    let current_bonus = 0;
                    
                    while current_bonus < bonus_iterations {
                        final_allocation = final_allocation + (base_allocation / 20);
                        current_bonus = current_bonus + 1;
                    }
                } else if activity_score < 50 {
                    final_allocation = final_allocation * 80 / 100;
                }
                
                allocated = allocated + final_allocation;
                member_index = member_index + 1;
            }
            
            return allocated;
        }

        fn run() -> Mana {
            return calculate_mana_distribution_complex([1, 2, 1], [90, 60, 85], 1000);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_deeply_nested_control_structures() {
    let src = r#"
        fn complex_decision_tree(
            category: Integer,
            subcategory: Integer,
            data_values: [Integer; 2]
        ) -> Integer {
            let result = 0;
            
            result = match category {
                1 => {
                    for value in data_values {
                        if value > 50 {
                            let temp_sum = 0;
                            let counter = 0;
                            
                            while counter < value / 10 {
                                temp_sum = temp_sum + match subcategory {
                                    1 => counter * 2,
                                    2 => counter * 3,
                                    _ => counter
                                };
                                counter = counter + 1;
                            }
                            
                            result = result + temp_sum;
                        }
                    }
                    result
                },
                2 => {
                    let accumulator = 0;
                    
                    for i in [0, 1] {
                        let data_value = data_values[i];
                        
                        accumulator = accumulator + match data_value {
                            100 => {
                                let inner_result = 0;
                                let j = 0;
                                while j < 3 {
                                    inner_result = inner_result + j * subcategory;
                                    j = j + 1;
                                }
                                inner_result
                            },
                            _ => data_value * subcategory
                        };
                    }
                    
                    accumulator
                },
                _ => 0
            };
            
            return result;
        }

        fn run() -> Integer {
            return complex_decision_tree(1, 2, [60, 30]);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}