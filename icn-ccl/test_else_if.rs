// test_else_if.rs
// Simple test to check if else-if chains work

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Simple else-if chain
    let source1 = r#"
        contract TestElseIf {
            scope: "test";
            version: "1.0.0";
            
            fn classify(x: Integer) -> Integer {
                if x < 0 {
                    return -1;
                } else if x == 0 {
                    return 0;
                } else {
                    return 1;
                }
            }
        }
    "#;

    println!("Testing else-if chain compilation...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, metadata)) => {
            println!("✅ Else-if chain compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
            println!("Metadata: {metadata:?}");
        }
        Err(e) => {
            println!("❌ Else-if chain compilation failed: {e}");
        }
    }

    // Test 2: More complex else-if chain
    let source2 = r#"
        contract TestGrading {
            scope: "test";
            version: "1.0.0";
            
            fn get_grade(score: Integer) -> Integer {
                if score > 89 {
                    return 90;
                } else if score > 79 {
                    return 80;
                } else if score > 69 {
                    return 70;
                } else if score > 59 {
                    return 60;
                } else {
                    return 0;
                }
            }
        }
    "#;

    println!("\nTesting complex else-if chain...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Complex else-if chain compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Complex else-if chain compilation failed: {e}");
        }
    }

    // Test 3: Nested if with else-if
    let source3 = r#"
        contract TestDiscount {
            scope: "test";
            version: "1.0.0";
            
            fn calculate_discount(member_type: Integer, amount: Integer) -> Integer {
                if member_type < 2 {
                    if amount > 100 {
                        return 90;
                    } else {
                        return 95;
                    }
                } else if member_type < 3 {
                    return 85;
                } else {
                    return amount;
                }
            }
        }
    "#;

    println!("\nTesting nested if with else-if...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Nested if with else-if compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Nested if with else-if compilation failed: {e}");
        }
    }
}
