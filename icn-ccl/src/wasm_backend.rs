// icn-ccl/src/wasm_backend.rs
use crate::ast::{
    AstNode, BinaryOperator, BlockNode, ExpressionNode, PolicyStatementNode, StatementNode,
    TypeAnnotationNode, UnaryOperator,
};
use crate::error::CclError;
use crate::metadata::ContractMetadata;
use std::cmp::min;
use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction,
    Module, TypeSection, ValType,
};

use std::collections::HashMap;

struct LocalEnv {
    locals: HashMap<String, (u32, ValType)>,
    order: Vec<ValType>,
    next_local_index: u32,
}

impl LocalEnv {
    fn new() -> Self {
        LocalEnv {
            locals: HashMap::new(),
            order: Vec::new(),
            next_local_index: 0,
        }
    }

    fn get_or_add(&mut self, name: &str, ty: ValType) -> u32 {
        if let Some((idx, _)) = self.locals.get(name) {
            *idx
        } else {
            let idx = self.next_local_index + self.order.len() as u32;
            self.locals.insert(name.to_string(), (idx, ty));
            self.order.push(ty);
            idx
        }
    }

    fn get(&self, name: &str) -> Option<(u32, ValType)> {
        self.locals.get(name).copied()
    }
}

const IMPORT_COUNT: u32 = 4;

pub struct WasmBackend {
    data: wasm_encoder::DataSection,
    data_offset: u32,
}

impl WasmBackend {
    pub fn new() -> Self {
        WasmBackend {
            data: wasm_encoder::DataSection::new(),
            data_offset: 0,
        }
    }

    pub fn compile_to_wasm(
        &mut self,
        ast: &AstNode,
    ) -> Result<(Vec<u8>, ContractMetadata), CclError> {
        let mut types = TypeSection::new();
        let mut imports = ImportSection::new();
        let mut functions = FunctionSection::new();
        let mut codes = CodeSection::new();
        let mut exports = ExportSection::new();
        let mut export_names = Vec::new();
        let mut memories = wasm_encoder::MemorySection::new();
        let mut globals = wasm_encoder::GlobalSection::new();
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

        let ty_get_rep = types.len() as u32;
        types
            .ty()
            .function(Vec::<ValType>::new(), vec![ValType::I64]);
        imports.import(
            "icn",
            "host_get_reputation",
            wasm_encoder::EntityType::Function(ty_get_rep),
        );
        fn_indices.insert("host_get_reputation".to_string(), next_index);
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
                parameters,
                return_type,
                body,
            }) = item
            {
                let ret_ty = map_val_type(return_type)?;

                // Build parameter types for WASM function signature
                let mut param_types = Vec::new();
                for param in parameters {
                    param_types.push(map_val_type(&param.type_ann)?);
                }

                let type_index = types.len();
                types.ty().function(param_types.clone(), vec![ret_ty]);
                functions.function(type_index as u32);
                let func_index = next_index;
                fn_indices.insert(name.clone(), func_index);
                next_index += 1;

                let mut locals = LocalEnv::new();

                // Register function parameters (they don't go in locals.order, only in the name mapping)
                for (i, param) in parameters.iter().enumerate() {
                    let param_type = map_val_type(&param.type_ann)?;
                    locals
                        .locals
                        .insert(param.name.clone(), (i as u32, param_type));
                }

                // Set the starting index for additional local variables after parameters
                locals.next_local_index = parameters.len() as u32;

                let mut instrs = Vec::<Instruction>::new();
                self.emit_block(body, &mut instrs, &mut locals, return_type, &fn_indices)?;
                instrs.push(Instruction::End);

                let mut func = Function::new_with_locals_types(locals.order.clone());
                for inst in instrs {
                    func.instruction(&inst);
                }
                codes.function(&func);

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
        globals.global(
            wasm_encoder::GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &wasm_encoder::ConstExpr::i32_const(self.data_offset as i32),
        );
        module.section(&globals);
        if exports.len() > 0 {
            module.section(&exports);
        }
        if codes.len() > 0 {
            module.section(&codes);
        }
        if self.data.len() > 0 {
            module.section(&self.data);
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
        &mut self,
        expr: &ExpressionNode,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
        indices: &HashMap<String, u32>,
    ) -> Result<ValType, CclError> {
        match expr {
            ExpressionNode::IntegerLiteral(i) => {
                instrs.push(Instruction::I64Const(*i));
                Ok(ValType::I64)
            }
            ExpressionNode::BooleanLiteral(b) => {
                instrs.push(Instruction::I32Const(if *b { 1 } else { 0 }));
                Ok(ValType::I32)
            }
            ExpressionNode::Identifier(name) => {
                let (idx, ty) = locals.get(name).ok_or_else(|| {
                    CclError::WasmGenerationError(format!("Unknown variable {}", name))
                })?;
                instrs.push(Instruction::LocalGet(idx));
                Ok(ty)
            }
            ExpressionNode::FunctionCall { name, arguments } => {
                match name.as_str() {
                    "array_len" => {
                        let ptr_ty =
                            self.emit_expression(&arguments[0], instrs, locals, indices)?;
                        let _ = ptr_ty;
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        Ok(ValType::I32)
                    }
                    "array_push" => {
                        let arr_ptr = locals.get_or_add("__push_ptr", ValType::I32);
                        self.emit_expression(&arguments[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalTee(arr_ptr));
                        let val_ty =
                            self.emit_expression(&arguments[1], instrs, locals, indices)?;
                        let val_is_i64 = val_ty == ValType::I64;
                        // load length
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        let len_local = locals.get_or_add("__push_len", ValType::I32);
                        instrs.push(Instruction::LocalTee(len_local));
                        // store value
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        if !val_is_i64 {
                            instrs.push(Instruction::I64ExtendI32U);
                        }
                        instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        // len + 1
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalTee(len_local));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalGet(len_local));
                        Ok(ValType::I32)
                    }
                    "array_pop" => {
                        let arr_ptr = locals.get_or_add("__pop_ptr", ValType::I32);
                        self.emit_expression(&arguments[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalTee(arr_ptr));
                        // len
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        let len_local = locals.get_or_add("__pop_len", ValType::I32);
                        instrs.push(Instruction::LocalTee(len_local));
                        // len - 1
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Sub);
                        instrs.push(Instruction::LocalTee(len_local));
                        // store new len
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        // load value
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        Ok(ValType::I64)
                    }
                    _ => {
                        let idx = indices.get(name).ok_or_else(|| {
                            CclError::WasmGenerationError(format!("Unknown function {}", name))
                        })?;
                        for arg in arguments {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        instrs.push(Instruction::Call(*idx));
                        let ret = match name.as_str() {
                            "host_account_get_mana" | "host_get_reputation" => ValType::I64,
                            "host_submit_mesh_job" | "host_anchor_receipt" => ValType::I32,
                            _ => ValType::I64,
                        };
                        Ok(ret)
                    }
                }
            }
            ExpressionNode::BinaryOp {
                left,
                operator,
                right,
            } => {
                let l_ty = self.emit_expression(left, instrs, locals, indices)?;
                let r_ty = self.emit_expression(right, instrs, locals, indices)?;
                match (l_ty, r_ty, operator) {
                    (ValType::I64, ValType::I64, BinaryOperator::Add) => {
                        instrs.push(Instruction::I64Add);
                        Ok(ValType::I64)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Sub) => {
                        instrs.push(Instruction::I64Sub);
                        Ok(ValType::I64)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Mul) => {
                        instrs.push(Instruction::I64Mul);
                        Ok(ValType::I64)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Div) => {
                        instrs.push(Instruction::I64DivS);
                        Ok(ValType::I64)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Eq) => {
                        instrs.push(Instruction::I64Eq);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Neq) => {
                        instrs.push(Instruction::I64Ne);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Lt) => {
                        instrs.push(Instruction::I64LtS);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Lte) => {
                        instrs.push(Instruction::I64LeS);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Gt) => {
                        instrs.push(Instruction::I64GtS);
                        Ok(ValType::I32)
                    }
                    (ValType::I64, ValType::I64, BinaryOperator::Gte) => {
                        instrs.push(Instruction::I64GeS);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::And) => {
                        instrs.push(Instruction::I32And);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Or) => {
                        instrs.push(Instruction::I32Or);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Eq) => {
                        instrs.push(Instruction::I32Eq);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Neq) => {
                        instrs.push(Instruction::I32Ne);
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Concat) => {
                        // String concatenation - simplified: combine pointers
                        instrs.push(Instruction::I32Add);
                        Ok(ValType::I32)
                    }
                    _ => Err(CclError::WasmGenerationError(
                        "Unsupported binary operation".to_string(),
                    )),
                }
            }
            ExpressionNode::StringLiteral(s) => {
                // Allocate a data segment for the string and push the pointer
                let mut bytes = (s.len() as u32).to_le_bytes().to_vec();
                bytes.extend_from_slice(s.as_bytes());
                let ptr = self.data_offset;
                let len = bytes.len() as u32;
                let offset = wasm_encoder::ConstExpr::i32_const(ptr as i32);
                self.data.active(0, &offset, bytes.into_boxed_slice());
                self.data_offset += len;
                instrs.push(Instruction::I32Const(ptr as i32));
                Ok(ValType::I32)
            }
            ExpressionNode::ArrayLiteral(elements) => {
                // Allocate array in guest memory: [len][elements]
                let size = 4 + elements.len() * 8;
                instrs.push(Instruction::GlobalGet(0));
                let tmp = locals.get_or_add("__arr_ptr", ValType::I32);
                instrs.push(Instruction::LocalTee(tmp));
                instrs.push(Instruction::GlobalGet(0));
                instrs.push(Instruction::I32Const(size as i32));
                instrs.push(Instruction::I32Add);
                instrs.push(Instruction::GlobalSet(0));

                // store length
                instrs.push(Instruction::LocalGet(tmp));
                instrs.push(Instruction::I32Const(elements.len() as i32));
                instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));

                for (i, el) in elements.iter().enumerate() {
                    self.emit_expression(el, instrs, locals, indices)?;
                    instrs.push(Instruction::LocalGet(tmp));
                    instrs.push(Instruction::I32Const(4 + (i as i32) * 8));
                    instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                }

                instrs.push(Instruction::LocalGet(tmp));
                Ok(ValType::I32)
            }
            ExpressionNode::ArrayAccess { array, index } => {
                let arr_ty = self.emit_expression(array, instrs, locals, indices)?;
                let arr_local = locals.get_or_add("__arr", ValType::I32);
                instrs.push(Instruction::LocalTee(arr_local));
                let _ = arr_ty;
                let idx_ty = self.emit_expression(index, instrs, locals, indices)?;
                if idx_ty == ValType::I64 {
                    instrs.push(Instruction::I32WrapI64);
                }
                let idx_local = locals.get_or_add("__idx", ValType::I32);
                instrs.push(Instruction::LocalTee(idx_local));
                instrs.push(Instruction::I32Const(8));
                instrs.push(Instruction::I32Mul);
                instrs.push(Instruction::LocalGet(arr_local));
                instrs.push(Instruction::I32Const(4));
                instrs.push(Instruction::I32Add);
                instrs.push(Instruction::I32Add);
                instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));
                Ok(ValType::I64)
            }
            ExpressionNode::UnaryOp { operator, operand } => {
                let operand_ty = self.emit_expression(operand, instrs, locals, indices)?;
                match (operator, operand_ty) {
                    (UnaryOperator::Not, ValType::I32) => {
                        // Boolean negation: !x = x == 0
                        instrs.push(Instruction::I32Eqz);
                        Ok(ValType::I32)
                    }
                    (UnaryOperator::Neg, ValType::I32) => {
                        // Integer negation: -x = 0 - x
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::I32Sub);
                        Ok(ValType::I32)
                    }
                    (UnaryOperator::Neg, ValType::I64) => {
                        // 64-bit integer negation: -x = 0 - x
                        instrs.push(Instruction::I64Const(0));
                        instrs.push(Instruction::I64Sub);
                        Ok(ValType::I64)
                    }
                    _ => Err(CclError::WasmGenerationError(format!(
                        "Unsupported unary operation: {:?} on {:?}",
                        operator, operand_ty
                    ))),
                }
            }
        }
    }

    fn emit_statement(
        &mut self,
        stmt: &StatementNode,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
        return_ty: &TypeAnnotationNode,
        indices: &HashMap<String, u32>,
    ) -> Result<(), CclError> {
        match stmt {
            StatementNode::Let { name, value } => {
                let ty = self.emit_expression(value, instrs, locals, indices)?;
                let idx = locals.get_or_add(name, ty);
                instrs.push(Instruction::LocalSet(idx));
            }
            StatementNode::ExpressionStatement(expr) => {
                self.emit_expression(expr, instrs, locals, indices)?;
                instrs.push(Instruction::Drop);
            }
            StatementNode::Return(expr) => {
                let ty = self.emit_expression(expr, instrs, locals, indices)?;
                let expected = map_val_type(return_ty)?;
                if ty != expected {
                    return Err(CclError::WasmGenerationError(
                        "Return type mismatch during codegen".to_string(),
                    ));
                }
                instrs.push(Instruction::Return);
            }
            StatementNode::If {
                condition,
                then_block,
                else_block,
            } => {
                self.emit_if_statement(
                    condition, then_block, else_block, instrs, locals, return_ty, indices,
                )?;
            }
            StatementNode::WhileLoop { condition, body } => {
                instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
                instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
                let cond_ty = self.emit_expression(condition, instrs, locals, indices)?;
                if cond_ty != ValType::I32 {
                    return Err(CclError::WasmGenerationError(
                        "While condition must be boolean".to_string(),
                    ));
                }
                instrs.push(Instruction::I32Eqz);
                instrs.push(Instruction::BrIf(1));
                self.emit_block(body, instrs, locals, return_ty, indices)?;
                instrs.push(Instruction::Br(0));
                instrs.push(Instruction::End);
                instrs.push(Instruction::End);
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn emit_if_statement(
        &mut self,
        condition: &ExpressionNode,
        then_block: &BlockNode,
        else_block: &Option<BlockNode>,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
        return_ty: &TypeAnnotationNode,
        indices: &HashMap<String, u32>,
    ) -> Result<(), CclError> {
        // Emit condition
        let cond_ty = self.emit_expression(condition, instrs, locals, indices)?;
        if cond_ty != ValType::I32 {
            return Err(CclError::WasmGenerationError(
                "If condition must be boolean".to_string(),
            ));
        }

        if let Some(else_blk) = else_block {
            instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
            self.emit_block(then_block, instrs, locals, return_ty, indices)?;
            instrs.push(Instruction::Else);
            if else_blk.statements.len() == 1 {
                if let StatementNode::If {
                    condition: c,
                    then_block: t,
                    else_block: e,
                } = &else_blk.statements[0]
                {
                    self.emit_if_statement(c, t, e, instrs, locals, return_ty, indices)?;
                } else {
                    self.emit_block(else_blk, instrs, locals, return_ty, indices)?;
                }
            } else {
                self.emit_block(else_blk, instrs, locals, return_ty, indices)?;
            }
            instrs.push(Instruction::End);
        } else {
            instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
            self.emit_block(then_block, instrs, locals, return_ty, indices)?;
            instrs.push(Instruction::End);
        }
        Ok(())
    }

    fn emit_block(
        &mut self,
        block: &BlockNode,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
        return_ty: &TypeAnnotationNode,
        indices: &HashMap<String, u32>,
    ) -> Result<(), CclError> {
        for stmt in &block.statements {
            self.emit_statement(stmt, instrs, locals, return_ty, indices)?;
        }
        Ok(())
    }
}

fn map_val_type(ty: &TypeAnnotationNode) -> Result<ValType, CclError> {
    match ty {
        TypeAnnotationNode::Mana | TypeAnnotationNode::Integer | TypeAnnotationNode::Did => {
            Ok(ValType::I64)
        }
        TypeAnnotationNode::Bool => Ok(ValType::I32),
        TypeAnnotationNode::String => Ok(ValType::I32),
        TypeAnnotationNode::Array(_) => {
            // Arrays represented as i32 pointer to array metadata
            Ok(ValType::I32)
        }
        TypeAnnotationNode::Proposal | TypeAnnotationNode::Vote => {
            // Governance types represented as i64 handles
            Ok(ValType::I64)
        }
        TypeAnnotationNode::Custom(name) => Err(CclError::WasmGenerationError(format!(
            "Unsupported type {}",
            name
        ))),
    }
}
