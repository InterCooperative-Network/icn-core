use icn_ccl::compile_ccl_source_to_wasm;

#[test]
fn compile_match_with_option() {
    let src = r#"
        fn run() -> Integer {
            let v = Some(2);
            match v { 2 => 10, _ => 0 }
        }
    "#;
    let (wasm, _meta) = compile_ccl_source_to_wasm(src).expect("compile");
    assert!(wasm.starts_with(b"\0asm"));
}
