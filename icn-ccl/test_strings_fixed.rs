// test_strings_fixed.rs
// Test string operations with corrected types

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Basic string creation and assignment
    let source1 = r#"
        contract TestStringBasics {
            scope: "test";
            version: "1.0.0";
            
            fn test_string_creation() -> String {
                let message = "Hello, ICN!";
                return message;
            }
        }
    "#;

    println!("Testing basic string creation...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ String creation compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ String creation failed: {e}");
        }
    }

    // Test 2: String concatenation with + operator
    let source2 = r#"
        contract TestStringConcat {
            scope: "test";
            version: "1.0.0";
            
            fn test_string_concat() -> String {
                let greeting = "Hello";
                let target = "World";
                return greeting + " " + target;
            }
        }
    "#;

    println!("\nTesting string concatenation...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ String concatenation compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ String concatenation failed: {e}");
        }
    }

    // Test 3: String length method
    let source3 = r#"
        contract TestStringLength {
            scope: "test";
            version: "1.0.0";
            
            fn test_string_length() -> Integer {
                let message = "Hello World";
                return message.length();
            }
        }
    "#;

    println!("\nTesting string length...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ String length compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ String length failed: {e}");
        }
    }

    // Test 4: String comparison (FIXED - using Boolean instead of Bool)
    let source4 = r#"
        contract TestStringComparison {
            scope: "test";
            version: "1.0.0";
            
            fn test_string_comparison() -> Boolean {
                let str1 = "hello";
                let str2 = "hello";
                return str1 == str2;
            }
        }
    "#;

    println!("\nTesting string comparison...");
    match compile_ccl_source_to_wasm(source4) {
        Ok((wasm, _metadata)) => {
            println!("✅ String comparison compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ String comparison failed: {e}");
        }
    }
}
