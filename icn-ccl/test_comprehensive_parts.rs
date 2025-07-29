use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔍 TESTING COMPREHENSIVE COMPONENTS");
    println!("=====================================");

    // Test the exact structs from comprehensive test
    let test1 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        struct Proposal {
            id: Integer,
            title: String,
            votes: Array<Vote>,
            status: String
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;

    match compile_ccl_source_to_wasm(test1) {
        Ok(_) => println!("✅ Test 1 (struct definitions): PASSED"),
        Err(e) => println!("❌ Test 1 (struct definitions): FAILED - {e}"),
    }

    // Test the exact function from comprehensive test
    let test2 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        fn calculate_result(votes: Array<Vote>) -> String {
            let yes_weight = 0;
            let no_weight = 0;
            let i = 0;
            let len = array_len(votes);
            
            while i < len {
                let vote = votes[i];
                if vote.choice == "yes" {
                    yes_weight = yes_weight + vote.weight;
                } else if vote.choice == "no" {
                    no_weight = no_weight + vote.weight;
                }
                i = i + 1;
            }
            
            if yes_weight > no_weight {
                return "passed";
            } else {
                return "rejected";
            }
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;

    match compile_ccl_source_to_wasm(test2) {
        Ok(_) => println!("✅ Test 2 (calculate_result function): PASSED"),
        Err(e) => println!("❌ Test 2 (calculate_result function): FAILED - {e}"),
    }

    // Test constants
    let test3 = r#"
        const SCALING: Bool = true;
        const WEEK: Integer = 604800;
        
        fn run() -> Bool {
            return SCALING;
        }
    "#;

    match compile_ccl_source_to_wasm(test3) {
        Ok(_) => println!("✅ Test 3 (constants): PASSED"),
        Err(e) => println!("❌ Test 3 (constants): FAILED - {e}"),
    }
}
