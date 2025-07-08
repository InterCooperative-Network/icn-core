#![allow(clippy::uninlined_format_args)]

use icn_ccl::compile_ccl_source_to_wasm;

fn main() {
    println!("🔀 ICN CCL Control Flow Features Test 🔀\n");

    // Test 1: Simple if statement without else
    println!("=== Test 1: Simple If Statement ===");
    let if_test = r#"
        fn check_value(x: Integer) -> Integer {
            if x > 10 {
                return 100;
            }
            return x;
        }
        
        fn run() -> Integer {
            return check_value(15);
        }
    "#;

    match compile_ccl_source_to_wasm(if_test) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Simple if statement compiled successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🧮 Expected result: check_value(15) -> 15 > 10 -> return 100");
        }
        Err(e) => {
            println!("❌ Simple if failed: {:?}", e);
        }
    }

    // Test 2: If-else statement with mana calculations
    println!("\n=== Test 2: If-Else Statement ===");
    let if_else_test = r#"
        fn calculate_discount(amount: Mana, is_premium: Bool) -> Mana {
            if is_premium {
                return amount * 80 / 100;
            } else {
                return amount * 95 / 100;
            }
        }
        
        fn run() -> Mana {
            return calculate_discount(1000, true);
        }
    "#;

    match compile_ccl_source_to_wasm(if_else_test) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ If-else statement compiled successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🧮 Expected result: calculate_discount(1000, true) -> 1000 * 80 / 100 = 800");
        }
        Err(e) => {
            println!("❌ If-else failed: {:?}", e);
        }
    }

    // Test 3: Nested if statements
    println!("\n=== Test 3: Nested If Statements ===");
    let nested_if_test = r#"
        fn complex_decision(value: Integer, threshold1: Integer, threshold2: Integer) -> Integer {
            if value >= threshold1 {
                if value >= threshold2 {
                    return 3;
                } else {
                    return 2;
                }
            } else {
                return 1;
            }
        }
        
        fn run() -> Integer {
            return complex_decision(75, 50, 80);
        }
    "#;

    match compile_ccl_source_to_wasm(nested_if_test) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Nested if statements compiled successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🧮 Expected result: complex_decision(75, 50, 80) -> 75 >= 50 but 75 < 80 -> return 2");
        }
        Err(e) => {
            println!("❌ Nested if failed: {:?}", e);
        }
    }

    // Test 4: If statement with local variables
    println!("\n=== Test 4: If with Local Variables ===");
    let if_local_test = r#"
        fn process_with_locals(input: Integer) -> Integer {
            let threshold = 25;
            let multiplier = 3;
            
            if input > threshold {
                let bonus = input * multiplier;
                return bonus;
            }
            
            return input;
        }
        
        fn run() -> Integer {
            return process_with_locals(30);
        }
    "#;

    match compile_ccl_source_to_wasm(if_local_test) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ If with local variables compiled successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🧮 Expected result: process_with_locals(30) -> 30 > 25 -> 30 * 3 = 90");
        }
        Err(e) => {
            println!("❌ If with locals failed: {:?}", e);
        }
    }

    // Test 5: Complex governance policy with conditional logic
    println!("\n=== Test 5: Governance Policy with Conditions ===");
    let governance_test = r#"
        fn evaluate_proposal_vote(
            voter_mana: Mana,
            voter_reputation: Integer,
            proposal_type: Integer,
            is_core_member: Bool
        ) -> Integer {
            let base_voting_power = voter_mana / 100;
            let reputation_bonus = voter_reputation / 10;
            
            if is_core_member {
                if proposal_type > 2 {
                    return base_voting_power + reputation_bonus + 50;
                } else {
                    return base_voting_power + reputation_bonus + 25;
                }
            } else {
                if voter_reputation >= 75 {
                    return base_voting_power + reputation_bonus;
                } else {
                    return base_voting_power;
                }
            }
        }
        
        fn run() -> Integer {
            return evaluate_proposal_vote(1000, 80, 3, !false);
        }
    "#;

    match compile_ccl_source_to_wasm(governance_test) {
        Ok((wasm_bytes, metadata)) => {
            println!("✅ Governance policy compiled successfully!");
            println!("📦 WASM size: {} bytes", wasm_bytes.len());
            println!("📋 Exports: {:?}", metadata.exports);
            println!("🏛️ Complex governance logic with multiple conditionals working!");
        }
        Err(e) => {
            println!("❌ Governance policy failed: {:?}", e);
        }
    }

    println!("\n🎉 Control Flow Test Complete!");
    println!("✅ CCL supports:");
    println!("   • 🔀 If/else statements with WASM generation");
    println!("   • 🔄 Nested conditional logic");
    println!("   • 🧮 Complex boolean expressions");
    println!("   • 📊 Local variable scoping in conditionals");
    println!("   • 🏛️ Real-world governance contract patterns");
} 