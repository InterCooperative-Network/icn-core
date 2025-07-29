// test_original_problems.rs
// Test the exact issues mentioned in CCL_FEATURE_ANALYSIS.md

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    // Test from the original analysis: else-if with >= that supposedly failed
    let source1 = r#"
        contract TestOriginalProblem {
            scope: "test";
            version: "1.0.0";
            
            fn classify_score(score: Integer) -> Integer {
                if score >= 90 {
                    return 90;
                } else if score >= 80 {
                    return 80;
                } else if score >= 70 {
                    return 70;
                } else if score >= 60 {
                    return 60;
                } else {
                    return 0;
                }
            }
        }
    "#;

    println!("Testing original problematic else-if with >=...");
    match compile_ccl_source_to_wasm(source1) {
        Ok((wasm, _metadata)) => {
            println!("✅ Original problematic pattern compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Original problematic pattern failed: {e}");
        }
    }

    // Test complex governance logic with == that was mentioned as failing
    let source2 = r#"
        contract TestGovernanceLogic {
            scope: "test";
            version: "1.0.0";
            
            fn process_vote(member_type: Integer, reputation: Integer) -> Integer {
                if member_type == 1 {
                    if reputation >= 75 {
                        return 100;
                    } else if reputation >= 50 {
                        return 80;
                    } else {
                        return 60;
                    }
                } else if member_type == 2 {
                    if reputation >= 80 {
                        return 90;
                    } else {
                        return 70;
                    }
                } else {
                    return 50;
                }
            }
        }
    "#;

    println!("\nTesting complex governance logic with == and >=...");
    match compile_ccl_source_to_wasm(source2) {
        Ok((wasm, _metadata)) => {
            println!("✅ Complex governance logic compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Complex governance logic failed: {e}");
        }
    }

    // Test that supposedly required mathematical workarounds
    let source3 = r#"
        contract TestMathematicalWorkaround {
            scope: "test";
            version: "1.0.0";
            
            fn calculate_discount_modern(amount: Integer, member_level: Integer) -> Integer {
                if member_level >= 3 {
                    return amount * 70 / 100;
                } else if member_level >= 2 {
                    return amount * 80 / 100;
                } else if member_level >= 1 {
                    return amount * 90 / 100;
                } else {
                    return amount;
                }
            }
        }
    "#;

    println!("\nTesting complex arithmetic that needed mathematical workarounds...");
    match compile_ccl_source_to_wasm(source3) {
        Ok((wasm, _metadata)) => {
            println!("✅ Complex arithmetic with operators compiled successfully!");
            println!("WASM size: {} bytes", wasm.len());
        }
        Err(e) => {
            println!("❌ Complex arithmetic failed: {e}");
        }
    }
}
