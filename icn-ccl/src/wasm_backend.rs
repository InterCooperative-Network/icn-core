// icn-ccl/src/wasm_backend.rs
use crate::ast::{AstNode, PolicyStatementNode, TypeAnnotationNode};
use crate::error::CclError;
use crate::metadata::ContractMetadata;
use std::cmp::min;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, Instruction, Module,
    TypeSection, ValType,
};

pub struct WasmBackend {}

impl WasmBackend {
    pub fn new() -> Self {
        WasmBackend {}
    }

    pub fn compile_to_wasm(&self, ast: &AstNode) -> Result<(Vec<u8>, ContractMetadata), CclError> {
        let mut types = TypeSection::new();
        let mut functions = FunctionSection::new();
        let mut codes = CodeSection::new();
        let mut exports = ExportSection::new();
        let mut export_names = Vec::new();

        let policy_items = match ast {
            AstNode::Policy(items) => items,
            _ => {
                return Err(CclError::WasmGenerationError(
                    "Expected policy as top level AST".to_string(),
                ))
            }
        };

        for item in policy_items {
            if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition {
                name,
                return_type,
                ..
            }) = item
            {
                let ret_ty = map_val_type(return_type)?;
                let type_index = types.len();
                types.function(Vec::<ValType>::new(), vec![ret_ty.clone()]);
                functions.add(type_index as u32);

                let mut func = Function::new(Vec::new());
                match ret_ty {
                    ValType::I32 => {
                        func.instruction(&Instruction::I32Const(0));
                    }
                    ValType::I64 => {
                        func.instruction(&Instruction::I64Const(0));
                    }
                    _ => {}
                }
                func.instruction(&Instruction::End);
                codes.add(&func);

                let func_index = (functions.len() - 1) as u32;
                exports.export(name, ExportKind::Func, func_index);
                export_names.push(name.clone());
            }
        }

        let mut module = Module::new();
        if types.len() > 0 {
            module.section(&types);
        }
        if functions.len() > 0 {
            module.section(&functions);
        }
        if exports.len() > 0 {
            module.section(&exports);
        }
        if codes.len() > 0 {
            module.section(&codes);
        }

        let wasm_bytes = module.finish();

        let metadata = ContractMetadata {
            cid: format!(
                "bafy2bzace{}",
                hex::encode(&wasm_bytes[0..min(10, wasm_bytes.len())])
            ),
            exports: export_names,
            inputs: Vec::new(),
            version: "0.1.0".to_string(),
            source_hash: "sha256_of_ccl_source_code".to_string(),
        };

        Ok((wasm_bytes, metadata))
    }
}

fn map_val_type(ty: &TypeAnnotationNode) -> Result<ValType, CclError> {
    match ty {
        TypeAnnotationNode::Mana | TypeAnnotationNode::Integer | TypeAnnotationNode::Did => {
            Ok(ValType::I64)
        }
        TypeAnnotationNode::Bool => Ok(ValType::I32),
        TypeAnnotationNode::String => Ok(ValType::I64),
        TypeAnnotationNode::Custom(name) => Err(CclError::WasmGenerationError(format!(
            "Unsupported type {}",
            name
        ))),
    }
}
