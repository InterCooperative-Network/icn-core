// icn-ccl/src/cli.rs
use crate::ast::{
    AstNode, BlockNode, ExpressionNode, PolicyStatementNode, StatementNode, TypeAnnotationNode,
    UnaryOperator,
};
use crate::error::CclError;
use crate::metadata::ContractMetadata;
use crate::optimizer::Optimizer;
use crate::parser::parse_ccl_source;
use crate::semantic_analyzer::SemanticAnalyzer;
use crate::wasm_backend::WasmBackend;
use icn_common::{compute_merkle_cid, Did};
use log::info;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

// This function would be called by `icn-cli ccl compile ...`
pub fn compile_ccl_file(
    source_path: &PathBuf,
    output_wasm_path: &PathBuf,
    output_meta_path: &PathBuf,
) -> Result<ContractMetadata, CclError> {
    info!(
        "[CCL CLI Lib] Compiling {} to {} (meta: {})",
        source_path.display(),
        output_wasm_path.display(),
        output_meta_path.display()
    );

    let source_code = fs::read_to_string(source_path).map_err(|e| {
        CclError::IoError(format!(
            "Failed to read source file {}: {}",
            source_path.display(),
            e
        ))
    })?;

    let ast = parse_ccl_source(&source_code)?;

    let mut semantic_analyzer = SemanticAnalyzer::new();
    match semantic_analyzer.analyze(&ast) {
        Ok(()) => {}
        Err(errors) => {
            return Err(errors
                .into_iter()
                .next()
                .unwrap_or_else(|| CclError::SemanticError("Unknown semantic error".to_string())))
        }
    }

    let mut optimizer = Optimizer::new(crate::optimizer::OptimizationLevel::Basic);
    let optimized_ast = optimizer.optimize(ast); // AST might change to IR here

    let mut wasm_backend = WasmBackend::new();
    let (wasm_bytecode, mut metadata) = wasm_backend.compile_to_wasm(&optimized_ast)?;

    // Calculate CID of the generated WASM using icn_common utilities
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = compute_merkle_cid(0x71, &wasm_bytecode, &[], ts, &author, &sig_opt, &None);
    metadata.cid = cid.to_string();

    // Calculate SHA-256 hash of the source code
    let hash = Sha256::digest(source_code.as_bytes());
    metadata.source_hash = format!("sha256:{:x}", hash);

    fs::write(output_wasm_path, &wasm_bytecode).map_err(|e| {
        CclError::IoError(format!(
            "Failed to write WASM file {}: {}",
            output_wasm_path.display(),
            e
        ))
    })?;

    let metadata_json = serde_json::to_string_pretty(&metadata).map_err(|e| {
        CclError::InternalCompilerError(format!("Failed to serialize metadata: {}", e))
    })?;
    fs::write(output_meta_path, metadata_json).map_err(|e| {
        CclError::IoError(format!(
            "Failed to write metadata file {}: {}",
            output_meta_path.display(),
            e
        ))
    })?;

    info!(
        "[CCL CLI Lib] Compilation successful. WASM: {}, Meta: {}",
        output_wasm_path.display(),
        output_meta_path.display()
    );
    Ok(metadata)
}

// This function would be called by `icn-cli ccl lint ...` or `icn-cli ccl check ...`
pub fn check_ccl_file(source_path: &PathBuf) -> Result<(), CclError> {
    info!("[CCL CLI Lib] Checking/Linting {}", source_path.display());
    let source_code = fs::read_to_string(source_path)?;
    let ast = parse_ccl_source(&source_code)?;
    let mut semantic_analyzer = SemanticAnalyzer::new();
    match semantic_analyzer.analyze(&ast) {
        Ok(()) => {}
        Err(errors) => {
            return Err(errors
                .into_iter()
                .next()
                .unwrap_or_else(|| CclError::SemanticError("Unknown semantic error".to_string())))
        }
    }
    info!("[CCL CLI Lib] {} passed checks.", source_path.display());
    Ok(())
}

// This function would be called by `icn-cli ccl fmt ...`
pub fn format_ccl_file(source_path: &PathBuf, _inplace: bool) -> Result<String, CclError> {
    info!(
        "[CCL CLI Lib] Formatting {} (Inplace: {})",
        source_path.display(),
        _inplace
    );
    let source_code = fs::read_to_string(source_path)?;
    let ast = parse_ccl_source(&source_code)?;
    let formatted = ast_to_string(&ast, 0);
    if _inplace {
        fs::write(source_path, &formatted).map_err(|e| {
            CclError::IoError(format!(
                "Failed to write formatted file {}: {}",
                source_path.display(),
                e
            ))
        })?;
    }
    Ok(formatted)
}

// This function would be called by `icn-cli ccl explain ...`
pub fn explain_ccl_policy(
    source_path: &PathBuf,
    _target_construct: Option<String>,
) -> Result<String, CclError> {
    info!(
        "[CCL CLI Lib] Explaining {} (Target: {:?})",
        source_path.display(),
        _target_construct
    );
    let source_code = fs::read_to_string(source_path)?;
    let ast = parse_ccl_source(&source_code)?;
    let target = _target_construct.as_deref();
    let explanation = explain_ast(&ast, target);
    Ok(explanation)
}

fn ast_to_string(ast: &AstNode, indent: usize) -> String {
    match ast {
        AstNode::Policy(items) => items
            .iter()
            .map(|i| policy_stmt_to_string(i, indent))
            .collect::<Vec<_>>()
            .join("\n"),
        AstNode::FunctionDefinition {
            name,
            return_type,
            body,
            ..
        } => {
            let mut s = String::new();
            s.push_str(&format!(
                "fn {}() -> {} ",
                name,
                return_type
                    .as_ref()
                    .map(type_expr_to_string)
                    .unwrap_or_else(|| "()".to_string())
            ));
            s.push_str(&block_to_string(body, indent));
            s
        }
        // CCL 0.1 AST variants
        AstNode::Program(_items) => {
            // TODO: Implement TopLevelNode to string conversion
            "program".to_string()
        }
        AstNode::ContractDeclaration { name, .. } => format!("contract {}", name),
        AstNode::RoleDeclaration { name, .. } => format!("role {}", name),
        AstNode::ProposalDeclaration { name, .. } => format!("proposal {}", name),
        AstNode::ImportStatement { path, .. } => format!("import {}", path),
        AstNode::Block(b) => block_to_string(b, indent),
        AstNode::StructDefinition { name, .. } => format!("struct {}", name),
        AstNode::EnumDefinition { name, .. } => format!("enum {}", name),
        AstNode::StateDeclaration { name, .. } => format!("state {}", name),
        AstNode::ConstDeclaration { name, .. } => format!("const {}", name),
    }
}

fn policy_stmt_to_string(stmt: &PolicyStatementNode, indent: usize) -> String {
    match stmt {
        PolicyStatementNode::FunctionDef(n) | PolicyStatementNode::RuleDef(n) => {
            ast_to_string(n, indent)
        }
        PolicyStatementNode::Import { path, alias } => {
            format!("import \"{}\" as {};", path, alias)
        }
        PolicyStatementNode::StructDef(_) => "struct".to_string(),
        PolicyStatementNode::ConstDef { name, type_ann, .. } => {
            format!("const {}: {}", name, type_to_string(type_ann))
        }
        PolicyStatementNode::MacroDef { name, params, .. } => {
            format!("macro {}({})", name, params.join(", "))
        }
        // Placeholder implementations for governance DSL
        PolicyStatementNode::EventDef { name, .. } => {
            format!("event {}", name)
        }
        PolicyStatementNode::StateDef { name, .. } => {
            format!("state {}", name)
        }
        PolicyStatementNode::TriggerDef { name, .. } => {
            format!("trigger {}", name)
        }
    }
}

fn block_to_string(block: &BlockNode, indent: usize) -> String {
    let mut s = String::from("{\n");
    for st in &block.statements {
        s.push_str(&format!(
            "{}{}\n",
            " ".repeat(indent + 4),
            stmt_to_string(st, indent + 4)
        ));
    }
    s.push_str(&format!("{}{}", " ".repeat(indent), "}"));
    s
}

fn stmt_to_string(stmt: &StatementNode, indent: usize) -> String {
    match stmt {
        StatementNode::Let {
            mutable: _,
            name,
            type_expr: _,
            value,
        } => {
            format!("let {} = {};", name, expr_to_string(value))
        }
        StatementNode::ExpressionStatement(e) => {
            format!("{};", expr_to_string(e))
        }
        StatementNode::Return(e) => {
            if let Some(expr) = e {
                format!("return {};", expr_to_string(expr))
            } else {
                "return;".to_string()
            }
        }
        StatementNode::If {
            condition,
            then_block,
            else_ifs: _,
            else_block,
        } => {
            let mut s = String::new();
            s.push_str(&format!("if {} ", expr_to_string(condition)));
            s.push_str(&block_to_string(then_block, indent));
            if let Some(b) = else_block {
                s.push_str(" else ");
                s.push_str(&block_to_string(b, indent));
            }
            s
        }
        StatementNode::Assignment { lvalue: _, value } => {
            format!("assignment = {};", expr_to_string(value))
        }
        StatementNode::While { condition, body } => {
            let mut s = String::new();
            s.push_str(&format!("while {} ", expr_to_string(condition)));
            s.push_str(&block_to_string(body, indent));
            s
        }
        StatementNode::For {
            iterator,
            iterable,
            body,
        } => {
            let mut s = String::new();
            s.push_str(&format!(
                "for {} in {} ",
                iterator,
                expr_to_string(iterable)
            ));
            s.push_str(&block_to_string(body, indent));
            s
        }
        StatementNode::Match { expr, arms: _ } => {
            format!("match {} {{ ... }}", expr_to_string(expr))
        }
        StatementNode::Emit {
            event_name,
            fields: _,
        } => {
            format!("emit {} {{ ... }};", event_name)
        }
        StatementNode::Require(condition) => {
            format!("require({});", expr_to_string(condition))
        }
        StatementNode::Break => "break;".to_string(),
        StatementNode::Continue => "continue;".to_string(),

        // Legacy statements for backward compatibility
        StatementNode::WhileLoop { condition, body } => {
            let mut s = String::new();
            s.push_str(&format!("while {} ", expr_to_string(condition)));
            s.push_str(&block_to_string(body, indent));
            s
        }
        StatementNode::ForLoop {
            iterator,
            iterable,
            body,
        } => {
            let mut s = String::new();
            s.push_str(&format!(
                "for {} in {} ",
                iterator,
                expr_to_string(iterable)
            ));
            s.push_str(&block_to_string(body, indent));
            s
        }
    }
}

fn expr_to_string(expr: &ExpressionNode) -> String {
    match expr {
        // New unified literal handling
        ExpressionNode::Literal(lit) => match lit {
            crate::ast::LiteralNode::Integer(i) => i.to_string(),
            crate::ast::LiteralNode::Float(f) => f.to_string(),
            crate::ast::LiteralNode::String(s) => format!("\"{}\"", s),
            crate::ast::LiteralNode::Boolean(b) => b.to_string(),
            crate::ast::LiteralNode::Did(did) => did.clone(),
            crate::ast::LiteralNode::Timestamp(ts) => ts.clone(),
        },

        // Legacy literal variants (for backward compatibility)
        ExpressionNode::IntegerLiteral(i) => i.to_string(),
        ExpressionNode::BooleanLiteral(b) => b.to_string(),
        ExpressionNode::StringLiteral(s) => format!("\"{}\"", s),

        ExpressionNode::ArrayLiteral(elements) => {
            let items = elements
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", items)
        }
        ExpressionNode::Identifier(s) => s.clone(),
        ExpressionNode::FunctionCall { name, args } => {
            let arg_strs = args
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, arg_strs)
        }
        ExpressionNode::MethodCall {
            object,
            method,
            args,
        } => {
            let arg_strs = args
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}.{}({})", expr_to_string(object), method, arg_strs)
        }
        ExpressionNode::BinaryOp {
            left,
            operator,
            right,
        } => {
            let op = match operator {
                crate::ast::BinaryOperator::Add => "+",
                crate::ast::BinaryOperator::Sub => "-",
                crate::ast::BinaryOperator::Mul => "*",
                crate::ast::BinaryOperator::Div => "/",
                crate::ast::BinaryOperator::Mod => "%",
                crate::ast::BinaryOperator::Eq => "==",
                crate::ast::BinaryOperator::Neq => "!=",
                crate::ast::BinaryOperator::Lt => "<",
                crate::ast::BinaryOperator::Gt => ">",
                crate::ast::BinaryOperator::Lte => "<=",
                crate::ast::BinaryOperator::Gte => ">=",
                crate::ast::BinaryOperator::And => "&&",
                crate::ast::BinaryOperator::Or => "||",
                crate::ast::BinaryOperator::Concat => "++",
            };
            format!(
                "({} {} {})",
                expr_to_string(left),
                op,
                expr_to_string(right)
            )
        }
        ExpressionNode::UnaryOp { operator, operand } => {
            let op = match operator {
                UnaryOperator::Not => "!",
                UnaryOperator::Neg => "-",
            };
            format!("({}{})", op, expr_to_string(operand))
        }
        ExpressionNode::ArrayAccess { array, index } => {
            format!("{}[{}]", expr_to_string(array), expr_to_string(index))
        }

        // New AST variants
        ExpressionNode::MemberAccess { object, member } => {
            format!("{}.{}", expr_to_string(object), member)
        }
        ExpressionNode::IndexAccess { object, index } => {
            format!("{}[{}]", expr_to_string(object), expr_to_string(index))
        }
        ExpressionNode::StructLiteral {
            type_name,
            fields: _,
        } => {
            format!("{} {{ ... }}", type_name)
        }

        // Governance expressions
        ExpressionNode::Transfer { from, to, amount } => {
            format!(
                "transfer({}, {}, {})",
                expr_to_string(from),
                expr_to_string(to),
                expr_to_string(amount)
            )
        }
        ExpressionNode::Mint { to, amount } => {
            format!("mint({}, {})", expr_to_string(to), expr_to_string(amount))
        }
        ExpressionNode::Burn { from, amount } => {
            format!("burn({}, {})", expr_to_string(from), expr_to_string(amount))
        }

        ExpressionNode::Some(inner) => format!("Some({})", expr_to_string(inner)),
        ExpressionNode::None => "None".to_string(),
        ExpressionNode::Ok(inner) => format!("Ok({})", expr_to_string(inner)),
        ExpressionNode::Err(inner) => format!("Err({})", expr_to_string(inner)),
        ExpressionNode::MapLiteral(pairs) => {
            let items = pairs
                .iter()
                .map(|(k, v)| format!("{}: {}", expr_to_string(k), expr_to_string(v)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{{}}}", items)
        }
        ExpressionNode::EnumValue { enum_name, variant } => format!("{}::{}", enum_name, variant),
        ExpressionNode::Match { .. } => "match { ... }".to_string(), // Placeholder for pattern matching expressions
    }
}

fn type_expr_to_string(ty: &crate::ast::TypeExprNode) -> String {
    use crate::ast::TypeExprNode;
    match ty {
        TypeExprNode::Integer => "Integer".to_string(),
        TypeExprNode::String => "String".to_string(),
        TypeExprNode::Boolean => "Boolean".to_string(),
        TypeExprNode::Mana => "Mana".to_string(),
        TypeExprNode::Did => "Did".to_string(),
        TypeExprNode::Timestamp => "Timestamp".to_string(),
        TypeExprNode::Duration => "Duration".to_string(),
        TypeExprNode::Custom(name) => name.clone(),
        TypeExprNode::Array(inner) => format!("[{}]", type_expr_to_string(inner)),
        TypeExprNode::Map {
            key_type,
            value_type,
        } => {
            format!(
                "Map<{}, {}>",
                type_expr_to_string(key_type),
                type_expr_to_string(value_type)
            )
        }
        TypeExprNode::Option(inner) => format!("Option<{}>", type_expr_to_string(inner)),
        TypeExprNode::Result { ok_type, err_type } => {
            format!(
                "Result<{}, {}>",
                type_expr_to_string(ok_type),
                type_expr_to_string(err_type)
            )
        }
        TypeExprNode::TypeParameter(name) => name.clone(),
        TypeExprNode::GenericInstantiation {
            base_type,
            type_args,
        } => {
            let args = type_args
                .iter()
                .map(type_expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}<{}>", base_type, args)
        }
    }
}

fn type_to_string(ty: &TypeAnnotationNode) -> String {
    match ty {
        TypeAnnotationNode::Mana => "Mana".to_string(),
        TypeAnnotationNode::Bool => "Bool".to_string(),
        TypeAnnotationNode::Did => "Did".to_string(),
        TypeAnnotationNode::String => "String".to_string(),
        TypeAnnotationNode::Integer => "Integer".to_string(),
        TypeAnnotationNode::Array(inner_ty) => format!("Array<{}>", type_to_string(inner_ty)),
        TypeAnnotationNode::Map {
            key_type,
            value_type,
        } => {
            format!(
                "Map<{}, {}>",
                type_to_string(key_type),
                type_to_string(value_type)
            )
        }
        TypeAnnotationNode::Proposal => "Proposal".to_string(),
        TypeAnnotationNode::Vote => "Vote".to_string(),
        TypeAnnotationNode::Custom(s) => s.clone(),
        TypeAnnotationNode::Option(_) => "Option".to_string(),
        TypeAnnotationNode::Result { .. } => "Result".to_string(),
    }
}

fn explain_ast(ast: &AstNode, target: Option<&str>) -> String {
    match ast {
        AstNode::Policy(items) => {
            let mut lines = Vec::new();
            for item in items {
                match item {
                    PolicyStatementNode::FunctionDef(inner) => {
                        if let AstNode::FunctionDefinition {
                            name, return_type, ..
                        } = inner
                        {
                            if target.is_none() || target == Some(name) {
                                lines.push(format!(
                                    "Function `{}` returns `{}`.",
                                    name,
                                    return_type
                                        .as_ref()
                                        .map(type_expr_to_string)
                                        .unwrap_or_else(|| "void".to_string())
                                ));
                            }
                        }
                    }
                    PolicyStatementNode::RuleDef(_) => {
                        // Legacy RuleDefinition removed in CCL 0.1
                        lines.push(
                            "Legacy rule definition found - no longer supported in CCL 0.1"
                                .to_string(),
                        );
                    }
                    PolicyStatementNode::Import { path, alias } => {
                        if target.is_none() {
                            lines.push(format!("Imports `{}` as `{}`.", path, alias));
                        }
                    }
                    PolicyStatementNode::StructDef(_) => {
                        if target.is_none() {
                            lines.push("Struct definition".to_string());
                        }
                    }
                    PolicyStatementNode::ConstDef { name, type_ann, .. } => {
                        if target.is_none() || target == Some(name) {
                            lines.push(format!(
                                "Constant `{}` of type `{}`.",
                                name,
                                type_to_string(type_ann)
                            ));
                        }
                    }
                    PolicyStatementNode::MacroDef { name, params, .. } => {
                        if target.is_none() || target == Some(name) {
                            lines.push(format!(
                                "Macro `{}` with {} parameters.",
                                name,
                                params.len()
                            ));
                        }
                    }
                    // Placeholder implementations for governance DSL
                    PolicyStatementNode::EventDef { name, .. } => {
                        if target.is_none() || target == Some(name) {
                            lines.push(format!("Event `{}` definition.", name));
                        }
                    }
                    PolicyStatementNode::StateDef { name, .. } => {
                        if target.is_none() || target == Some(name) {
                            lines.push(format!("State variable `{}`.", name));
                        }
                    }
                    PolicyStatementNode::TriggerDef { name, .. } => {
                        if target.is_none() || target == Some(name) {
                            lines.push(format!("Trigger `{}` definition.", name));
                        }
                    }
                }
            }
            lines.join("\n")
        }
        node => {
            if target.is_none() {
                ast_to_string(node, 0)
            } else {
                String::new()
            }
        }
    }
}
