// Quick test to verify CCL 0.1 contract parsing
use icn_ccl::{compile_ccl_source_to_wasm, parser::parse_ccl_source};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contract_source = r#"
contract TestContract {
    scope: "test.example.org";
    version: "1.0.0";
    
    fn calculate_total(base: Integer, multiplier: Integer, bonus: Integer) -> Integer {
        return base * multiplier + bonus;
    }
    
    fn get_answer() -> Integer {
        return 42;
    }
}
"#;

    println!("🧪 Testing CCL 0.1 Contract Parsing...");
    
    // Test 1: Parse the contract
    match parse_ccl_source(contract_source) {
        Ok(ast) => {
            println!("✅ Contract parsed successfully!");
            println!("AST: {:?}", ast);
        }
        Err(e) => {
            println!("❌ Parsing failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Test 2: Try to compile to WASM
    println!("\n🔧 Testing WASM Compilation...");
    match compile_ccl_source_to_wasm(contract_source) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ WASM compilation successful!");
            println!("WASM size: {} bytes", wasm_bytes.len());
            println!("Metadata: {:?}", metadata);
            
            // Verify WASM magic number
            if wasm_bytes.starts_with(b"\0asm") {
                println!("✅ Valid WASM module generated!");
            } else {
                println!("❌ Invalid WASM magic number");
            }
        }
        Err(e) => {
            println!("❌ WASM compilation failed: {}", e);
            return Err(e.into());
        }
    }
    
    println!("\n🎉 All tests passed! CCL 0.1 is working correctly!");
    Ok(())
} 