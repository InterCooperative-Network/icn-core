// test_forward_reference.rs
// Test forward reference handling

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔄 Testing Forward Reference");
    println!("============================");
    
    let source = r#"
        fn caller() -> Bool {
            return callee();
        }

        fn callee() -> Bool {
            return true;
        }

        fn run() -> Bool {
            return caller();
        }
    "#;

    match compile_ccl_source_to_wasm(source) {
        Ok((wasm, metadata)) => {
            println!("✅ SUCCESS: Forward reference compiled successfully!");
            println!("📊 WASM size: {} bytes", wasm.len());
            println!("📋 Contract metadata: {:?}", metadata);
        }
        Err(e) => {
            println!("❌ FAILED: Forward reference failed to compile");
            println!("🐛 Error: {}", e);
        }
    }
}
