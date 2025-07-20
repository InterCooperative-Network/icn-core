use icn_ccl::compile_ccl_source_to_wasm;

#[test]
fn test_enhanced_array_operations_compilation() {
    let source = r#"
        fn run() -> Integer {
            let mut nums = [10, 20, 30];
            nums[1] = 99;                  // Assignment
            let value = nums[1];           // Access
            array_push(nums, 40);          // Growth
            let length = array_len(nums);  // Length
            return value + length;
        }
    "#;
    
    let result = compile_ccl_source_to_wasm(source);
    assert!(result.is_ok(), "Enhanced array operations should compile successfully");
    
    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}

#[test]
fn test_enhanced_string_operations_compilation() {
    let source = r#"
        fn run() -> Boolean {
            let name1 = "alice";
            let name2 = "bob";
            let are_equal = name1 == name2;    // Equality
            let is_less = name1 < name2;       // Ordering
            let first_char = name1[0];         // Indexing
            return are_equal && is_less;
        }
    "#;
    
    let result = compile_ccl_source_to_wasm(source);
    assert!(result.is_ok(), "Enhanced string operations should compile successfully");
    
    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}

#[test]
fn test_enhanced_map_operations_compilation() {
    let source = r#"
        fn run() -> Integer {
            let mut scores = map_new();
            map_insert(scores, "alice", 100);  // Insert
            let alice_score = map_get(scores, "alice");  // Get
            return if alice_score == None { 0 } else { 100 };
        }
    "#;
    
    let result = compile_ccl_source_to_wasm(source);
    assert!(result.is_ok(), "Enhanced map operations should compile successfully");
    
    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}

#[test]
fn test_complex_data_structure_operations() {
    let source = r#"
        fn run() -> Integer {
            // Create a map of string arrays
            let mut groups = map_new();
            let alice_friends = ["bob", "charlie"];
            map_insert(groups, "alice", alice_friends);
            
            // Retrieve and manipulate
            let friends = map_get(groups, "alice");
            if friends != None {
                // String operations and array access
                let first_friend = alice_friends[0];
                if first_friend == "bob" {
                    return array_len(alice_friends);
                }
            }
            
            return 0;
        }
    "#;
    
    let result = compile_ccl_source_to_wasm(source);
    assert!(result.is_ok(), "Complex data structure operations should compile successfully");
    
    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}

#[test]  
fn test_governance_scenario_with_enhanced_features() {
    let source = r#"
        fn process_proposal() -> Boolean {
            // Create a map for vote tallies
            let mut vote_counts = map_new();
            map_insert(vote_counts, "yes", 0);
            map_insert(vote_counts, "no", 0);
            
            // Process vote array
            let votes = ["yes", "no", "yes", "yes", "no"];
            let mut i = 0;
            let vote_length = array_len(votes);
            
            while i < vote_length {
                let vote = votes[i];
                if vote == "yes" {
                    let current = map_get(vote_counts, "yes");
                    // In a full implementation, we'd unwrap the Option
                    map_insert(vote_counts, "yes", 1);
                } else {
                    let current = map_get(vote_counts, "no");
                    map_insert(vote_counts, "no", 1);
                }
                i = i + 1;
            }
            
            // Check if proposal passes
            let yes_votes = map_get(vote_counts, "yes");
            let no_votes = map_get(vote_counts, "no");
            
            return true; // Simplified result
        }
    "#;
    
    let result = compile_ccl_source_to_wasm(source);
    assert!(result.is_ok(), "Governance scenario with enhanced features should compile successfully");
    
    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}