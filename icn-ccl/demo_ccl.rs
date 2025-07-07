#![allow(clippy::uninlined_format_args)]

use icn_ccl::{compile_ccl_file_to_wasm, compile_ccl_source_to_wasm};
use std::path::Path;

fn main() {
    println!("🌟 ICN CCL (Cooperative Contract Language) Demo 🌟\n");

    // Test 1: Simple inline contract
    println!("=== Test 1: Simple Contract ===");
    let simple_contract = "fn run() -> Integer { return 42; }";
    match compile_ccl_source_to_wasm(simple_contract) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Successfully compiled simple contract!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("🔍 CID: {}", metadata.cid);
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🔐 Source hash: {}\n", metadata.source_hash);
        }
        Err(e) => {
            println!("❌ Compilation failed: {:?}\n", e);
        }
    }

    // Test 2: Governance workflow contract
    println!("=== Test 2: Governance Workflow ===");
    let governance_contract = r#"
        fn create_proposal() -> Integer { return 1; }
        fn vote_on_proposal(proposal_id: Integer) -> Integer { return proposal_id + 1; }
        fn finalize_proposal(vote_count: Integer) -> Integer { return vote_count + 1; }
        fn run() -> Integer { 
            let proposal_id = create_proposal();
            let vote_count = vote_on_proposal(proposal_id);
            return finalize_proposal(vote_count);
        }
    "#;
    match compile_ccl_source_to_wasm(governance_contract) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Successfully compiled governance contract!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("🔍 CID: {}", metadata.cid);
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🔐 Source hash: {}\n", metadata.source_hash);
        }
        Err(e) => {
            println!("❌ Compilation failed: {:?}\n", e);
        }
    }

    // Test 3: File compilation
    println!("=== Test 3: File Compilation ===");
    let example_path = Path::new("tests/contracts/example.ccl");
    if example_path.exists() {
        match compile_ccl_file_to_wasm(example_path) {
            Ok((wasm_bytes, metadata)) => {
                println!("✅ Successfully compiled example.ccl!");
                println!("📦 WASM size: {} bytes", wasm_bytes.len());
                println!("🔍 CID: {}", metadata.cid);
                println!("📋 Exports: {:?}", metadata.exports);
                println!("🔐 Source hash: {}", metadata.source_hash);
            }
            Err(e) => {
                println!("❌ File compilation failed: {:?}", e);
            }
        }
    } else {
        println!("⚠️  Example file not found at {}", example_path.display());
    }

    println!("\n🎉 CCL Demo Complete!");
    println!("🔗 CCL enables:");
    println!("   • Governance as Code");
    println!("   • Deterministic WASM execution");
    println!("   • Policy templates for cooperatives");
    println!("   • Composable governance logic");
}
