// test_assignment.rs
// Test different assignment patterns to isolate the parsing issue

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Simple variable assignment
    let source1 = r#"
        contract TestSimpleAssign {
            scope: "test";
            version: "1.0.0";
            
            fn test_simple() -> Integer {
                let mut x = 10;
                x = 20;
                return x;
            }
        }
    "#;

    println!("Testing simple variable assignment...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Simple assignment compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Simple assignment failed: {}", e);
        }
    }

    // Test 2: Array indexing in assignment (simplified)
    let source2 = r#"
        contract TestArrayAssign {
            scope: "test";
            version: "1.0.0";
            
            fn test_array() -> Integer {
                let mut arr = [1, 2, 3];
                arr[0] = 99;
                return arr[0];
            }
        }
    "#;

    println!("\nTesting array assignment...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Array assignment compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Array assignment failed: {}", e);
        }
    }

    // Test 3: Array assignment with expression
    let source3 = r#"
        contract TestArrayExprAssign {
            scope: "test";
            version: "1.0.0";
            
            fn test_expr() -> Integer {
                let mut arr = [1, 2, 3];
                let i = 1;
                arr[i] = 42;
                return arr[i];
            }
        }
    "#;

    println!("\nTesting array assignment with expression...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Array expression assignment compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Array expression assignment failed: {}", e);
        }
    }
}
