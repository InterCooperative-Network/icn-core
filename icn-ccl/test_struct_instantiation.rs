// test_struct_instantiation.rs
// Test struct instantiation syntax

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Very simple struct instantiation
    let source1 = r#"
        contract TestStructInstantiation {
            scope: "test";
            version: "1.0.0";
            
            struct Point {
                x: Integer,
                y: Integer
            }
            
            fn create_point() -> Integer {
                let p = Point { x: 10, y: 20 };
                return p.x;
            }
        }
    "#;
    
    println!("Testing simple struct instantiation...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Struct instantiation compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Struct instantiation failed: {}", e);
            
            // Let's try even simpler - just the struct literal without assignment
            let simple_test = r#"
                contract TestSimpleStruct {
                    scope: "test";
                    version: "1.0.0";
                    
                    struct Point {
                        x: Integer
                    }
                    
                    fn test() -> Integer {
                        let p = 42;  // This should work
                        return p;
                    }
                }
            "#;
            
            println!("\nTesting simplified version (no struct instantiation)...");
            match compile_ccl_source_to_wasm(simple_test) {
                Ok((wasm, _metadata)) => {
                    println!("✅ Simplified version works - issue is with struct instantiation syntax");
                    println!("WASM size: {} bytes", wasm.len());
                }
                Err(e2) => {
                    println!("❌ Even simplified version failed: {}", e2);
                }
            }
        }
    }
} 