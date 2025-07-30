// test_simple_struct.rs
// Simple struct test to isolate parsing issues

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Just struct definition (no instantiation)
    let source1 = r#"
        contract TestStructDef {
            scope: "test";
            version: "1.0.0";
            
            struct Member {
                name: String,
                reputation: Integer
            }
            
            fn test() -> Integer {
                return 42;
            }
        }
    "#;

    println!("Testing struct definition only...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Struct definition compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Struct definition failed: {e}");
        }
    }

    // Test 2: Just constant usage
    let source2 = r#"
        contract TestConstant {
            scope: "test";
            version: "1.0.0";
            
            const MY_CONSTANT: Integer = 42;
            
            fn test() -> Integer {
                return MY_CONSTANT;
            }
        }
    "#;

    println!("\nTesting constant definition and usage...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Constant compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Constant failed: {e}");
        }
    }

    // Test 3: Just enum definition
    let source3 = r#"
        contract TestEnum {
            scope: "test";
            version: "1.0.0";
            
            enum Status {
                Active,
                Inactive
            }
            
            fn test() -> Integer {
                return 1;
            }
        }
    "#;

    println!("\nTesting enum definition only...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Enum definition compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Enum definition failed: {e}");
        }
    }
}
