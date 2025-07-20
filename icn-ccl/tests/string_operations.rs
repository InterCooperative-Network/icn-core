use icn_ccl::compile_ccl_source_to_wasm;

#[test]
fn compile_string_concatenation() {
    let src = r#"
        fn run() -> String {
            let greeting = "Hello";
            let target = "World";
            return greeting + " " + target + "!";
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_concatenation_with_variables() {
    let src = r#"
        fn format_member_id(member_name: String, prefix: String) -> String {
            let separator = "-";
            return prefix + separator + member_name;
        }

        fn run() -> String {
            return format_member_id("alice", "COOP");
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_comparison() {
    let src = r#"
        fn run() -> Bool {
            let str1 = "cooperative";
            let str2 = "cooperative";
            return str1 == str2;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_operations_in_governance() {
    let src = r#"
        fn validate_proposal_title(title: String) -> String {
            let prefix = "PROPOSAL: ";
            let suffix = " [PENDING]";
            return prefix + title + suffix;
        }

        fn run() -> String {
            return validate_proposal_title("Increase Mana Regeneration Rate");
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_conditional_string_building() {
    let src = r#"
        fn format_status_message(is_approved: Bool, proposal_name: String) -> String {
            let status = "";
            
            if is_approved {
                status = "APPROVED: ";
            } else {
                status = "REJECTED: ";
            }
            
            return status + proposal_name;
        }

        fn run() -> String {
            return format_status_message(true, "Budget Amendment");
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_with_empty_values() {
    let src = r#"
        fn run() -> String {
            let empty = "";
            let text = "Hello";
            return empty + text + empty;
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_complex_string_building() {
    let src = r#"
        fn build_governance_message(
            proposal_id: String,
            vote_count: String,
            threshold: String
        ) -> String {
            let part1 = "Proposal " + proposal_id;
            let part2 = " received " + vote_count + " votes";
            let part3 = " (threshold: " + threshold + ")";
            return part1 + part2 + part3;
        }

        fn run() -> String {
            return build_governance_message("P001", "25", "20");
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}
