// icn-ccl/src/lsp/diagnostics.rs
//! Diagnostic generation for CCL LSP

use super::server::DocumentState;
use crate::error::CclError;
use tower_lsp::lsp_types::*;

/// Generate LSP diagnostics from CCL parse and semantic errors
pub fn generate_diagnostics(doc_state: &DocumentState) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    // Convert parse errors to diagnostics
    for parse_error in &doc_state.parse_errors {
        diagnostics.push(error_to_diagnostic(parse_error, DiagnosticSeverity::ERROR));
    }

    // Convert semantic errors to diagnostics
    for semantic_error in &doc_state.semantic_errors {
        diagnostics.push(error_to_diagnostic(
            semantic_error,
            DiagnosticSeverity::ERROR,
        ));
    }

    diagnostics
}

/// Convert a CclError to an LSP Diagnostic
fn error_to_diagnostic(error: &CclError, severity: DiagnosticSeverity) -> Diagnostic {
    let (message, range) = match error {
        CclError::ParsingError(msg) => (msg.clone(), create_default_range()),
        CclError::SemanticError(msg) => (msg.clone(), create_default_range()),
        CclError::TypeMismatch {
            expected,
            found,
            line,
        } => (
            format!("Type mismatch: expected {}, found {}", expected, found),
            create_range_for_line(*line),
        ),
        CclError::UndefinedVariableError { variable } => (
            format!("Undefined variable: {}", variable),
            create_default_range(),
        ),
        CclError::UndefinedFunctionError { function } => (
            format!("Undefined function: {}", function),
            create_default_range(),
        ),
        CclError::ArgumentCountMismatchError {
            function,
            expected,
            found,
        } => (
            format!(
                "Invalid arity: function '{}' expected {} arguments, found {}",
                function, expected, found
            ),
            create_default_range(),
        ),
        CclError::ImmutableAssignmentError { variable } => (
            format!("Cannot assign to immutable variable: {}", variable),
            create_default_range(),
        ),
        CclError::InvalidBinaryOperationError {
            left_type,
            operator,
            right_type,
        } => (
            format!(
                "Invalid operation '{:?}' between types '{:?}' and '{:?}'",
                operator, left_type, right_type
            ),
            create_default_range(),
        ),
        CclError::CircularDependency { cycle } => (
            format!("Circular dependency detected: {}", cycle),
            create_default_range(),
        ),
        _ => (format!("{}", error), create_default_range()),
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
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    }
}

/// Create a range for a specific line
fn create_range_for_line(line: usize) -> Range {
    Range {
        start: Position {
            line: line.saturating_sub(1) as u32,
            character: 0,
        },
        end: Position {
            line: line.saturating_sub(1) as u32,
            character: 100,
        },
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
