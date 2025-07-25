use icn_ccl::compile_ccl_source_to_wasm;
use tower_lsp::lsp_types::*;

pub fn validate_ccl_source(source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    
    // Attempt to compile the source to get diagnostics
    match compile_ccl_source_to_wasm(source) {
        Ok(_) => {
            // Compilation successful - no errors
        }
        Err(err) => {
            // Convert CCL error to LSP diagnostic
            let diagnostic = convert_ccl_error_to_diagnostic(&err);
            diagnostics.push(diagnostic);
        }
    }
    
    // Additional basic syntax checks
    let lines: Vec<&str> = source.lines().collect();
    for (line_num, line) in lines.iter().enumerate() {
        // Check for common syntax issues
        if line.trim_end() != line {
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line_num as u32,
                        character: line.trim_end().len() as u32,
                    },
                    end: Position {
                        line: line_num as u32,
                        character: line.len() as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::INFORMATION),
                message: "Trailing whitespace".to_string(),
                source: Some("ccl-lsp".to_string()),
                ..Default::default()
            });
        }
        
        // Check for unmatched braces (basic check)
        let open_braces = line.matches('{').count();
        let close_braces = line.matches('}').count();
        if open_braces != close_braces && (open_braces > 0 || close_braces > 0) {
            // This is a very basic check - a full implementation would track brace balance across lines
            if open_braces > close_braces && !line.contains("if") && !line.contains("fn") && !line.contains("while") {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position {
                            line: line_num as u32,
                            character: 0,
                        },
                        end: Position {
                            line: line_num as u32,
                            character: line.len() as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: "Possible unmatched opening brace".to_string(),
                    source: Some("ccl-lsp".to_string()),
                    ..Default::default()
                });
            }
        }
    }
    
    diagnostics
}

fn convert_ccl_error_to_diagnostic(error: &icn_ccl::CclError) -> Diagnostic {
    let (severity, message) = match error {
        icn_ccl::CclError::ParseError(msg) => (DiagnosticSeverity::ERROR, format!("Parse error: {}", msg)),
        icn_ccl::CclError::SemanticError(msg) => (DiagnosticSeverity::ERROR, format!("Semantic error: {}", msg)),
        icn_ccl::CclError::WasmGenerationError(msg) => (DiagnosticSeverity::ERROR, format!("WASM generation error: {}", msg)),
        icn_ccl::CclError::IoError(err) => (DiagnosticSeverity::ERROR, format!("I/O error: {}", err)),
        icn_ccl::CclError::ValidationError(msg) => (DiagnosticSeverity::WARNING, format!("Validation warning: {}", msg)),
    };
    
    Diagnostic {
        range: Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 0 },
        },
        severity: Some(severity),
        message,
        source: Some("ccl-compiler".to_string()),
        ..Default::default()
    }
}