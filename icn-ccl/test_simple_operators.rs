// test_simple_operators.rs
// Test the simplest operator cases to isolate parsing issues

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Simple assignment with >=
    let source1 = r#"
        contract TestSimple {
            scope: "test";
            version: "1.0.0";
            
            fn test_simple() -> Integer {
                let x = 10;
                let y = 5;
                let result = x >= y;
                return 1;
            }
        }
    "#;
    
    println!("Testing >= in assignment...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ >= in assignment compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ >= in assignment compilation failed: {}", e);
        }
    }
    
    // Test 2: Just return a comparison
    let source2 = r#"
        contract TestReturn {
            scope: "test";
            version: "1.0.0";
            
            fn test_return(x: Integer) -> Integer {
                return x > 5;
            }
        }
    "#;
    
    println!("\nTesting > in return...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ > in return compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ > in return compilation failed: {}", e);
        }
    }
    
    // Test 3: Test without spaces
    let source3 = r#"
        contract TestNoSpaces {
            scope: "test";
            version: "1.0.0";
            
            fn test_no_spaces(x: Integer) -> Integer {
                if x>=5 {
                    return 1;
                } else {
                    return 0;
                }
            }
        }
    "#;
    
    println!("\nTesting >= without spaces...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ >= without spaces compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ >= without spaces compilation failed: {}", e);
        }
    }
    
    // Test 4: Simple equality
    let source4 = r#"
        contract TestSimpleEq {
            scope: "test";
            version: "1.0.0";
            
            fn test_eq() -> Integer {
                let a = 5;
                let b = 5;
                return a == b;
            }
        }
    "#;
    
    println!("\nTesting == in assignment...");
    match compile_ccl_source_to_wasm(source4) {
        Ok((wasm, _metadata)) => {
            println!("✅ == in assignment compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ == in assignment compilation failed: {}", e);
        }
    }
} 