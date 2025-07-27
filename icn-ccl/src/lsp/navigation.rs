// icn-ccl/src/lsp/navigation.rs
//! Navigation features for CCL LSP (go-to-definition, find references)

use tower_lsp::lsp_types::*;
use super::server::DocumentState;
use crate::ast::{AstNode, TopLevelNode, FunctionDefinitionNode, ExpressionNode, StatementNode, ContractBodyNode};

/// Symbol information for navigation
#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    symbol_type: SymbolType,
    location: Range,
    definition_location: Option<Range>,
}

#[derive(Debug, Clone)]
enum SymbolType {
    Function,
    Variable,
    Type,
    Contract,
    Field,
}

/// Provide go-to-definition functionality
pub fn goto_definition(doc_state: &DocumentState, position: Position) -> Option<Location> {
    // Get the word at the cursor position
    let word = get_word_at_position(&doc_state.text, position)?;
    
    // Find the symbol definition in the AST
    if let Some(ast) = &doc_state.ast {
        if let Some(definition_range) = find_symbol_definition(ast, &word, position) {
            return Some(Location {
                uri: doc_state.uri.clone(),
                range: definition_range,
            });
        }
    }
    
    None
}

/// Find all references to a symbol
pub fn find_references(doc_state: &DocumentState, position: Position) -> Vec<Location> {
    // Get the word at the cursor position
    if let Some(word) = get_word_at_position(&doc_state.text, position) {
        // Find all references to this symbol in the AST
        if let Some(ast) = &doc_state.ast {
            let ranges = find_symbol_references(ast, &word);
            return ranges.into_iter()
                .map(|range| Location {
                    uri: doc_state.uri.clone(),
                    range,
                })
                .collect();
        }
    }
    
    Vec::new()
}

/// Find the definition of a symbol in the AST
fn find_symbol_definition(ast: &AstNode, symbol_name: &str, _cursor_position: Position) -> Option<Range> {
    match ast {
        AstNode::Program(top_level_nodes) => {
            for node in top_level_nodes {
                if let Some(range) = find_definition_in_top_level(node, symbol_name) {
                    return Some(range);
                }
            }
        }
        AstNode::ContractDeclaration { name, body, .. } => {
            // Check if we're looking for the contract itself
            if name == symbol_name {
                return Some(create_range(0, 0, 0, name.len()));
            }
            
            // Look inside contract body
            for body_node in body {
                if let Some(range) = find_definition_in_contract_body(body_node, symbol_name) {
                    return Some(range);
                }
            }
        }
        _ => {}
    }
    
    None
}

/// Find definition in top-level nodes
fn find_definition_in_top_level(node: &TopLevelNode, symbol_name: &str) -> Option<Range> {
    match node {
        TopLevelNode::Contract(contract) => {
            if contract.name == symbol_name {
                return Some(create_range(0, 0, 0, contract.name.len()));
            }
            
            // Look inside contract body
            for body_node in &contract.body {
                if let Some(range) = find_definition_in_contract_body(body_node, symbol_name) {
                    return Some(range);
                }
            }
        }
        TopLevelNode::Function(func) => {
            if func.name == symbol_name {
                return Some(create_range(0, 0, 0, func.name.len()));
            }
        }
        TopLevelNode::Struct(struct_def) => {
            if struct_def.name == symbol_name {
                return Some(create_range(0, 0, 0, struct_def.name.len()));
            }
        }
        _ => {}
    }
    
    None
}

/// Find definition in contract body nodes
fn find_definition_in_contract_body(node: &ContractBodyNode, symbol_name: &str) -> Option<Range> {
    match node {
        ContractBodyNode::Function(func) => {
            if func.name == symbol_name {
                return Some(create_range(0, 0, 0, func.name.len()));
            }
        }
        ContractBodyNode::State(state) => {
            if state.name == symbol_name {
                return Some(create_range(0, 0, 0, state.name.len()));
            }
        }
        ContractBodyNode::Const(const_decl) => {
            if const_decl.name == symbol_name {
                return Some(create_range(0, 0, 0, const_decl.name.len()));
            }
        }
        _ => {}
    }
    
    None
}

/// Find all references to a symbol in the AST
fn find_symbol_references(ast: &AstNode, symbol_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    
    match ast {
        AstNode::Program(top_level_nodes) => {
            for node in top_level_nodes {
                references.extend(find_references_in_top_level(node, symbol_name));
            }
        }
        AstNode::ContractDeclaration { body, .. } => {
            for body_node in body {
                references.extend(find_references_in_contract_body(body_node, symbol_name));
            }
        }
        _ => {}
    }
    
    references
}

/// Find references in top-level nodes
fn find_references_in_top_level(node: &TopLevelNode, symbol_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    
    match node {
        TopLevelNode::Contract(contract) => {
            for body_node in &contract.body {
                references.extend(find_references_in_contract_body(body_node, symbol_name));
            }
        }
        TopLevelNode::Function(func) => {
            references.extend(find_references_in_function(func, symbol_name));
        }
        _ => {}
    }
    
    references
}

/// Find references in contract body nodes
fn find_references_in_contract_body(node: &ContractBodyNode, symbol_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    
    match node {
        ContractBodyNode::Function(func) => {
            references.extend(find_references_in_function(func, symbol_name));
        }
        ContractBodyNode::State(state) => {
            if let Some(initial_value) = &state.initial_value {
                references.extend(find_references_in_expression(initial_value, symbol_name));
            }
        }
        ContractBodyNode::Const(const_decl) => {
            references.extend(find_references_in_expression(&const_decl.value, symbol_name));
        }
        _ => {}
    }
    
    references
}

/// Find references in function definition
fn find_references_in_function(func: &FunctionDefinitionNode, symbol_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    
    // Check function body
    references.extend(find_references_in_block(&func.body, symbol_name));
    
    references
}

/// Find references in a block of statements
fn find_references_in_block(block: &crate::ast::BlockNode, symbol_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    
    for statement in &block.statements {
        references.extend(find_references_in_statement(statement, symbol_name));
    }
    
    references
}

/// Find references in a statement
fn find_references_in_statement(statement: &StatementNode, symbol_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    
    match statement {
        StatementNode::ExpressionStatement(expr) => {
            references.extend(find_references_in_expression(expr, symbol_name));
        }
        StatementNode::Assignment { value, .. } => {
            references.extend(find_references_in_expression(value, symbol_name));
        }
        StatementNode::If { condition, then_block, else_ifs, else_block } => {
            references.extend(find_references_in_expression(condition, symbol_name));
            references.extend(find_references_in_block(then_block, symbol_name));
            for (else_if_condition, else_if_block) in else_ifs {
                references.extend(find_references_in_expression(else_if_condition, symbol_name));
                references.extend(find_references_in_block(else_if_block, symbol_name));
            }
            if let Some(else_block) = else_block {
                references.extend(find_references_in_block(else_block, symbol_name));
            }
        }
        StatementNode::While { condition, body } => {
            references.extend(find_references_in_expression(condition, symbol_name));
            references.extend(find_references_in_block(body, symbol_name));
        }
        StatementNode::Return(expr) => {
            if let Some(expr) = expr {
                references.extend(find_references_in_expression(expr, symbol_name));
            }
        }
        _ => {}
    }
    
    references
}

/// Find references in an expression
fn find_references_in_expression(expr: &ExpressionNode, symbol_name: &str) -> Vec<Range> {
    let mut references = Vec::new();
    
    match expr {
        ExpressionNode::Identifier(name) => {
            if name == symbol_name {
                // Create a placeholder range - in a real implementation, 
                // you'd track source positions through the parser
                references.push(create_range(0, 0, 0, name.len()));
            }
        }
        ExpressionNode::BinaryOp { left, right, .. } => {
            references.extend(find_references_in_expression(left, symbol_name));
            references.extend(find_references_in_expression(right, symbol_name));
        }
        ExpressionNode::UnaryOp { operand, .. } => {
            references.extend(find_references_in_expression(operand, symbol_name));
        }
        ExpressionNode::FunctionCall { name, args } => {
            if name == symbol_name {
                references.push(create_range(0, 0, 0, name.len()));
            }
            for arg in args {
                references.extend(find_references_in_expression(arg, symbol_name));
            }
        }
        ExpressionNode::ArrayAccess { array, index } => {
            references.extend(find_references_in_expression(array, symbol_name));
            references.extend(find_references_in_expression(index, symbol_name));
        }
        _ => {}
    }
    
    references
}

/// Extract the word at a given position in the text
fn get_word_at_position(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }
    
    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    
    // Find word boundaries
    let chars: Vec<char> = line.chars().collect();
    
    // Check bounds using character count, not byte count
    if char_pos >= chars.len() {
        return None;
    }
    
    // Find start of word
    let mut start = char_pos;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    
    // Find end of word
    let mut end = char_pos;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    
    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

/// Convert a text position (line, character) to LSP Position
pub fn text_position_to_lsp(line: usize, character: usize) -> Position {
    Position {
        line: line as u32,
        character: character as u32,
    }
}

/// Create an LSP Range from start and end positions
pub fn create_range(start_line: usize, start_char: usize, end_line: usize, end_char: usize) -> Range {
    Range {
        start: text_position_to_lsp(start_line, start_char),
        end: text_position_to_lsp(end_line, end_char),
    }
}

/// Create an LSP Location from URI and range
pub fn create_location(uri: &Url, range: Range) -> Location {
    Location {
        uri: uri.clone(),
        range,
    }
}