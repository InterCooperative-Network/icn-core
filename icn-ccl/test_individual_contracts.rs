#![allow(clippy::uninlined_format_args)]
#![allow(clippy::manual_flatten)]
use icn_ccl::*;
use std::fs;

fn main() {
    println!("🌟 ICN Cooperative Contracts Testing 🌟\n");
    
    let contracts = vec![
        "cooperative_dividend_distribution.ccl",
        "cooperative_membership_management.ccl",
        "cooperative_resource_allocation.ccl",
        "cooperative_treasury_management.ccl",
        "cooperative_work_assignment.ccl",
        "cooperative_reputation_access_control.ccl",
        "cooperative_banking_credit_union.ccl",
        "cooperative_supply_chain_coordination.ccl",
        "cooperative_conflict_resolution.ccl",
        "cooperative_educational_governance.ccl",
        "cooperative_simple_governance.ccl",
    ];
    
    let mut successful_contracts = 0;
    let mut failed_contracts = 0;
    
    for contract_file in contracts {
        println!("=== Testing {} ===", contract_file);
        
        // Read the contract file
        let ccl_source = match fs::read_to_string(contract_file) {
            Ok(source) => source,
            Err(e) => {
                println!("❌ Failed to read {}: {}", contract_file, e);
                continue;
            }
        };
        
        // Try to compile the contract
        match compile_ccl_source_to_wasm(&ccl_source) {
            Ok((wasm_bytes, metadata)) => {
                println!("✅ Successfully compiled {}!", contract_file);
                println!("📦 WASM size: {} bytes", wasm_bytes.len());
                
                // Try to parse the WASM to get export information
                let mut exports = Vec::new();
                for payload in wasmparser::Parser::new(0).parse_all(&wasm_bytes) {
                    if let Ok(wasmparser::Payload::ExportSection(export_reader)) = payload {
                        for export in export_reader {
                            if let Ok(export) = export {
                                exports.push(export.name.to_string());
                            }
                        }
                    }
                }
                println!("📋 Exports: {:?}", exports);
                println!("📋 Metadata: {:?}", metadata);
                
                successful_contracts += 1;
            }
            Err(e) => {
                println!("❌ Failed to compile {}: {}", contract_file, e);
                failed_contracts += 1;
            }
        }
        
        println!();
    }
    
    println!("🎉 Testing Complete!");
    println!("✅ Successful contracts: {}", successful_contracts);
    println!("❌ Failed contracts: {}", failed_contracts);
    println!("📊 Success rate: {:.1}%", (successful_contracts as f64 / (successful_contracts + failed_contracts) as f64) * 100.0);
} 