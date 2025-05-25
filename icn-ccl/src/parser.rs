// icn-ccl/src/parser.rs
use pest::Parser;
use crate::ast::AstNode; // Assuming AstNode is the root of your AST
use crate::error::CclError;

#[derive(Parser)]
#[grammar = "grammar/ccl.pest"] // Path to your Pest grammar file
pub struct CclParser;

pub fn parse_ccl_source(source: &str) -> Result<AstNode, CclError> {
    match CclParser::parse(Rule::policy, source) {
        Ok(mut pairs) => {
            let policy_pair = pairs.next().ok_or_else(|| CclError::ParsingError("No policy rule matched".to_string()))?;
            // TODO: Implement recursive descent or other strategy to convert Pest `pairs` to your `AstNode`
            // For now, returning a placeholder
            println!("[CCL Parser STUB] Successfully parsed source (Pest pairs available). AST conversion pending.");
            println!("[CCL Parser STUB] Top-level pair: {:?}", policy_pair.as_rule());
            // let ast = crate::ast::pair_to_ast(policy_pair)?; // Example if you have this helper
            Ok(AstNode::Policy(vec![])) // Placeholder AST
        }
        Err(e) => {
            Err(CclError::ParsingError(e.to_string()))
        }
    }
}

// Example test for the parser
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_policy_stub() {
        let source = r#"
            // A very simple CCL policy stub
            fn always_true() -> Bool {
                return true;
            }

            rule check_mana when always_true() then allow;
        "#;
        match parse_ccl_source(source) {
            Ok(_ast) => {
                // In a real test, you'd assert the structure of the generated AST
                println!("Successfully parsed simple policy stub.");
            }
            Err(e) => panic!("Parsing failed: {}", e),
        }
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let source = "fn broken { return 1 }"; // Invalid syntax
        let result = parse_ccl_source(source);
        assert!(matches!(result, Err(CclError::ParsingError(_))));
    }
} 