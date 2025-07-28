use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔍 TESTING PROBLEMATIC OPERATIONS");
    println!("==================================");

    // Test array indexing with struct access
    let test1 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        fn calculate_result(votes: Array<Vote>) -> String {
            let i = 0;
            let vote = votes[i];
            return "test";
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;

    match compile_ccl_source_to_wasm(test1) {
        Ok(_) => println!("✅ Test 1 (array indexing): PASSED"),
        Err(e) => println!("❌ Test 1 (array indexing): FAILED - {}", e),
    }

    // Test struct field access after array indexing
    let test2 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        fn calculate_result(votes: Array<Vote>) -> String {
            let i = 0;
            let vote = votes[i];
            let choice = vote.choice;
            return "test";
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;

    match compile_ccl_source_to_wasm(test2) {
        Ok(_) => println!("✅ Test 2 (struct field access): PASSED"),
        Err(e) => println!("❌ Test 2 (struct field access): FAILED - {}", e),
    }

    // Test string comparison
    let test3 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        fn calculate_result(votes: Array<Vote>) -> String {
            let i = 0;
            let vote = votes[i];
            if vote.choice == "yes" {
                return "yes";
            }
            return "test";
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;

    match compile_ccl_source_to_wasm(test3) {
        Ok(_) => println!("✅ Test 3 (string comparison): PASSED"),
        Err(e) => println!("❌ Test 3 (string comparison): FAILED - {}", e),
    }

    // Test the arithmetic operations
    let test4 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        fn calculate_result(votes: Array<Vote>) -> String {
            let yes_weight = 0;
            let i = 0;
            let vote = votes[i];
            yes_weight = yes_weight + vote.weight;
            return "test";
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;

    match compile_ccl_source_to_wasm(test4) {
        Ok(_) => println!("✅ Test 4 (arithmetic with struct field): PASSED"),
        Err(e) => println!("❌ Test 4 (arithmetic with struct field): FAILED - {}", e),
    }
}
