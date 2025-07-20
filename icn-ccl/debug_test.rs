// Debug test for comparison operators
use icn_ccl::compile_ccl_source_to_wasm;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contract_source = r#"
contract DebugComparison {
    scope: "debug.example.org";
    version: "1.0.0";
    
    fn test_comparison() -> Boolean {
        let x = 5;
        let y = 3;
        return x < y;
    }
}
"#;

    println!("🔍 Testing comparison operator parsing...");
    
    match compile_ccl_source_to_wasm(contract_source) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ SUCCESS: Comparison compiled correctly!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("🔍 CID: {}", metadata.cid);
            println!("📋 Exports: {:?}", metadata.exports);
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
} 