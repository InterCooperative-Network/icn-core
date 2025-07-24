use icn_ccl::compile_ccl_file_to_wasm;
use std::path::PathBuf;

fn main() {
    println!("🧪 Testing Simple Democracy Function...");
    
    let test_path = PathBuf::from("simple_democracy_test.ccl");
    
    match compile_ccl_file_to_wasm(&test_path) {
        Ok((wasm, metadata)) => {
            println!("✅ Simple democracy test compiled successfully!");
            println!("   WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Simple democracy test failed: {}", e);
        }
    }
}