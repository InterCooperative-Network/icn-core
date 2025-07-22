use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔍 TESTING CALCULATE_RESULT COMPONENTS");
    println!("========================================");
    
    // Test the minimal version with just the structure
    let test1 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        fn calculate_result(votes: Array<Vote>) -> String {
            return "test";
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;
    
    match compile_ccl_source_to_wasm(test1) {
        Ok(_) => println!("✅ Test 1 (function skeleton): PASSED"),
        Err(e) => println!("❌ Test 1 (function skeleton): FAILED - {}", e),
    }
    
    // Test with basic variables
    let test2 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        fn calculate_result(votes: Array<Vote>) -> String {
            let yes_weight = 0;
            let no_weight = 0;
            return "test";
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;
    
    match compile_ccl_source_to_wasm(test2) {
        Ok(_) => println!("✅ Test 2 (with variables): PASSED"),
        Err(e) => println!("❌ Test 2 (with variables): FAILED - {}", e),
    }
    
    // Test with array_len call
    let test3 = r#"
        struct Vote {
            voter: String,
            choice: String,
            weight: Integer
        }

        fn calculate_result(votes: Array<Vote>) -> String {
            let yes_weight = 0;
            let no_weight = 0;
            let len = array_len(votes);
            return "test";
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;
    
    match compile_ccl_source_to_wasm(test3) {
        Ok(_) => println!("✅ Test 3 (with array_len): PASSED"),
        Err(e) => println!("❌ Test 3 (with array_len): FAILED - {}", e),
    }
    
    // Test with while loop (no body)
    let test4 = r#"
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
                i = i + 1;
            }
            
            return "test";
        }
        
        fn run() -> Bool {
            return true;
        }
    "#;
    
    match compile_ccl_source_to_wasm(test4) {
        Ok(_) => println!("✅ Test 4 (with while loop): PASSED"),
        Err(e) => println!("❌ Test 4 (with while loop): FAILED - {}", e),
    }
}
