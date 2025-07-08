use icn_ccl::compile_ccl_source_to_wasm;

#[test]
fn compile_advanced_function_params() {
    let src = r#"
        fn process_data(base: Integer, multiplier: Integer, label: String) -> Integer {
            let result = base * multiplier;
            return result;
        }

        fn calculate_bonus(score: Integer, level: Integer) -> Integer {
            let bonus = score + level * 10;
            return bonus;
        }

        fn run() -> Integer {
            let total = process_data(5, 3, "test");
            let bonus = calculate_bonus(total, 2);
            return bonus;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_operations() {
    let src = r#"
        fn build_message(prefix: String, suffix: String) -> String {
            let message = prefix + suffix;
            return message;
        }

        fn run() -> String {
            return build_message("Hello ", "ICN!");
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_array_literals() {
    let src = r#"
        fn process_scores(scores: Array<Integer>) -> Integer {
            return 100;
        }

        fn run() -> Integer {
            let numbers = [1, 2, 3, 4, 5];
            return process_scores(numbers);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_governance_contract() {
    let src = r#"
        fn calculate_voting_power(mana: Mana, reputation: Integer) -> Integer {
            let base_power = mana / 100;
            let bonus = reputation * 2;
            return base_power + bonus;
        }

        fn check_quorum(total_votes: Integer, min_threshold: Integer) -> Bool {
            return total_votes >= min_threshold;
        }

        fn validate_proposal(submitter_mana: Mana, min_mana: Mana) -> Bool {
            return submitter_mana >= min_mana;
        }

        fn run() -> Integer {
            let voter_mana = 1000;
            let voter_reputation = 50;
            let voting_power = calculate_voting_power(voter_mana, voter_reputation);

            if check_quorum(voting_power, 25) {
                return voting_power;
            } else {
                return 0;
            }
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_economic_policy() {
    let src = r#"
        fn calculate_mana_cost(cpu_cores: Integer, memory_mb: Integer, duration_ms: Integer) -> Mana {
            let cpu_cost = cpu_cores * 10;
            let memory_cost = memory_mb / 256;
            let time_cost = duration_ms / 1000;
            return cpu_cost + memory_cost + time_cost;
        }

        fn apply_reputation_discount(base_cost: Mana, reputation: Integer) -> Mana {
            if reputation > 80 {
                return base_cost * 80 / 100;
            } else {
                return base_cost;
            }
        }

        fn run() -> Mana {
            let base_cost = calculate_mana_cost(4, 1024, 30000);
            let final_cost = apply_reputation_discount(base_cost, 85);
            return final_cost;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}
