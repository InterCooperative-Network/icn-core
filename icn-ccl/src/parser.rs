// icn-ccl/src/parser.rs
use pest::iterators::Pair;
use pest::Parser;
use crate::ast::{AstNode, BlockNode, ExpressionNode, PolicyStatementNode, StatementNode, TypeAnnotationNode};
use crate::error::CclError;

#[derive(Parser)]
#[grammar = "grammar/ccl.pest"] // Path to your Pest grammar file
pub struct CclParser;

fn parse_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    match pair.as_rule() {
        Rule::integer_literal => {
            let value = pair.as_str().parse::<i64>()
                .map_err(|e| CclError::ParsingError(format!("Invalid integer: {}", e)))?;
            Ok(ExpressionNode::IntegerLiteral(value))
        }
        // TODO: Add other expression types as grammar expands
        _ => Err(CclError::ParsingError(format!("Unsupported expression type: {:?}", pair.as_rule()))),
    }
}

fn parse_block(pair: Pair<Rule>) -> Result<BlockNode, CclError> {
    let mut statements = vec![];
    // block = { "{" ~ statement* ~ "}" }
    // statement = { let_statement | expression_statement | return_statement }
    // return_statement = { "return" ~ expression ~ ";" }
    // We expect the block's inner rules to be statements.
    for statement_pair in pair.into_inner() { // Iterates over rules inside the block
        match statement_pair.as_rule() {
            Rule::return_statement => {
                // return_statement has 'return', expression, and ';' as inner rules.
                // We need the expression part.
                let expression_pair = statement_pair.into_inner().next() // Skips "return" token if it's explicit
                    .ok_or_else(|| CclError::ParsingError("Return statement missing expression".to_string()))?;
                 // If Pest grammar for return_statement is ` { "return" ~ expression ~ ";" } `
                 // then `statement_pair.into_inner()` would yield `expression` then `;`.
                 // If Pest grammar captures `expression` directly under `return_statement` like `return_statement = { "return" ~ ^expression ~ ";" }`
                 // then the .next() might not be needed or might point to the expression directly.
                 // Assuming grammar `return_statement = { "return" ~ expression ~ ";" }`, and expression is the first meaningful inner rule.
                 // Let's refine this based on how `return_statement` is structured.
                 // If `return_statement` itself *is* the rule passed, and its inner is the expression:
                 let mut inner_return = statement_pair.into_inner();
                 let expr_candidate = inner_return.next().ok_or_else(||CclError::ParsingError("Malformed return statement, expected expression".to_string()))?;


                statements.push(StatementNode::Return(parse_expression(expr_candidate)?));
            }
            // TODO: Add other statement types as grammar expands (let_statement, expression_statement)
            _ => return Err(CclError::ParsingError(format!("Unexpected statement type in block: {:?}", statement_pair.as_rule()))),
        }
    }
    Ok(BlockNode { statements })
}

fn parse_type_annotation(pair: Pair<Rule>) -> Result<TypeAnnotationNode, CclError> {
    // Assuming type_annotation rule directly matches one of the types
    // type_annotation = { "Mana" | "Bool" | "DID" | "String" | "Integer" }
    match pair.as_str() { // The matched string itself
        "Integer" => Ok(TypeAnnotationNode::Integer),
        "Bool" => Ok(TypeAnnotationNode::Bool),
        "String" => Ok(TypeAnnotationNode::String),
        "Mana" => Ok(TypeAnnotationNode::Mana),
        "DID" => Ok(TypeAnnotationNode::Did),
        other => Err(CclError::TypeError(format!("Unknown type: {}", other))),
    }
}

fn parse_function_definition(pair: Pair<Rule>) -> Result<AstNode, CclError> {
    // function_definition = { "fn" ~ identifier ~ "(" ~ (parameter ~ ("," ~ parameter)*)? ~ ")" ~ "->" ~ type_annotation ~ block }
    let mut inner_rules = pair.into_inner(); // Consumes the function_definition pair

    let name_token = inner_rules.next().ok_or_else(|| CclError::ParsingError("Function definition missing name".to_string()))?;
    assert_eq!(name_token.as_rule(), Rule::identifier);
    let name = name_token.as_str().to_string();

    // Parameters are optional and not handled in this minimal version, so we skip to return type
    // This needs to be more robust if parameters are present.
    // For now, assume no parameters as per the simple example.
    // "fn" -> name -> "(" -> ")" -> "->" -> type_annotation -> block
    // If parameters were `(parameter ~ ("," ~ parameter)*)?`, this part is tricky.
    // Let's assume for the minimal example, the grammar for no params is just `()`
    // and the parser skips these tokens or the grammar is simplified.
    // Example `fn get_value() -> Integer { return 42; }`
    // `inner_rules` items: `identifier` (`get_value`), `type_annotation` (`Integer`), `block`
    // This assumes the `(`, `)`, `->` tokens are not captured as separate `Pair<Rule>` items if not explicitly named or captured in Pest.
    // If they are, they need to be skipped. Let's check Pest behavior.
    // Typically, literals like "(" are consumed.
    
    // If Pest grammar has `identifier ~ "(" ~ parameters_list? ~ ")" ~ "->" ~ type_annotation ~ block`
    // We'd do:
    // name = inner_rules.next().unwrap().as_str().to_string(); // identifier
    // _open_paren = inner_rules.next().unwrap(); // (
    // parameters_list_or_close_paren = inner_rules.next().unwrap();
    // if parameters_list_or_close_paren.as_rule() == Rule::parameters_list { ... } else { /* it's ) */ }
    // ... and so on. This is complex.

    // For the given minimal grammar:
    // function_definition = { "fn" ~ identifier ~ "(" ~ ")" ~ "->" ~ type_annotation ~ block }
    // inner_rules will give: identifier, type_annotation, block
    // Because "fn", "(", ")", "->" are silent (not captured by a rule name).
    
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
    // Try to parse the source as a single function_definition first for this minimal case
    // Or, parse as `policy` and expect it to contain one function_definition
    match CclParser::parse(Rule::policy, source) {
        Ok(mut pairs) => {
            let policy_content = pairs.next().ok_or_else(|| CclError::ParsingError("Empty policy source".to_string()))?;
            
            let mut ast_nodes_for_policy = vec![];

            // `policy` rule is `SOI ~ (function_definition | policy_statement)* ~ EOI`
            // So `policy_content.into_inner()` will give us the sequence of function_definition or policy_statement
            for pair_in_policy in policy_content.into_inner() {
                match pair_in_policy.as_rule() {
                    Rule::function_definition => {
                        ast_nodes_for_policy.push(PolicyStatementNode::FunctionDef(parse_function_definition(pair_in_policy)?));
                    }
                    // Rule::policy_statement => { ... } // For later expansion
                    Rule::EOI => (), // End Of Input, ignore
                    _ => return Err(CclError::ParsingError(format!("Unexpected rule in policy: {:?}", pair_in_policy.as_rule()))),
                }
            }
            if ast_nodes_for_policy.is_empty() {
                 return Err(CclError::ParsingError("No function definitions found in policy".to_string()));
            }
            // For this minimal phase, we expect one function, so we construct a Policy node
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
        let source = "fn broken { return 1 }"; // Invalid syntax according to full grammar
        let result = parse_ccl_source(source);
        // This will fail if the grammar rule `policy` doesn't match at all.
        // The error might be about not matching `policy` or an inner rule.
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
                return ; // Missing expression
            }
        "#;
        // This depends on how robust `parse_expression` and `parse_block` are.
        // The current Pest grammar for `return_statement` requires an `expression`.
        // So Pest itself would likely fail to parse `return ;` under `return_statement`.
        // The error would be a Pest parsing error.
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