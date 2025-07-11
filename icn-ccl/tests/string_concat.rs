use icn_ccl::compile_ccl_source_to_wasm;
use wasmtime::{Engine, Linker, Module, Store};

#[test]
fn concat_literal_strings() {
    let src = r#"fn run() -> String { return \"foo\" + \"bar\"; }"#;
    let (wasm, _) = compile_ccl_source_to_wasm(src).expect("compile");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm).expect("module");

    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("icn", "host_account_get_mana", || -> i64 { 0 })
        .unwrap();
    linker
        .func_wrap("icn", "host_get_reputation", || -> i64 { 0 })
        .unwrap();
    linker
        .func_wrap("icn", "host_submit_mesh_job", |_: i32, _: i32| {})
        .unwrap();
    linker
        .func_wrap("icn", "host_anchor_receipt", |_: i32, _: i32| {})
        .unwrap();

    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .expect("instantiate");
    let run = instance
        .get_typed_func::<(), i32>(&mut store, "run")
        .unwrap();
    let ptr = run.call(&mut store, ()).expect("run");
    let memory = instance.get_memory(&mut store, "memory").unwrap();

    let mut len_buf = [0u8; 4];
    memory.read(&mut store, ptr as usize, &mut len_buf).unwrap();
    let len = u32::from_le_bytes(len_buf);
    let mut bytes = vec![0u8; len as usize];
    memory
        .read(&mut store, ptr as usize + 4, &mut bytes)
        .unwrap();

    assert_eq!(std::str::from_utf8(&bytes).unwrap(), "foobar");
}
