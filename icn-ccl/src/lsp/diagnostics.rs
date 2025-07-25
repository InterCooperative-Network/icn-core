// icn-ccl/src/lsp/diagnostics.rs
//! Diagnostic generation for CCL LSP

use tower_lsp::lsp_types::*;
use crate::error::CclError;
use super::server::DocumentState;

/// Generate LSP diagnostics from CCL parse and semantic errors
pub fn generate_diagnostics(doc_state: &DocumentState) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Convert parse errors to diagnostics
    for parse_error in &doc_state.parse_errors {
        diagnostics.push(error_to_diagnostic(parse_error, DiagnosticSeverity::ERROR));
    }

    // Convert semantic errors to diagnostics
    for semantic_error in &doc_state.semantic_errors {
        diagnostics.push(error_to_diagnostic(semantic_error, DiagnosticSeverity::ERROR));
    }

    diagnostics
}

/// Convert a CclError to an LSP Diagnostic
fn error_to_diagnostic(error: &CclError, severity: DiagnosticSeverity) -> Diagnostic {
    let (message, range) = match error {
        CclError::ParsingError(msg) => {
            (msg.clone(), create_default_range())
        }
        CclError::SemanticError(msg) => {
            (msg.clone(), create_default_range())
        }
        CclError::TypeMismatch { expected, found, location: _ } => {
            (format!("Type mismatch: expected {}, found {}", expected, found), create_default_range())
        }
        CclError::UndefinedVariable { name, location: _ } => {
            (format!("Undefined variable: {}", name), create_default_range())
        }
        CclError::UndefinedFunction { name, location: _ } => {
            (format!("Undefined function: {}", name), create_default_range())
        }
        CclError::UndefinedType { name, location: _ } => {
            (format!("Undefined type: {}", name), create_default_range())
        }
        CclError::DuplicateDefinition { name, location: _ } => {
            (format!("Duplicate definition: {}", name), create_default_range())
        }
        CclError::InvalidArity { expected, found, location: _ } => {
            (format!("Invalid arity: expected {} arguments, found {}", expected, found), create_default_range())
        }
        CclError::ImmutableAssignment { name, location: _ } => {
            (format!("Cannot assign to immutable variable: {}", name), create_default_range())
        }
        CclError::InvalidOperation { operation, type_name, location: _ } => {
            (format!("Invalid operation '{}' for type '{}'", operation, type_name), create_default_range())
        }
        CclError::CircularDependency { names } => {
            (format!("Circular dependency detected: {}", names.join(" -> ")), create_default_range())
        }
        _ => {
            (format!("{}", error), create_default_range())
        }
    };

    Diagnostic {
        range,
        severity: Some(severity),
        code: None,
        code_description: None,
        source: Some("ccl".to_string()),
        message,
        related_information: None,
        tags: None,
        data: None,
    }
}

/// Create a default range for errors without location information
fn create_default_range() -> Range {
    Range {
        start: Position { line: 0, character: 0 },
        end: Position { line: 0, character: 0 },
    }
}

/// Convert a line/character position to LSP Position
pub fn position_to_lsp(line: usize, character: usize) -> Position {
    Position {
        line: line as u32,
        character: character as u32,
    }
}

/// Convert an LSP Position to line/character
pub fn lsp_to_position(pos: Position) -> (usize, usize) {
    (pos.line as usize, pos.character as usize)
}