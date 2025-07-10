use icn_ccl::compile_ccl_source_to_wasm;
use icn_governance_templates::{treasury_rules_template, voting_logic_template};

#[test]
fn compile_voting_logic_template() {
    let src = voting_logic_template();
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_treasury_rules_template() {
    let src = treasury_rules_template();
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}
