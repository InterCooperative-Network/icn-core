#![allow(
    clippy::uninlined_format_args,
    clippy::needless_borrows_for_generic_args
)]

use icn_ccl::compile_ccl_source_to_wasm;
use std::process::Command;

fn main() {
    println!("🚀 ICN CCL + Devnet Integration Demo 🚀\n");

    // Create a variety of CCL contracts
    let contracts = vec![
        ("simple_calculator", "fn run() -> Integer { return 10 + 5; }"),
        ("counter", "fn run() -> Integer { let i = 0; let j = i + 1; return j; }"),
        ("mana_policy", "fn calculate_base_mana() -> Integer { return 100; } fn run() -> Integer { return calculate_base_mana(); }"),
        ("voting_threshold", "fn get_quorum() -> Integer { return 50; } fn run() -> Integer { return get_quorum(); }"),
    ];

    println!("=== Compiling CCL Contracts ===");
    let mut compiled_contracts = Vec::new();

    for (name, source) in contracts {
        println!("🔧 Compiling: {}", name);
        match compile_ccl_source_to_wasm(source) {
            Ok((wasm_bytes, metadata)) => {
                println!(
                    "  ✅ Success! WASM size: {} bytes, CID: {}",
                    wasm_bytes.len(),
                    metadata.cid
                );
                compiled_contracts.push((name, wasm_bytes, metadata));
            }
            Err(e) => {
                println!("  ❌ Failed: {:?}", e);
            }
        }
    }

    println!("\n=== CCL Compilation Summary ===");
    println!(
        "📊 Compiled {} out of {} contracts successfully",
        compiled_contracts.len(),
        4
    );

    if !compiled_contracts.is_empty() {
        println!("\n=== Contract Details ===");
        for (name, wasm_bytes, metadata) in &compiled_contracts {
            println!("🔷 Contract: {}", name);
            println!("  📦 WASM Size: {} bytes", wasm_bytes.len());
            println!("  🆔 CID: {}", metadata.cid);
            println!("  📋 Exports: {:?}", metadata.exports);
            println!("  🔐 Hash: {}", metadata.source_hash);
            println!();
        }
    }

    println!("=== Next Steps for Full Integration ===");
    println!("🔗 To fully integrate with ICN devnet:");
    println!("  1. Store compiled WASM in DAG via /dag/put API");
    println!("  2. Submit mesh job with CclWasm JobKind");
    println!("  3. Execute contract in secure WASM sandbox");
    println!("  4. Generate execution receipt with results");
    println!("  5. Anchor receipt back to DAG");

    println!("\n=== Testing devnet connectivity ===");
    let curl_result = Command::new("curl")
        .args(&[
            "-s",
            "-H",
            "X-API-Key: devnet-a-key",
            "http://localhost:5001/info",
        ])
        .output();

    match curl_result {
        Ok(output) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                if response.contains("status_message") {
                    println!("✅ ICN Devnet is running and accessible!");
                    println!("🔗 Ready for CCL contract deployment");
                } else {
                    println!("⚠️  Devnet responded but format unexpected");
                }
            } else {
                println!("❌ Devnet not responding properly");
            }
        }
        Err(_) => {
            println!(
                "⚠️  Could not test devnet connectivity (curl not available or devnet not running)"
            );
        }
    }

    println!("\n🎯 CCL Capabilities Demonstrated:");
    println!("  ✨ Governance as Code - Write bylaws in CCL");
    println!("  ⚡ Deterministic Execution - WASM compilation ensures consistency");
    println!("  🏛️ Policy Templates - Reusable governance patterns");
    println!("  🔗 Mesh Integration - Deploy and execute across the network");
    println!("  🔒 Security - Sandboxed execution with resource limits");
    println!("  📜 Auditability - Source code hashing and receipts");
}
