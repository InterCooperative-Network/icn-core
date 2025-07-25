use icn_ccl::{compile_ccl_source_to_wasm, StandardLibrary};

#[test]
fn test_advanced_democracy_compilation() {
    let source = r#"
fn test_liquid_democracy() -> Integer {
    let voter = did:key:z6MkTest1;
    let delegate = did:key:z6MkTest2;
    
    // Create a delegation
    let delegation_id = create_delegation(voter, delegate, "budget", 100);
    
    // Resolve the final vote
    let final_voter = resolve_delegated_vote(voter, "proposal_123", "budget");
    let delegated_power = calculate_delegated_power(delegate, "budget");
    
    return delegated_power;
}

fn test_quadratic_voting() -> Integer {
    let voter = did:key:z6MkVoter1;
    
    // Calculate cost for allocating 5 votes (should be 25 credits)
    let cost = quadratic_vote_cost(5);
    
    // Submit quadratic vote
    submit_quadratic_vote(voter, "proposal_456", 5, cost);
    
    // Calculate result from multiple allocations
    let allocations = [5, 3, -2, 4];
    let result = calculate_quadratic_result(allocations);
    
    return result;
}

fn test_budget_management() -> Integer {
    let budget_id = create_budget("quarterly", 10000, "cooperative_token", 
                                 ["operations", "development"], [6000, 4000]);
    
    allocate_budget_funds(budget_id, "operations", did:key:z6MkRecipient, 1000, "office rent");
    
    let remaining = get_budget_balance(budget_id, "operations");
    return remaining;
}

fn main() -> Integer {
    let liquid_result = test_liquid_democracy();
    let quadratic_result = test_quadratic_voting();
    let budget_result = test_budget_management();
    
    return liquid_result + quadratic_result + budget_result;
}
"#;

    // Test that the source compiles successfully
    let result = compile_ccl_source_to_wasm(source);
    
    match result {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Advanced democracy features compile successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("🆔 Contract CID: {}", metadata.cid);
            assert!(!wasm_bytes.is_empty());
        }
        Err(e) => {
            panic!("❌ Compilation failed: {:?}", e);
        }
    }
}

#[test]
fn test_stdlib_functions_exist() {
    let stdlib = StandardLibrary::new();
    
    // Test that advanced democracy functions are registered
    let liquid_democracy_functions = vec![
        "create_delegation",
        "revoke_delegation", 
        "get_delegation_chain",
        "resolve_delegated_vote",
        "calculate_delegated_power",
    ];
    
    for func_name in liquid_democracy_functions {
        assert!(stdlib.get_function(func_name).is_some(), 
                "Function '{}' not found in stdlib", func_name);
        println!("✅ Found function: {}", func_name);
    }
    
    let quadratic_voting_functions = vec![
        "quadratic_vote_cost",
        "submit_quadratic_vote",
        "calculate_quadratic_result",
    ];
    
    for func_name in quadratic_voting_functions {
        assert!(stdlib.get_function(func_name).is_some(),
                "Function '{}' not found in stdlib", func_name);
        println!("✅ Found function: {}", func_name);
    }
    
    let economic_functions = vec![
        "create_budget",
        "allocate_budget_funds", 
        "get_budget_balance",
        "calculate_surplus",
        "distribute_dividends",
        "create_dividend_pool",
    ];
    
    for func_name in economic_functions {
        assert!(stdlib.get_function(func_name).is_some(),
                "Function '{}' not found in stdlib", func_name);
        println!("✅ Found function: {}", func_name);
    }
    
    let multi_stage_functions = vec![
        "create_multi_stage_proposal",
        "advance_proposal_stage",
        "get_proposal_stage",
    ];
    
    for func_name in multi_stage_functions {
        assert!(stdlib.get_function(func_name).is_some(),
                "Function '{}' not found in stdlib", func_name);
        println!("✅ Found function: {}", func_name);
    }
    
    println!("🎉 All advanced democracy and economic primitives are available!");
}