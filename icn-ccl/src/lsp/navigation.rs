// icn-ccl/src/lsp/navigation.rs
//! Navigation features for CCL LSP (go-to-definition, find references)

use tower_lsp::lsp_types::*;
use super::server::DocumentState;

/// Provide go-to-definition functionality
pub fn goto_definition(doc_state: &DocumentState, position: Position) -> Option<Location> {
    // Get the word at the cursor position
    let _word = get_word_at_position(&doc_state.text, position)?;
    
    // TODO: Implement go-to-definition by analyzing the AST
    // This would involve:
    // 1. Finding the symbol at the cursor position
    // 2. Looking up its definition in the AST
    // 3. Converting the AST location to LSP Position/Range
    // 4. Returning a Location pointing to the definition
    
    // For now, return None (no definition found)
    None
}

/// Find all references to a symbol
pub fn find_references(doc_state: &DocumentState, position: Position) -> Vec<Location> {
    // Get the word at the cursor position
    let _word = get_word_at_position(&doc_state.text, position)?;
    
    // TODO: Implement find references by analyzing the AST
    // This would involve:
    // 1. Finding the symbol at the cursor position
    // 2. Searching the entire AST for references to this symbol
    // 3. Converting all found locations to LSP Locations
    
    // For now, return empty list (no references found)
    Vec::new()
}

/// Extract the word at a given position in the text
fn get_word_at_position(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }
    
    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    
    if char_pos > line.len() {
        return None;
    }
    
    // Find word boundaries
    let chars: Vec<char> = line.chars().collect();
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