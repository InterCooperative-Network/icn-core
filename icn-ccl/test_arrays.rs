// test_arrays.rs
// Test array operations to identify what's missing

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Basic array creation and access
    let source1 = r#"
        contract TestArrayBasics {
            scope: "test";
            version: "1.0.0";
            
            fn test_array_creation() -> Integer {
                let numbers = [1, 2, 3, 4, 5];
                return 42;
            }
        }
    "#;

    println!("Testing basic array creation...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Array creation compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Array creation failed: {e}");
        }
    }

    // Test 2: Array length access
    let source2 = r#"
        contract TestArrayLength {
            scope: "test";
            version: "1.0.0";
            
            fn test_array_length() -> Integer {
                let numbers = [1, 2, 3, 4, 5];
                return numbers.length();
            }
        }
    "#;

    println!("\nTesting array length access...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Array length compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Array length failed: {e}");
        }
    }

    // Test 3: Array indexing
    let source3 = r#"
        contract TestArrayIndexing {
            scope: "test";
            version: "1.0.0";
            
            fn test_array_indexing() -> Integer {
                let numbers = [10, 20, 30, 40, 50];
                return numbers[2];
            }
        }
    "#;

    println!("\nTesting array indexing...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Array indexing compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Array indexing failed: {e}");
        }
    }

    // Test 4: Array modification
    let source4 = r#"
        contract TestArrayModification {
            scope: "test";
            version: "1.0.0";
            
            fn test_array_modification() -> Integer {
                let mut numbers = [10, 20, 30];
                numbers[1] = 99;
                return numbers[1];
            }
        }
    "#;

    println!("\nTesting array modification...");
    match compile_ccl_source_to_wasm(source4) {
        Ok((wasm, _metadata)) => {
            println!("✅ Array modification compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Array modification failed: {e}");
        }
    }

    // Test 5: Array with variables
    let source5 = r#"
        contract TestArrayVariables {
            scope: "test";
            version: "1.0.0";
            
            fn test_array_variables() -> Integer {
                let numbers = [1, 2, 3, 4, 5];
                let index = 2;
                return numbers[index];
            }
        }
    "#;

    println!("\nTesting array indexing with variables...");
    match compile_ccl_source_to_wasm(source5) {
        Ok((wasm, _metadata)) => {
            println!("✅ Array variable indexing compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Array variable indexing failed: {e}");
        }
    }
}
