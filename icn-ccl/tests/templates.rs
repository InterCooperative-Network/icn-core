use icn_ccl::compile_ccl_source_to_wasm;
use icn_templates::{SIMPLE_VOTING, TREASURY_RULES};

#[test]
fn compile_voting_template() {
    let (wasm, _meta) = compile_ccl_source_to_wasm(SIMPLE_VOTING).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_treasury_template() {
    let (wasm, _meta) = compile_ccl_source_to_wasm(TREASURY_RULES).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}
