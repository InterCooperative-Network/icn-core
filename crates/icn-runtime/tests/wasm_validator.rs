use icn_common::CommonError;
use icn_runtime::executor::{WasmModuleValidator, WasmSecurityLimits};

#[test]
fn validator_rejects_excess_memory() {
    let limits = WasmSecurityLimits {
        max_memory_pages: 1,
        ..Default::default()
    };
    let validator = WasmModuleValidator::new(limits);
    let wasm = "(module (memory 2) (func (export \"run\") (result i64) i64.const 1))";
    let bytes = wat::parse_str(wasm).unwrap();
    let res = validator.validate(&bytes);
    assert!(matches!(res, Err(CommonError::PolicyDenied(_))));
}

#[test]
fn validator_rejects_excess_functions() {
    let limits = WasmSecurityLimits {
        max_functions: 1,
        ..Default::default()
    };
    let validator = WasmModuleValidator::new(limits);
    let wasm = "(module (func $a) (func (export \"run\")))";
    let bytes = wat::parse_str(wasm).unwrap();
    let res = validator.validate(&bytes);
    assert!(matches!(res, Err(CommonError::PolicyDenied(_))));
}

#[test]
fn validator_accepts_valid_module() {
    let limits = WasmSecurityLimits::default();
    let validator = WasmModuleValidator::new(limits);
    let wasm = "(module (memory 1) (func (export \"run\") (result i64) i64.const 0))";
    let bytes = wat::parse_str(wasm).unwrap();
    assert!(validator.validate(&bytes).is_ok());
}
