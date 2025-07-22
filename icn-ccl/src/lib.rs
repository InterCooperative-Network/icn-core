// icn-ccl/src/lib.rs
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::new_without_default)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::len_zero)]

pub mod ast;
pub mod error;
// pub mod grammar; // If ccl.pest is directly included or re-exported - REMOVING THIS
pub mod cli;
pub mod governance_std;
pub mod metadata;
pub mod optimizer;
pub mod parser;
pub mod semantic_analyzer;
pub mod stdlib;
pub mod wasm_backend; // Expose functions for CLI layer

pub use error::CclError;
pub use metadata::ContractMetadata;
pub use stdlib::StdLibrary as StandardLibrary;

/// Compiles a CCL source string into WASM bytecode and metadata.
pub fn compile_ccl_source_to_wasm(source: &str) -> Result<(Vec<u8>, ContractMetadata), CclError> {
    use icn_common::{compute_merkle_cid, Did};
    use sha2::{Digest, Sha256};

    let mut ast_node = parser::parse_ccl_source(source)?;
    ast_node = expand_macros(ast_node, &StandardLibrary::new())?;

    let mut semantic_analyzer = semantic_analyzer::SemanticAnalyzer::new();
    match semantic_analyzer.analyze(&ast_node) {
        Ok(()) => {}
        Err(errors) => {
            return Err(errors
                .into_iter()
                .next()
                .unwrap_or_else(|| CclError::SemanticError("Unknown semantic error".to_string())))
        }
    }

    let mut optimizer = optimizer::Optimizer::new(optimizer::OptimizationLevel::Basic);
    let optimized_ast = optimizer.optimize(ast_node);

    let mut backend = wasm_backend::WasmBackend::new();
    let (wasm, mut meta) = backend.compile_to_wasm(&optimized_ast)?;

    // Compute the CID for the generated WASM so executors can fetch it via the
    // runtime DAG APIs. This mirrors the behavior in the CLI helper.
    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    meta.cid = cid.to_string();

    // Hash the source code for auditing purposes
    let digest = Sha256::digest(source.as_bytes());
    meta.source_hash = format!("sha256:{:x}", digest);

    Ok((wasm, meta))
}

/// Reads a CCL source file from disk and compiles it to WASM bytecode and metadata.
///
/// This is similar to [`cli::compile_ccl_file`] but returns the generated WASM
/// bytes directly instead of writing them to disk. It also fills out the
/// [`ContractMetadata`] with a placeholder CID and the SHA-256 hash of the
/// source for auditing purposes.
pub fn compile_ccl_file_to_wasm(
    path: &std::path::Path,
) -> Result<(Vec<u8>, ContractMetadata), CclError> {
    use icn_common::{compute_merkle_cid, Did};
    use sha2::{Digest, Sha256};

    let ast_node = parser::parse_ccl_file(path)?;

    let mut semantic_analyzer = semantic_analyzer::SemanticAnalyzer::new();
    match semantic_analyzer.analyze(&ast_node) {
        Ok(()) => {}
        Err(errors) => {
            return Err(errors
                .into_iter()
                .next()
                .unwrap_or_else(|| CclError::SemanticError("Unknown semantic error".to_string())))
        }
    }

    let mut optimizer = optimizer::Optimizer::new(optimizer::OptimizationLevel::Basic);
    let optimized_ast = optimizer.optimize(ast_node);

    let mut backend = wasm_backend::WasmBackend::new();
    let (wasm, mut meta) = backend.compile_to_wasm(&optimized_ast)?;

    let ts = 0u64;
    let author = Did::new("key", "tester");
    let sig_opt = None;
    let cid = compute_merkle_cid(0x71, &wasm, &[], ts, &author, &sig_opt, &None);
    meta.cid = cid.to_string();

    let source_code = std::fs::read_to_string(path)?;
    let digest = Sha256::digest(source_code.as_bytes());
    meta.source_hash = format!("sha256:{:x}", digest);

    Ok((wasm, meta))
}

// Re-export CLI helper functions for easier access by icn-cli
pub use cli::{check_ccl_file, compile_ccl_file, explain_ccl_policy, format_ccl_file};

/// Expand macro definitions in the given AST using the standard library.
fn expand_macros(ast: ast::AstNode, stdlib: &StandardLibrary) -> Result<ast::AstNode, CclError> {
    use ast::{AstNode, PolicyStatementNode, ExpressionNode, StatementNode};

    if let AstNode::Policy(stmts) = ast {
        let mut expanded = Vec::new();
        let mut local_stdlib = stdlib.clone(); // Clone so we can register new macros

        // First pass: collect macro definitions and register them
        for stmt in &stmts {
            if let PolicyStatementNode::MacroDef {
                name,
                params,
                body,
            } = stmt
            {
                // Convert body statements to appropriate type
                let body_stmts = body.statements.clone();
                local_stdlib.register_macro(name.clone(), params.clone(), body_stmts);
            }
        }

        // Second pass: process statements and expand macro calls
        for stmt in stmts {
            match stmt {
                PolicyStatementNode::MacroDef { .. } => {
                    // Keep macro definitions in the output for potential runtime use
                    expanded.push(stmt);
                }
                other => {
                    // Recursively expand macros in this statement
                    let expanded_stmt = expand_macros_in_statement(other, &local_stdlib)?;
                    expanded.push(expanded_stmt);
                }
            }
        }
        Ok(AstNode::Policy(expanded))
    } else {
        // For non-Policy nodes, recursively expand macros
        expand_macros_in_ast_node(ast, stdlib)
    }
}

/// Expand macros within a single statement
fn expand_macros_in_statement(stmt: PolicyStatementNode, stdlib: &StandardLibrary) -> Result<PolicyStatementNode, CclError> {
    use ast::{PolicyStatementNode, ExpressionNode};
    
    match stmt {
        PolicyStatementNode::FunctionDefinition { name, params, return_type, body } => {
            let expanded_body = expand_macros_in_block(body, stdlib)?;
            Ok(PolicyStatementNode::FunctionDefinition {
                name,
                params,
                return_type,
                body: expanded_body,
            })
        }
        PolicyStatementNode::StructDefinition { .. } => Ok(stmt), // No macro expansion in struct definitions
        PolicyStatementNode::EnumDefinition { .. } => Ok(stmt),   // No macro expansion in enum definitions
        PolicyStatementNode::ConstDeclaration { name, value, type_annotation } => {
            let expanded_value = expand_macros_in_expression(value, stdlib)?;
            Ok(PolicyStatementNode::ConstDeclaration {
                name,
                value: expanded_value,
                type_annotation,
            })
        }
        PolicyStatementNode::MacroDef { .. } => Ok(stmt), // Macro definitions don't need expansion
        other => Ok(other), // Other statement types don't currently support macro expansion
    }
}

/// Expand macros within a block
fn expand_macros_in_block(block: ast::BlockNode, stdlib: &StandardLibrary) -> Result<ast::BlockNode, CclError> {
    let mut expanded_statements = Vec::new();
    
    for stmt in block.statements {
        let expanded_stmt = expand_macros_in_statement_node(stmt, stdlib)?;
        expanded_statements.push(expanded_stmt);
    }
    
    Ok(ast::BlockNode {
        statements: expanded_statements,
    })
}

/// Expand macros within a statement node
fn expand_macros_in_statement_node(stmt: ast::StatementNode, stdlib: &StandardLibrary) -> Result<ast::StatementNode, CclError> {
    use ast::{StatementNode, ExpressionNode};
    
    match stmt {
        StatementNode::Let { name, value, type_expr, mutable } => {
            let expanded_value = expand_macros_in_expression(value, stdlib)?;
            Ok(StatementNode::Let {
                name,
                value: expanded_value,
                type_expr,
                mutable,
            })
        }
        StatementNode::Assignment { lvalue, value } => {
            let expanded_value = expand_macros_in_expression(value, stdlib)?;
            Ok(StatementNode::Assignment {
                lvalue,
                value: expanded_value,
            })
        }
        StatementNode::Return(expr_opt) => {
            let expanded_expr = if let Some(expr) = expr_opt {
                Some(expand_macros_in_expression(expr, stdlib)?)
            } else { None };
            Ok(StatementNode::Return(expanded_expr))
        }
        StatementNode::If { condition, then_block, else_ifs, else_block } => {
            let expanded_condition = expand_macros_in_expression(condition, stdlib)?;
            let expanded_then = expand_macros_in_block(then_block, stdlib)?;
            let mut expanded_else_ifs = Vec::new();
            for (cond, block) in else_ifs {
                expanded_else_ifs.push((
                    expand_macros_in_expression(cond, stdlib)?,
                    expand_macros_in_block(block, stdlib)?
                ));
            }
            let expanded_else = if let Some(else_blk) = else_block {
                Some(expand_macros_in_block(else_blk, stdlib)?)
            } else { None };
            Ok(StatementNode::If {
                condition: expanded_condition,
                then_block: expanded_then,
                else_ifs: expanded_else_ifs,
                else_block: expanded_else,
            })
        }
        other => Ok(other), // Other statement types don't need macro expansion yet
    }
}

/// Expand macros within an expression
fn expand_macros_in_expression(expr: ExpressionNode, stdlib: &StandardLibrary) -> Result<ExpressionNode, CclError> {
    use ast::{ExpressionNode, LiteralNode};
    
    match expr {
        ExpressionNode::FunctionCall { name, args } => {
            // Check if this is a macro call
            if let Some(macro_def) = stdlib.get_macro(&name) {
                // Expand macro call by substituting parameters
                return expand_macro_call(macro_def, args, stdlib);
            }
            
            // Regular function call - expand arguments
            let mut expanded_args = Vec::new();
            for arg in args {
                expanded_args.push(expand_macros_in_expression(arg, stdlib)?);
            }
            Ok(ExpressionNode::FunctionCall {
                name,
                args: expanded_args,
            })
        }
        ExpressionNode::BinaryOp { left, operator, right } => {
            let expanded_left = Box::new(expand_macros_in_expression(*left, stdlib)?);
            let expanded_right = Box::new(expand_macros_in_expression(*right, stdlib)?);
            Ok(ExpressionNode::BinaryOp {
                left: expanded_left,
                operator,
                right: expanded_right,
            })
        }
        ExpressionNode::UnaryOp { operator, operand } => {
            let expanded_operand = Box::new(expand_macros_in_expression(*operand, stdlib)?);
            Ok(ExpressionNode::UnaryOp {
                operator,
                operand: expanded_operand,
            })
        }
        ExpressionNode::ArrayLiteral(elements) => {
            let mut expanded_elements = Vec::new();
            for elem in elements {
                expanded_elements.push(expand_macros_in_expression(elem, stdlib)?);
            }
            Ok(ExpressionNode::ArrayLiteral(expanded_elements))
        }
        ExpressionNode::MethodCall { object, method, args } => {
            let expanded_object = Box::new(expand_macros_in_expression(*object, stdlib)?);
            let mut expanded_args = Vec::new();
            for arg in args {
                expanded_args.push(expand_macros_in_expression(arg, stdlib)?);
            }
            Ok(ExpressionNode::MethodCall {
                object: expanded_object,
                method,
                args: expanded_args,
            })
        }
        ExpressionNode::MemberAccess { object, member } => {
            let expanded_object = Box::new(expand_macros_in_expression(*object, stdlib)?);
            Ok(ExpressionNode::MemberAccess {
                object: expanded_object,
                member,
            })
        }
        ExpressionNode::IndexAccess { object, index } => {
            let expanded_object = Box::new(expand_macros_in_expression(*object, stdlib)?);
            let expanded_index = Box::new(expand_macros_in_expression(*index, stdlib)?);
            Ok(ExpressionNode::IndexAccess {
                object: expanded_object,
                index: expanded_index,
            })
        }
        ExpressionNode::ArrayAccess { array, index } => {
            let expanded_array = Box::new(expand_macros_in_expression(*array, stdlib)?);
            let expanded_index = Box::new(expand_macros_in_expression(*index, stdlib)?);
            Ok(ExpressionNode::ArrayAccess {
                array: expanded_array,
                index: expanded_index,
            })
        }
        ExpressionNode::Match { expr, arms } => {
            let expanded_expr = Box::new(expand_macros_in_expression(*expr, stdlib)?);
            let mut expanded_arms = Vec::new();
            for arm in arms {
                // Expand the body expression of each match arm
                let expanded_body = expand_macros_in_expression(arm.body, stdlib)?;
                expanded_arms.push(ast::MatchArmNode {
                    pattern: arm.pattern,
                    guard: if let Some(guard) = arm.guard {
                        Some(expand_macros_in_expression(guard, stdlib)?)
                    } else { None },
                    body: expanded_body,
                });
            }
            Ok(ExpressionNode::Match {
                expr: expanded_expr,
                arms: expanded_arms,
            })
        }
        // Other expression types don't need macro expansion
        other => Ok(other),
    }
}

/// Expand a macro call by substituting parameters
fn expand_macro_call(macro_def: &stdlib::MacroDefinition, args: Vec<ExpressionNode>, _stdlib: &StandardLibrary) -> Result<ExpressionNode, CclError> {
    // For now, implement basic parameter substitution
    // In a full implementation, this would create a proper substitution context
    // and recursively expand the macro body
    
    if args.len() != macro_def.params.len() {
        return Err(CclError::SemanticError(format!(
            "Macro '{}' expects {} arguments, got {}",
            macro_def.name, macro_def.params.len(), args.len()
        )));
    }
    
    // For this MVP implementation, we'll return a function call that represents the expanded macro
    // A more sophisticated implementation would perform proper parameter substitution
    // and inline the macro body
    Ok(ExpressionNode::FunctionCall {
        name: format!("__expanded_macro_{}", macro_def.name),
        args,
    })
}

/// Expand macros in other AST node types
fn expand_macros_in_ast_node(ast: ast::AstNode, stdlib: &StandardLibrary) -> Result<ast::AstNode, CclError> {
    // For now, just return the AST as-is
    // This can be extended to handle other node types as needed
    Ok(ast)
}
