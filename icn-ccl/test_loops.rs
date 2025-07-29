// test_loops.rs
// Test the current loop implementation status in CCL

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Simple while loop
    let source1 = r#"
        contract TestWhileLoop {
            scope: "test";
            version: "1.0.0";
            
            fn count_up() -> Integer {
                let mut counter = 0;
                while counter < 5 {
                    counter = counter + 1;
                }
                return counter;
            }
        }
    "#;

    println!("Testing while loop compilation...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ While loop compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ While loop compilation failed: {e}");
        }
    }

    // Test 2: Simple for loop
    let source2 = r#"
        contract TestForLoop {
            scope: "test";
            version: "1.0.0";
            
            fn sum_array() -> Integer {
                let numbers = [1, 2, 3, 4, 5];
                let mut total = 0;
                for num in numbers {
                    total = total + num;
                }
                return total;
            }
        }
    "#;

    println!("\nTesting for loop compilation...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ For loop compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ For loop compilation failed: {e}");
        }
    }

    // Test 3: Nested loops
    let source3 = r#"
        contract TestNestedLoops {
            scope: "test";
            version: "1.0.0";
            
            fn nested_count() -> Integer {
                let mut total = 0;
                let mut outer = 0;
                while outer < 3 {
                    let mut inner = 0;
                    while inner < 2 {
                        total = total + 1;
                        inner = inner + 1;
                    }
                    outer = outer + 1;
                }
                return total;
            }
        }
    "#;

    println!("\nTesting nested loops...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Nested loops compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Nested loops compilation failed: {e}");
        }
    }
}
