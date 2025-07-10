#![allow(clippy::uninlined_format_args)]

use icn_ccl::{compile_ccl_file_to_wasm, compile_ccl_source_to_wasm};
use std::path::Path;

fn main() {
    println!("🏛️ ICN Governance Policy Test 🏛️\n");

    // Test 1: Compile the governance policy file
    println!("=== Test 1: Compile Governance Policy File ===");
    match compile_ccl_file_to_wasm(Path::new("governance_policy.ccl")) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Governance policy compiled successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("🔍 CID: {}", metadata.cid);
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🔐 Source hash: {}", metadata.source_hash);
            
            // Expected calculation:
            // base = (2*10) + (512*2) = 20 + 1024 = 1044
            // with_congestion = 1044 + (1044 * (75/25) / 100) = 1044 + (1044 * 3 / 100) = 1044 + 31 = 1075
            // final_cost = 1075 - (1075 * 20 / 100) = 1075 - 215 = 860
            println!("🧮 Expected result: ~860 mana (after reputation discount)");
        }
        Err(e) => {
            println!("❌ File compilation failed: {:?}", e);
        }
    }

    // Test 2: Simple inline policy test
    println!("\n=== Test 2: Simple Mana Policy ===");
    let simple_policy = r#"
        fn calculate_job_cost(cores: Integer, memory: Integer) -> Mana {
            let base_cpu = cores * 5;
            let base_memory = memory * 1;
            let total = base_cpu + base_memory;
            return total;
        }
        
        fn run() -> Mana {
            return calculate_job_cost(4, 256);
        }
    "#;

    match compile_ccl_source_to_wasm(simple_policy) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Simple policy compiled successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🧮 Expected result: (4*5) + (256*1) = 20 + 256 = 276 mana");
        }
        Err(e) => {
            println!("❌ Simple policy failed: {:?}", e);
        }
    }

    // Test 3: Advanced economic policy
    println!("\n=== Test 3: Advanced Economic Policy ===");
    let economic_policy = r#"
        fn calculate_network_fee(base_fee: Mana, volume: Integer, peak_hours: Bool) -> Mana {
            let volume_multiplier = volume / 100;
            let volume_fee = base_fee * volume_multiplier / 10;
            let total_fee = base_fee + volume_fee;
            
            if peak_hours {
                let peak_surcharge = total_fee * 50 / 100;
                return total_fee + peak_surcharge;
            } else {
                return total_fee;
            }
        }
        
        fn run() -> Mana {
            return calculate_network_fee(100, 500, true);
        }
    "#;

    match compile_ccl_source_to_wasm(economic_policy) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Economic policy compiled successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("📋 Exports: {:?}", metadata.exports);
            println!("💼 Expected result: base=100, volume_fee=50, total=150, peak_surcharge=75, final=225 mana");
        }
        Err(e) => {
            println!("❌ Economic policy failed: {:?}", e);
        }
    }

    println!("\n🎉 Governance Policy Test Complete!");
    println!("✅ CCL enables:");
    println!("   • 🏛️ Governance as Code");
    println!("   • 💰 Dynamic economic policies");
    println!("   • 📊 Complex calculations with reputation factors");
    println!("   • 🔄 Multi-parameter policy functions");
    println!("   • ⚖️ Fair resource allocation algorithms");
} 