// test_for_arithmetic.rs
// Test for loop arithmetic operations to find where it breaks

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: For loop with simple variable assignment
    let source1 = r#"
        contract TestForAssign {
            scope: "test";
            version: "1.0.0";
            
            fn test_assign() -> Integer {
                let numbers = [1, 2, 3];
                let mut total = 0;
                for num in numbers {
                    total = 42;  // Simple constant assignment
                }
                return total;
            }
        }
    "#;

    println!("Testing for loop with constant assignment...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ For loop with assignment compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ For loop with assignment failed: {e}");
        }
    }

    // Test 2: For loop with iterator addition
    let source2 = r#"
        contract TestForIteratorAdd {
            scope: "test";
            version: "1.0.0";
            
            fn test_iterator_add() -> Integer {
                let numbers = [1, 2, 3];
                let mut total = 0;
                for num in numbers {
                    total = num + 0;  // Iterator + constant
                }
                return total;
            }
        }
    "#;

    println!("\nTesting for loop with iterator addition...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ For loop with iterator addition compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ For loop with iterator addition failed: {e}");
        }
    }

    // Test 3: For loop with variable addition
    let source3 = r#"
        contract TestForVarAdd {
            scope: "test";
            version: "1.0.0";
            
            fn test_var_add() -> Integer {
                let numbers = [1, 2, 3];
                let mut total = 0;
                for num in numbers {
                    total = total + 1;  // Variable + constant
                }
                return total;
            }
        }
    "#;

    println!("\nTesting for loop with variable addition...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ For loop with variable addition compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ For loop with variable addition failed: {e}");
        }
    }

    // Test 4: For loop with iterator + variable
    let source4 = r#"
        contract TestForFullAdd {
            scope: "test";
            version: "1.0.0";
            
            fn test_full_add() -> Integer {
                let numbers = [1, 2, 3];
                let mut total = 0;
                for num in numbers {
                    total = total + num;  // Variable + iterator
                }
                return total;
            }
        }
    "#;

    println!("\nTesting for loop with variable + iterator...");
    match compile_ccl_source_to_wasm(source4) {
        Ok((wasm, _metadata)) => {
            println!("✅ For loop with variable + iterator compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ For loop with variable + iterator failed: {e}");
        }
    }
}
