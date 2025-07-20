// test_structs.rs
// Test custom struct support in CCL

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Basic struct definition and instantiation
    let source1 = r#"
        contract TestStructs {
            scope: "test";
            version: "1.0.0";
            
            struct Member {
                name: String,
                reputation: Integer,
                active: Boolean
            }
            
            fn create_member() -> Integer {
                let member = Member {
                    name: "Alice",
                    reputation: 100,
                    active: true
                };
                return member.reputation;
            }
        }
    "#;

    println!("Testing basic struct definition and instantiation...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Basic structs compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Basic structs failed: {}", e);
        }
    }

    // Test 2: Struct with array fields
    let source2 = r#"
        contract TestStructArrays {
            scope: "test";
            version: "1.0.0";
            
            struct Cooperative {
                name: String,
                members: [String],
                established: Integer
            }
            
            fn get_member_count() -> Integer {
                let coop = Cooperative {
                    name: "Tech Cooperative",
                    members: ["Alice", "Bob", "Charlie"],
                    established: 2023
                };
                return coop.members.length();
            }
        }
    "#;

    println!("\nTesting struct with array fields...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Struct arrays compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Struct arrays failed: {}", e);
        }
    }

    // Test 3: Enum definitions
    let source3 = r#"
        contract TestEnums {
            scope: "test";
            version: "1.0.0";
            
            enum ProposalStatus {
                Pending,
                Active,
                Passed,
                Rejected
            }
            
            fn get_default_status() -> Integer {
                let status = ProposalStatus::Pending;
                return 1; // Return 1 for pending
            }
        }
    "#;

    println!("\nTesting enum definitions...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Enums compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Enums failed: {}", e);
        }
    }

    // Test 4: Constants
    let source4 = r#"
        contract TestConstants {
            scope: "test";
            version: "1.0.0";
            
            const QUORUM_THRESHOLD: Integer = 66;
            const COOPERATIVE_NAME: String = "ICN Cooperative";
            
            fn check_quorum(votes: Integer, total_members: Integer) -> Boolean {
                let participation = (votes * 100) / total_members;
                return participation >= QUORUM_THRESHOLD;
            }
        }
    "#;

    println!("\nTesting constants...");
    match compile_ccl_source_to_wasm(source4) {
        Ok((wasm, _metadata)) => {
            println!("✅ Constants compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Constants failed: {}", e);
        }
    }

    // Test 5: Complex governance struct
    let source5 = r#"
        contract TestGovernanceStructs {
            scope: "test";
            version: "1.0.0";
            
            struct Proposal {
                id: Integer,
                title: String,
                votes_for: Integer,
                votes_against: Integer,
                status: Integer  // 0=pending, 1=active, 2=passed, 3=rejected
            }
            
            fn evaluate_proposal() -> Boolean {
                let proposal = Proposal {
                    id: 1,
                    title: "Increase membership fees",
                    votes_for: 75,
                    votes_against: 25,
                    status: 1
                };
                
                let total_votes = proposal.votes_for + proposal.votes_against;
                let approval_rate = (proposal.votes_for * 100) / total_votes;
                
                return approval_rate >= 60;
            }
        }
    "#;

    println!("\nTesting complex governance structs...");
    match compile_ccl_source_to_wasm(source5) {
        Ok((wasm, _metadata)) => {
            println!("✅ Complex governance structs compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Complex governance structs failed: {}", e);
        }
    }
}
