// test_simple_for.rs
// Test the simplest possible for loop to isolate issues

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Simplest possible for loop - no arithmetic
    let source1 = r#"
        contract TestSimpleFor {
            scope: "test";
            version: "1.0.0";
            
            fn simple_for() -> Integer {
                let numbers = [1, 2, 3];
                for num in numbers {
                    // Do nothing with num, just loop
                }
                return 5;
            }
        }
    "#;
    
    println!("Testing simplest for loop...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Simple for loop compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Simple for loop compilation failed: {}", e);
        }
    }
    
    // Test 2: For loop with variable access but no arithmetic
    let source2 = r#"
        contract TestForAccess {
            scope: "test";
            version: "1.0.0";
            
            fn access_for() -> Integer {
                let numbers = [1, 2, 3];
                let mut result = 0;
                for num in numbers {
                    result = num;
                }
                return result;
            }
        }
    "#;
    
    println!("\nTesting for loop with variable access...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ For loop with access compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ For loop with access compilation failed: {}", e);
        }
    }
} 