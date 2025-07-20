// test_operators.rs
// Test specific operator support in CCL

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Greater than or equal (>=)
    let source1 = r#"
        contract TestOperators {
            scope: "test";
            version: "1.0.0";
            
            fn test_gte(x: Integer) -> Integer {
                if x >= 5 {
                    return 1;
                } else {
                    return 0;
                }
            }
        }
    "#;

    println!("Testing >= operator...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ >= operator compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ >= operator compilation failed: {}", e);
        }
    }

    // Test 2: Equal to (==)
    let source2 = r#"
        contract TestEquals {
            scope: "test";
            version: "1.0.0";
            
            fn test_eq(x: Integer) -> Integer {
                if x == 5 {
                    return 1;
                } else {
                    return 0;
                }
            }
        }
    "#;

    println!("\nTesting == operator...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ == operator compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ == operator compilation failed: {}", e);
        }
    }

    // Test 3: Less than or equal (<=)
    let source3 = r#"
        contract TestLte {
            scope: "test";
            version: "1.0.0";
            
            fn test_lte(x: Integer) -> Integer {
                if x <= 10 {
                    return 1;
                } else {
                    return 0;
                }
            }
        }
    "#;

    println!("\nTesting <= operator...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ <= operator compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ <= operator compilation failed: {}", e);
        }
    }

    // Test 4: Complex arithmetic with operators
    let source4 = r#"
        contract TestArithmetic {
            scope: "test";
            version: "1.0.0";
            
            fn test_arithmetic(a: Integer, b: Integer) -> Integer {
                let result = a * b + 10;
                if result >= 100 {
                    return result / 2;
                } else {
                    return result;
                }
            }
        }
    "#;

    println!("\nTesting complex arithmetic with >=...");
    match compile_ccl_source_to_wasm(source4) {
        Ok((wasm, _metadata)) => {
            println!("✅ Complex arithmetic compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Complex arithmetic compilation failed: {}", e);
        }
    }
}
