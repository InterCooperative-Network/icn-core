// test_simple_debug.rs
// Simple test to debug the binary operation issue

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔍 DEBUGGING BINARY OPERATION ISSUE");
    println!("====================================");
    
    let simple_test = r#"
        fn test_add() -> Integer {
            let a = 5;
            let b = 3;
            return a + b;
        }

        fn run() -> Bool {
            let result = test_add();
            return result == 8;
        }
    "#;

    match compile_ccl_source_to_wasm(simple_test) {
        Ok((wasm, metadata)) => {
            println!("✅ SUCCESS: Simple binary operation test passed!");
            println!("📊 WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ FAILED: Simple binary operation test failed");
            println!("🐛 Error: {}", e);
        }
    }
}
