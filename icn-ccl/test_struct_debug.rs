use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔧 Debugging Struct Parsing Issues");
    println!("==================================");

    // Test 1: Just a struct
    let struct_only = r#"
        struct SimpleStruct {
            value: Integer,
        }
        
        fn run() -> Integer {
            return 42;
        }
    "#;

    println!("Test 1: Simple struct");
    match compile_ccl_source_to_wasm(struct_only) {
        Ok((wasm, _)) => println!("✅ Simple struct works! ({} bytes)", wasm.len()),
        Err(e) => println!("❌ Simple struct failed: {e}"),
    }

    // Test 2: Struct with Bool
    let struct_with_bool = r#"
        struct StructWithBool {
            active: Bool,
        }
        
        fn run() -> Integer {
            return 42;
        }
    "#;

    println!("\nTest 2: Struct with Bool");
    match compile_ccl_source_to_wasm(struct_with_bool) {
        Ok((wasm, _)) => println!("✅ Struct with Bool works! ({} bytes)", wasm.len()),
        Err(e) => println!("❌ Struct with Bool failed: {e}"),
    }

    // Test 3: Multiple fields
    let struct_multi_field = r#"
        struct MultiField {
            value: Integer,
            name: String,
        }
        
        fn run() -> Integer {
            return 42;
        }
    "#;

    println!("\nTest 3: Multi-field struct");
    match compile_ccl_source_to_wasm(struct_multi_field) {
        Ok((wasm, _)) => println!("✅ Multi-field struct works! ({} bytes)", wasm.len()),
        Err(e) => println!("❌ Multi-field struct failed: {e}"),
    }

    // Test 4: Struct with array
    let struct_with_array = r#"
        struct StructWithArray {
            items: Array<Integer>,
        }
        
        fn run() -> Integer {
            return 42;
        }
    "#;

    println!("\nTest 4: Struct with Array");
    match compile_ccl_source_to_wasm(struct_with_array) {
        Ok((wasm, _)) => println!("✅ Struct with Array works! ({} bytes)", wasm.len()),
        Err(e) => println!("❌ Struct with Array failed: {e}"),
    }
}
