use icn_ccl::{parser::parse_ccl_source, compile_ccl_source_to_wasm};

fn main() {
    // Test pattern matching parsing
    let source = r#"
        fn test_match(value: Option<Integer>) -> Integer {
            return match value {
                Some(x) => x,
                None => 0
            };
        }
    "#;
    
    let result = parse_ccl_source(source);
    match result {
        Ok(_) => println!("✓ Pattern matching parsed successfully!"),
        Err(e) => println!("✗ Failed to parse pattern matching: {:?}", e),
    }

    match compile_ccl_source_to_wasm(source) {
        Ok((wasm, _)) => assert!(wasm.starts_with(b"\0asm")),
        Err(e) => panic!("Compilation failed: {}", e),
    }
    
    // Test simple enum pattern
    let source2 = r#"
        fn handle_result(result: Result<Integer, String>) -> Integer {
            return match result {
                Ok(value) => value,
                Err(_) => -1
            };
        }
    "#;
    
    let result2 = parse_ccl_source(source2);
    match result2 {
        Ok(_) => println!("✓ Enum pattern matching parsed successfully!"),
        Err(e) => println!("✗ Failed to parse enum patterns: {:?}", e),
    }

    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _)) => assert!(wasm.starts_with(b"\0asm")),
        Err(e) => panic!("Compilation failed: {}", e),
    }
}