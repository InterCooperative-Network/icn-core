// icn-ccl/src/cli.rs
use crate::ast::{
    ActionNode, AstNode, BlockNode, ExpressionNode, PolicyStatementNode, StatementNode,
    TypeAnnotationNode,
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
    semantic_analyzer.analyze(&ast)?;

    let optimizer = Optimizer::new();
    let optimized_ast = optimizer.optimize(ast)?; // AST might change to IR here

    let wasm_backend = WasmBackend::new();
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
    semantic_analyzer.analyze(&ast)?;
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
                type_to_string(return_type)
            ));
            s.push_str(&block_to_string(body, indent));
            s
        }
        AstNode::RuleDefinition {
            name,
            condition,
            action,
        } => {
            format!(
                "rule {} when {} then {}",
                name,
                expr_to_string(condition),
                action_to_string(action)
            )
        }
        AstNode::Block(b) => block_to_string(b, indent),
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
        StatementNode::Let { name, value } => {
            format!("let {} = {};", name, expr_to_string(value))
        }
        StatementNode::ExpressionStatement(e) => {
            format!("{};", expr_to_string(e))
        }
        StatementNode::Return(e) => {
            format!("return {};", expr_to_string(e))
        }
        StatementNode::If {
            condition,
            then_block,
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
        StatementNode::WhileLoop { condition, body } => {
            let mut s = String::new();
            s.push_str(&format!("while {} ", expr_to_string(condition)));
            s.push_str(&block_to_string(body, indent));
            s
        }
    }
}

fn expr_to_string(expr: &ExpressionNode) -> String {
    match expr {
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
        ExpressionNode::FunctionCall { name, arguments } => {
            let args = arguments
                .iter()
                .map(expr_to_string)
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}({})", name, args)
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
                crate::ast::UnaryOperator::Not => "!",
                crate::ast::UnaryOperator::Neg => "-",
            };
            format!("({}{})", op, expr_to_string(operand))
        }
        ExpressionNode::ArrayAccess { array, index } => {
            format!("{}[{}]", expr_to_string(array), expr_to_string(index))
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
        TypeAnnotationNode::Proposal => "Proposal".to_string(),
        TypeAnnotationNode::Vote => "Vote".to_string(),
        TypeAnnotationNode::Custom(s) => s.clone(),
    }
}

fn action_to_string(action: &ActionNode) -> String {
    match action {
        ActionNode::Allow => "allow".to_string(),
        ActionNode::Deny => "deny".to_string(),
        ActionNode::Charge(e) => format!("charge {}", expr_to_string(e)),
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
                                    type_to_string(return_type)
                                ));
                            }
                        }
                    }
                    PolicyStatementNode::RuleDef(inner) => {
                        if let AstNode::RuleDefinition {
                            name,
                            condition,
                            action,
                        } = inner
                        {
                            if target.is_none() || target == Some(name) {
                                lines.push(format!(
                                    "Rule `{}` when {} then {}.",
                                    name,
                                    expr_to_string(condition),
                                    action_to_string(action)
                                ));
                            }
                        }
                    }
                    PolicyStatementNode::Import { path, alias } => {
                        if target.is_none() {
                            lines.push(format!("Imports `{}` as `{}`.", path, alias));
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
