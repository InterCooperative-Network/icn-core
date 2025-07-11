#![allow(clippy::uninlined_format_args)]

use icn_ccl::compile_ccl_file_to_wasm;
use std::path::Path;

fn main() {
    println!("🏛️  ICN Cooperative Contracts Test Suite 🏛️\n");
    println!("Testing comprehensive cooperative governance and management contracts...\n");

    let contracts = vec![
        ("Dividend Distribution", "cooperative_dividend_distribution.ccl", "Profit sharing and member compensation based on contribution, seniority, and democratic decisions"),
        ("Membership Management", "cooperative_membership_management.ccl", "Member onboarding, probationary evaluation, and status progression with voting weight calculation"),
        ("Resource Allocation", "cooperative_resource_allocation.ccl", "Fair distribution of workspace, equipment, budget, and time resources"),
        ("Treasury Management", "cooperative_treasury_management.ccl", "Financial management including reserves, investments, emergency funds, and expense approval"),
        ("Work Assignment", "cooperative_work_assignment.ccl", "Task distribution based on skills, workload balance, and learning opportunities"),
        ("Reputation Access Control", "cooperative_reputation_access_control.ccl", "Reputation-based access to resources and privilege escalation")
    ];

    let mut successful_contracts = 0;
    let mut total_wasm_size = 0;

    for (name, filename, description) in contracts {
        println!("=== Testing {} Contract ===", name);
        println!("📄 Description: {}", description);
        
        let contract_path = Path::new(filename);
        
        if !contract_path.exists() {
            println!("❌ Contract file not found: {}", filename);
            println!();
            continue;
        }

        match compile_ccl_file_to_wasm(contract_path) {
            Ok((wasm_bytes, metadata)) => {
                println!("✅ Successfully compiled {} contract!", name);
                println!("📦 WASM size: {} bytes", wasm_bytes.len());
                println!("🔍 CID: {}", metadata.cid);
                println!("📋 Exports: {:?}", metadata.exports);
                println!("🔐 Source hash: {}", metadata.source_hash);
                
                // Analyze contract features
                analyze_contract_features(name, &wasm_bytes);
                
                successful_contracts += 1;
                total_wasm_size += wasm_bytes.len();
                println!("✨ Contract ready for deployment!\n");
            }
            Err(e) => {
                println!("❌ Compilation failed for {}: {:?}", name, e);
                println!("🔧 Check contract syntax and fix any errors\n");
            }
        }
    }

    // Summary report
    println!("=== Test Suite Summary ===");
    println!("📊 Successful contracts: {}/6", successful_contracts);
    println!("📦 Total WASM size: {} bytes", total_wasm_size);
    println!("📈 Average contract size: {} bytes", if successful_contracts > 0 { total_wasm_size / successful_contracts } else { 0 });
    
    if successful_contracts == 6 {
        println!("\n🎉 All cooperative contracts compiled successfully!");
        println!("🚀 Ready for deployment to ICN network");
        
        println!("\n🏗️  Contract Deployment Scenarios:");
        println!("1. 📈 Dividend Distribution: Quarterly profit sharing for tech cooperative");
        println!("2. 👥 Membership Management: New member onboarding and progression");
        println!("3. 🏢 Resource Allocation: Workspace and equipment scheduling");
        println!("4. 💰 Treasury Management: Emergency fund and investment decisions");
        println!("5. 📋 Work Assignment: Fair task distribution with skill matching");
        println!("6. 🔐 Access Control: Reputation-based resource access and privileges");
        
        println!("\n🔗 Integration Points:");
        println!("• Mesh job execution for contract computation");
        println!("• DAG anchoring for governance decisions");
        println!("• Mana-based cost calculation for contract execution");
        println!("• DID-based authentication for member actions");
        println!("• Reputation tracking across all contract interactions");
        
        println!("\n💡 Next Steps:");
        println!("• Deploy contracts to ICN devnet");
        println!("• Create governance proposals for contract adoption");
        println!("• Test cross-contract interactions");
        println!("• Implement contract-based policy enforcement");
    } else {
        println!("\n⚠️  Some contracts failed compilation");
        println!("🔧 Review and fix contract syntax before deployment");
    }
}

fn analyze_contract_features(contract_name: &str, _wasm_bytes: &[u8]) {
    // Contract-specific feature analysis
    match contract_name {
        "Dividend Distribution" => {
            println!("🎯 Features: Contribution scoring, seniority bonuses, democratic adjustments, minimum wage protection");
        }
        "Membership Management" => {
            println!("🎯 Features: Application screening, probationary evaluation, voting weight calculation, status progression");
        }
        "Resource Allocation" => {
            println!("🎯 Features: Workspace priority, budget allocation, equipment scheduling, meeting time management");
        }
        "Treasury Management" => {
            println!("🎯 Features: Reserve allocation, emergency fund authorization, investment assessment, financial health scoring");
        }
        "Work Assignment" => {
            println!("🎯 Features: Skill matching, workload balancing, learning opportunities, task priority assessment");
        }
        "Reputation Access Control" => {
            println!("🎯 Features: Multi-factor reputation scoring, resource access evaluation, privilege escalation, delegation authority");
        }
        _ => {
            println!("🎯 Features: Advanced cooperative governance logic");
        }
    }
} 