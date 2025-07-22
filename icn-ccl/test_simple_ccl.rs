use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔧 Testing Simple CCL Contract");
    println!("==============================");

    let simple_contract = r#"
        fn run() -> Integer {
            return 42;
        }
    "#;

    match compile_ccl_source_to_wasm(simple_contract) {
        Ok((wasm, metadata)) => {
            println!("✅ SUCCESS: Simple contract compiled!");
            println!("📦 WASM size: {} bytes", wasm.len());
            println!("📋 Exports: {:?}", metadata.exports);
        }
        Err(e) => {
            println!("❌ FAILED: Compilation error:");
            println!("   {}", e);
        }
    }

    // Now test with a more complex contract
    let complex_contract = r#"
        const TEST_CONSTANT: Integer = 100;
        
        struct TestStruct {
            value: Integer,
            active: Bool,
        }
        
        fn helper_function(input: Integer) -> Integer {
            return input * 2;
        }
        
        fn run() -> Integer {
            let test_value = helper_function(TEST_CONSTANT);
            return test_value;
        }
    "#;

    println!();
    println!("🔧 Testing Complex CCL Contract");
    println!("===============================");

    match compile_ccl_source_to_wasm(complex_contract) {
        Ok((wasm, metadata)) => {
            println!("✅ SUCCESS: Complex contract compiled!");
            println!("📦 WASM size: {} bytes", wasm.len());
            println!("📋 Exports: {:?}", metadata.exports);
        }
        Err(e) => {
            println!("❌ FAILED: Complex compilation error:");
            println!("   {}", e);
        }
    }
}
