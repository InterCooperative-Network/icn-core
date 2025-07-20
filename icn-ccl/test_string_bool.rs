// test_string_bool.rs
// Test Bool type usage to isolate the type mismatch issue

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Using Boolean (as defined in grammar)
    let source1 = r#"
        contract TestBoolean {
            scope: "test";
            version: "1.0.0";
            
            fn test_boolean_type() -> Boolean {
                return true;
            }
        }
    "#;

    println!("Testing Boolean type...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Boolean type compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Boolean type failed: {}", e);
        }
    }

    // Test 2: Using Bool (might not be supported)
    let source2 = r#"
        contract TestBool {
            scope: "test";
            version: "1.0.0";
            
            fn test_bool_type() -> Bool {
                return true;
            }
        }
    "#;

    println!("\nTesting Bool type...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Bool type compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Bool type failed: {}", e);
        }
    }

    // Test 3: String comparison with Boolean return type
    let source3 = r#"
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

    println!("\nTesting string comparison with Boolean...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ String comparison with Boolean compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ String comparison with Boolean failed: {}", e);
        }
    }
}
