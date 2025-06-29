// icn-ccl/src/wasm_backend.rs
use crate::ast::{
    AstNode, BinaryOperator, ExpressionNode, PolicyStatementNode, StatementNode, TypeAnnotationNode,
};
use crate::error::CclError;
use crate::metadata::ContractMetadata;
use std::cmp::min;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    Module, TypeSection, ValType,
};

use std::collections::HashMap;

const IMPORT_COUNT: u32 = 3;

pub struct WasmBackend {}

impl WasmBackend {
    pub fn new() -> Self {
        WasmBackend {}
    }

    pub fn compile_to_wasm(&self, ast: &AstNode) -> Result<(Vec<u8>, ContractMetadata), CclError> {
        let mut types = TypeSection::new();
        let mut imports = ImportSection::new();
        let mut functions = FunctionSection::new();
        let mut codes = CodeSection::new();
        let mut exports = ExportSection::new();
        let mut export_names = Vec::new();
        let mut memories = wasm_encoder::MemorySection::new();
        memories.memory(wasm_encoder::MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });

        // Map of function name -> index in the function table
        let mut fn_indices = HashMap::<String, u32>::new();
        let mut next_index: u32 = 0;

        // Host function imports expected by WasmExecutor
        let ty_get_mana = types.len() as u32;
        types
            .ty()
            .function(Vec::<ValType>::new(), vec![ValType::I64]);
        imports.import(
            "icn",
            "host_account_get_mana",
            wasm_encoder::EntityType::Function(ty_get_mana),
        );
        fn_indices.insert("host_account_get_mana".to_string(), next_index);
        next_index += 1;

        let ty_submit = types.len() as u32;
        types
            .ty()
            .function(vec![ValType::I32, ValType::I32], Vec::<ValType>::new());
        imports.import(
            "icn",
            "host_submit_mesh_job",
            wasm_encoder::EntityType::Function(ty_submit),
        );
        fn_indices.insert("host_submit_mesh_job".to_string(), next_index);
        next_index += 1;

        let ty_anchor = types.len() as u32;
        types
            .ty()
            .function(vec![ValType::I32, ValType::I32], Vec::<ValType>::new());
        imports.import(
            "icn",
            "host_anchor_receipt",
            wasm_encoder::EntityType::Function(ty_anchor),
        );
        fn_indices.insert("host_anchor_receipt".to_string(), next_index);
        next_index += 1;

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
                body,
                ..
            }) = item
            {
                let ret_ty = map_val_type(return_type)?;
                let type_index = types.len();
                types.ty().function(Vec::<ValType>::new(), vec![ret_ty]);
                functions.function(type_index as u32);
                let func_index = next_index;
                fn_indices.insert(name.clone(), func_index);
                next_index += 1;

                let mut func = Function::new(Vec::new());

                // Expect a single return statement for now
                if let Some(StatementNode::Return(expr)) = body.statements.first() {
                    let expr_ty = self.emit_expression(expr, &mut func, &fn_indices)?;
                    let expected = map_val_type(return_type)?;
                    if expr_ty != expected {
                        return Err(CclError::WasmGenerationError(
                            "Return type mismatch during codegen".to_string(),
                        ));
                    }
                } else {
                    return Err(CclError::WasmGenerationError(
                        "Only simple return-only functions supported".to_string(),
                    ));
                }

                func.instruction(&Instruction::End);
                codes.function(&func);

                // Final index within the module = imported count + position in function section
                let func_index = IMPORT_COUNT + (functions.len() - 1) as u32;
                exports.export(name, ExportKind::Func, func_index);
                export_names.push(name.clone());
            }
        }

        let mut module = Module::new();
        if types.len() > 0 {
            module.section(&types);
        }
        if imports.len() > 0 {
            module.section(&imports);
        }
        if functions.len() > 0 {
            module.section(&functions);
        }
        if memories.len() > 0 {
            module.section(&memories);
            exports.export("memory", ExportKind::Memory, 0);
            export_names.push("memory".to_string());
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

    fn emit_expression(
        &self,
        expr: &ExpressionNode,
        func: &mut Function,
        indices: &HashMap<String, u32>,
    ) -> Result<ValType, CclError> {
        match expr {
            ExpressionNode::IntegerLiteral(i) => {
                func.instruction(&Instruction::I64Const(*i));
                Ok(ValType::I64)
            }
            ExpressionNode::BooleanLiteral(b) => {
                func.instruction(&Instruction::I32Const(if *b { 1 } else { 0 }));
                Ok(ValType::I32)
            }
            ExpressionNode::FunctionCall { name, arguments } => {
                let idx = indices.get(name).ok_or_else(|| {
                    CclError::WasmGenerationError(format!("Unknown function {}", name))
                })?;
                for arg in arguments {
                    self.emit_expression(arg, func, indices)?;
                }
                func.instruction(&Instruction::Call(*idx));
                let ret = match name.as_str() {
                    "host_account_get_mana" => ValType::I64,
                    "host_submit_mesh_job" | "host_anchor_receipt" => ValType::I32,
                    _ => ValType::I64,
                };
                Ok(ret)
            }
            ExpressionNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                let l_ty = self.emit_expression(left, func, indices)?;
                let r_ty = self.emit_expression(right, func, indices)?;
                match (l_ty, r_ty, operator) {
                    (ValType::I64, ValType::I64, BinaryOperator::Add) => {
                        func.instruction(&Instruction::I64Add);
                        Ok(ValType::I64)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Sub) => {
                        func.instruction(&Instruction::I64Sub);
                        Ok(ValType::I64)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Mul) => {
                        func.instruction(&Instruction::I64Mul);
                        Ok(ValType::I64)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Div) => {
                        func.instruction(&Instruction::I64DivS);
                        Ok(ValType::I64)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Eq) => {
                        func.instruction(&Instruction::I64Eq);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Neq) => {
                        func.instruction(&Instruction::I64Ne);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Lt) => {
                        func.instruction(&Instruction::I64LtS);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Lte) => {
                        func.instruction(&Instruction::I64LeS);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Gt) => {
                        func.instruction(&Instruction::I64GtS);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Gte) => {
                        func.instruction(&Instruction::I64GeS);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::And) => {
                        func.instruction(&Instruction::I32And);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Or) => {
                        func.instruction(&Instruction::I32Or);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Eq) => {
                        func.instruction(&Instruction::I32Eq);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Neq) => {
                        func.instruction(&Instruction::I32Ne);
                        Ok(ValType::I32)
                    }
                    _ => Err(CclError::WasmGenerationError(
                        "Unsupported binary operation".to_string(),
                    )),
                }
            }
            _ => Err(CclError::WasmGenerationError(
                "Unsupported expression type".to_string(),
            )),
        }
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
