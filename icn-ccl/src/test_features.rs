#[cfg(test)]
mod tests {
    use icn_ccl::StandardLibrary;

    #[test]
    fn test_advanced_democracy_functions_exist() {
        let stdlib = StandardLibrary::new();
        
        // Test liquid democracy functions
        assert!(stdlib.get_function("create_delegation").is_some());
        assert!(stdlib.get_function("revoke_delegation").is_some());
        assert!(stdlib.get_function("resolve_delegated_vote").is_some());
        assert!(stdlib.get_function("calculate_delegated_power").is_some());
        
        // Test quadratic voting functions  
        assert!(stdlib.get_function("quadratic_vote_cost").is_some());
        assert!(stdlib.get_function("submit_quadratic_vote").is_some());
        assert!(stdlib.get_function("calculate_quadratic_result").is_some());
        
        // Test multi-stage proposals
        assert!(stdlib.get_function("create_multi_stage_proposal").is_some());
        assert!(stdlib.get_function("advance_proposal_stage").is_some());
        
        // Test budget management
        assert!(stdlib.get_function("create_budget").is_some());
        assert!(stdlib.get_function("allocate_budget_funds").is_some());
        
        // Test surplus distribution
        assert!(stdlib.get_function("calculate_surplus").is_some());
        assert!(stdlib.get_function("distribute_dividends").is_some());
        
        println!("✅ All advanced democracy and economic primitives found in stdlib!");
    }
}