use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔍 TESTING SIMPLE INTEGER OPERATIONS");
    println!("====================================");

    // Test 1: Simple integer literal
    let test1 = r#"
        fn run() -> Integer {
            return 42;
        }
    "#;

    match compile_ccl_source_to_wasm(test1) {
        Ok(_) => println!("✅ Test 1 (integer literal): PASSED"),
        Err(e) => println!("❌ Test 1 (integer literal): FAILED - {}", e),
    }

    // Test 2: Simple addition
    let test2 = r#"
        fn run() -> Integer {
            return 5 + 3;
        }
    "#;

    match compile_ccl_source_to_wasm(test2) {
        Ok(_) => println!("✅ Test 2 (integer addition): PASSED"),
        Err(e) => println!("❌ Test 2 (integer addition): FAILED - {}", e),
    }

    // Test 3: Variable assignment and addition
    let test3 = r#"
        fn run() -> Integer {
            let a = 5;
            let b = 3;
            return a + b;
        }
    "#;

    match compile_ccl_source_to_wasm(test3) {
        Ok(_) => println!("✅ Test 3 (variable addition): PASSED"),
        Err(e) => println!("❌ Test 3 (variable addition): FAILED - {}", e),
    }

    // Test 4: Integer comparison
    let test4 = r#"
        fn run() -> Bool {
            return 5 > 3;
        }
    "#;

    match compile_ccl_source_to_wasm(test4) {
        Ok(_) => println!("✅ Test 4 (integer comparison): PASSED"),
        Err(e) => println!("❌ Test 4 (integer comparison): FAILED - {}", e),
    }
}
