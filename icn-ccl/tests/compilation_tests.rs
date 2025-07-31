use icn_ccl::compile_ccl_source_to_wasm;

#[test]
fn test_else_if_chain_compilation() {
    let source = r#"
        fn test_grades(score: Integer) -> String {
            if score >= 90 {
                return "A";
            } else if score >= 80 {
                return "B";
            } else if score >= 70 {
                return "C";
            } else if score >= 60 {
                return "D";
            } else {
                return "F";
            }
        }
        
        fn run() -> Integer {
            let grade = test_grades(85);
            return 1; // Simple success test
        }
    "#;

    let result = compile_ccl_source_to_wasm(source);
    assert!(
        result.is_ok(),
        "Else-if chain should compile successfully: {result:?}"
    );

    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}

#[test]
fn test_array_assignment_compilation() {
    let source = r#"
        fn run() -> Integer {
            let mut nums = [10, 20, 30];
            nums[1] = 99;
            return nums[1];
        }
    "#;

    let result = compile_ccl_source_to_wasm(source);
    assert!(
        result.is_ok(),
        "Array assignment should compile successfully"
    );

    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}

#[test]
fn test_string_comparison_compilation() {
    let source = r#"
        fn run() -> Boolean {
            let name1 = "alice";
            let name2 = "bob";
            return name1 == name2;
        }
    "#;

    let result = compile_ccl_source_to_wasm(source);
    assert!(
        result.is_ok(),
        "String comparison should compile successfully"
    );

    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}

#[test]
fn test_map_operations_compilation() {
    let source = r#"
        fn run() -> Integer {
            let mut scores = map_new();
            map_insert(scores, "alice", 100);
            let alice_score = map_get(scores, "alice");
            return if alice_score == None { 0 } else { 100 };
        }
    "#;

    let result = compile_ccl_source_to_wasm(source);
    assert!(result.is_ok(), "Map operations should compile successfully");

    let (wasm, _meta) = result.unwrap();
    assert!(wasm.starts_with(b"\0asm"), "Should produce valid WASM");
}
