use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔍 TESTING STRING COMPARISON TYPES");
    println!("====================================");

    // Test simple string comparison
    let test1 = r#"
        fn run() -> Bool {
            let choice = "yes";
            return choice == "yes";
        }
    "#;

    match compile_ccl_source_to_wasm(test1) {
        Ok(_) => println!("✅ Test 1 (variable string comparison): PASSED"),
        Err(e) => println!("❌ Test 1 (variable string comparison): FAILED - {e}"),
    }

    // Test direct string comparison
    let test2 = r#"
        fn run() -> Bool {
            return "hello" == "world";
        }
    "#;

    match compile_ccl_source_to_wasm(test2) {
        Ok(_) => println!("✅ Test 2 (direct string comparison): PASSED"),
        Err(e) => println!("❌ Test 2 (direct string comparison): FAILED - {e}"),
    }

    // Test the actual struct field case
    let test3 = r#"
        struct Vote {
            choice: String
        }

        fn run() -> Bool {
            let vote = Vote { choice: "yes" };
            return vote.choice == "yes";
        }
    "#;

    match compile_ccl_source_to_wasm(test3) {
        Ok(_) => println!("✅ Test 3 (struct field comparison): PASSED"),
        Err(e) => println!("❌ Test 3 (struct field comparison): FAILED - {e}"),
    }
}
