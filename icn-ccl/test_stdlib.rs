// test_stdlib.rs
// Test standard library function accessibility in CCL contracts

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Governance functions
    let source1 = r#"
        contract TestGovernance {
            scope: "test";
            version: "1.0.0";
            
            fn test_governance() -> Boolean {
                let member_did = "did:key:example";
                return has_role(member_did, "admin");
            }
        }
    "#;

    println!("Testing governance functions...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Governance functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Governance functions failed: {e}");
        }
    }

    // Test 2: Economics functions
    let source2 = r#"
        contract TestEconomics {
            scope: "test";
            version: "1.0.0";
            
            fn test_economics() -> Integer {
                let member_did = "did:key:example";
                let balance = get_balance(member_did);
                return balance;
            }
        }
    "#;

    println!("\nTesting economics functions...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Economics functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Economics functions failed: {e}");
        }
    }

    // Test 3: Utility functions
    let source3 = r#"
        contract TestUtilities {
            scope: "test";
            version: "1.0.0";
            
            fn test_utilities() -> Boolean {
                let did_str = "did:key:example123";
                return validate_did(did_str);
            }
        }
    "#;

    println!("\nTesting utility functions...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Utility functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Utility functions failed: {e}");
        }
    }

    // Test 4: Crypto functions
    let source4 = r#"
        contract TestCrypto {
            scope: "test";
            version: "1.0.0";
            
            fn test_crypto() -> String {
                let data = "Hello World";
                return hash_sha256(data);
            }
        }
    "#;

    println!("\nTesting crypto functions...");
    match compile_ccl_source_to_wasm(source4) {
        Ok((wasm, _metadata)) => {
            println!("✅ Crypto functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Crypto functions failed: {e}");
        }
    }

    // Test 5: Math functions
    let source5 = r#"
        contract TestMath {
            scope: "test";
            version: "1.0.0";
            
            fn test_math() -> Integer {
                let values = [10, 20, 30, 40, 50];
                return sum(values);
            }
        }
    "#;

    println!("\nTesting math functions...");
    match compile_ccl_source_to_wasm(source5) {
        Ok((wasm, _metadata)) => {
            println!("✅ Math functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Math functions failed: {e}");
        }
    }
}
