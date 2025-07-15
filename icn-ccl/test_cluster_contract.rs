use icn_ccl::{CCLCompiler, CompilerConfig};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏛️ Testing Custom Governance Contract 🏛️");
    
    // Read the contract file
    let contract_source = fs::read_to_string("test_cluster_governance.ccl")?;
    
    // Initialize compiler
    let config = CompilerConfig::default();
    let compiler = CCLCompiler::new(config);
    
    // Compile the contract
    println!("🔧 Compiling test_cluster_governance.ccl...");
    match compiler.compile(&contract_source) {
        Ok(compiled_contract) => {
            println!("✅ Contract compiled successfully!");
            println!("📦 WASM size: {} bytes", compiled_contract.wasm_bytes.len());
            println!("🔍 CID: {}", compiled_contract.cid);
            println!("📋 Exports: {:?}", compiled_contract.metadata.exports);
            
            // Save WASM to file
            fs::write("test_cluster_governance.wasm", &compiled_contract.wasm_bytes)?;
            println!("💾 WASM saved to test_cluster_governance.wasm");
            
            // Test the contract calculation
            println!("🧮 Expected calculation:");
            println!("   - Total members: 25");
            println!("   - Proposal type: 2");
            println!("   - Threshold: 25/2 + 2*10 = 12 + 20 = 32");
            println!("   - Votes: 18 for, 5 against (total 23)");
            println!("   - Meets threshold: 23 >= 32? No (0)");
            println!("   - Expected result: 2 (insufficient participation)");
            println!("   - Mana cost: 2*50 + 25*5 = 100 + 125 = 225");
            println!("   - Final result: 2*100 + 225 = 425");
            
            Ok(())
        }
        Err(e) => {
            println!("❌ Compilation failed: {}", e);
            Err(e.into())
        }
    }
} 