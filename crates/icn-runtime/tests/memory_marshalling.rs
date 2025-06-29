use icn_runtime::{memory, context::RuntimeContext};
use wasmtime::{Engine, Linker, Module, Store};
use std::sync::Arc;

#[tokio::test]
async fn write_string_limited_truncates() {
    let ctx = RuntimeContext::new_with_stubs("did:key:zMemTest");
    let engine = Engine::default();
    let module_wat = r#"(module
        (import "t" "write" (func $write (param i32 i32) (result i32)))
        (memory (export "memory") 1)
        (func (export "run") (param i32 i32) (result i32) (local.get 0) (local.get 1) call $write)
    )"#;
    let module = Module::new(&engine, wat::parse_str(module_wat).unwrap()).unwrap();
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("t", "write", |mut caller: wasmtime::Caller<'_, Arc<RuntimeContext>>, ptr: u32, len: u32| -> i32 {
            memory::write_string_limited(&mut caller, ptr, "hello world", len).unwrap() as i32
        })
        .unwrap();
    let mut store = Store::new(&engine, ctx.clone());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let run = instance.get_typed_func::<(i32, i32), i32>(&mut store, "run").unwrap();
    let memory = instance.get_memory(&mut store, "memory").unwrap();
    let written = run.call(&mut store, (0, 5)).unwrap();
    assert_eq!(written, 5);
    let mut buf = [0u8; 5];
    memory.read(&mut store, 0, &mut buf).unwrap();
    assert_eq!(&buf, b"hello");
}

#[tokio::test]
async fn read_string_safe_empty() {
    let ctx = RuntimeContext::new_with_stubs("did:key:zMemRead");
    let engine = Engine::default();
    let module_wat = r#"(module
        (import "t" "read" (func $read (param i32 i32) (result i32)))
        (memory (export "memory") 1)
        (func (export "run") (param i32 i32) (result i32) (local.get 0) (local.get 1) call $read)
    )"#;
    let module = Module::new(&engine, wat::parse_str(module_wat).unwrap()).unwrap();
    let mut linker = Linker::new(&engine);
    linker
        .func_wrap("t", "read", |mut caller: wasmtime::Caller<'_, Arc<RuntimeContext>>, ptr: u32, len: u32| -> i32 {
            let s = memory::read_string_safe(&mut caller, ptr, len).unwrap();
            if s.is_empty() { 1 } else { 0 }
        })
        .unwrap();
    let mut store = Store::new(&engine, ctx.clone());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let run = instance.get_typed_func::<(i32, i32), i32>(&mut store, "run").unwrap();
    let result = run.call(&mut store, (0, 0)).unwrap();
    assert_eq!(result, 1);
}
