use icn_ccl::compile_ccl_file_to_wasm;
use std::path::PathBuf;
use wasmtime::{Engine, Linker, Module, Store};

fn run_contract(path: &str) -> i32 {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path);
    let (wasm, _) = compile_ccl_file_to_wasm(&path).expect("compile");
    let engine = Engine::default();
    let module = Module::new(&engine, &wasm).expect("module");
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("icn", "host_account_get_mana", || -> i64 { 0 })
        .unwrap();
    linker
        .func_wrap("icn", "host_get_reputation", || -> i64 { 1 })
        .unwrap();
    linker
        .func_wrap("icn", "host_submit_mesh_job", |_: i32, _: i32| {})
        .unwrap();
    linker
        .func_wrap("icn", "host_anchor_receipt", |_: i32, _: i32| {})
        .unwrap();
    linker
        .func_wrap("icn", "host_get_current_time", || -> i64 { 0 })
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).expect("inst");
    let run = instance
        .get_typed_func::<(), i32>(&mut store, "run")
        .unwrap();
    run.call(&mut store, ()).expect("run")
}

#[test]
fn delegation_increases_votes() {
    let with_del = run_contract("tests/contracts/assembly_delegation.ccl");
    let without_del = run_contract("tests/contracts/assembly_no_delegation.ccl");
    assert_eq!(with_del, 2);
    assert_eq!(without_del, 1);
}
