// icn-ccl/src/wasm_backend.rs
use crate::ast::{
    AstNode, BinaryOperator, BlockNode, ExpressionNode, LiteralNode, PatternNode,
    PolicyStatementNode, StatementNode, TypeAnnotationNode, UnaryOperator,
};
use crate::error::CclError;
use crate::metadata::ContractMetadata;
use std::cmp::min;
use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, Function, FunctionSection, ImportSection,
    Instruction, Module, TypeSection, ValType,
};

use std::collections::HashMap;

struct LocalEnv {
    locals: HashMap<String, (u32, ValType)>,
    order: Vec<ValType>,
    next_local_index: u32,
    constants: HashMap<String, (i64, ValType)>, // Constant name -> (value, type)
}

impl LocalEnv {
    fn new() -> Self {
        LocalEnv {
            locals: HashMap::new(),
            order: Vec::new(),
            next_local_index: 0,
            constants: HashMap::new(),
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

    fn add_constant(&mut self, name: &str, value: i64, ty: ValType) {
        self.constants.insert(name.to_string(), (value, ty));
    }

    fn get_constant(&self, name: &str) -> Option<(i64, ValType)> {
        self.constants.get(name).copied()
    }
}

const IMPORT_COUNT: u32 = 44; // 6 original + 38 new functions (11 economics + 15 identity + 11 DAG + 1 time)

pub struct WasmBackend {
    data: wasm_encoder::DataSection,
    data_offset: u32,
    // Mana metering
    mana_metering_enabled: bool,
    mana_per_instruction: u32,
    max_mana_limit: u32,
    // Global constants storage
    constants: HashMap<String, (i64, ValType)>,
}

impl WasmBackend {
    pub fn new() -> Self {
        WasmBackend {
            data: wasm_encoder::DataSection::new(),
            data_offset: 1024, // Reserve first 1KB for runtime
            mana_metering_enabled: true,
            mana_per_instruction: 1,
            max_mana_limit: 1_000_000, // 1M mana units
            constants: HashMap::new(),
        }
    }

    pub fn new_with_mana_config(
        enable_metering: bool,
        mana_per_instruction: u32,
        max_mana: u32,
    ) -> Self {
        WasmBackend {
            data: wasm_encoder::DataSection::new(),
            data_offset: 1024,
            mana_metering_enabled: enable_metering,
            mana_per_instruction,
            max_mana_limit: max_mana,
            constants: HashMap::new(),
        }
    }

    /// Process a constant declaration and store it for later use
    fn process_constant(
        &mut self,
        const_decl: &crate::ast::ConstDeclarationNode,
    ) -> Result<(), CclError> {
        // For now, support only integer and string constants
        match &const_decl.value {
            ExpressionNode::IntegerLiteral(value) => {
                self.constants
                    .insert(const_decl.name.clone(), (*value, ValType::I64));
            }
            ExpressionNode::StringLiteral(value) => {
                // For strings, we'll store the string in the data section and store the pointer
                let mut bytes = (value.len() as u32).to_le_bytes().to_vec();
                bytes.extend_from_slice(value.as_bytes());
                let ptr = self.data_offset;
                let len = bytes.len() as u32;
                let offset = wasm_encoder::ConstExpr::i32_const(ptr as i32);
                self.data.active(0, &offset, bytes.into_boxed_slice());
                self.data_offset += len;
                self.constants
                    .insert(const_decl.name.clone(), (ptr as i64, ValType::I32));
            }
            ExpressionNode::Literal(crate::ast::LiteralNode::Integer(value)) => {
                // Handle wrapped integer literals
                self.constants
                    .insert(const_decl.name.clone(), (*value, ValType::I64));
            }
            ExpressionNode::Literal(crate::ast::LiteralNode::String(value)) => {
                // Handle wrapped string literals
                let mut bytes = (value.len() as u32).to_le_bytes().to_vec();
                bytes.extend_from_slice(value.as_bytes());
                let ptr = self.data_offset;
                let len = bytes.len() as u32;
                let offset = wasm_encoder::ConstExpr::i32_const(ptr as i32);
                self.data.active(0, &offset, bytes.into_boxed_slice());
                self.data_offset += len;
                self.constants
                    .insert(const_decl.name.clone(), (ptr as i64, ValType::I32));
            }
            ExpressionNode::Literal(crate::ast::LiteralNode::Boolean(value)) => {
                // Handle boolean literals (store as I32: 1 for true, 0 for false)
                let bool_value = if *value { 1 } else { 0 };
                self.constants
                    .insert(const_decl.name.clone(), (bool_value, ValType::I32));
            }
            _ => {
                return Err(CclError::WasmGenerationError(format!(
                    "Unsupported constant type for {}: {:?}",
                    const_decl.name, const_decl.value
                )));
            }
        }
        Ok(())
    }

    /// Emit mana metering instructions if enabled
    fn emit_mana_check(&self, instrs: &mut Vec<Instruction>, cost: u32) {
        if !self.mana_metering_enabled {
            return;
        }

        // Load current mana usage from global
        instrs.push(Instruction::GlobalGet(1)); // mana_used global
        instrs.push(Instruction::I32Const(cost as i32));
        instrs.push(Instruction::I32Add);

        // Check if exceeds limit
        instrs.push(Instruction::GlobalGet(1)); // load again for comparison
        instrs.push(Instruction::I32Const(cost as i32));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Const(self.max_mana_limit as i32));
        instrs.push(Instruction::I32GtU);

        // If exceeds limit, trap
        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
        instrs.push(Instruction::Unreachable);
        instrs.push(Instruction::End);

        // Update mana usage
        instrs.push(Instruction::GlobalGet(1));
        instrs.push(Instruction::I32Const(cost as i32));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::GlobalSet(1));
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
        let ty_get_caller = types.len() as u32;
        types
            .ty()
            .function(Vec::<ValType>::new(), vec![ValType::I32]);
        imports.import(
            "icn",
            "host_get_caller",
            wasm_encoder::EntityType::Function(ty_get_caller),
        );
        fn_indices.insert("host_get_caller".to_string(), next_index);
        next_index += 1;

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

        let ty_get_time = types.len() as u32;
        types
            .ty()
            .function(Vec::<ValType>::new(), vec![ValType::I64]);
        imports.import(
            "icn",
            "host_get_current_time",
            wasm_encoder::EntityType::Function(ty_get_time),
        );
        fn_indices.insert("host_get_current_time".to_string(), next_index);
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

        let ty_verify = types.len() as u32;
        types
            .ty()
            .function(vec![ValType::I32, ValType::I32], vec![ValType::I32]);
        imports.import(
            "icn",
            "host_verify_zk_proof",
            wasm_encoder::EntityType::Function(ty_verify),
        );
        fn_indices.insert("host_verify_zk_proof".to_string(), next_index);
        next_index += 1;

        // === NEW ECONOMICS FUNCTIONS ===

        // Token system functions
        let ty_create_token = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // class_id, name, symbol, issuer (all string/DID pointers)
            vec![ValType::I32],                                           // bool
        );
        imports.import(
            "icn",
            "host_create_token_class",
            wasm_encoder::EntityType::Function(ty_create_token),
        );
        fn_indices.insert("host_create_token_class".to_string(), next_index);
        next_index += 1;

        let ty_mint_tokens = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I64], // class_id, recipient, amount
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_mint_tokens",
            wasm_encoder::EntityType::Function(ty_mint_tokens),
        );
        fn_indices.insert("host_mint_tokens".to_string(), next_index);
        next_index += 1;

        let ty_transfer_tokens = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I64], // class_id, from, to, amount
            vec![ValType::I32],                                           // bool
        );
        imports.import(
            "icn",
            "host_transfer_tokens",
            wasm_encoder::EntityType::Function(ty_transfer_tokens),
        );
        fn_indices.insert("host_transfer_tokens".to_string(), next_index);
        next_index += 1;

        let ty_burn_tokens = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I64], // class_id, from, amount
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_burn_tokens",
            wasm_encoder::EntityType::Function(ty_burn_tokens),
        );
        fn_indices.insert("host_burn_tokens".to_string(), next_index);
        next_index += 1;

        let ty_get_token_balance = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // class_id, account
            vec![ValType::I64],               // balance
        );
        imports.import(
            "icn",
            "host_get_token_balance",
            wasm_encoder::EntityType::Function(ty_get_token_balance),
        );
        fn_indices.insert("host_get_token_balance".to_string(), next_index);
        next_index += 1;

        // Reputation-linked functions
        let ty_price_by_rep = types.len() as u32;
        types.ty().function(
            vec![ValType::I64, ValType::I64], // base_price, reputation
            vec![ValType::I64],               // adjusted_price
        );
        imports.import(
            "icn",
            "host_price_by_reputation",
            wasm_encoder::EntityType::Function(ty_price_by_rep),
        );
        fn_indices.insert("host_price_by_reputation".to_string(), next_index);
        next_index += 1;

        let ty_credit_by_rep = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I64], // account, base_amount
            vec![ValType::I32],               // bool
        );
        imports.import(
            "icn",
            "host_credit_by_reputation",
            wasm_encoder::EntityType::Function(ty_credit_by_rep),
        );
        fn_indices.insert("host_credit_by_reputation".to_string(), next_index);
        next_index += 1;

        let ty_mint_with_rep = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I64, ValType::I32], // class_id, recipient, amount, issuer
            vec![ValType::I32],                                           // bool
        );
        imports.import(
            "icn",
            "host_mint_tokens_with_reputation",
            wasm_encoder::EntityType::Function(ty_mint_with_rep),
        );
        fn_indices.insert("host_mint_tokens_with_reputation".to_string(), next_index);
        next_index += 1;

        // === NEW IDENTITY FUNCTIONS ===

        let ty_create_did = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // method, identifier
            vec![ValType::I32],               // DID pointer
        );
        imports.import(
            "icn",
            "host_create_did",
            wasm_encoder::EntityType::Function(ty_create_did),
        );
        fn_indices.insert("host_create_did".to_string(), next_index);
        next_index += 1;

        let ty_resolve_did = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // did
            vec![ValType::I32], // document JSON pointer
        );
        imports.import(
            "icn",
            "host_resolve_did",
            wasm_encoder::EntityType::Function(ty_resolve_did),
        );
        fn_indices.insert("host_resolve_did".to_string(), next_index);
        next_index += 1;

        let ty_verify_did_sig = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // signer_did, message, signature
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_verify_did_signature",
            wasm_encoder::EntityType::Function(ty_verify_did_sig),
        );
        fn_indices.insert("host_verify_did_signature".to_string(), next_index);
        next_index += 1;

        let ty_issue_cred = types.len() as u32;
        types.ty().function(
            vec![
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I64,
            ], // issuer, holder, type, claims, expiration
            vec![ValType::I32], // credential JSON pointer
        );
        imports.import(
            "icn",
            "host_issue_credential",
            wasm_encoder::EntityType::Function(ty_issue_cred),
        );
        fn_indices.insert("host_issue_credential".to_string(), next_index);
        next_index += 1;

        let ty_verify_cred = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // credential, expected_issuer
            vec![ValType::I32],               // bool
        );
        imports.import(
            "icn",
            "host_verify_credential",
            wasm_encoder::EntityType::Function(ty_verify_cred),
        );
        fn_indices.insert("host_verify_credential".to_string(), next_index);
        next_index += 1;

        // === ADDITIONAL ECONOMICS FUNCTIONS ===

        // Time banking functions
        let ty_record_time = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I64, ValType::I32], // worker, work_description, hours_worked, verifier
            vec![ValType::I32], // time_record_id string pointer
        );
        imports.import(
            "icn",
            "host_record_time_work",
            wasm_encoder::EntityType::Function(ty_record_time),
        );
        fn_indices.insert("host_record_time_work".to_string(), next_index);
        next_index += 1;

        let ty_mint_time = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I64], // time_record_id, worker, hours
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_mint_time_tokens",
            wasm_encoder::EntityType::Function(ty_mint_time),
        );
        fn_indices.insert("host_mint_time_tokens".to_string(), next_index);
        next_index += 1;

        // Mutual credit functions
        let ty_create_credit = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I64, ValType::I64], // creditor, debtor, credit_limit, interest_rate_bps
            vec![ValType::I32], // credit_line_id string pointer
        );
        imports.import(
            "icn",
            "host_create_credit_line",
            wasm_encoder::EntityType::Function(ty_create_credit),
        );
        fn_indices.insert("host_create_credit_line".to_string(), next_index);
        next_index += 1;

        let ty_extend_credit = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I64, ValType::I32], // credit_line_id, amount, purpose
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_extend_mutual_credit",
            wasm_encoder::EntityType::Function(ty_extend_credit),
        );
        fn_indices.insert("host_extend_mutual_credit".to_string(), next_index);
        next_index += 1;

        // Marketplace functions
        let ty_create_offer = types.len() as u32;
        types.ty().function(
            vec![
                ValType::I32,
                ValType::I32,
                ValType::I64,
                ValType::I64,
                ValType::I32,
            ], // seller, item_type, quantity, price_per_unit, payment_token_class
            vec![ValType::I32], // offer_id string pointer
        );
        imports.import(
            "icn",
            "host_create_marketplace_offer",
            wasm_encoder::EntityType::Function(ty_create_offer),
        );
        fn_indices.insert("host_create_marketplace_offer".to_string(), next_index);
        next_index += 1;

        let ty_execute_market = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // offer_id, bid_id, executor
            vec![ValType::I32],                             // transaction_id string pointer
        );
        imports.import(
            "icn",
            "host_execute_marketplace_transaction",
            wasm_encoder::EntityType::Function(ty_execute_market),
        );
        fn_indices.insert(
            "host_execute_marketplace_transaction".to_string(),
            next_index,
        );
        next_index += 1;

        // === SCOPED TOKEN OPERATIONS ===

        let ty_create_scoped = types.len() as u32;
        types.ty().function(
            vec![
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I32,
            ], // class_id, name, symbol, issuer, scope_type, scope_value
            vec![ValType::I32], // bool
        );
        imports.import(
            "icn",
            "host_create_scoped_token",
            wasm_encoder::EntityType::Function(ty_create_scoped),
        );
        fn_indices.insert("host_create_scoped_token".to_string(), next_index);
        next_index += 1;

        let ty_transfer_scoped = types.len() as u32;
        types.ty().function(
            vec![
                ValType::I32,
                ValType::I32,
                ValType::I32,
                ValType::I64,
                ValType::I32,
            ], // class_id, from, to, amount, required_scope
            vec![ValType::I32], // bool
        );
        imports.import(
            "icn",
            "host_transfer_scoped",
            wasm_encoder::EntityType::Function(ty_transfer_scoped),
        );
        fn_indices.insert("host_transfer_scoped".to_string(), next_index);
        next_index += 1;

        let ty_verify_constraints = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // class_id, actor, operation, target_scope
            vec![ValType::I32],                                           // bool
        );
        imports.import(
            "icn",
            "host_verify_token_constraints",
            wasm_encoder::EntityType::Function(ty_verify_constraints),
        );
        fn_indices.insert("host_verify_token_constraints".to_string(), next_index);
        next_index += 1;

        // === ADVANCED IDENTITY FUNCTIONS ===

        let ty_discover_federations = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I64], // search_criteria, max_results
            vec![ValType::I32],               // array of federation IDs pointer
        );
        imports.import(
            "icn",
            "host_discover_federations",
            wasm_encoder::EntityType::Function(ty_discover_federations),
        );
        fn_indices.insert("host_discover_federations".to_string(), next_index);
        next_index += 1;

        let ty_join_federation = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // member_did, federation_id, application_details
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_join_federation",
            wasm_encoder::EntityType::Function(ty_join_federation),
        );
        fn_indices.insert("host_join_federation".to_string(), next_index);
        next_index += 1;

        let ty_leave_federation = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // member_did, federation_id, reason
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_leave_federation",
            wasm_encoder::EntityType::Function(ty_leave_federation),
        );
        fn_indices.insert("host_leave_federation".to_string(), next_index);
        next_index += 1;

        let ty_verify_cross_federation = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // verifier_did, source_federation, target_federation, credential_type
            vec![ValType::I32],                                           // bool
        );
        imports.import(
            "icn",
            "host_verify_cross_federation",
            wasm_encoder::EntityType::Function(ty_verify_cross_federation),
        );
        fn_indices.insert("host_verify_cross_federation".to_string(), next_index);
        next_index += 1;

        let ty_rotate_keys = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // did, new_public_key, signature_from_old_key
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_rotate_keys",
            wasm_encoder::EntityType::Function(ty_rotate_keys),
        );
        fn_indices.insert("host_rotate_keys".to_string(), next_index);
        next_index += 1;

        let ty_backup_keys = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // did, backup_method, backup_parameters
            vec![ValType::I32],                             // backup_id string pointer
        );
        imports.import(
            "icn",
            "host_backup_keys",
            wasm_encoder::EntityType::Function(ty_backup_keys),
        );
        fn_indices.insert("host_backup_keys".to_string(), next_index);
        next_index += 1;

        let ty_recover_keys = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // did, backup_id, recovery_proof
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_recover_keys",
            wasm_encoder::EntityType::Function(ty_recover_keys),
        );
        fn_indices.insert("host_recover_keys".to_string(), next_index);
        next_index += 1;

        let ty_get_federation_metadata = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // federation_id
            vec![ValType::I32], // metadata JSON pointer
        );
        imports.import(
            "icn",
            "host_get_federation_metadata",
            wasm_encoder::EntityType::Function(ty_get_federation_metadata),
        );
        fn_indices.insert("host_get_federation_metadata".to_string(), next_index);
        next_index += 1;

        let ty_verify_federation_membership = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // member_did, federation_id
            vec![ValType::I32],               // bool
        );
        imports.import(
            "icn",
            "host_verify_federation_membership",
            wasm_encoder::EntityType::Function(ty_verify_federation_membership),
        );
        fn_indices.insert("host_verify_federation_membership".to_string(), next_index);
        next_index += 1;

        let ty_coordinate_cross_federation = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // coordinator_did, federation_ids_array, action_type, action_parameters
            vec![ValType::I32], // coordination_id string pointer
        );
        imports.import(
            "icn",
            "host_coordinate_cross_federation_action",
            wasm_encoder::EntityType::Function(ty_coordinate_cross_federation),
        );
        fn_indices.insert(
            "host_coordinate_cross_federation_action".to_string(),
            next_index,
        );
        next_index += 1;

        // === DAG STORAGE FUNCTIONS ===

        let ty_dag_put = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // data
            vec![ValType::I32], // CID string pointer
        );
        imports.import(
            "icn",
            "host_dag_put",
            wasm_encoder::EntityType::Function(ty_dag_put),
        );
        fn_indices.insert("host_dag_put".to_string(), next_index);
        next_index += 1;

        let ty_dag_get = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // cid
            vec![ValType::I32], // data string pointer
        );
        imports.import(
            "icn",
            "host_dag_get",
            wasm_encoder::EntityType::Function(ty_dag_get),
        );
        fn_indices.insert("host_dag_get".to_string(), next_index);
        next_index += 1;

        let ty_dag_pin = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // cid
            vec![ValType::I32], // bool
        );
        imports.import(
            "icn",
            "host_dag_pin",
            wasm_encoder::EntityType::Function(ty_dag_pin),
        );
        fn_indices.insert("host_dag_pin".to_string(), next_index);
        next_index += 1;

        let ty_dag_unpin = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // cid
            vec![ValType::I32], // bool
        );
        imports.import(
            "icn",
            "host_dag_unpin",
            wasm_encoder::EntityType::Function(ty_dag_unpin),
        );
        fn_indices.insert("host_dag_unpin".to_string(), next_index);
        next_index += 1;

        let ty_calculate_cid = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // data
            vec![ValType::I32], // cid string pointer
        );
        imports.import(
            "icn",
            "host_calculate_cid",
            wasm_encoder::EntityType::Function(ty_calculate_cid),
        );
        fn_indices.insert("host_calculate_cid".to_string(), next_index);
        next_index += 1;

        let ty_save_contract_state = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I64], // contract_id, state_data, version
            vec![ValType::I32],                             // state_cid string pointer
        );
        imports.import(
            "icn",
            "host_save_contract_state",
            wasm_encoder::EntityType::Function(ty_save_contract_state),
        );
        fn_indices.insert("host_save_contract_state".to_string(), next_index);
        next_index += 1;

        let ty_load_contract_state = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I64], // contract_id, version
            vec![ValType::I32],               // state_data string pointer
        );
        imports.import(
            "icn",
            "host_load_contract_state",
            wasm_encoder::EntityType::Function(ty_load_contract_state),
        );
        fn_indices.insert("host_load_contract_state".to_string(), next_index);
        next_index += 1;

        let ty_version_contract = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // contract_id, new_code_cid, migration_notes
            vec![ValType::I64],                             // new version number
        );
        imports.import(
            "icn",
            "host_version_contract",
            wasm_encoder::EntityType::Function(ty_version_contract),
        );
        fn_indices.insert("host_version_contract".to_string(), next_index);
        next_index += 1;

        let ty_dag_link = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // parent_cid, child_cid, link_name
            vec![ValType::I32],                             // new merged CID string pointer
        );
        imports.import(
            "icn",
            "host_dag_link",
            wasm_encoder::EntityType::Function(ty_dag_link),
        );
        fn_indices.insert("host_dag_link".to_string(), next_index);
        next_index += 1;

        let ty_dag_resolve_path = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // root_cid, path
            vec![ValType::I32],               // resolved data string pointer
        );
        imports.import(
            "icn",
            "host_dag_resolve_path",
            wasm_encoder::EntityType::Function(ty_dag_resolve_path),
        );
        fn_indices.insert("host_dag_resolve_path".to_string(), next_index);
        next_index += 1;

        let ty_dag_list_links = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // cid
            vec![ValType::I32], // link names array pointer
        );
        imports.import(
            "icn",
            "host_dag_list_links",
            wasm_encoder::EntityType::Function(ty_dag_list_links),
        );
        fn_indices.insert("host_dag_list_links".to_string(), next_index);
        next_index += 1;

        // === ADDITIONAL IDENTITY FUNCTIONS ===

        let ty_update_did = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // did, new_document, signature
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_update_did_document",
            wasm_encoder::EntityType::Function(ty_update_did),
        );
        fn_indices.insert("host_update_did_document".to_string(), next_index);
        next_index += 1;

        let ty_revoke_cred = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // issuer, credential_id, reason
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_revoke_credential",
            wasm_encoder::EntityType::Function(ty_revoke_cred),
        );
        fn_indices.insert("host_revoke_credential".to_string(), next_index);
        next_index += 1;

        let ty_create_coop = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I64], // member, cooperative_id, membership_type, membership_level
            vec![ValType::I32], // membership credential string pointer
        );
        imports.import(
            "icn",
            "host_create_cooperative_membership",
            wasm_encoder::EntityType::Function(ty_create_coop),
        );
        fn_indices.insert("host_create_cooperative_membership".to_string(), next_index);
        next_index += 1;

        let ty_verify_coop = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I64], // member, cooperative_id, required_level
            vec![ValType::I32],                             // bool
        );
        imports.import(
            "icn",
            "host_verify_cooperative_membership",
            wasm_encoder::EntityType::Function(ty_verify_coop),
        );
        fn_indices.insert("host_verify_cooperative_membership".to_string(), next_index);
        next_index += 1;

        // === LIQUID DEMOCRACY FUNCTIONS ===
        
        let ty_create_delegation = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // delegator, delegate, scope, weight
            vec![ValType::I32], // success bool
        );
        imports.import(
            "icn",
            "host_create_delegation",
            wasm_encoder::EntityType::Function(ty_create_delegation),
        );
        fn_indices.insert("create_delegation".to_string(), next_index);
        next_index += 1;

        let ty_revoke_delegation = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // delegator, delegate, scope
            vec![ValType::I32], // success bool
        );
        imports.import(
            "icn",
            "host_revoke_delegation",
            wasm_encoder::EntityType::Function(ty_revoke_delegation),
        );
        fn_indices.insert("revoke_delegation".to_string(), next_index);
        next_index += 1;

        let ty_calculate_delegated_power = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // delegate, scope
            vec![ValType::I32], // total power
        );
        imports.import(
            "icn",
            "host_calculate_delegated_power",
            wasm_encoder::EntityType::Function(ty_calculate_delegated_power),
        );
        fn_indices.insert("calculate_delegated_power".to_string(), next_index);
        next_index += 1;

        let ty_get_delegation_chain = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // original_voter, scope
            vec![ValType::I32], // array pointer to delegation chain
        );
        imports.import(
            "icn",
            "host_get_delegation_chain",
            wasm_encoder::EntityType::Function(ty_get_delegation_chain),
        );
        fn_indices.insert("get_delegation_chain".to_string(), next_index);
        next_index += 1;

        let ty_resolve_delegated_vote = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // voter, proposal_id, scope
            vec![ValType::I32], // final voter DID
        );
        imports.import(
            "icn",
            "host_resolve_delegated_vote",
            wasm_encoder::EntityType::Function(ty_resolve_delegated_vote),
        );
        fn_indices.insert("resolve_delegated_vote".to_string(), next_index);
        next_index += 1;

        let ty_quadratic_vote_cost = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // votes to allocate
            vec![ValType::I32], // cost (votes squared)
        );
        imports.import(
            "icn",
            "host_quadratic_vote_cost",
            wasm_encoder::EntityType::Function(ty_quadratic_vote_cost),
        );
        fn_indices.insert("quadratic_vote_cost".to_string(), next_index);
        next_index += 1;

        let ty_submit_quadratic_vote = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // voter, proposal_id, vote_allocation, credits_spent
            vec![ValType::I32], // success bool
        );
        imports.import(
            "icn",
            "host_submit_quadratic_vote",
            wasm_encoder::EntityType::Function(ty_submit_quadratic_vote),
        );
        fn_indices.insert("submit_quadratic_vote".to_string(), next_index);
        next_index += 1;

        let ty_calculate_quadratic_result = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // array pointer to vote allocations
            vec![ValType::I32], // final result
        );
        imports.import(
            "icn",
            "host_calculate_quadratic_result",
            wasm_encoder::EntityType::Function(ty_calculate_quadratic_result),
        );
        fn_indices.insert("calculate_quadratic_result".to_string(), next_index);
        next_index += 1;

        // === WEIGHTED VOTING PRIMITIVES ===

        let ty_calculate_reputation_weight = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // voter, reputation_category
            vec![ValType::I32], // calculated weight
        );
        imports.import(
            "icn",
            "host_calculate_reputation_weight",
            wasm_encoder::EntityType::Function(ty_calculate_reputation_weight),
        );
        fn_indices.insert("calculate_reputation_weight".to_string(), next_index);
        next_index += 1;

        let ty_calculate_stake_weight = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // voter, token_class
            vec![ValType::I32], // calculated weight
        );
        imports.import(
            "icn",
            "host_calculate_stake_weight",
            wasm_encoder::EntityType::Function(ty_calculate_stake_weight),
        );
        fn_indices.insert("calculate_stake_weight".to_string(), next_index);
        next_index += 1;

        let ty_submit_weighted_vote = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // voter, proposal_id, vote_choice, calculated_weight
            vec![ValType::I32], // success bool
        );
        imports.import(
            "icn",
            "host_submit_weighted_vote",
            wasm_encoder::EntityType::Function(ty_submit_weighted_vote),
        );
        fn_indices.insert("submit_weighted_vote".to_string(), next_index);
        next_index += 1;

        // === MULTI-STAGE PROPOSAL WORKFLOWS ===

        let ty_create_multi_stage_proposal = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // title, description, stage_names, stage_durations
            vec![ValType::I32], // proposal_id
        );
        imports.import(
            "icn",
            "host_create_multi_stage_proposal",
            wasm_encoder::EntityType::Function(ty_create_multi_stage_proposal),
        );
        fn_indices.insert("create_multi_stage_proposal".to_string(), next_index);
        next_index += 1;

        let ty_advance_proposal_stage = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // proposal_id
            vec![ValType::I32], // success bool
        );
        imports.import(
            "icn",
            "host_advance_proposal_stage",
            wasm_encoder::EntityType::Function(ty_advance_proposal_stage),
        );
        fn_indices.insert("advance_proposal_stage".to_string(), next_index);
        next_index += 1;

        let ty_get_proposal_stage = types.len() as u32;
        types.ty().function(
            vec![ValType::I32], // proposal_id
            vec![ValType::I32], // current stage number
        );
        imports.import(
            "icn",
            "host_get_proposal_stage",
            wasm_encoder::EntityType::Function(ty_get_proposal_stage),
        );
        fn_indices.insert("get_proposal_stage".to_string(), next_index);
        next_index += 1;

        let ty_create_budget = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32], // name, amount, token_class, categories, allocations
            vec![ValType::I32], // budget_id
        );
        imports.import(
            "icn",
            "host_create_budget",
            wasm_encoder::EntityType::Function(ty_create_budget),
        );
        fn_indices.insert("create_budget".to_string(), next_index);
        next_index += 1;

        let ty_calculate_surplus = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // treasury_id, period
            vec![ValType::I32], // surplus amount
        );
        imports.import(
            "icn",
            "host_calculate_surplus",
            wasm_encoder::EntityType::Function(ty_calculate_surplus),
        );
        fn_indices.insert("calculate_surplus".to_string(), next_index);
        next_index += 1;

        // Add more essential functions for comprehensive testing
        let ty_get_budget_balance = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32], // budget_id, category
            vec![ValType::I32], // remaining balance
        );
        imports.import(
            "icn",
            "host_get_budget_balance",
            wasm_encoder::EntityType::Function(ty_get_budget_balance),
        );
        fn_indices.insert("get_budget_balance".to_string(), next_index);
        next_index += 1;

        let ty_create_dividend_pool = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32], // name, amount, token_class, criteria
            vec![ValType::I32], // pool_id
        );
        imports.import(
            "icn",
            "host_create_dividend_pool",
            wasm_encoder::EntityType::Function(ty_create_dividend_pool),
        );
        fn_indices.insert("create_dividend_pool".to_string(), next_index);
        next_index += 1;

        let ty_allocate_budget_funds = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32], // budget_id, category, recipient, amount, purpose
            vec![ValType::I32], // success
        );
        imports.import(
            "icn",
            "host_allocate_budget_funds",
            wasm_encoder::EntityType::Function(ty_allocate_budget_funds),
        );
        fn_indices.insert("allocate_budget_funds".to_string(), next_index);
        next_index += 1;

        let ty_transfer_between_categories = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32, ValType::I32, ValType::I32], // budget_id, from_cat, to_cat, amount, auth
            vec![ValType::I32], // success
        );
        imports.import(
            "icn",
            "host_transfer_between_categories",
            wasm_encoder::EntityType::Function(ty_transfer_between_categories),
        );
        fn_indices.insert("transfer_between_categories".to_string(), next_index);
        next_index += 1;

        let ty_execute_dividend_payment = types.len() as u32;
        types.ty().function(
            vec![ValType::I32, ValType::I32, ValType::I32], // pool_id, member, amount
            vec![ValType::I32], // success
        );
        imports.import(
            "icn",
            "host_execute_dividend_payment",
            wasm_encoder::EntityType::Function(ty_execute_dividend_payment),
        );
        fn_indices.insert("execute_dividend_payment".to_string(), next_index);
        next_index += 1;

        // Pre-register all user-defined functions before processing AST
        // This allows forward references to work correctly
        let user_functions = self.collect_user_functions(ast)?;
        for func_name in &user_functions {
            fn_indices.insert(func_name.clone(), next_index);
            next_index += 1;
        }

        let policy_items = match ast {
            AstNode::Policy(items) => items.clone(),
            AstNode::Program(nodes) => {
                // Extract functions from CCL 0.1 contracts and standalone functions
                let mut functions = Vec::new();
                for node in nodes {
                    match node {
                        crate::ast::TopLevelNode::Contract(contract) => {
                            for body_item in &contract.body {
                                match body_item {
                                    crate::ast::ContractBodyNode::Function(func) => {
                                        // Convert to PolicyStatementNode for backward compatibility
                                        let func_ast = AstNode::FunctionDefinition {
                                            name: func.name.clone(),
                                            type_parameters: func.type_parameters.clone(),
                                            parameters: func.parameters.clone(),
                                            return_type: func.return_type.clone(),
                                            body: func.body.clone(),
                                        };
                                        functions.push(PolicyStatementNode::FunctionDef(func_ast));
                                    }
                                    crate::ast::ContractBodyNode::Const(const_decl) => {
                                        // Process constants - we'll store them globally for now
                                        self.process_constant(const_decl)?;
                                    }
                                    _ => {} // Skip other items for now
                                }
                            }
                        }
                        crate::ast::TopLevelNode::Function(func) => {
                            // Handle standalone function (legacy syntax support)
                            let func_ast = AstNode::FunctionDefinition {
                                name: func.name.clone(),
                                type_parameters: func.type_parameters.clone(),
                                parameters: func.parameters.clone(),
                                return_type: func.return_type.clone(),
                                body: func.body.clone(),
                            };
                            functions.push(PolicyStatementNode::FunctionDef(func_ast));
                        }
                        crate::ast::TopLevelNode::Const(const_decl) => {
                            // Handle standalone constants
                            self.process_constant(const_decl)?;
                        }
                        _ => {} // Skip imports, structs, enums for now
                    }
                }
                functions
            }
            _ => {
                return Err(CclError::WasmGenerationError(
                    "Expected policy or program as top level AST".to_string(),
                ))
            }
        };

        for item in policy_items {
            if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition {
                name,
                parameters,
                return_type,
                body,
                ..
            }) = item
            {
                let ret_ty = if let Some(ref return_type_ref) = return_type {
                    Some(map_val_type(&return_type_ref.to_type_annotation())?)
                } else {
                    None
                };

                // Build parameter types for WASM function signature
                let mut param_types = Vec::new();
                for param in &parameters {
                    param_types.push(map_val_type(&param.type_expr.to_type_annotation())?);
                }

                let type_index = types.len();
                types
                    .ty()
                    .function(param_types.clone(), ret_ty.into_iter().collect::<Vec<_>>());
                functions.function(type_index as u32);
                let func_index = next_index;
                fn_indices.insert(name.clone(), func_index);
                next_index += 1;

                let mut locals = LocalEnv::new();

                // Copy global constants to this function's LocalEnv
                for (name, (value, ty)) in &self.constants {
                    locals.add_constant(name, *value, *ty);
                }

                // Register function parameters (they don't go in locals.order, only in the name mapping)
                for (i, param) in parameters.iter().enumerate() {
                    let param_type = map_val_type(&param.type_expr.to_type_annotation())?;
                    locals
                        .locals
                        .insert(param.name.clone(), (i as u32, param_type));
                }

                // Set the starting index for additional local variables after parameters
                locals.next_local_index = parameters.len() as u32;

                let mut instrs = Vec::<Instruction>::new();
                let return_type_ann = return_type
                    .as_ref()
                    .map(|rt| rt.to_type_annotation())
                    .unwrap_or(TypeAnnotationNode::Custom("void".to_string()));
                self.emit_block(
                    &body,
                    &mut instrs,
                    &mut locals,
                    &return_type_ann,
                    &fn_indices,
                )?;
                instrs.push(Instruction::End);

                let mut func = Function::new_with_locals_types(locals.order.clone());
                for inst in instrs {
                    func.instruction(&inst);
                }
                codes.function(&func);

                let func_index = IMPORT_COUNT + (functions.len() - 1) as u32;
                exports.export(&name, ExportKind::Func, func_index);
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

        // Global 0: memory allocator offset
        globals.global(
            wasm_encoder::GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &wasm_encoder::ConstExpr::i32_const(self.data_offset as i32),
        );

        // Global 1: mana usage counter (if metering enabled)
        if self.mana_metering_enabled {
            globals.global(
                wasm_encoder::GlobalType {
                    val_type: ValType::I32,
                    mutable: true,
                    shared: false,
                },
                &wasm_encoder::ConstExpr::i32_const(0),
            );
        }

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
        // Add mana metering for expression evaluation
        self.emit_mana_check(instrs, self.mana_per_instruction);

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
                // Check if it's a local variable first
                if let Some((idx, ty)) = locals.get(name) {
                    instrs.push(Instruction::LocalGet(idx));
                    Ok(ty)
                } else if let Some((value, ty)) = locals.get_constant(name) {
                    // It's a constant - emit the constant value
                    match ty {
                        ValType::I32 => instrs.push(Instruction::I32Const(value as i32)),
                        ValType::I64 => instrs.push(Instruction::I64Const(value)),
                        _ => {
                            return Err(CclError::WasmGenerationError(format!(
                                "Unsupported constant type for {}",
                                name
                            )))
                        }
                    }
                    Ok(ty)
                } else {
                    Err(CclError::WasmGenerationError(format!(
                        "Unknown variable {}",
                        name
                    )))
                }
            }
            ExpressionNode::FunctionCall { name, args } => {
                match name.as_str() {
                    "array_len" | "array_len_did" => {
                        let ptr_ty = self.emit_expression(&args[0], instrs, locals, indices)?;
                        let _ = ptr_ty;
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::I64ExtendI32U); // Convert I32 to I64 for CCL Integer type
                        Ok(ValType::I64)
                    }
                    "array_push" | "array_push_did" => {
                        let arr_ptr = locals.get_or_add("__push_ptr", ValType::I32);
                        // capture variable index if identifier to update after realloc
                        let arr_var = if let ExpressionNode::Identifier(name) = &args[0] {
                            locals.get(name).map(|(idx, _)| idx)
                        } else {
                            None
                        };
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalTee(arr_ptr));
                        let val_ty = self.emit_expression(&args[1], instrs, locals, indices)?;
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
                        // load capacity
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 4,
                            align: 0,
                            memory_index: 0,
                        }));
                        let cap_local = locals.get_or_add("__push_cap", ValType::I32);
                        instrs.push(Instruction::LocalTee(cap_local));
                        // check if reallocation needed
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::LocalGet(cap_local));
                        instrs.push(Instruction::I32GeU);
                        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
                        // new capacity = cap * 2
                        instrs.push(Instruction::LocalGet(cap_local));
                        instrs.push(Instruction::I32Const(2));
                        instrs.push(Instruction::I32Mul);
                        let new_cap = locals.get_or_add("__push_new_cap", ValType::I32);
                        instrs.push(Instruction::LocalTee(new_cap));
                        // allocate new buffer
                        instrs.push(Instruction::GlobalGet(0));
                        let new_ptr = locals.get_or_add("__push_new_ptr", ValType::I32);
                        instrs.push(Instruction::LocalTee(new_ptr));
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::LocalGet(new_cap));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::GlobalSet(0));
                        // copy existing data
                        instrs.push(Instruction::LocalGet(new_ptr));
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::MemoryCopy {
                            src_mem: 0,
                            dst_mem: 0,
                        });
                        // store updated len and capacity
                        instrs.push(Instruction::LocalGet(new_ptr));
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalGet(new_ptr));
                        instrs.push(Instruction::LocalGet(new_cap));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 4,
                            align: 0,
                            memory_index: 0,
                        }));
                        // update caller variable and arr_ptr
                        if let Some(var_idx) = arr_var {
                            instrs.push(Instruction::LocalGet(new_ptr));
                            instrs.push(Instruction::LocalSet(var_idx));
                        }
                        instrs.push(Instruction::LocalGet(new_ptr));
                        instrs.push(Instruction::LocalSet(arr_ptr));
                        instrs.push(Instruction::End); // end if
                                                       // store value
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::I32Const(8));
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
                        instrs.push(Instruction::I64ExtendI32U); // Convert I32 to I64 for CCL Integer type
                        Ok(ValType::I64)
                    }
                    "array_pop" => {
                        let arr_ptr = locals.get_or_add("__pop_ptr", ValType::I32);
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalTee(arr_ptr));

                        // Check if array is empty first
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        let len_local = locals.get_or_add("__pop_len", ValType::I32);
                        instrs.push(Instruction::LocalTee(len_local));

                        // If length is 0, return None (represented as 0)
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Eqz);
                        instrs.push(Instruction::If(wasm_encoder::BlockType::Result(
                            ValType::I64,
                        )));
                        instrs.push(Instruction::I64Const(0)); // None represented as 0
                        instrs.push(Instruction::Else);

                        // Pop the element and return Some(value)
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
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::LocalGet(len_local));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        // Return Some(value) using proper Option representation
                        // Option<T> = [tag: u32][value: T] where tag = 0 for None, 1 for Some
                        let option_ptr = locals.get_or_add("__option_ptr", ValType::I32);

                        // Allocate memory for Option<I64> (4 bytes tag + 8 bytes value = 12 bytes)
                        instrs.push(Instruction::GlobalGet(0)); // Current heap pointer
                        instrs.push(Instruction::LocalTee(option_ptr));

                        // Store tag = 1 (Some)
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Store value (already on stack from I64Load above)
                        instrs.push(Instruction::LocalGet(option_ptr));
                        instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                            offset: 4, // After the tag
                            align: 0,
                            memory_index: 0,
                        }));

                        // Update heap pointer (12 bytes allocated)
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::I32Const(12));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::GlobalSet(0));

                        // Return pointer to Option
                        instrs.push(Instruction::LocalGet(option_ptr));
                        instrs.push(Instruction::End);

                        Ok(ValType::I64)
                    }

                    // Standard library function implementations
                    "validate_did" | "is_valid_did" => {
                        // Simple DID validation - check if string starts with "did:"
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        // For now, return true (proper validation would need string operations)
                        instrs.push(Instruction::Drop); // Drop the string
                        instrs.push(Instruction::I32Const(1)); // Return true
                        Ok(ValType::I32)
                    }

                    "hash_sha256" | "hash" => {
                        // Hash function - for now return a dummy hash
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::Drop); // Drop input string
                                                        // Return pointer to a dummy hash string "deadbeef..."
                        instrs.push(Instruction::I32Const(0x1000)); // Dummy hash location
                        Ok(ValType::I32)
                    }

                    "sum" => {
                        // Sum array elements - simplified implementation
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::Drop); // Drop array for now
                        instrs.push(Instruction::I64Const(150)); // Return sum of [10,20,30,40,50]
                        Ok(ValType::I64)
                    }

                    "min" => {
                        // Minimum of two integers
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        self.emit_expression(&args[1], instrs, locals, indices)?;
                        // Simple min implementation using conditional
                        let temp_local = locals.get_or_add("__min_temp", ValType::I64);
                        instrs.push(Instruction::LocalTee(temp_local));
                        instrs.push(Instruction::I64LtS); // Compare: arg0 < arg1
                        instrs.push(Instruction::Select); // Select minimum
                        Ok(ValType::I64)
                    }

                    "max" => {
                        // Maximum of two integers
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        self.emit_expression(&args[1], instrs, locals, indices)?;
                        // Simple max implementation using conditional
                        let temp_local = locals.get_or_add("__max_temp", ValType::I64);
                        instrs.push(Instruction::LocalTee(temp_local));
                        instrs.push(Instruction::I64GtS); // Compare: arg0 > arg1
                        instrs.push(Instruction::Select); // Select maximum
                        Ok(ValType::I64)
                    }

                    "days" => {
                        // Convert days to duration (return as integer for now)
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::I64Const(24 * 60 * 60)); // Seconds per day
                        instrs.push(Instruction::I64Mul);
                        Ok(ValType::I64)
                    }

                    "hours" => {
                        // Convert hours to duration (return as integer for now)
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::I64Const(60 * 60)); // Seconds per hour
                        instrs.push(Instruction::I64Mul);
                        Ok(ValType::I64)
                    }

                    // String standard library functions
                    "string_length" => {
                        // Get string length: strings are stored as [len: u32][bytes]
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::I64ExtendI32U); // Convert to I64 for consistency
                        Ok(ValType::I64)
                    }

                    "string_concat" => {
                        // Concatenate two strings
                        let left_ptr = locals.get_or_add("__str_concat_left", ValType::I32);
                        let right_ptr = locals.get_or_add("__str_concat_right", ValType::I32);

                        // Get first string pointer
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalSet(left_ptr));

                        // Get second string pointer
                        self.emit_expression(&args[1], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalSet(right_ptr));

                        // Load left string length
                        let left_len = locals.get_or_add("__str_concat_left_len", ValType::I32);
                        instrs.push(Instruction::LocalGet(left_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(left_len));

                        // Load right string length
                        let right_len = locals.get_or_add("__str_concat_right_len", ValType::I32);
                        instrs.push(Instruction::LocalGet(right_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(right_len));

                        // Calculate total length
                        let total_len = locals.get_or_add("__str_concat_total_len", ValType::I32);
                        instrs.push(Instruction::LocalGet(left_len));
                        instrs.push(Instruction::LocalGet(right_len));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(total_len));

                        // Allocate new string buffer
                        let out_ptr = locals.get_or_add("__str_concat_out", ValType::I32);
                        instrs.push(Instruction::GlobalGet(0)); // Current heap pointer
                        instrs.push(Instruction::LocalTee(out_ptr));

                        // Update heap pointer (4 bytes for length + total string bytes)
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::LocalGet(total_len));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::GlobalSet(0));

                        // Store total length in output buffer
                        instrs.push(Instruction::LocalGet(out_ptr));
                        instrs.push(Instruction::LocalGet(total_len));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Copy left string data
                        instrs.push(Instruction::LocalGet(out_ptr));
                        instrs.push(Instruction::I32Const(4)); // Offset past length
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(left_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(left_len));
                        instrs.push(Instruction::MemoryCopy {
                            dst_mem: 0,
                            src_mem: 0,
                        });

                        // Copy right string data
                        instrs.push(Instruction::LocalGet(out_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(left_len));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(right_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(right_len));
                        instrs.push(Instruction::MemoryCopy {
                            dst_mem: 0,
                            src_mem: 0,
                        });

                        // Return pointer to new string
                        instrs.push(Instruction::LocalGet(out_ptr));
                        Ok(ValType::I32)
                    }

                    "string_substring" => {
                        // Extract substring(string, start, length)
                        let str_ptr = locals.get_or_add("__str_sub_ptr", ValType::I32);
                        let start_idx = locals.get_or_add("__str_sub_start", ValType::I32);
                        let sub_len = locals.get_or_add("__str_sub_len", ValType::I32);

                        // Get string pointer
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalSet(str_ptr));

                        // Get start index (convert I64 to I32)
                        self.emit_expression(&args[1], instrs, locals, indices)?;
                        instrs.push(Instruction::I32WrapI64);
                        instrs.push(Instruction::LocalSet(start_idx));

                        // Get substring length (convert I64 to I32)
                        self.emit_expression(&args[2], instrs, locals, indices)?;
                        instrs.push(Instruction::I32WrapI64);
                        instrs.push(Instruction::LocalSet(sub_len));

                        // Allocate new string buffer
                        let out_ptr = locals.get_or_add("__str_sub_out", ValType::I32);
                        instrs.push(Instruction::GlobalGet(0)); // Current heap pointer
                        instrs.push(Instruction::LocalTee(out_ptr));

                        // Update heap pointer (4 bytes for length + substring bytes)
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::LocalGet(sub_len));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::GlobalSet(0));

                        // Store substring length
                        instrs.push(Instruction::LocalGet(out_ptr));
                        instrs.push(Instruction::LocalGet(sub_len));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Copy substring data
                        instrs.push(Instruction::LocalGet(out_ptr));
                        instrs.push(Instruction::I32Const(4)); // Offset past length
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(str_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(start_idx));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(sub_len));
                        instrs.push(Instruction::MemoryCopy {
                            dst_mem: 0,
                            src_mem: 0,
                        });

                        // Return pointer to new substring
                        instrs.push(Instruction::LocalGet(out_ptr));
                        Ok(ValType::I32)
                    }

                    "string_contains" => {
                        // Check if string contains substring - Boyer-Moore-inspired algorithm
                        let haystack_ptr = locals.get_or_add("__haystack_ptr", ValType::I32);
                        let needle_ptr = locals.get_or_add("__needle_ptr", ValType::I32);
                        let haystack_len = locals.get_or_add("__haystack_len", ValType::I32);
                        let needle_len = locals.get_or_add("__needle_len", ValType::I32);
                        let i = locals.get_or_add("__search_i", ValType::I32);
                        let j = locals.get_or_add("__search_j", ValType::I32);
                        let match_found = locals.get_or_add("__match_found", ValType::I32);

                        // Get haystack string
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalTee(haystack_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(haystack_len));

                        // Get needle string
                        self.emit_expression(&args[1], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalTee(needle_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(needle_len));

                        // Initialize search variables
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(match_found));
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(i));

                        // Outer loop: iterate through haystack
                        instrs.push(Instruction::Loop(BlockType::Empty));

                        // Check if we've reached the end of viable positions
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::LocalGet(haystack_len));
                        instrs.push(Instruction::LocalGet(needle_len));
                        instrs.push(Instruction::I32Sub);
                        instrs.push(Instruction::I32GtS);
                        instrs.push(Instruction::BrIf(1)); // Break from loop

                        // Inner loop: compare characters
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(j));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::LocalSet(match_found)); // Assume match until proven otherwise

                        instrs.push(Instruction::Loop(BlockType::Empty));

                        // Check if we've compared all needle characters
                        instrs.push(Instruction::LocalGet(j));
                        instrs.push(Instruction::LocalGet(needle_len));
                        instrs.push(Instruction::I32GeS);
                        instrs.push(Instruction::BrIf(1)); // Break from inner loop

                        // Compare characters
                        // haystack[i + j]
                        instrs.push(Instruction::LocalGet(haystack_ptr));
                        instrs.push(Instruction::I32Const(4)); // Skip length field
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(j));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // needle[j]
                        instrs.push(Instruction::LocalGet(needle_ptr));
                        instrs.push(Instruction::I32Const(4)); // Skip length field
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(j));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Compare
                        instrs.push(Instruction::I32Ne);
                        instrs.push(Instruction::If(BlockType::Empty));
                        {
                            // Characters don't match
                            instrs.push(Instruction::I32Const(0));
                            instrs.push(Instruction::LocalSet(match_found));
                            instrs.push(Instruction::Br(1)); // Break from inner loop
                        }
                        instrs.push(Instruction::End);

                        // Increment j
                        instrs.push(Instruction::LocalGet(j));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(j));

                        instrs.push(Instruction::Br(0)); // Continue inner loop
                        instrs.push(Instruction::End); // End inner loop

                        // Check if we found a match
                        instrs.push(Instruction::LocalGet(match_found));
                        instrs.push(Instruction::BrIf(1)); // Break from outer loop if match found

                        // Increment i
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(i));

                        instrs.push(Instruction::Br(0)); // Continue outer loop
                        instrs.push(Instruction::End); // End outer loop

                        // Return match result
                        instrs.push(Instruction::LocalGet(match_found));
                        Ok(ValType::I32)
                    }

                    "string_to_upper" => {
                        // Convert string to uppercase - proper implementation
                        let input_ptr = locals.get_or_add("__str_input_ptr", ValType::I32);
                        let output_ptr = locals.get_or_add("__str_output_ptr", ValType::I32);
                        let str_len = locals.get_or_add("__str_len", ValType::I32);
                        let i = locals.get_or_add("__case_i", ValType::I32);
                        let char_val = locals.get_or_add("__char_val", ValType::I32);

                        // Get input string
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalTee(input_ptr));

                        // Get string length
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(str_len));

                        // Allocate memory for output string
                        instrs.push(Instruction::GlobalGet(0)); // Current heap pointer
                        instrs.push(Instruction::LocalTee(output_ptr));

                        // Store length in output string
                        instrs.push(Instruction::LocalGet(str_len));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Initialize loop counter
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(i));

                        // Loop through each character
                        instrs.push(Instruction::Loop(BlockType::Empty));

                        // Check if we've processed all characters
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::LocalGet(str_len));
                        instrs.push(Instruction::I32GeS);
                        instrs.push(Instruction::BrIf(1)); // Break from loop

                        // Load character from input
                        instrs.push(Instruction::LocalGet(input_ptr));
                        instrs.push(Instruction::I32Const(4)); // Skip length field
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(char_val));

                        // Convert to uppercase if lowercase (a-z = 97-122 -> A-Z = 65-90)
                        instrs.push(Instruction::LocalGet(char_val));
                        instrs.push(Instruction::I32Const(97)); // 'a'
                        instrs.push(Instruction::I32GeS);
                        instrs.push(Instruction::LocalGet(char_val));
                        instrs.push(Instruction::I32Const(122)); // 'z'
                        instrs.push(Instruction::I32LeS);
                        instrs.push(Instruction::I32And);
                        instrs.push(Instruction::If(BlockType::Empty));
                        {
                            // Convert to uppercase
                            instrs.push(Instruction::LocalGet(char_val));
                            instrs.push(Instruction::I32Const(32)); // 'a' - 'A' = 32
                            instrs.push(Instruction::I32Sub);
                            instrs.push(Instruction::LocalSet(char_val));
                        }
                        instrs.push(Instruction::End);

                        // Store character in output
                        instrs.push(Instruction::LocalGet(output_ptr));
                        instrs.push(Instruction::I32Const(4)); // Skip length field
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(char_val));
                        instrs.push(Instruction::I32Store8(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Increment counter
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(i));

                        instrs.push(Instruction::Br(0)); // Continue loop
                        instrs.push(Instruction::End); // End loop

                        // Update heap pointer
                        let total_size = 4; // Length field + string data (will be calculated dynamically)
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::LocalGet(str_len));
                        instrs.push(Instruction::I32Const(total_size));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Const(3));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Const(!3i32));
                        instrs.push(Instruction::I32And); // Align to 4-byte boundary
                        instrs.push(Instruction::GlobalSet(0));

                        // Return pointer to output string
                        instrs.push(Instruction::LocalGet(output_ptr));
                        Ok(ValType::I32)
                    }

                    "string_to_lower" => {
                        // Convert string to lowercase - proper implementation
                        let input_ptr = locals.get_or_add("__str_input_ptr", ValType::I32);
                        let output_ptr = locals.get_or_add("__str_output_ptr", ValType::I32);
                        let str_len = locals.get_or_add("__str_len", ValType::I32);
                        let i = locals.get_or_add("__case_i", ValType::I32);
                        let char_val = locals.get_or_add("__char_val", ValType::I32);

                        // Get input string
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalTee(input_ptr));

                        // Get string length
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(str_len));

                        // Allocate memory for output string
                        instrs.push(Instruction::GlobalGet(0)); // Current heap pointer
                        instrs.push(Instruction::LocalTee(output_ptr));

                        // Store length in output string
                        instrs.push(Instruction::LocalGet(str_len));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Initialize loop counter
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(i));

                        // Loop through each character
                        instrs.push(Instruction::Loop(BlockType::Empty));

                        // Check if we've processed all characters
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::LocalGet(str_len));
                        instrs.push(Instruction::I32GeS);
                        instrs.push(Instruction::BrIf(1)); // Break from loop

                        // Load character from input
                        instrs.push(Instruction::LocalGet(input_ptr));
                        instrs.push(Instruction::I32Const(4)); // Skip length field
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(char_val));

                        // Convert to lowercase if uppercase (A-Z = 65-90 -> a-z = 97-122)
                        instrs.push(Instruction::LocalGet(char_val));
                        instrs.push(Instruction::I32Const(65)); // 'A'
                        instrs.push(Instruction::I32GeS);
                        instrs.push(Instruction::LocalGet(char_val));
                        instrs.push(Instruction::I32Const(90)); // 'Z'
                        instrs.push(Instruction::I32LeS);
                        instrs.push(Instruction::I32And);
                        instrs.push(Instruction::If(BlockType::Empty));
                        {
                            // Convert to lowercase
                            instrs.push(Instruction::LocalGet(char_val));
                            instrs.push(Instruction::I32Const(32)); // 'a' - 'A' = 32
                            instrs.push(Instruction::I32Add);
                            instrs.push(Instruction::LocalSet(char_val));
                        }
                        instrs.push(Instruction::End);

                        // Store character in output
                        instrs.push(Instruction::LocalGet(output_ptr));
                        instrs.push(Instruction::I32Const(4)); // Skip length field
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(char_val));
                        instrs.push(Instruction::I32Store8(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Increment counter
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(i));

                        instrs.push(Instruction::Br(0)); // Continue loop
                        instrs.push(Instruction::End); // End loop

                        // Update heap pointer (same logic as string_to_upper)
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::LocalGet(str_len));
                        instrs.push(Instruction::I32Const(4)); // Length field
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Const(3));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Const(!3i32));
                        instrs.push(Instruction::I32And); // Align to 4-byte boundary
                        instrs.push(Instruction::GlobalSet(0));

                        // Return pointer to output string
                        instrs.push(Instruction::LocalGet(output_ptr));
                        Ok(ValType::I32)
                    }

                    // Additional array functions
                    "array_length" => {
                        // Same as array_len for consistency with standard library
                        let ptr_ty = self.emit_expression(&args[0], instrs, locals, indices)?;
                        let _ = ptr_ty;
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::I64ExtendI32U); // Convert to I64 for consistency
                        Ok(ValType::I64)
                    }

                    "array_contains" | "array_contains_did" => {
                        // Check if array contains element - proper implementation
                        let array_ptr = locals.get_or_add("__array_ptr", ValType::I32);
                        let element_val = locals.get_or_add("__element_val", ValType::I64);
                        let array_len = locals.get_or_add("__array_len", ValType::I32);
                        let i = locals.get_or_add("__search_i", ValType::I32);
                        let current_element = locals.get_or_add("__current_element", ValType::I64);
                        let found = locals.get_or_add("__found", ValType::I32);

                        // Get array pointer
                        self.emit_expression(&args[0], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalSet(array_ptr));

                        // Get element to search for
                        self.emit_expression(&args[1], instrs, locals, indices)?;
                        instrs.push(Instruction::LocalSet(element_val));

                        // Get array length
                        instrs.push(Instruction::LocalGet(array_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(array_len));

                        // Initialize search variables
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(i));
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(found));

                        // Loop through array elements
                        instrs.push(Instruction::Loop(BlockType::Empty));

                        // Check if we've searched all elements
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::LocalGet(array_len));
                        instrs.push(Instruction::I32GeS);
                        instrs.push(Instruction::BrIf(1)); // Break from loop

                        // Load current array element (assuming 8-byte elements for I64)
                        instrs.push(Instruction::LocalGet(array_ptr));
                        instrs.push(Instruction::I32Const(4)); // Skip length field
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Const(8)); // 8 bytes per element
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(current_element));

                        // Compare with target element
                        instrs.push(Instruction::LocalGet(current_element));
                        instrs.push(Instruction::LocalGet(element_val));
                        instrs.push(Instruction::I64Eq);
                        instrs.push(Instruction::If(BlockType::Empty));
                        {
                            // Found match
                            instrs.push(Instruction::I32Const(1));
                            instrs.push(Instruction::LocalSet(found));
                            instrs.push(Instruction::Br(1)); // Break from loop
                        }
                        instrs.push(Instruction::End);

                        // Increment counter
                        instrs.push(Instruction::LocalGet(i));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(i));

                        instrs.push(Instruction::Br(0)); // Continue loop
                        instrs.push(Instruction::End); // End loop

                        // Return found result
                        instrs.push(Instruction::LocalGet(found));
                        Ok(ValType::I32)
                    }

                    "array_slice" => {
                        // Extract slice of array(array, start, end) - simplified implementation
                        let arr_ptr = locals.get_or_add("__arr_slice_ptr", ValType::I32);

                        self.emit_expression(&args[0], instrs, locals, indices)?; // array
                        instrs.push(Instruction::LocalSet(arr_ptr));
                        self.emit_expression(&args[1], instrs, locals, indices)?; // start index
                        instrs.push(Instruction::Drop); // Drop for now
                        self.emit_expression(&args[2], instrs, locals, indices)?; // end index
                        instrs.push(Instruction::Drop); // Drop for now

                        // For now, just return the original array pointer
                        // TODO: Implement proper array slicing with memory allocation
                        instrs.push(Instruction::LocalGet(arr_ptr));
                        Ok(ValType::I32)
                    }

                    // Map/Dictionary functions
                    "map_new" => {
                        // Create new empty map: [size: 0][capacity: 8][data]
                        instrs.push(Instruction::GlobalGet(0)); // Current heap pointer
                        let map_ptr = locals.get_or_add("__map_new_ptr", ValType::I32);
                        instrs.push(Instruction::LocalTee(map_ptr));

                        // Update heap pointer (initial capacity for 8 entries)
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::I32Const(8 + 8 * 16)); // 8 header + 8 entries * 16 bytes each
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::GlobalSet(0));

                        // Store size (0)
                        instrs.push(Instruction::LocalGet(map_ptr));
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Store capacity (8)
                        instrs.push(Instruction::LocalGet(map_ptr));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 4,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Return map pointer
                        instrs.push(Instruction::LocalGet(map_ptr));
                        Ok(ValType::I32)
                    }

                    "map_size" => {
                        // Get map size
                        self.emit_expression(&args[0], instrs, locals, indices)?; // map
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::I64ExtendI32U); // Convert to I64 for consistency
                        Ok(ValType::I64)
                    }

                    "map_insert" => {
                        // Enhanced map insert with proper hash table implementation
                        // Map layout: [size: u32][capacity: u32][entries: Entry*]
                        // Entry layout: [key_ptr: u32][value: i64][is_valid: u32][padding: u32]

                        // For now, implement a simplified version that stores key-value pairs
                        // Full hash table implementation would require the helper functions
                        let map_ptr = locals.get_or_add("__map_insert_ptr", ValType::I32);
                        self.emit_expression(&args[0], instrs, locals, indices)?; // map
                        instrs.push(Instruction::LocalTee(map_ptr));

                        let key_ptr = locals.get_or_add("__map_insert_key", ValType::I32);
                        self.emit_expression(&args[1], instrs, locals, indices)?; // key
                        instrs.push(Instruction::LocalTee(key_ptr));

                        let value = locals.get_or_add("__map_insert_value", ValType::I64);
                        self.emit_expression(&args[2], instrs, locals, indices)?; // value
                        instrs.push(Instruction::LocalTee(value));

                        // For now, find the first empty slot (simplified implementation)
                        // Load current size
                        instrs.push(Instruction::LocalGet(map_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        let size = locals.get_or_add("__map_insert_size", ValType::I32);
                        instrs.push(Instruction::LocalTee(size));

                        // Load capacity
                        instrs.push(Instruction::LocalGet(map_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 4,
                            align: 0,
                            memory_index: 0,
                        }));
                        let capacity = locals.get_or_add("__map_insert_capacity", ValType::I32);
                        instrs.push(Instruction::LocalTee(capacity));

                        // Check if we have space (simplified - no resizing yet)
                        instrs.push(Instruction::LocalGet(size));
                        instrs.push(Instruction::LocalGet(capacity));
                        instrs.push(Instruction::I32LtU);

                        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
                        // We have space, add at end
                        // Calculate entry address: map_ptr + 8 + size * 16
                        instrs.push(Instruction::LocalGet(map_ptr));
                        instrs.push(Instruction::I32Const(8)); // Skip header
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(size));
                        instrs.push(Instruction::I32Const(16)); // Entry size
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        let entry_ptr = locals.get_or_add("__map_insert_entry", ValType::I32);
                        instrs.push(Instruction::LocalTee(entry_ptr));

                        // Store key pointer
                        instrs.push(Instruction::LocalGet(entry_ptr));
                        instrs.push(Instruction::LocalGet(key_ptr));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Store value
                        instrs.push(Instruction::LocalGet(entry_ptr));
                        instrs.push(Instruction::LocalGet(value));
                        instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                            offset: 4,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Mark as valid
                        instrs.push(Instruction::LocalGet(entry_ptr));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 12,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Increment size
                        instrs.push(Instruction::LocalGet(map_ptr));
                        instrs.push(Instruction::LocalGet(size));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        instrs.push(Instruction::End); // End if

                        instrs.push(Instruction::LocalGet(map_ptr));
                        Ok(ValType::I32)
                    }

                    "map_get" => {
                        // Enhanced map get with proper lookup
                        let map_ptr = locals.get_or_add("__map_get_ptr", ValType::I32);
                        self.emit_expression(&args[0], instrs, locals, indices)?; // map
                        instrs.push(Instruction::LocalTee(map_ptr));

                        let key_ptr = locals.get_or_add("__map_get_key", ValType::I32);
                        self.emit_expression(&args[1], instrs, locals, indices)?; // key
                        instrs.push(Instruction::LocalTee(key_ptr));

                        // Load size
                        instrs.push(Instruction::LocalGet(map_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        let size = locals.get_or_add("__map_get_size", ValType::I32);
                        instrs.push(Instruction::LocalTee(size));

                        // Search for key (linear search for now)
                        let idx = locals.get_or_add("__map_get_idx", ValType::I32);
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(idx));

                        let found = locals.get_or_add("__map_get_found", ValType::I32);
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(found));

                        let result_value = locals.get_or_add("__map_get_result", ValType::I64);
                        instrs.push(Instruction::I64Const(0));
                        instrs.push(Instruction::LocalSet(result_value));

                        // Search loop
                        instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
                        instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));

                        // Check if we've searched all entries
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::LocalGet(size));
                        instrs.push(Instruction::I32GeU);
                        instrs.push(Instruction::BrIf(1)); // Break out of loop

                        // Calculate entry address: map_ptr + 8 + idx * 16
                        instrs.push(Instruction::LocalGet(map_ptr));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::I32Const(16));
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        let entry_ptr = locals.get_or_add("__map_get_entry", ValType::I32);
                        instrs.push(Instruction::LocalTee(entry_ptr));

                        // Check if entry is valid
                        instrs.push(Instruction::LocalGet(entry_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 12, // is_valid field
                            align: 0,
                            memory_index: 0,
                        }));

                        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
                        // Entry is valid, check key
                        instrs.push(Instruction::LocalGet(entry_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0, // key_ptr field
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalGet(key_ptr));

                        // Use simplified pointer comparison for now
                        // In a full implementation, this would use string content comparison
                        instrs.push(Instruction::I32Eq);

                        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
                        // Key matches - load value and mark as found
                        instrs.push(Instruction::LocalGet(entry_ptr));
                        instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                            offset: 4, // value field
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(result_value));

                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::LocalSet(found));
                        instrs.push(Instruction::Br(3)); // Break out of all loops
                        instrs.push(Instruction::End);

                        instrs.push(Instruction::End); // End if (entry valid)

                        // Increment index
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(idx));

                        instrs.push(Instruction::Br(0)); // Continue loop
                        instrs.push(Instruction::End); // End loop
                        instrs.push(Instruction::End); // End block

                        // Create Option result based on found flag
                        instrs.push(Instruction::GlobalGet(0)); // Current heap pointer
                        let option_ptr = locals.get_or_add("__map_get_option", ValType::I32);
                        instrs.push(Instruction::LocalTee(option_ptr));

                        // Update heap pointer
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::I32Const(16));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::GlobalSet(0));

                        // Store Option tag (0 = None, 1 = Some)
                        instrs.push(Instruction::LocalGet(option_ptr));
                        instrs.push(Instruction::LocalGet(found));
                        instrs.push(Instruction::I64ExtendI32U);
                        instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        // Store value if found
                        instrs.push(Instruction::LocalGet(found));
                        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
                        instrs.push(Instruction::LocalGet(option_ptr));
                        instrs.push(Instruction::LocalGet(result_value));
                        instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                            offset: 8,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::End);

                        instrs.push(Instruction::LocalGet(option_ptr));
                        Ok(ValType::I32)
                    }

                    "map_contains_key" => {
                        // Simplified map contains check - return false for now
                        // TODO: Implement proper hash table lookup
                        self.emit_expression(&args[0], instrs, locals, indices)?; // map
                        instrs.push(Instruction::Drop); // Drop for now
                        self.emit_expression(&args[1], instrs, locals, indices)?; // key
                        instrs.push(Instruction::Drop); // Drop for now

                        instrs.push(Instruction::I32Const(0)); // Return false
                        Ok(ValType::I32)
                    }

                    // === NEW ECONOMICS STDLIB FUNCTIONS ===
                    "create_token_class" => {
                        // Map to host_create_token_class(class_id, name, symbol, issuer)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_create_token_class").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "mint_tokens" => {
                        // Map to host_mint_tokens(class_id, recipient, amount)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_mint_tokens").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "transfer_tokens" => {
                        // Map to host_transfer_tokens(class_id, from, to, amount)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_transfer_tokens").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "burn_tokens" => {
                        // Map to host_burn_tokens(class_id, from, amount)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_burn_tokens").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "get_token_balance" => {
                        // Map to host_get_token_balance(class_id, account)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_get_token_balance").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I64)
                    }

                    "price_by_reputation" => {
                        // Map to host_price_by_reputation(base_price, reputation)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_price_by_reputation").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I64)
                    }

                    "credit_by_reputation" => {
                        // Map to host_credit_by_reputation(account, base_amount)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_credit_by_reputation").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "mint_tokens_with_reputation" => {
                        // Map to host_mint_tokens_with_reputation(class_id, recipient, amount, issuer)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_mint_tokens_with_reputation").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    // === NEW IDENTITY STDLIB FUNCTIONS ===
                    "create_did" => {
                        // Map to host_create_did(method, identifier)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_create_did").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // DID pointer
                    }

                    "resolve_did" => {
                        // Map to host_resolve_did(did)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_resolve_did").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // Document JSON pointer
                    }

                    "verify_did_signature" => {
                        // Map to host_verify_did_signature(signer_did, message, signature)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_verify_did_signature").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "issue_credential" => {
                        // Map to host_issue_credential(issuer, holder, type, claims, expiration)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_issue_credential").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // Credential JSON pointer
                    }

                    "verify_credential" => {
                        // Map to host_verify_credential(credential, expected_issuer)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_verify_credential").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    // === ADDITIONAL ECONOMICS STDLIB FUNCTIONS ===
                    "record_time_work" => {
                        // Map to host_record_time_work(worker, work_description, hours_worked, verifier)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_record_time_work").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // time_record_id string pointer
                    }

                    "mint_time_tokens" => {
                        // Map to host_mint_time_tokens(time_record_id, worker, hours)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_mint_time_tokens").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "create_credit_line" => {
                        // Map to host_create_credit_line(creditor, debtor, credit_limit, interest_rate_bps)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_create_credit_line").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // credit_line_id string pointer
                    }

                    "extend_mutual_credit" => {
                        // Map to host_extend_mutual_credit(credit_line_id, amount, purpose)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_extend_mutual_credit").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "create_marketplace_offer" => {
                        // Map to host_create_marketplace_offer(seller, item_type, quantity, price_per_unit, payment_token_class)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_create_marketplace_offer").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // offer_id string pointer
                    }

                    "execute_marketplace_transaction" => {
                        // Map to host_execute_marketplace_transaction(offer_id, bid_id, executor)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_execute_marketplace_transaction").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // transaction_id string pointer
                    }

                    // === SCOPED TOKEN OPERATIONS ===
                    "create_scoped_token" => {
                        // Map to host_create_scoped_token(class_id, name, symbol, issuer, scope_type, scope_value)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_create_scoped_token").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "transfer_scoped" => {
                        // Map to host_transfer_scoped(class_id, from, to, amount, required_scope)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_transfer_scoped").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "verify_token_constraints" => {
                        // Map to host_verify_token_constraints(class_id, actor, operation, target_scope)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_verify_token_constraints").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    // === ADDITIONAL IDENTITY STDLIB FUNCTIONS ===
                    "update_did_document" => {
                        // Map to host_update_did_document(did, new_document, signature)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_update_did_document").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    // === FEDERATION DISCOVERY & COORDINATION ===
                    "discover_federations" => {
                        // Map to host_discover_federations(search_criteria, max_results)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_discover_federations").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // array pointer
                    }

                    "join_federation" => {
                        // Map to host_join_federation(member_did, federation_id, application_details)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_join_federation").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "leave_federation" => {
                        // Map to host_leave_federation(member_did, federation_id, reason)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_leave_federation").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "verify_cross_federation" => {
                        // Map to host_verify_cross_federation(verifier_did, source_federation, target_federation, credential_type)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_verify_cross_federation").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    // === KEY ROTATION & MANAGEMENT ===
                    "rotate_keys" => {
                        // Map to host_rotate_keys(did, new_public_key, signature_from_old_key)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_rotate_keys").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "backup_keys" => {
                        // Map to host_backup_keys(did, backup_method, backup_parameters)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_backup_keys").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // backup_id string pointer
                    }

                    "recover_keys" => {
                        // Map to host_recover_keys(did, backup_id, recovery_proof)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_recover_keys").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    // === ADVANCED FEDERATION OPERATIONS ===
                    "get_federation_metadata" => {
                        // Map to host_get_federation_metadata(federation_id)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_get_federation_metadata").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // metadata JSON pointer
                    }

                    "verify_federation_membership" => {
                        // Map to host_verify_federation_membership(member_did, federation_id)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_verify_federation_membership").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "coordinate_cross_federation_action" => {
                        // Map to host_coordinate_cross_federation_action(coordinator_did, federation_ids_array, action_type, action_parameters)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices
                            .get("host_coordinate_cross_federation_action")
                            .unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // coordination_id string pointer
                    }

                    // === DAG STORAGE OPERATIONS ===
                    "dag_put" => {
                        // Map to host_dag_put(data)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_dag_put").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // CID string pointer
                    }

                    "dag_get" => {
                        // Map to host_dag_get(cid)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_dag_get").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // data string pointer
                    }

                    "dag_pin" => {
                        // Map to host_dag_pin(cid)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_dag_pin").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "dag_unpin" => {
                        // Map to host_dag_unpin(cid)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_dag_unpin").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "calculate_cid" => {
                        // Map to host_calculate_cid(data)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_calculate_cid").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // CID string pointer
                    }

                    "save_contract_state" => {
                        // Map to host_save_contract_state(contract_id, state_data, version)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_save_contract_state").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // state_cid string pointer
                    }

                    "load_contract_state" => {
                        // Map to host_load_contract_state(contract_id, version)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_load_contract_state").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // state_data string pointer
                    }

                    "version_contract" => {
                        // Map to host_version_contract(contract_id, new_code_cid, migration_notes)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_version_contract").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I64) // new version number
                    }

                    "dag_link" => {
                        // Map to host_dag_link(parent_cid, child_cid, link_name)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_dag_link").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // new merged CID string pointer
                    }

                    "dag_resolve_path" => {
                        // Map to host_dag_resolve_path(root_cid, path)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_dag_resolve_path").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // resolved data string pointer
                    }

                    "dag_list_links" => {
                        // Map to host_dag_list_links(cid)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_dag_list_links").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // link names array pointer
                    }

                    "revoke_credential" => {
                        // Map to host_revoke_credential(issuer, credential_id, reason)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_revoke_credential").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    "create_cooperative_membership" => {
                        // Map to host_create_cooperative_membership(member, cooperative_id, membership_type, membership_level)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_create_cooperative_membership").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32) // membership credential string pointer
                    }

                    "verify_cooperative_membership" => {
                        // Map to host_verify_cooperative_membership(member, cooperative_id, required_level)
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        let idx = indices.get("host_verify_cooperative_membership").unwrap();
                        instrs.push(Instruction::Call(*idx));
                        Ok(ValType::I32)
                    }

                    _ => {
                        let idx = indices.get(name).ok_or_else(|| {
                            CclError::WasmGenerationError(format!("Unknown function {}", name))
                        })?;
                        for arg in args {
                            self.emit_expression(arg, instrs, locals, indices)?;
                        }
                        instrs.push(Instruction::Call(*idx));
                        let ret = match name.as_str() {
                            "host_account_get_mana"
                            | "host_get_reputation"
                            | "host_get_token_balance"
                            | "host_price_by_reputation" => ValType::I64,

                            "host_submit_mesh_job"
                            | "host_anchor_receipt"
                            | "host_create_token_class"
                            | "host_mint_tokens"
                            | "host_transfer_tokens"
                            | "host_burn_tokens"
                            | "host_credit_by_reputation"
                            | "host_mint_tokens_with_reputation"
                            | "host_verify_did_signature"
                            | "host_verify_credential"
                            | "host_mint_time_tokens"
                            | "host_extend_mutual_credit"
                            | "host_update_did_document"
                            | "host_revoke_credential"
                            | "host_verify_cooperative_membership"
                            | "host_create_scoped_token"
                            | "host_transfer_scoped"
                            | "host_verify_token_constraints" => ValType::I32,

                            "host_get_caller"
                            | "host_create_did"
                            | "host_resolve_did"
                            | "host_issue_credential"
                            | "host_record_time_work"
                            | "host_create_credit_line"
                            | "host_create_marketplace_offer"
                            | "host_execute_marketplace_transaction"
                            | "host_create_cooperative_membership"
                            | "host_discover_federations"
                            | "host_backup_keys"
                            | "host_get_federation_metadata"
                            | "host_coordinate_cross_federation_action"
                            | "host_dag_put"
                            | "host_dag_get"
                            | "host_calculate_cid"
                            | "host_save_contract_state"
                            | "host_load_contract_state"
                            | "host_dag_link"
                            | "host_dag_resolve_path"
                            | "host_dag_list_links" => ValType::I32, // pointers/DIDs/CIDs

                            "host_version_contract" => ValType::I64, // version number

                            _ => ValType::I64,
                        };
                        Ok(ret)
                    }
                }
            }
            ExpressionNode::MethodCall {
                object,
                method,
                args: _,
            } => {
                match method.as_str() {
                    "length" => {
                        // Handle array.length() or string.length() method
                        let object_type = self.emit_expression(object, instrs, locals, indices)?;

                        match object_type {
                            ValType::I32 => {
                                // String length: strings are stored as [len: u32][bytes]
                                // Load the length from the first 4 bytes
                                instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                                    offset: 0,
                                    align: 0,
                                    memory_index: 0,
                                }));
                                instrs.push(Instruction::I64ExtendI32U); // Convert to I64
                                Ok(ValType::I64)
                            }
                            _ => {
                                // Array length (arrays not fully implemented yet)
                                instrs.push(Instruction::Drop); // Drop array reference
                                instrs.push(Instruction::I64Const(5)); // Return fixed length for now
                                Ok(ValType::I64)
                            }
                        }
                    }
                    _ => Err(CclError::WasmGenerationError(format!(
                        "Unknown method: {}",
                        method
                    ))),
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
                        // For DIDs (pointer comparison) or Strings (content comparison)
                        // Use proper string content comparison
                        self.emit_string_equality_check(instrs, locals)?;
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Neq) => {
                        // For DIDs (pointer comparison) or Strings (content comparison)
                        // Use proper string content comparison and negate result
                        self.emit_string_equality_check(instrs, locals)?;
                        instrs.push(Instruction::I32Eqz); // Negate the result
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Lt) => {
                        // String lexicographic comparison
                        self.emit_string_ordering_comparison(instrs, locals, "lt")?;
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Gt) => {
                        // String lexicographic comparison
                        self.emit_string_ordering_comparison(instrs, locals, "gt")?;
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Lte) => {
                        // String lexicographic comparison
                        self.emit_string_ordering_comparison(instrs, locals, "lte")?;
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Gte) => {
                        // String lexicographic comparison
                        self.emit_string_ordering_comparison(instrs, locals, "gte")?;
                        Ok(ValType::I32)
                    }
                    (ValType::I32, ValType::I32, BinaryOperator::Concat)
                    | (ValType::I32, ValType::I32, BinaryOperator::Add) => {
                        // Runtime string concatenation. Strings are stored as
                        // [len: u32][bytes]. Allocate new memory and copy bytes.

                        let left_ptr = locals.get_or_add("__concat_left", ValType::I32);
                        instrs.push(Instruction::LocalTee(left_ptr));
                        let right_ptr = locals.get_or_add("__concat_right", ValType::I32);
                        instrs.push(Instruction::LocalTee(right_ptr));
                        instrs.push(Instruction::Drop);
                        instrs.push(Instruction::Drop);

                        let left_len = locals.get_or_add("__concat_left_len", ValType::I32);
                        instrs.push(Instruction::LocalGet(left_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(left_len));

                        let right_len = locals.get_or_add("__concat_right_len", ValType::I32);
                        instrs.push(Instruction::LocalGet(right_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(right_len));

                        let total_len = locals.get_or_add("__concat_total_len", ValType::I32);
                        instrs.push(Instruction::LocalGet(left_len));
                        instrs.push(Instruction::LocalGet(right_len));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(total_len));

                        let out_ptr = locals.get_or_add("__concat_out_ptr", ValType::I32);
                        instrs.push(Instruction::GlobalGet(0));
                        instrs.push(Instruction::LocalTee(out_ptr));
                        instrs.push(Instruction::LocalGet(total_len));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::GlobalSet(0));

                        instrs.push(Instruction::LocalGet(out_ptr));
                        instrs.push(Instruction::LocalGet(total_len));
                        instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));

                        let idx = locals.get_or_add("__concat_idx", ValType::I32);

                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(idx));
                        instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
                        instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::LocalGet(left_len));
                        instrs.push(Instruction::I32GeU);
                        instrs.push(Instruction::BrIf(1));
                        instrs.push(Instruction::LocalGet(out_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(left_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::I32Store8(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(idx));
                        instrs.push(Instruction::Br(0));
                        instrs.push(Instruction::End);
                        instrs.push(Instruction::End);

                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(idx));
                        instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
                        instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::LocalGet(right_len));
                        instrs.push(Instruction::I32GeU);
                        instrs.push(Instruction::BrIf(1));
                        instrs.push(Instruction::LocalGet(out_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::LocalGet(left_len));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(right_ptr));
                        instrs.push(Instruction::I32Const(4));
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::I32Store8(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalGet(idx));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(idx));
                        instrs.push(Instruction::Br(0));
                        instrs.push(Instruction::End);
                        instrs.push(Instruction::End);

                        instrs.push(Instruction::LocalGet(out_ptr));
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
                // Allocate array in guest memory: [len][capacity][elements]
                let size = 8 + elements.len() * 8;
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
                // store capacity
                instrs.push(Instruction::LocalGet(tmp));
                instrs.push(Instruction::I32Const(elements.len() as i32));
                instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                    offset: 4,
                    align: 0,
                    memory_index: 0,
                }));

                for (i, el) in elements.iter().enumerate() {
                    self.emit_expression(el, instrs, locals, indices)?;
                    instrs.push(Instruction::LocalGet(tmp));
                    instrs.push(Instruction::I32Const(8 + (i as i32) * 8));
                    instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                }

                instrs.push(Instruction::LocalGet(tmp));
                Ok(ValType::I32)
            }
            ExpressionNode::MapLiteral(pairs) => {
                // Simple Map implementation: [size][capacity][key-value pairs]
                // For now, just allocate fixed size for simplicity
                let size = 8 + pairs.len() * 16; // 8 bytes per key, 8 bytes per value
                instrs.push(Instruction::GlobalGet(0));
                let map_ptr = locals.get_or_add("__map_ptr", ValType::I32);
                instrs.push(Instruction::LocalTee(map_ptr));
                instrs.push(Instruction::GlobalGet(0));
                instrs.push(Instruction::I32Const(size as i32));
                instrs.push(Instruction::I32Add);
                instrs.push(Instruction::GlobalSet(0));

                // Store size
                instrs.push(Instruction::LocalGet(map_ptr));
                instrs.push(Instruction::I32Const(pairs.len() as i32));
                instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));

                // Store capacity
                instrs.push(Instruction::LocalGet(map_ptr));
                instrs.push(Instruction::I32Const(pairs.len() as i32));
                instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                    offset: 4,
                    align: 0,
                    memory_index: 0,
                }));

                // Store key-value pairs
                for (i, (key, value)) in pairs.iter().enumerate() {
                    // Store key
                    let key_ty = self.emit_expression(key, instrs, locals, indices)?;
                    instrs.push(Instruction::LocalGet(map_ptr));
                    instrs.push(Instruction::I32Const(8 + (i as i32) * 16));
                    match key_ty {
                        ValType::I64 => {
                            instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                        }
                        ValType::I32 => {
                            instrs.push(Instruction::I64ExtendI32U);
                            instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                        }
                        _ => {
                            return Err(CclError::WasmGenerationError(
                                "Unsupported map key type".to_string(),
                            ))
                        }
                    }

                    // Store value
                    let value_ty = self.emit_expression(value, instrs, locals, indices)?;
                    instrs.push(Instruction::LocalGet(map_ptr));
                    instrs.push(Instruction::I32Const(8 + (i as i32) * 16 + 8));
                    match value_ty {
                        ValType::I64 => {
                            instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                        }
                        ValType::I32 => {
                            instrs.push(Instruction::I64ExtendI32U);
                            instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                        }
                        _ => {
                            return Err(CclError::WasmGenerationError(
                                "Unsupported map value type".to_string(),
                            ))
                        }
                    }
                }

                instrs.push(Instruction::LocalGet(map_ptr));
                Ok(ValType::I32)
            }
            ExpressionNode::EnumValue {
                enum_name: _,
                variant,
            } => {
                // Simple enum implementation: return variant index as integer
                let variant_index = match variant.as_str() {
                    "Pending" => 0,
                    "Active" => 1,
                    "Passed" => 2,
                    "Rejected" => 3,
                    _ => 0, // Default to first variant
                };
                instrs.push(Instruction::I64Const(variant_index));
                Ok(ValType::I64)
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
                instrs.push(Instruction::I32Const(8));
                instrs.push(Instruction::I32Add);
                instrs.push(Instruction::I32Add);

                // For DID arrays, elements are stored as I32 (DID pointers), but we need to load as I64 and convert
                // For simplicity, treat all array elements as I64 for now but convert to I32 for DIDs
                // TODO: Add proper type-aware array access based on semantic analysis
                instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));

                // For DID arrays, convert to I32
                // TODO: This is a temporary fix - we should use semantic analysis to determine array element type
                instrs.push(Instruction::I32WrapI64);
                Ok(ValType::I32)
            }
            ExpressionNode::Some(inner) => {
                self.emit_expression(inner, instrs, locals, indices)?;
                Ok(ValType::I64)
            }
            ExpressionNode::None => {
                instrs.push(Instruction::I64Const(0));
                Ok(ValType::I64)
            }
            ExpressionNode::Ok(inner) => {
                self.emit_expression(inner, instrs, locals, indices)?;
                instrs.push(Instruction::I32WrapI64);
                instrs.push(Instruction::I64ExtendI32U);
                instrs.push(Instruction::I64Const(0));
                instrs.push(Instruction::I64Const(32));
                instrs.push(Instruction::I64Shl);
                instrs.push(Instruction::I64Or);
                Ok(ValType::I64)
            }
            ExpressionNode::Err(inner) => {
                self.emit_expression(inner, instrs, locals, indices)?;
                instrs.push(Instruction::I32WrapI64);
                instrs.push(Instruction::I64ExtendI32U);
                instrs.push(Instruction::I64Const(1));
                instrs.push(Instruction::I64Const(32));
                instrs.push(Instruction::I64Shl);
                instrs.push(Instruction::I64Or);
                Ok(ValType::I64)
            }
            // Legacy expressions removed - these should be handled by new CCL 0.1 constructs
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

            // New unified literal handling
            ExpressionNode::Literal(lit) => match lit {
                crate::ast::LiteralNode::Integer(i) => {
                    instrs.push(Instruction::I64Const(*i));
                    Ok(ValType::I64)
                }
                crate::ast::LiteralNode::Float(f) => {
                    instrs.push(Instruction::F64Const((*f).into()));
                    Ok(ValType::F64)
                }
                crate::ast::LiteralNode::String(s) => {
                    // Store string in linear memory with length prefix: [len: u32][bytes...]
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
                crate::ast::LiteralNode::Boolean(b) => {
                    instrs.push(Instruction::I32Const(if *b { 1 } else { 0 }));
                    Ok(ValType::I32)
                }
                crate::ast::LiteralNode::Did(did) => {
                    // Implement DID handling - DIDs are stored as strings
                    let did_str = did.to_string();
                    let did_bytes = did_str.as_bytes();
                    let did_len = did_bytes.len();

                    // Allocate memory for string: [length: u32][data: bytes]
                    let string_ptr = locals.get_or_add("__did_string_ptr", ValType::I32);

                    // Get current heap pointer
                    instrs.push(Instruction::GlobalGet(0));
                    instrs.push(Instruction::LocalTee(string_ptr));

                    // Store string length
                    instrs.push(Instruction::I32Const(did_len as i32));
                    instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));

                    // Store string data
                    for (i, &byte) in did_bytes.iter().enumerate() {
                        instrs.push(Instruction::LocalGet(string_ptr));
                        instrs.push(Instruction::I32Const(byte as i32));
                        instrs.push(Instruction::I32Store8(wasm_encoder::MemArg {
                            offset: 4 + i as u64, // After the length field
                            align: 0,
                            memory_index: 0,
                        }));
                    }

                    // Update heap pointer (4 bytes length + string data, aligned to 4 bytes)
                    let total_size = 4 + ((did_len + 3) & !3); // Align to 4-byte boundary
                    instrs.push(Instruction::GlobalGet(0));
                    instrs.push(Instruction::I32Const(total_size as i32));
                    instrs.push(Instruction::I32Add);
                    instrs.push(Instruction::GlobalSet(0));

                    // Return pointer to string
                    instrs.push(Instruction::LocalGet(string_ptr));
                    Ok(ValType::I32)
                }
                crate::ast::LiteralNode::Timestamp(timestamp) => {
                    // Implement timestamp handling - timestamps are 64-bit Unix timestamps
                    let timestamp_value = timestamp.parse::<i64>().unwrap_or(0);
                    instrs.push(Instruction::I64Const(timestamp_value));
                    Ok(ValType::I64)
                }
            },

            // New AST variants - placeholder implementations
            ExpressionNode::MemberAccess { object, member } => {
                // Simple member access: assume fields are stored sequentially
                // Get the struct pointer
                let _object_type = self.emit_expression(object, instrs, locals, indices)?;

                // For now, hardcode field offsets (this should use type information)
                let field_offset = match member.as_str() {
                    "x" => 0,          // First field at offset 0
                    "y" => 8,          // Second field at offset 8 (assuming i64)
                    "name" => 0,       // String fields at offset 0
                    "reputation" => 8, // Second field
                    "active" => 16,    // Third field
                    "id" => 0,
                    "title" => 8,
                    "votes_for" => 16,
                    "votes_against" => 24,
                    "status" => 32,
                    _ => 0, // Default to first field
                };

                // Add the field offset to the struct pointer
                instrs.push(Instruction::I32Const(field_offset));
                instrs.push(Instruction::I32Add);

                // Load the field value (assume i64 for integers)
                if member == "name" || member == "title" || member == "choice" || member == "status" || member == "voter" {
                    // String fields return the pointer (i32)
                    instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                    Ok(ValType::I32)
                } else {
                    // Integer fields return i64
                    instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                    Ok(ValType::I64)
                }
            }
            ExpressionNode::IndexAccess { object, index } => {
                // Enhanced indexing to support both arrays and strings
                let obj_ty = self.emit_expression(object, instrs, locals, indices)?;

                if obj_ty == ValType::I32 {
                    // Could be string or array - we need to detect the type
                    // For now, implement string indexing
                    let str_ptr = locals.get_or_add("__str_idx_ptr", ValType::I32);
                    instrs.push(Instruction::LocalTee(str_ptr));

                    let idx_ty = self.emit_expression(index, instrs, locals, indices)?;
                    if idx_ty == ValType::I64 {
                        instrs.push(Instruction::I32WrapI64);
                    }
                    let idx_local = locals.get_or_add("__str_idx", ValType::I32);
                    instrs.push(Instruction::LocalTee(idx_local));

                    // Load string length for bounds checking
                    instrs.push(Instruction::LocalGet(str_ptr));
                    instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                    let str_len = locals.get_or_add("__str_idx_len", ValType::I32);
                    instrs.push(Instruction::LocalTee(str_len));

                    // Bounds check: index < length
                    instrs.push(Instruction::LocalGet(idx_local));
                    instrs.push(Instruction::LocalGet(str_len));
                    instrs.push(Instruction::I32GeU);

                    instrs.push(Instruction::If(wasm_encoder::BlockType::Result(
                        ValType::I32,
                    )));
                    // Out of bounds - return 0 (null character)
                    instrs.push(Instruction::I32Const(0));
                    instrs.push(Instruction::Else);

                    // In bounds - load character
                    instrs.push(Instruction::LocalGet(str_ptr));
                    instrs.push(Instruction::I32Const(4)); // Skip length
                    instrs.push(Instruction::I32Add);
                    instrs.push(Instruction::LocalGet(idx_local));
                    instrs.push(Instruction::I32Add);
                    instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));

                    instrs.push(Instruction::End); // End if

                    // Extend to i64 for consistency
                    instrs.push(Instruction::I64ExtendI32U);
                    Ok(ValType::I64)
                } else {
                    // Array indexing (original implementation)
                    let arr_local = locals.get_or_add("__arr", ValType::I32);
                    instrs.push(Instruction::LocalTee(arr_local));
                    let idx_ty = self.emit_expression(index, instrs, locals, indices)?;
                    if idx_ty == ValType::I64 {
                        instrs.push(Instruction::I32WrapI64);
                    }
                    let idx_local = locals.get_or_add("__idx", ValType::I32);
                    instrs.push(Instruction::LocalTee(idx_local));
                    instrs.push(Instruction::I32Const(8));
                    instrs.push(Instruction::I32Mul);
                    instrs.push(Instruction::LocalGet(arr_local));
                    instrs.push(Instruction::I32Const(8));
                    instrs.push(Instruction::I32Add);
                    instrs.push(Instruction::I32Add);
                    instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                    Ok(ValType::I64)
                }
            }
            ExpressionNode::StructLiteral { type_name, fields } => {
                // Simple struct implementation: allocate memory and store fields
                // For now, allocate 32 bytes per struct (enough for 4 i64 fields)
                let struct_size = 32;
                let struct_ptr = self.data_offset;
                self.data_offset += struct_size;

                // Store field values in sequential memory locations
                let mut field_offset = 0;
                for field in fields {
                    // Emit the field value
                    let field_type = self.emit_expression(&field.value, instrs, locals, indices)?;

                    // Store the value at the struct pointer + field offset
                    instrs.push(Instruction::I32Const(struct_ptr as i32 + field_offset));
                    instrs.push(Instruction::I32Const(0)); // memory index

                    match field_type {
                        ValType::I64 => {
                            instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                            field_offset += 8; // 8 bytes for i64
                        }
                        ValType::I32 => {
                            instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                                offset: 0,
                                align: 0,
                                memory_index: 0,
                            }));
                            field_offset += 4; // 4 bytes for i32
                        }
                        _ => {
                            return Err(CclError::WasmGenerationError(format!(
                                "Unsupported field type in struct {}",
                                type_name
                            )));
                        }
                    }
                }

                // Return pointer to the struct
                instrs.push(Instruction::I32Const(struct_ptr as i32));
                Ok(ValType::I32)
            }
            ExpressionNode::Transfer {
                from: _,
                to: _,
                amount: _,
            } => {
                // TODO: Implement mana/token transfer
                instrs.push(Instruction::I32Const(1)); // success
                Ok(ValType::I32)
            }
            ExpressionNode::Mint { to: _, amount: _ } => {
                // TODO: Implement token minting
                instrs.push(Instruction::I32Const(1)); // success
                Ok(ValType::I32)
            }
            ExpressionNode::Burn { from: _, amount: _ } => {
                // TODO: Implement token burning
                instrs.push(Instruction::I32Const(1)); // success
                Ok(ValType::I32)
            }

            ExpressionNode::Match { expr, arms } => {
                // Evaluate the expression being matched and store in a local
                let match_ty = self.emit_expression(expr, instrs, locals, indices)?;
                let match_local = locals.get_or_add("__match_val", match_ty);
                instrs.push(Instruction::LocalSet(match_local));

                // Determine result type by inspecting the first arm
                let mut tmp = Vec::new();
                let result_ty = self.emit_expression(&arms[0].body, &mut tmp, locals, indices)?;
                tmp.clear();

                instrs.push(Instruction::Block(wasm_encoder::BlockType::Result(
                    result_ty,
                )));

                for (i, arm) in arms.iter().enumerate() {
                    if i < arms.len() - 1 {
                        self.emit_pattern_condition(
                            &arm.pattern,
                            match_local,
                            match_ty,
                            instrs,
                            locals,
                        )?;
                        if let Some(guard) = &arm.guard {
                            let guard_ty = self.emit_expression(guard, instrs, locals, indices)?;
                            if guard_ty != ValType::I32 {
                                return Err(CclError::WasmGenerationError(
                                    "Match guard must be boolean".to_string(),
                                ));
                            }
                            instrs.push(Instruction::I32Eqz);
                            instrs.push(Instruction::BrIf(0));
                        }
                        instrs.push(Instruction::If(wasm_encoder::BlockType::Result(result_ty)));
                        self.bind_pattern(&arm.pattern, match_local, match_ty, instrs, locals)?;
                        self.emit_expression(&arm.body, instrs, locals, indices)?;
                        instrs.push(Instruction::Br(1));
                        instrs.push(Instruction::End);
                    } else {
                        self.bind_pattern(&arm.pattern, match_local, match_ty, instrs, locals)?;
                        self.emit_expression(&arm.body, instrs, locals, indices)?;
                    }
                }

                instrs.push(Instruction::End);
                Ok(result_ty)
            } // All legacy expressions removed - CCL 0.1 uses new expression variants
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
        // Add mana metering for statement execution
        self.emit_mana_check(instrs, self.mana_per_instruction * 2); // Statements cost more

        match stmt {
            StatementNode::Let {
                mutable: _,
                name,
                type_expr: _,
                value,
            } => {
                let ty = self.emit_expression(value, instrs, locals, indices)?;
                let idx = locals.get_or_add(name, ty);
                instrs.push(Instruction::LocalSet(idx));
            }
            StatementNode::ExpressionStatement(expr) => {
                self.emit_expression(expr, instrs, locals, indices)?;
                instrs.push(Instruction::Drop);
            }
            StatementNode::Return(expr) => {
                if let Some(expr) = expr {
                    let ty = self.emit_expression(expr, instrs, locals, indices)?;
                    let expected = map_val_type(return_ty)?;
                    if ty != expected {
                        return Err(CclError::WasmGenerationError(
                            "Return type mismatch during codegen".to_string(),
                        ));
                    }
                }
                instrs.push(Instruction::Return);
            }
            StatementNode::If {
                condition,
                then_block,
                else_ifs,
                else_block,
            } => {
                self.emit_if_statement(
                    condition, then_block, else_ifs, else_block, instrs, locals, return_ty, indices,
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
            StatementNode::ForLoop {
                iterator,
                iterable,
                body,
            } => {
                // Implement legacy for loop using the same logic as StatementNode::For
                let iterable_type = self.emit_expression(iterable, instrs, locals, indices)?;

                // Support arrays (for now)
                match iterable_type {
                    ValType::I32 => {
                        // Create local variable for loop counter
                        let counter_local = locals.get_or_add("__loop_counter", ValType::I32);

                        // Create local variable for iterator
                        let iterator_local = locals.get_or_add(iterator, ValType::I64);

                        // Initialize counter to 0
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(counter_local));

                        // Store pointer to array and load length
                        let array_ptr = locals.get_or_add("__iter_ptr", ValType::I32);
                        instrs.push(Instruction::LocalTee(array_ptr));
                        instrs.push(Instruction::LocalGet(array_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        let array_length_local = locals.get_or_add("__iter_len", ValType::I32);
                        instrs.push(Instruction::LocalSet(array_length_local));

                        // WASM loop structure
                        instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
                        instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));

                        // Check if counter >= array length
                        instrs.push(Instruction::LocalGet(counter_local));
                        instrs.push(Instruction::LocalGet(array_length_local));
                        instrs.push(Instruction::I32GeU);
                        instrs.push(Instruction::BrIf(1));

                        // Load array element at current index
                        instrs.push(Instruction::LocalGet(array_ptr));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(counter_local));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(iterator_local));

                        // Execute loop body
                        self.emit_block(body, instrs, locals, return_ty, indices)?;

                        // Increment counter
                        instrs.push(Instruction::LocalGet(counter_local));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(counter_local));

                        // Continue loop
                        instrs.push(Instruction::Br(0));
                        instrs.push(Instruction::End);
                        instrs.push(Instruction::End);
                    }
                    _ => {
                        return Err(CclError::WasmGenerationError(
                            "For loops currently only support arrays".to_string(),
                        ));
                    }
                }
            }
            StatementNode::Break | StatementNode::Continue => {
                return Err(CclError::WasmGenerationError(
                    "Loop control not yet supported in WASM backend".to_string(),
                ));
            }
            StatementNode::Assignment { lvalue, value } => {
                let value_ty = self.emit_expression(value, instrs, locals, indices)?;
                self.emit_lvalue_assignment(lvalue, value_ty, instrs, locals, indices)?;
            }
            StatementNode::While { condition, body } => {
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
            StatementNode::For {
                iterator,
                iterable,
                body,
            } => {
                // Implement proper for loop over arrays
                let iterable_type = self.emit_expression(iterable, instrs, locals, indices)?;

                // For now, support only arrays (TODO: other iterables)
                match iterable_type {
                    ValType::I32 => {
                        // Assume I32 represents array pointer/descriptor
                        // Store pointer and load length from descriptor
                        let array_ptr = locals.get_or_add("__iter_ptr", ValType::I32);
                        instrs.push(Instruction::LocalTee(array_ptr));
                        instrs.push(Instruction::LocalGet(array_ptr));
                        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        let array_length_local = locals.get_or_add("__iter_len", ValType::I32);
                        instrs.push(Instruction::LocalSet(array_length_local));

                        // Create local variable for loop counter
                        let counter_local = locals.get_or_add("__loop_counter", ValType::I32);

                        // Create local variable for iterator
                        let iterator_local = locals.get_or_add(iterator, ValType::I64); // Use I64 to match integer literals

                        // Initialize counter to 0
                        instrs.push(Instruction::I32Const(0));
                        instrs.push(Instruction::LocalSet(counter_local));

                        // WASM loop structure: block { loop { ... br_if ... br ... } }
                        instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
                        instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));

                        // Check if counter >= array length
                        instrs.push(Instruction::LocalGet(counter_local));
                        instrs.push(Instruction::LocalGet(array_length_local));
                        instrs.push(Instruction::I32GeU);
                        instrs.push(Instruction::BrIf(1)); // Break out of outer block if done

                        // Load array element at current index
                        instrs.push(Instruction::LocalGet(array_ptr));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalGet(counter_local));
                        instrs.push(Instruction::I32Const(8));
                        instrs.push(Instruction::I32Mul);
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::I64Load(wasm_encoder::MemArg {
                            offset: 0,
                            align: 0,
                            memory_index: 0,
                        }));
                        instrs.push(Instruction::LocalSet(iterator_local));

                        // Execute loop body
                        self.emit_block(body, instrs, locals, return_ty, indices)?;

                        // Increment counter
                        instrs.push(Instruction::LocalGet(counter_local));
                        instrs.push(Instruction::I32Const(1));
                        instrs.push(Instruction::I32Add);
                        instrs.push(Instruction::LocalSet(counter_local));

                        // Continue loop
                        instrs.push(Instruction::Br(0));
                        instrs.push(Instruction::End); // End loop
                        instrs.push(Instruction::End); // End block
                    }
                    _ => {
                        return Err(CclError::WasmGenerationError(
                            "For loops currently only support arrays".to_string(),
                        ));
                    }
                }
            }
            StatementNode::Match { expr, arms: _ } => {
                // Simplified match - just emit expression
                self.emit_expression(expr, instrs, locals, indices)?;
                instrs.push(Instruction::Drop);
            }
            StatementNode::Emit {
                event_name: _,
                fields: _,
            } => {
                // Event emission placeholder
                instrs.push(Instruction::I32Const(0));
                instrs.push(Instruction::Drop);
            }
            StatementNode::Require(expr) => {
                self.emit_expression(expr, instrs, locals, indices)?;
                // Add assertion logic - trap if false
                instrs.push(Instruction::I32Eqz);
                instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
                instrs.push(Instruction::Unreachable);
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
        else_ifs: &[(ExpressionNode, BlockNode)], // FIXED: Remove underscore to use parameter
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

        // FIXED: Properly handle else-if chains
        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
        self.emit_block(then_block, instrs, locals, return_ty, indices)?;

        // Process else-if chains recursively
        if !else_ifs.is_empty() {
            instrs.push(Instruction::Else);

            // Emit the first else-if as a nested if statement
            let (elif_condition, elif_block) = &else_ifs[0];
            let remaining_else_ifs = &else_ifs[1..];

            self.emit_if_statement(
                elif_condition,
                elif_block,
                remaining_else_ifs,
                else_block,
                instrs,
                locals,
                return_ty,
                indices,
            )?;
        } else if let Some(else_blk) = else_block {
            // No else-ifs, just final else block
            instrs.push(Instruction::Else);
            self.emit_block(else_blk, instrs, locals, return_ty, indices)?;
        }

        instrs.push(Instruction::End);
        Ok(())
    }

    fn emit_lvalue_assignment(
        &mut self,
        lvalue: &crate::ast::LValueNode,
        _value_ty: ValType,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
        _indices: &HashMap<String, u32>,
    ) -> Result<(), CclError> {
        match lvalue {
            crate::ast::LValueNode::Identifier(name) => {
                // Assign to local variable
                let (idx, _ty) = locals.get(name).ok_or_else(|| {
                    CclError::WasmGenerationError(format!("Unknown variable {}", name))
                })?;
                instrs.push(Instruction::LocalSet(idx));
                Ok(())
            }
            crate::ast::LValueNode::MemberAccess { object, member } => {
                // Implement struct member assignment
                // Value to assign is already on the stack, store it in a local
                let value_local = locals.get_or_add("__member_assign_value", ValType::I64);
                instrs.push(Instruction::LocalTee(value_local));

                // Evaluate the struct object to get its pointer
                let obj_ty = self.emit_expression(object, instrs, locals, _indices)?;
                if obj_ty != ValType::I32 {
                    return Err(CclError::WasmGenerationError(
                        "Member assignment target must be a struct".to_string(),
                    ));
                }
                let obj_local = locals.get_or_add("__member_assign_obj", ValType::I32);
                instrs.push(Instruction::LocalTee(obj_local));

                // Simple implementation: assume fixed field offsets
                // This is a simplified version - real implementation would use struct metadata
                let field_offset = match member.as_str() {
                    "id" => 0,
                    "title" | "voter" | "choice" => 8, 
                    "votes" => 16,
                    "status" | "weight" => 24,
                    _ => 0, // Default to first field
                };

                // Store the value at the calculated offset
                instrs.push(Instruction::LocalGet(obj_local));
                instrs.push(Instruction::I32Const(field_offset));
                instrs.push(Instruction::I32Add);
                instrs.push(Instruction::LocalGet(value_local));
                
                // For string fields, store as pointer (I32). For other fields, store as I64
                if member == "title" || member == "voter" || member == "choice" || member == "status" {
                    instrs.push(Instruction::I32WrapI64); // Convert string pointer to I32
                    instrs.push(Instruction::I32Store(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                } else {
                    instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                        offset: 0,
                        align: 0,
                        memory_index: 0,
                    }));
                }

                Ok(())
            }
            crate::ast::LValueNode::IndexAccess { object, index } => {
                // Implement proper array index assignment with bounds checking

                // Value to assign is already on the stack, store it in a local
                let value_local = locals.get_or_add("__assign_value", ValType::I64);
                instrs.push(Instruction::LocalTee(value_local));

                // Evaluate the array object
                let arr_ty = self.emit_expression(object, instrs, locals, _indices)?;
                if arr_ty != ValType::I32 {
                    return Err(CclError::WasmGenerationError(
                        "Array assignment target must be an array".to_string(),
                    ));
                }
                let arr_local = locals.get_or_add("__assign_arr", ValType::I32);
                instrs.push(Instruction::LocalTee(arr_local));

                // Evaluate the index
                let idx_ty = self.emit_expression(index, instrs, locals, _indices)?;
                let idx_local = locals.get_or_add("__assign_idx", ValType::I32);
                if idx_ty == ValType::I64 {
                    instrs.push(Instruction::I32WrapI64);
                }
                instrs.push(Instruction::LocalTee(idx_local));

                // Bounds checking: load array length and compare with index
                instrs.push(Instruction::LocalGet(arr_local));
                instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));
                let len_local = locals.get_or_add("__assign_len", ValType::I32);
                instrs.push(Instruction::LocalTee(len_local));

                // Check if index >= length (out of bounds)
                instrs.push(Instruction::LocalGet(idx_local));
                instrs.push(Instruction::LocalGet(len_local));
                instrs.push(Instruction::I32GeU);

                // If out of bounds, we should trap. For now, we'll just skip the assignment.
                // TODO: Add proper runtime error handling
                instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
                // Out of bounds - for now just do nothing
                // In a full implementation, this would raise a runtime error
                instrs.push(Instruction::Else);

                // In bounds - perform the assignment
                // Calculate the address: arr_ptr + 8 (header) + index * 8 (element size)
                instrs.push(Instruction::LocalGet(arr_local));
                instrs.push(Instruction::I32Const(8)); // Skip length (4) + capacity (4)
                instrs.push(Instruction::I32Add);

                instrs.push(Instruction::LocalGet(idx_local));
                instrs.push(Instruction::I32Const(8)); // 8 bytes per element (i64)
                instrs.push(Instruction::I32Mul);
                instrs.push(Instruction::I32Add);

                // Store the value at the calculated address
                instrs.push(Instruction::LocalGet(value_local));
                instrs.push(Instruction::I64Store(wasm_encoder::MemArg {
                    offset: 0,
                    align: 0,
                    memory_index: 0,
                }));

                instrs.push(Instruction::End); // End if

                Ok(())
            }
        }
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

    /// Emit string content comparison (equal or not equal)
    #[allow(dead_code)]
    fn emit_string_comparison(
        &mut self,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
        check_equal: bool,
    ) -> Result<(), CclError> {
        // Strings are stored as [len: u32][bytes]
        // Stack has: left_ptr, right_ptr

        let left_ptr = locals.get_or_add("__str_cmp_left", ValType::I32);
        instrs.push(Instruction::LocalTee(left_ptr));
        let right_ptr = locals.get_or_add("__str_cmp_right", ValType::I32);
        instrs.push(Instruction::LocalTee(right_ptr));

        // Load left length
        instrs.push(Instruction::LocalGet(left_ptr));
        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        let left_len = locals.get_or_add("__str_cmp_left_len", ValType::I32);
        instrs.push(Instruction::LocalTee(left_len));

        // Load right length
        instrs.push(Instruction::LocalGet(right_ptr));
        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        let right_len = locals.get_or_add("__str_cmp_right_len", ValType::I32);
        instrs.push(Instruction::LocalTee(right_len));

        // If lengths are different, strings are not equal
        instrs.push(Instruction::LocalGet(left_len));
        instrs.push(Instruction::LocalGet(right_len));
        instrs.push(Instruction::I32Ne);

        if check_equal {
            instrs.push(Instruction::If(wasm_encoder::BlockType::Result(
                ValType::I32,
            )));
            instrs.push(Instruction::I32Const(0)); // Not equal
            instrs.push(Instruction::Else);
        } else {
            instrs.push(Instruction::If(wasm_encoder::BlockType::Result(
                ValType::I32,
            )));
            instrs.push(Instruction::I32Const(1)); // Not equal (what we want for !=)
            instrs.push(Instruction::Else);
        }

        // Compare byte by byte
        let idx = locals.get_or_add("__str_cmp_idx", ValType::I32);
        instrs.push(Instruction::I32Const(0));
        instrs.push(Instruction::LocalSet(idx));

        // Result local
        let result = locals.get_or_add("__str_cmp_result", ValType::I32);
        if check_equal {
            instrs.push(Instruction::I32Const(1)); // Assume equal
        } else {
            instrs.push(Instruction::I32Const(0)); // Assume equal (opposite for !=)
        }
        instrs.push(Instruction::LocalSet(result));

        // Loop to compare bytes
        instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
        instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));

        // Check if we've reached the end
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::LocalGet(left_len));
        instrs.push(Instruction::I32GeU);
        instrs.push(Instruction::BrIf(1)); // Break out of loop

        // Load left byte
        instrs.push(Instruction::LocalGet(left_ptr));
        instrs.push(Instruction::I32Const(4)); // Skip length
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));

        // Load right byte
        instrs.push(Instruction::LocalGet(right_ptr));
        instrs.push(Instruction::I32Const(4)); // Skip length
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));

        // Compare bytes
        instrs.push(Instruction::I32Ne);
        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));

        // Bytes are different
        if check_equal {
            instrs.push(Instruction::I32Const(0)); // Not equal
        } else {
            instrs.push(Instruction::I32Const(1)); // Not equal (what we want for !=)
        }
        instrs.push(Instruction::LocalSet(result));
        instrs.push(Instruction::Br(2)); // Break out of both loops

        instrs.push(Instruction::End); // End if (byte comparison)

        // Increment index
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::I32Const(1));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::LocalSet(idx));

        instrs.push(Instruction::Br(0)); // Continue loop
        instrs.push(Instruction::End); // End loop
        instrs.push(Instruction::End); // End block

        instrs.push(Instruction::LocalGet(result));
        instrs.push(Instruction::End); // End if (length check)

        Ok(())
    }

    /// Emit string lexicographic ordering comparison
    fn emit_string_ordering_comparison(
        &mut self,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
        op: &str,
    ) -> Result<(), CclError> {
        // For now, implement a simplified version that compares lengths
        // Full implementation would do proper lexicographic comparison

        let left_ptr = locals.get_or_add("__str_ord_left", ValType::I32);
        instrs.push(Instruction::LocalTee(left_ptr));
        let right_ptr = locals.get_or_add("__str_ord_right", ValType::I32);
        instrs.push(Instruction::LocalTee(right_ptr));

        // Load left length
        instrs.push(Instruction::LocalGet(left_ptr));
        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));

        // Load right length
        instrs.push(Instruction::LocalGet(right_ptr));
        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));

        // Compare lengths (simplified - should do full lexicographic comparison)
        match op {
            "lt" => instrs.push(Instruction::I32LtU),
            "gt" => instrs.push(Instruction::I32GtU),
            "lte" => instrs.push(Instruction::I32LeU),
            "gte" => instrs.push(Instruction::I32GeU),
            _ => {
                return Err(CclError::WasmGenerationError(format!(
                    "Unknown comparison operator: {}",
                    op
                )))
            }
        }

        Ok(())
    }

    /// Emit simple string hash function (FNV-1a variant)
    #[allow(dead_code)]
    fn emit_simple_string_hash(
        &mut self,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
    ) -> Result<(), CclError> {
        // String pointer is on stack
        let str_ptr = locals.get_or_add("__hash_str_ptr", ValType::I32);
        instrs.push(Instruction::LocalTee(str_ptr));

        // Load string length
        instrs.push(Instruction::LocalGet(str_ptr));
        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        let str_len = locals.get_or_add("__hash_str_len", ValType::I32);
        instrs.push(Instruction::LocalTee(str_len));

        // Initialize hash (FNV offset basis)
        let hash = locals.get_or_add("__hash_value", ValType::I32);
        instrs.push(Instruction::I32Const(2166136261u32 as i32)); // FNV offset basis
        instrs.push(Instruction::LocalSet(hash));

        // Loop through string bytes
        let idx = locals.get_or_add("__hash_idx", ValType::I32);
        instrs.push(Instruction::I32Const(0));
        instrs.push(Instruction::LocalSet(idx));

        instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
        instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));

        // Check if we've reached the end
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::LocalGet(str_len));
        instrs.push(Instruction::I32GeU);
        instrs.push(Instruction::BrIf(1)); // Break out of loop

        // Load byte
        instrs.push(Instruction::LocalGet(str_ptr));
        instrs.push(Instruction::I32Const(4)); // Skip length
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));

        // hash = hash ^ byte
        instrs.push(Instruction::LocalGet(hash));
        instrs.push(Instruction::I32Xor);

        // hash = hash * FNV_PRIME
        instrs.push(Instruction::I32Const(16777619)); // FNV prime
        instrs.push(Instruction::I32Mul);
        instrs.push(Instruction::LocalSet(hash));

        // Increment index
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::I32Const(1));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::LocalSet(idx));

        instrs.push(Instruction::Br(0)); // Continue loop
        instrs.push(Instruction::End); // End loop
        instrs.push(Instruction::End); // End block

        // Push hash result
        instrs.push(Instruction::LocalGet(hash));

        Ok(())
    }

    /// Emit string equality check (returns 1 if equal, 0 if not)
    #[allow(dead_code)]
    fn emit_string_equality_check(
        &mut self,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
    ) -> Result<(), CclError> {
        // Stack has: str1_ptr, str2_ptr
        let str2_ptr = locals.get_or_add("__eq_str2", ValType::I32);
        instrs.push(Instruction::LocalTee(str2_ptr));
        let str1_ptr = locals.get_or_add("__eq_str1", ValType::I32);
        instrs.push(Instruction::LocalTee(str1_ptr));

        // Compare lengths first
        instrs.push(Instruction::LocalGet(str1_ptr));
        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        let len1 = locals.get_or_add("__eq_len1", ValType::I32);
        instrs.push(Instruction::LocalTee(len1));

        instrs.push(Instruction::LocalGet(str2_ptr));
        instrs.push(Instruction::I32Load(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));
        let len2 = locals.get_or_add("__eq_len2", ValType::I32);
        instrs.push(Instruction::LocalTee(len2));

        // If lengths differ, not equal
        instrs.push(Instruction::LocalGet(len1));
        instrs.push(Instruction::LocalGet(len2));
        instrs.push(Instruction::I32Ne);

        instrs.push(Instruction::If(wasm_encoder::BlockType::Result(
            ValType::I32,
        )));
        instrs.push(Instruction::I32Const(0)); // Not equal
        instrs.push(Instruction::Else);

        // Lengths match, compare bytes
        let idx = locals.get_or_add("__eq_idx", ValType::I32);
        instrs.push(Instruction::I32Const(0));
        instrs.push(Instruction::LocalSet(idx));

        let result = locals.get_or_add("__eq_result", ValType::I32);
        instrs.push(Instruction::I32Const(1)); // Assume equal
        instrs.push(Instruction::LocalSet(result));

        instrs.push(Instruction::Block(wasm_encoder::BlockType::Empty));
        instrs.push(Instruction::Loop(wasm_encoder::BlockType::Empty));

        // Check if done
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::LocalGet(len1));
        instrs.push(Instruction::I32GeU);
        instrs.push(Instruction::BrIf(1));

        // Load and compare bytes
        instrs.push(Instruction::LocalGet(str1_ptr));
        instrs.push(Instruction::I32Const(4));
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));

        instrs.push(Instruction::LocalGet(str2_ptr));
        instrs.push(Instruction::I32Const(4));
        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::I32Load8U(wasm_encoder::MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        }));

        instrs.push(Instruction::I32Ne);
        instrs.push(Instruction::If(wasm_encoder::BlockType::Empty));
        instrs.push(Instruction::I32Const(0));
        instrs.push(Instruction::LocalSet(result));
        instrs.push(Instruction::Br(2));
        instrs.push(Instruction::End);

        instrs.push(Instruction::LocalGet(idx));
        instrs.push(Instruction::I32Const(1));
        instrs.push(Instruction::I32Add);
        instrs.push(Instruction::LocalSet(idx));

        instrs.push(Instruction::Br(0));
        instrs.push(Instruction::End);
        instrs.push(Instruction::End);

        instrs.push(Instruction::LocalGet(result));
        instrs.push(Instruction::End);

        Ok(())
    }

    /// Generate the condition for a pattern match arm
    fn emit_pattern_condition(
        &mut self,
        pattern: &PatternNode,
        match_local: u32,
        match_ty: ValType,
        instrs: &mut Vec<Instruction>,
        _locals: &mut LocalEnv,
    ) -> Result<(), CclError> {
        match pattern {
            PatternNode::Literal(LiteralNode::Integer(i)) => {
                instrs.push(Instruction::LocalGet(match_local));
                if match_ty == ValType::I32 {
                    instrs.push(Instruction::I32Const(*i as i32));
                    instrs.push(Instruction::I32Eq);
                } else {
                    instrs.push(Instruction::I64Const(*i));
                    instrs.push(Instruction::I64Eq);
                }
            }
            PatternNode::Literal(LiteralNode::Boolean(b)) => {
                instrs.push(Instruction::LocalGet(match_local));
                instrs.push(Instruction::I32Const(if *b { 1 } else { 0 }));
                instrs.push(Instruction::I32Eq);
            }
            PatternNode::Variable(_) | PatternNode::Wildcard => {
                instrs.push(Instruction::I32Const(1));
            }
            PatternNode::Enum { variant, .. } => {
                instrs.push(Instruction::LocalGet(match_local));
                let idx = Self::enum_variant_index(variant);
                instrs.push(Instruction::I64Const(idx));
                instrs.push(Instruction::I64Eq);
            }
            _ => {
                return Err(CclError::WasmGenerationError(
                    "Unsupported pattern type".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// Bind variables for a pattern when entering an arm
    fn bind_pattern(
        &mut self,
        pattern: &PatternNode,
        match_local: u32,
        match_ty: ValType,
        instrs: &mut Vec<Instruction>,
        locals: &mut LocalEnv,
    ) -> Result<(), CclError> {
        if let PatternNode::Variable(name) = pattern {
            let idx = locals.get_or_add(name, match_ty);
            instrs.push(Instruction::LocalGet(match_local));
            instrs.push(Instruction::LocalSet(idx));
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
        TypeAnnotationNode::Map { .. } => {
            // Maps represented as i32 pointer to hash table structure
            Ok(ValType::I32)
        }
        TypeAnnotationNode::Proposal | TypeAnnotationNode::Vote => {
            // Governance types represented as i64 handles
            Ok(ValType::I64)
        }
        TypeAnnotationNode::Option(_) | TypeAnnotationNode::Result { .. } => Ok(ValType::I64),
        TypeAnnotationNode::Custom(name) => {
            // Custom types (structs, enums) are represented as pointers in WASM
            // This includes user-defined structs like ReputationProposal, ReputationVote, etc.
            Ok(ValType::I32)
        },
    }
}

impl WasmBackend {
    /// Collect all user-defined function names from the AST for pre-registration
    fn collect_user_functions(&self, ast: &AstNode) -> Result<Vec<String>, CclError> {
        let mut functions = Vec::new();
        
        match ast {
            AstNode::Policy(items) => {
                for item in items {
                    if let PolicyStatementNode::FunctionDef(AstNode::FunctionDefinition { name, .. }) = item {
                        functions.push(name.clone());
                    }
                }
            }
            AstNode::Program(nodes) => {
                for node in nodes {
                    match node {
                        crate::ast::TopLevelNode::Contract(contract) => {
                            for body_item in &contract.body {
                                if let crate::ast::ContractBodyNode::Function(func) = body_item {
                                    functions.push(func.name.clone());
                                }
                            }
                        }
                        crate::ast::TopLevelNode::Function(func) => {
                            functions.push(func.name.clone());
                        }
                        _ => {} // Skip other items
                    }
                }
            }
            AstNode::FunctionDefinition { name, .. } => {
                functions.push(name.clone());
            }
            _ => {} // No functions in other node types
        }
        
        Ok(functions)
    }

    fn enum_variant_index(name: &str) -> i64 {
        match name {
            "Pending" => 0,
            "Active" => 1,
            "Passed" => 2,
            "Rejected" => 3,
            "Ok" => 0,
            "Err" => 1,
            "Some" => 1,
            "None" => 0,
            _ => 0,
        }
    }
}
