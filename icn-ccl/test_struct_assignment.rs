// test_struct_assignment.rs
// Test struct field assignment specifically

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔧 Testing Struct Field Assignment");
    println!("=================================");
    
    // Test 1: Just struct member access (no assignment)
    let source1 = r#"
        struct TestStruct {
            name: String
        }

        fn test_access() -> String {
            let my_struct = TestStruct {
                name: "test"
            };
            
            return my_struct.name;
        }

        fn run() -> Bool {
            return true;
        }
    "#;
    
    println!("Testing struct member access (read-only)...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Struct access compiled successfully!");
            println!("📊 WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Struct access failed: {}", e);
            return;
        }
    }
    
    // Test 2: Simple variable assignment (no structs)
    let source2 = r#"
        fn test_assignment() -> String {
            let name = "test";
            name = "updated";
            return name;
        }

        fn run() -> Bool {
            return true;
        }
    "#;
    
    println!("\nTesting simple variable assignment...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Variable assignment compiled successfully!");
            println!("📊 WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Variable assignment failed: {}", e);
            return;
        }
    }
    
    // Test 3: Struct field assignment (the problem case)
    let source3 = r#"
        struct TestStruct {
            name: String
        }

        fn test_field_assignment() -> String {
            let my_struct = TestStruct {
                name: "test"
            };
            
            my_struct.name = "updated";
            
            return my_struct.name;
        }

        fn run() -> Bool {
            return true;
        }
    "#;
    
    println!("\nTesting struct field assignment...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Struct field assignment compiled successfully!");
            println!("📊 WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Struct field assignment failed: {}", e);
        }
    }

}
