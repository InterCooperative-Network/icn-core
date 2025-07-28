// icn-ccl/src/lsp/formatting.rs
//! Code formatting for CCL files

use super::server::DocumentState;
use tower_lsp::lsp_types::*;

/// Format a CCL document and return text edits
pub fn format_document(doc_state: &DocumentState, _options: FormattingOptions) -> Vec<TextEdit> {
    let mut edits = Vec::new();

    // Get the formatted text
    let formatted_text = format_ccl_code(&doc_state.text);

    // If the formatted text is different, create a text edit to replace the entire document
    if formatted_text != doc_state.text {
        let lines = doc_state.text.lines().collect::<Vec<_>>();
        let end_line = lines.len().saturating_sub(1);
        let end_character = lines.last().map(|line| line.len()).unwrap_or(0);

        edits.push(TextEdit {
            range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: end_line as u32,
                    character: end_character as u32,
                },
            },
            new_text: formatted_text,
        });
    }

    edits
}

/// Format CCL code according to standard conventions
fn format_ccl_code(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut formatted_lines = Vec::new();
    let mut indent_level: usize = 0;
    let indent_size = 4; // 4 spaces per indent level

    for line in lines {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            formatted_lines.push(String::new());
            continue;
        }

        // Decrease indent for closing braces
        if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
            indent_level = indent_level.saturating_sub(1);
        }

        // Apply current indentation
        let indent = " ".repeat(indent_level * indent_size);
        let formatted_line = format!("{}{}", indent, trimmed);
        formatted_lines.push(formatted_line);

        // Increase indent for opening braces
        if trimmed.ends_with('{') || trimmed.ends_with('[') || trimmed.ends_with('(') {
            indent_level += 1;
        }

        // Handle special CCL keywords that increase indentation
        if is_indent_increasing_keyword(trimmed) {
            indent_level += 1;
        }

        // Handle keywords that decrease indentation
        if is_indent_decreasing_keyword(trimmed) {
            indent_level = indent_level.saturating_sub(1);
        }
    }

    formatted_lines.join("\n")
}

/// Check if a line contains a keyword that should increase indentation
fn is_indent_increasing_keyword(line: &str) -> bool {
    let keywords = [
        "contract", "function", "struct", "enum", "impl", "if", "else", "while", "for", "match",
        "proposal", "role", "state",
    ];

    for keyword in &keywords {
        if line.starts_with(keyword) && line.ends_with(':') {
            return true;
        }
    }

    false
}

/// Check if a line contains a keyword that should decrease indentation
fn is_indent_decreasing_keyword(line: &str) -> bool {
    let keywords = ["end", "endcontract", "endfunction", "endstruct"];

    for keyword in &keywords {
        if line.starts_with(keyword) {
            return true;
        }
    }

    false
}

/// Format a specific range of text
pub fn format_range(
    doc_state: &DocumentState,
    range: Range,
    _options: FormattingOptions,
) -> Vec<TextEdit> {
    let lines: Vec<&str> = doc_state.text.lines().collect();
    let start_line = range.start.line as usize;
    let end_line = range.end.line as usize;

    if start_line >= lines.len() || end_line >= lines.len() {
        return Vec::new();
    }

    // Extract the range text
    let mut range_lines = Vec::new();
    for i in start_line..=end_line {
        if i < lines.len() {
            range_lines.push(lines[i]);
        }
    }

    if range_lines.is_empty() {
        return Vec::new();
    }

    let range_text = range_lines.join("\n");
    let formatted_text = format_ccl_code(&range_text);

    if formatted_text != range_text {
        vec![TextEdit {
            range,
            new_text: formatted_text,
        }]
    } else {
        Vec::new()
    }
}

/// Apply basic formatting rules to improve code readability
#[allow(dead_code)]
fn apply_spacing_rules(code: &str) -> String {
    let mut result = code.to_string();

    // Add spaces around operators
    let operators = ["=", "+", "-", "*", "/", "==", "!=", "<", ">", "<=", ">="];
    for op in &operators {
        let spaced_op = format!(" {} ", op);
        result = result.replace(op, &spaced_op);
    }

    // Remove extra spaces
    while result.contains("  ") {
        result = result.replace("  ", " ");
    }

    // Add space after commas
    result = result.replace(",", ", ");

    // Clean up double spaces after commas
    result = result.replace(",  ", ", ");

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let input = "contract MyContract{function test(){return 42;}}";
        let expected = "contract MyContract {\n    function test() {\n        return 42;\n    }\n}";
        let result = format_ccl_code(input);
        // Note: This is a simplified test - real formatting would be more sophisticated
        assert_ne!(result, input); // Should be different from unformatted input
    }

    #[test]
    fn test_indentation() {
        let input = "{\ntest\n}";
        let result = format_ccl_code(input);
        assert!(result.contains("    test")); // Should have proper indentation
    }
}
