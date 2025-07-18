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
fn compile_string_length_operation() {
    let src = r#"
        fn run() -> Integer {
            let text = "Hello, World!";
            return length(text);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_substring_operation() {
    let src = r#"
        fn run() -> String {
            let text = "Hello, World!";
            return substring(text, 0, 5);
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
fn compile_string_contains_operation() {
    let src = r#"
        fn run() -> Bool {
            let text = "InterCooperative Network";
            let search = "Cooperative";
            return contains(text, search);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_indexof_operation() {
    let src = r#"
        fn run() -> Integer {
            let text = "Hello, World!";
            let pattern = "World";
            return indexof(text, pattern);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_tolower_operation() {
    let src = r#"
        fn run() -> String {
            let text = "Hello World";
            return tolower(text);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_toupper_operation() {
    let src = r#"
        fn run() -> String {
            let text = "hello world";
            return toupper(text);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_trim_operation() {
    let src = r#"
        fn run() -> String {
            let text = "  hello world  ";
            return trim(text);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_operations_in_governance() {
    let src = r#"
        fn validate_proposal_title(title: String) -> Bool {
            let min_length = 10;
            let max_length = 100;
            let title_length = length(title);
            
            if title_length < min_length || title_length > max_length {
                return false;
            }
            
            let trimmed_title = trim(title);
            if length(trimmed_title) == 0 {
                return false;
            }
            
            // Check for required keywords
            let lower_title = tolower(trimmed_title);
            if contains(lower_title, "proposal") || contains(lower_title, "motion") {
                return true;
            }
            
            return false;
        }

        fn run() -> Bool {
            return validate_proposal_title("  Proposal: Increase Mana Regeneration Rate  ");
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_concatenation_with_variables() {
    let src = r#"
        fn format_member_id(member_name: String, member_number: Integer) -> String {
            let prefix = "COOP-";
            let number_str = tostring(member_number);
            let separator = "-";
            
            return prefix + toupper(substring(member_name, 0, 3)) + separator + number_str;
        }

        fn run() -> String {
            return format_member_id("alice", 42);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_parsing_operations() {
    let src = r#"
        fn parse_governance_tag(tag: String) -> Integer {
            let cleaned_tag = trim(tolower(tag));
            
            if cleaned_tag == "urgent" {
                return 3;
            } else if cleaned_tag == "important" {
                return 2;
            } else if cleaned_tag == "normal" {
                return 1;
            }
            
            return 0;
        }

        fn run() -> Integer {
            return parse_governance_tag("  URGENT  ");
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_string_validation_functions() {
    let src = r#"
        fn validate_did_format(did: String) -> Bool {
            let prefix = "did:icn:";
            let prefix_length = length(prefix);
            let did_length = length(did);
            
            if did_length <= prefix_length {
                return false;
            }
            
            let did_prefix = substring(did, 0, prefix_length);
            if did_prefix != prefix {
                return false;
            }
            
            let identifier_part = substring(did, prefix_length, did_length - prefix_length);
            let identifier_length = length(identifier_part);
            
            return identifier_length >= 10 && identifier_length <= 50;
        }

        fn run() -> Bool {
            return validate_did_format("did:icn:1234567890abcdef");
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_complex_string_processing() {
    let src = r#"
        fn process_proposal_description(description: String) -> String {
            let trimmed = trim(description);
            let max_summary_length = 50;
            let description_length = length(trimmed);
            
            if description_length <= max_summary_length {
                return trimmed;
            }
            
            let summary = substring(trimmed, 0, max_summary_length - 3);
            return summary + "...";
        }

        fn categorize_by_keywords(text: String) -> Integer {
            let lower_text = tolower(text);
            
            if contains(lower_text, "budget") || contains(lower_text, "finance") {
                return 1; // Financial
            } else if contains(lower_text, "governance") || contains(lower_text, "voting") {
                return 2; // Governance
            } else if contains(lower_text, "technical") || contains(lower_text, "infrastructure") {
                return 3; // Technical
            }
            
            return 0; // General
        }

        fn run() -> Integer {
            let description = "This proposal addresses technical infrastructure improvements for the governance system";
            let summary = process_proposal_description(description);
            return categorize_by_keywords(summary);
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}