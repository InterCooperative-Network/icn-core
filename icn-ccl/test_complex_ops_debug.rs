use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔍 TESTING COMPLEX OPERATIONS");
    println!("===============================");

    // Test 1: String equality
    let test1 = r#"
        fn run() -> Bool {
            return "hello" == "world";
        }
    "#;

    match compile_ccl_source_to_wasm(test1) {
        Ok(_) => println!("✅ Test 1 (string equality): PASSED"),
        Err(e) => println!("❌ Test 1 (string equality): FAILED - {e}"),
    }

    // Test 2: Array operations
    let test2 = r#"
        fn run() -> Integer {
            let arr = [1, 2, 3];
            return array_len(arr);
        }
    "#;

    match compile_ccl_source_to_wasm(test2) {
        Ok(_) => println!("✅ Test 2 (array length): PASSED"),
        Err(e) => println!("❌ Test 2 (array length): FAILED - {e}"),
    }

    // Test 3: Struct field access with binary ops
    let test3 = r#"
        struct Vote {
            weight: Integer
        }
        
        fn run() -> Integer {
            let vote = Vote { weight: 5 };
            return vote.weight + 3;
        }
    "#;

    match compile_ccl_source_to_wasm(test3) {
        Ok(_) => println!("✅ Test 3 (struct field + number): PASSED"),
        Err(e) => println!("❌ Test 3 (struct field + number): FAILED - {e}"),
    }

    // Test 4: Array indexing with binary ops
    let test4 = r#"
        fn run() -> Integer {
            let arr = [1, 2, 3];
            let item = arr[0];
            return item + 1;
        }
    "#;

    match compile_ccl_source_to_wasm(test4) {
        Ok(_) => println!("✅ Test 4 (array index + number): PASSED"),
        Err(e) => println!("❌ Test 4 (array index + number): FAILED - {e}"),
    }

    // Test 5: Function call with binary ops
    let test5 = r#"
        fn get_number() -> Integer {
            return 10;
        }
        
        fn run() -> Integer {
            return get_number() + 5;
        }
    "#;

    match compile_ccl_source_to_wasm(test5) {
        Ok(_) => println!("✅ Test 5 (function call + number): PASSED"),
        Err(e) => println!("❌ Test 5 (function call + number): FAILED - {e}"),
    }
}
