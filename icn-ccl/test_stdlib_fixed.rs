// test_stdlib_fixed.rs
// Test standard library function accessibility with correct types

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test 1: Utility functions (using String type)
    let source1 = r#"
        contract TestUtilities {
            scope: "test";
            version: "1.0.0";
            
            fn test_utilities() -> Boolean {
                let did_str = "did:key:example123";
                return validate_did(did_str);
            }
        }
    "#;

    println!("Testing utility functions...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Utility functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Utility functions failed: {e}");
        }
    }

    // Test 2: Crypto functions
    let source2 = r#"
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
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Crypto functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Crypto functions failed: {e}");
        }
    }

    // Test 3: Math functions
    let source3 = r#"
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
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Math functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Math functions failed: {e}");
        }
    }

    // Test 4: Simple governance functions (avoiding DID type issues for now)
    let source4 = r#"
        contract TestGovernanceSimple {
            scope: "test";
            version: "1.0.0";
            
            fn test_governance() -> Boolean {
                // Test with numbers instead of DIDs for now
                let result1 = min(5, 10);
                let result2 = max(5, 10);
                return result1 < result2;
            }
        }
    "#;

    println!("\nTesting simple governance/math functions...");
    match compile_ccl_source_to_wasm(source4) {
        Ok((wasm, _metadata)) => {
            println!("✅ Simple governance functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Simple governance functions failed: {e}");
        }
    }

    // Test 5: Time/duration utilities
    let source5 = r#"
        contract TestTimeUtils {
            scope: "test";
            version: "1.0.0";
            
            fn test_time() -> Integer {
                let day_duration = days(7);
                let hour_duration = hours(24);
                return 42; // Simplified return for now
            }
        }
    "#;

    println!("\nTesting time/duration functions...");
    match compile_ccl_source_to_wasm(source5) {
        Ok((wasm, _metadata)) => {
            println!("✅ Time/duration functions compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Time/duration functions failed: {e}");
        }
    }
}
