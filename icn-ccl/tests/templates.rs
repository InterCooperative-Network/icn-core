use icn_ccl::compile_ccl_source_to_wasm;
use icn_ccl_templates::{TREASURY_TEMPLATE, VOTING_TEMPLATE};

#[test]
fn compile_voting_template() {
    let (wasm, _meta) = compile_ccl_source_to_wasm(VOTING_TEMPLATE).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}

#[test]
fn compile_treasury_template() {
    let (wasm, _meta) = compile_ccl_source_to_wasm(TREASURY_TEMPLATE).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}
