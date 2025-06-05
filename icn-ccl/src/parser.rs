// icn-ccl/src/parser.rs
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use crate::ast::{AstNode, BlockNode, ExpressionNode, PolicyStatementNode, StatementNode, TypeAnnotationNode};
use crate::error::CclError;

#[derive(Parser)]
#[grammar = "grammar/ccl.pest"] // Path to your Pest grammar file
pub struct CclParser;

// New helper function to handle the leaf nodes of an expression
fn parse_literal_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    match pair.as_rule() {
        Rule::integer_literal => {
            let value = pair.as_str().parse::<i64>()
                .map_err(|e| CclError::ParsingError(format!("Invalid integer: {}", e)))?;
            Ok(ExpressionNode::IntegerLiteral(value))
        }
        Rule::boolean_literal => {
            match pair.as_str() {
                "true" => Ok(ExpressionNode::BooleanLiteral(true)),
                "false" => Ok(ExpressionNode::BooleanLiteral(false)),
                _ => Err(CclError::ParsingError("Invalid boolean literal".to_string())),
            }
        }
        Rule::string_literal => {
            // Need to strip quotes from string_literal if they are part of the pair's string value
            let s = pair.as_str();
            if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                Ok(ExpressionNode::StringLiteral(s[1..s.len()-1].to_string()))
            } else {
                // This case should ideally not be hit if Pest grammar for string_literal is correct e.g. `"\"" ~ inner_chars ~ "\""`
                // If `string_literal = @{ "\"" ~ inner ~ "\"" }` then `as_str()` includes quotes.
                Ok(ExpressionNode::StringLiteral(s.to_string())) // Fallback, or error
            }
        }
        Rule::identifier => {
            Ok(ExpressionNode::Identifier(pair.as_str().to_string()))
        }
        // TODO: Rule::function_call, Rule::parenthesized_expression, etc.
        _ => Err(CclError::ParsingError(format!("Unsupported literal/factor expression type: {:?}", pair.as_rule()))),
    }
}

fn parse_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // expression = { term ~ (binary_operator ~ term)* }
    // term       = { factor ~ (multiplicative_operator ~ factor)* }
    // factor     = { primary_expression | unary_operator ~ primary_expression }
    // primary_expression = { literal | identifier | parenthesized_expression | function_call }
    // literal    = { integer_literal | boolean_literal | string_literal }

    match pair.as_rule() {
        Rule::expression => {
            // For this minimal implementation, we assume the expression is just a single literal without operators.
            // expression -> term -> factor -> literal_pair
            let mut inner_rules = pair.into_inner(); // consumes the expression pair
            let term_pair = inner_rules.next()
                .ok_or_else(|| CclError::ParsingError("Expression missing term".to_string()))?;
            
            // TODO: Handle binary operators if inner_rules.next() is not None here

            // Now parse the term_pair
            // term = { factor ~ (("*" | "/") ~ factor)* }
            let mut term_inner_rules = term_pair.into_inner();
            let factor_pair = term_inner_rules.next()
                .ok_or_else(|| CclError::ParsingError("Term missing factor".to_string()))?;

            // TODO: Handle multiplicative operators if term_inner_rules.next() is not None here

            // Now parse the factor_pair
            // factor = { integer_literal | boolean_literal | string_literal | identifier | "(" ~ expression ~ ")" | function_call }
            // For the simple case `return 42;`, factor_pair directly contains integer_literal.
            let literal_or_primary_pair = factor_pair.into_inner().next()
                 .ok_or_else(|| CclError::ParsingError("Factor missing primary expression".to_string()))?;

            parse_literal_expression(literal_or_primary_pair)
        }
        // This case allows calling parse_expression with a rule that is already a literal (e.g. from other parser functions if needed)
        // However, the main call from `parse_block` will pass `Rule::expression`.
        Rule::integer_literal | Rule::boolean_literal | Rule::string_literal | Rule::identifier => {
            parse_literal_expression(pair)
        }
        _ => Err(CclError::ParsingError(format!("Unsupported top-level rule for expression parsing: {:?}", pair.as_rule()))),
    }
}

fn parse_block(pair: Pair<Rule>) -> Result<BlockNode, CclError> {
    let mut statements = vec![];
    // block = { "{" ~ statement* ~ "}" }
    // We expect the block's inner rules to be `statement` rules.
    for statement_rule_pair in pair.into_inner() { // This `pair` is the `block`
        if statement_rule_pair.as_rule() == Rule::statement {
            // A `statement` rule can be `let_statement | expression_statement | return_statement`
            // We take the first (and only, for now) inner rule of the `statement`.
            let actual_statement_pair = statement_rule_pair.into_inner().next()
                .ok_or_else(|| CclError::ParsingError("Empty statement rule".to_string()))?;

            match actual_statement_pair.as_rule() {
                Rule::return_statement => {
                    let mut inner_return_rules = actual_statement_pair.into_inner();
                    let expression_rule = inner_return_rules.next()
                        .ok_or_else(|| CclError::ParsingError("Return statement missing expression".to_string()))?;
                    statements.push(StatementNode::Return(parse_expression(expression_rule)?));
                }
                // TODO: Add Rule::let_statement, Rule::expression_statement
                _ => return Err(CclError::ParsingError(format!("Unsupported statement type: {:?}", actual_statement_pair.as_rule()))),
            }
        } else {
            return Err(CclError::ParsingError(format!("Unexpected rule directly in block: {:?}, expected a statement", statement_rule_pair.as_rule())));
        }
    }
    Ok(BlockNode { statements })
}

fn parse_type_annotation(pair: Pair<Rule>) -> Result<TypeAnnotationNode, CclError> {
    // `type_annotation` rule in Pest is now `{ identifier }`.
    // So, the `pair` passed here should have `pair.as_rule() == Rule::identifier` if Pest rule `type_annotation = { identifier }` was used directly in `function_definition`.
    // However, the `function_definition` rule is `... ~ type_annotation ~ ...`
    // So the `pair` here *is* the `type_annotation` rule itself. Its inner rule should be `identifier`.
    let type_identifier_pair = pair.into_inner().next()
        .ok_or_else(|| CclError::ParsingError("Type annotation missing identifier".to_string()))?;
    
    if type_identifier_pair.as_rule() != Rule::identifier {
        return Err(CclError::ParsingError(format!("Expected identifier for type annotation, got {:?}", type_identifier_pair.as_rule())));
    }

    match type_identifier_pair.as_str() { // The matched identifier string
        "Integer" => Ok(TypeAnnotationNode::Integer),
        "Bool" => Ok(TypeAnnotationNode::Bool),
        "String" => Ok(TypeAnnotationNode::String),
        "Mana" => Ok(TypeAnnotationNode::Mana),
        "DID" => Ok(TypeAnnotationNode::Did),
        other => Err(CclError::TypeError(format!("Unknown type: {}", other))), // This error should now be correctly triggered
    }
}

fn parse_function_definition(pair: Pair<Rule>) -> Result<AstNode, CclError> {
    // function_definition = { "fn" ~ identifier ~ "(" ~ (parameter ~ ("," ~ parameter)*)? ~ ")" ~ "->" ~ type_annotation ~ block }
    let mut inner_rules = pair.into_inner(); // Consumes the function_definition pair

    let name_token = inner_rules.next().ok_or_else(|| CclError::ParsingError("Function definition missing name".to_string()))?;
    assert_eq!(name_token.as_rule(), Rule::identifier);
    let name = name_token.as_str().to_string();
    
    let return_type_pair = inner_rules.next().ok_or_else(|| CclError::ParsingError("Function definition missing return type".to_string()))?;
    let return_type = parse_type_annotation(return_type_pair)?;
    
    let block_pair = inner_rules.next().ok_or_else(|| CclError::ParsingError("Function definition missing body".to_string()))?;
    let body = parse_block(block_pair)?;
    
    Ok(AstNode::FunctionDefinition {
        name,
        parameters: vec![], // Minimal example has no parameters
        return_type,
        body,
    })
}

pub fn parse_ccl_source(source: &str) -> Result<AstNode, CclError> {
    match CclParser::parse(Rule::policy, source) {
        Ok(mut pairs) => {
            let policy_content = pairs.next().ok_or_else(|| CclError::ParsingError("Empty policy source".to_string()))?;
            
            let mut ast_nodes_for_policy = vec![];

            for pair_in_policy in policy_content.into_inner() {
                match pair_in_policy.as_rule() {
                    Rule::function_definition => {
                        ast_nodes_for_policy.push(PolicyStatementNode::FunctionDef(parse_function_definition(pair_in_policy)?));
                    }
                    Rule::EOI => (), 
                    _ => return Err(CclError::ParsingError(format!("Unexpected rule in policy: {:?}", pair_in_policy.as_rule()))),
                }
            }
            if ast_nodes_for_policy.is_empty() {
                 return Err(CclError::ParsingError("No function definitions found in policy".to_string()));
            }
            Ok(AstNode::Policy(ast_nodes_for_policy))

        }
        Err(e) => {
            Err(CclError::ParsingError(format!("Pest parsing error: {}", e)))
        }
    }
}


// Example test for the parser
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, BlockNode, ExpressionNode, PolicyStatementNode, StatementNode, TypeAnnotationNode};

    #[test]
    fn test_parse_simple_function_definition() {
        let source = r#"
            fn get_value() -> Integer {
                return 42;
            }
        "#;
        match parse_ccl_source(source) {
            Ok(ast) => {
                let expected_ast = AstNode::Policy(vec![
                    PolicyStatementNode::FunctionDef(
                        AstNode::FunctionDefinition {
                            name: "get_value".to_string(),
                            parameters: vec![],
                            return_type: TypeAnnotationNode::Integer,
                            body: BlockNode {
                                statements: vec![
                                    StatementNode::Return(ExpressionNode::IntegerLiteral(42))
                                ],
                            },
                        }
                    )
                ]);
                assert_eq!(ast, expected_ast);
            }
            Err(e) => panic!("Parsing failed: {}", e),
        }
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let source = "fn broken { return 1 }"; 
        let result = parse_ccl_source(source);
        assert!(matches!(result, Err(CclError::ParsingError(_))));
    }

    #[test]
    fn test_parse_unknown_type_annotation() {
        let source = r#"
            fn test_func() -> UnknownType {
                return 1;
            }
        "#;
        let result = parse_ccl_source(source);
        assert!(matches!(result, Err(CclError::TypeError(_))));
    }

    #[test]
    fn test_parse_block_missing_return_expression() {
         let source = r#"
            fn test_func() -> Integer {
                return ; 
            }
        "#;
        let result = parse_ccl_source(source);
        assert!(matches!(result, Err(CclError::ParsingError(_))));
    }
     #[test]
    fn test_malformed_function_no_body() {
        let source = "fn no_body() -> Integer";
        let result = parse_ccl_source(source);
        assert!(matches!(result, Err(CclError::ParsingError(_))));
    }

    #[test]
    fn test_malformed_function_no_return_type() {
        let source = "fn no_return_type() { return 1; }";
        let result = parse_ccl_source(source);
        assert!(matches!(result, Err(CclError::ParsingError(_))));
    }
} 