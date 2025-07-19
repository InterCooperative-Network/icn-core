// icn-ccl/src/parser.rs
#![allow(clippy::while_let_on_iterator)]
use crate::ast::{
    ActionNode, AstNode, BinaryOperator, BlockNode, ExpressionNode, ParameterNode,
    PolicyStatementNode, StatementNode, TypeAnnotationNode, UnaryOperator,
};
use crate::error::CclError;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/ccl.pest"] // Path to your Pest grammar file
pub struct CclParser;

/// Convert escaped sequences like `\n` or `\"` into their actual characters.
pub fn unescape_string(s: &str) -> Result<String, CclError> {
    let mut result = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            let next = chars
                .next()
                .ok_or_else(|| CclError::ParsingError("Incomplete escape sequence".to_string()))?;
            match next {
                'n' => result.push('\n'),
                't' => result.push('\t'),
                'r' => result.push('\r'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                other => result.push(other),
            }
        } else {
            result.push(c);
        }
    }
    Ok(result)
}

// Parse a simple literal or identifier expression
pub(crate) fn parse_literal_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    match pair.as_rule() {
        Rule::integer_literal => {
            let value = pair
                .as_str()
                .parse::<i64>()
                .map_err(|e| CclError::ParsingError(format!("Invalid integer: {}", e)))?;
            Ok(ExpressionNode::IntegerLiteral(value))
        }
        Rule::boolean_literal => match pair.as_str() {
            "true" => Ok(ExpressionNode::BooleanLiteral(true)),
            "false" => Ok(ExpressionNode::BooleanLiteral(false)),
            _ => Err(CclError::ParsingError(
                "Invalid boolean literal".to_string(),
            )),
        },
        Rule::string_literal => {
            let s = pair.as_str();
            let inner = if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
                &s[1..s.len() - 1]
            } else {
                s
            };
            let unescaped = unescape_string(inner)?;
            Ok(ExpressionNode::StringLiteral(unescaped))
        }
        Rule::identifier => Ok(ExpressionNode::Identifier(pair.as_str().to_string())),
        _ => Err(CclError::ParsingError(format!(
            "Unsupported literal expression type: {:?}",
            pair.as_rule()
        ))),
    }
}

pub(crate) fn parse_function_call(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Function call missing name".to_string()))?;
    let name = name_pair.as_str().to_string();
    let mut args = Vec::new();
    for arg_pair in inner {
        if arg_pair.as_rule() == Rule::expression {
            args.push(parse_expression(arg_pair)?);
        }
    }
    Ok(ExpressionNode::FunctionCall {
        name,
        arguments: args,
    })
}

pub(crate) fn parse_array_literal(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    let elements = pair
        .into_inner()
        .map(parse_expression)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ExpressionNode::ArrayLiteral(elements))
}

pub(crate) fn parse_primary(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    match pair.as_rule() {
        Rule::primary => {
            let mut inner = pair.into_inner();
            let base = inner.next().ok_or_else(|| {
                CclError::ParsingError("Primary expression missing base".to_string())
            })?;
            let mut expr = parse_primary(base)?;
            for index_pair in inner {
                let index_expr = parse_expression(index_pair)?;
                expr = ExpressionNode::ArrayAccess {
                    array: Box::new(expr),
                    index: Box::new(index_expr),
                };
            }
            Ok(expr)
        }
        Rule::atom => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| CclError::ParsingError("Atom missing inner".to_string()))?;
            parse_primary(inner)
        }
        Rule::array_literal => parse_array_literal(pair),
        Rule::some_expr => {
            let inner = pair.into_inner().next().unwrap();
            Ok(ExpressionNode::SomeExpr(Box::new(parse_expression(inner)?)))
        }
        Rule::none_expr => Ok(ExpressionNode::NoneExpr),
        Rule::ok_expr => {
            let inner = pair.into_inner().next().unwrap();
            Ok(ExpressionNode::OkExpr(Box::new(parse_expression(inner)?)))
        }
        Rule::err_expr => {
            let inner = pair.into_inner().next().unwrap();
            Ok(ExpressionNode::ErrExpr(Box::new(parse_expression(inner)?)))
        }
        Rule::integer_literal | Rule::boolean_literal | Rule::string_literal | Rule::identifier => {
            parse_literal_expression(pair)
        }
        Rule::require_proof_expr => {
            let inner = pair.into_inner().next().unwrap();
            Ok(ExpressionNode::RequireProof(Box::new(parse_expression(
                inner,
            )?)))
        }
        Rule::function_call => parse_function_call(pair),
        Rule::expression => parse_expression(pair),
        _ => Err(CclError::ParsingError(format!(
            "Unsupported primary expression: {:?}",
            pair.as_rule()
        ))),
    }
}

pub(crate) fn parse_unary(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    if pair.as_rule() != Rule::unary {
        return Err(CclError::ParsingError(format!(
            "Expected unary expression, got {:?}",
            pair.as_rule()
        )));
    }

    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Unary rule missing first element".to_string()))?;

    match first.as_rule() {
        Rule::NOT_OP => Ok(ExpressionNode::UnaryOp {
            operator: UnaryOperator::Not,
            operand: Box::new(parse_unary(inner.next().unwrap())?),
        }),
        Rule::SUB_OP => Ok(ExpressionNode::UnaryOp {
            operator: UnaryOperator::Neg,
            operand: Box::new(parse_unary(inner.next().unwrap())?),
        }),
        _ => parse_primary(first),
    }
}

pub(crate) fn parse_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    match pair.as_rule() {
        Rule::expression => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| CclError::ParsingError("Expression missing inner".to_string()))?;
            parse_expression(inner)
        }
        Rule::try_expression => parse_try_expression(pair),
        Rule::match_expression => {
            let mut inner = pair.into_inner();
            let value = parse_expression(inner.next().unwrap())?;
            let mut arms = Vec::new();
            for arm in inner {
                let mut arm_inner = arm.into_inner();
                let pat = parse_expression(arm_inner.next().unwrap())?;
                let expr = parse_expression(arm_inner.next().unwrap())?;
                arms.push((pat, expr));
            }
            Ok(ExpressionNode::Match {
                value: Box::new(value),
                arms,
            })
        }
        Rule::logical_or => {
            let mut inner = pair.into_inner();
            let mut expr = parse_expression(inner.next().unwrap())?;
            while let Some(op) = inner.next() {
                let right = parse_expression(inner.next().unwrap())?;
                let op = match op.as_rule() {
                    Rule::OR_OP => BinaryOperator::Or,
                    _ => unreachable!(),
                };
                expr = ExpressionNode::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
            Ok(expr)
        }
        Rule::logical_and => {
            let mut inner = pair.into_inner();
            let mut expr = parse_expression(inner.next().unwrap())?;
            while let Some(op) = inner.next() {
                let right = parse_expression(inner.next().unwrap())?;
                let op = match op.as_rule() {
                    Rule::AND_OP => BinaryOperator::And,
                    _ => unreachable!(),
                };
                expr = ExpressionNode::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
            Ok(expr)
        }
        Rule::equality => {
            let mut inner = pair.into_inner();
            let mut expr = parse_expression(inner.next().unwrap())?;
            while let Some(op) = inner.next() {
                let right = parse_expression(inner.next().unwrap())?;
                let op = match op.as_rule() {
                    Rule::EQ_OP => BinaryOperator::Eq,
                    Rule::NEQ_OP => BinaryOperator::Neq,
                    _ => unreachable!(),
                };
                expr = ExpressionNode::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
            Ok(expr)
        }
        Rule::comparison => {
            let mut inner = pair.into_inner();
            let mut expr = parse_expression(inner.next().unwrap())?;
            while let Some(op) = inner.next() {
                let right = parse_expression(inner.next().unwrap())?;
                let op = match op.as_rule() {
                    Rule::LT_OP => BinaryOperator::Lt,
                    Rule::LTE_OP => BinaryOperator::Lte,
                    Rule::GT_OP => BinaryOperator::Gt,
                    Rule::GTE_OP => BinaryOperator::Gte,
                    _ => unreachable!(),
                };
                expr = ExpressionNode::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
            Ok(expr)
        }
        Rule::addition => {
            let mut inner = pair.into_inner();
            let mut expr = parse_expression(inner.next().unwrap())?;
            while let Some(op) = inner.next() {
                let right = parse_expression(inner.next().unwrap())?;
                let op = match op.as_rule() {
                    Rule::ADD_OP => BinaryOperator::Add,
                    Rule::SUB_OP => BinaryOperator::Sub,
                    _ => unreachable!(),
                };
                expr = ExpressionNode::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
            Ok(expr)
        }
        Rule::multiplication => {
            let mut inner = pair.into_inner();
            let mut expr = parse_expression(inner.next().unwrap())?;
            while let Some(op) = inner.next() {
                let right = parse_expression(inner.next().unwrap())?;
                let op = match op.as_rule() {
                    Rule::MUL_OP => BinaryOperator::Mul,
                    Rule::DIV_OP => BinaryOperator::Div,
                    _ => unreachable!(),
                };
                expr = ExpressionNode::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
            Ok(expr)
        }
        Rule::unary => parse_unary(pair),
        Rule::primary => parse_primary(pair),
        Rule::function_call
        | Rule::integer_literal
        | Rule::boolean_literal
        | Rule::string_literal
        | Rule::identifier
        | Rule::array_literal => parse_primary(pair),
        _ => Err(CclError::ParsingError(format!(
            "Unsupported expression rule: {:?}",
            pair.as_rule()
        ))),
    }
}

pub(crate) fn parse_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    if pair.as_rule() != Rule::statement {
        return Err(CclError::ParsingError(format!(
            "Expected statement, got {:?}",
            pair.as_rule()
        )));
    }

    let actual_statement_pair = pair
        .into_inner()
        .next()
        .ok_or_else(|| CclError::ParsingError("Empty statement rule".to_string()))?;

    match actual_statement_pair.as_rule() {
        Rule::return_statement => {
            let mut inner_return_rules = actual_statement_pair.into_inner();
            let expression_rule = inner_return_rules.next().ok_or_else(|| {
                CclError::ParsingError("Return statement missing expression".to_string())
            })?;
            Ok(StatementNode::Return(parse_expression(expression_rule)?))
        }
        Rule::let_statement => {
            let mut inner = actual_statement_pair.into_inner();
            let name_pair = inner.next().ok_or_else(|| {
                CclError::ParsingError("Let statement missing identifier".to_string())
            })?;
            let expr_pair = inner.next().ok_or_else(|| {
                CclError::ParsingError("Let statement missing expression".to_string())
            })?;
            Ok(StatementNode::Let {
                name: name_pair.as_str().to_string(),
                value: parse_expression(expr_pair)?,
            })
        }
        Rule::expression_statement => {
            let expr_pair = actual_statement_pair.into_inner().next().ok_or_else(|| {
                CclError::ParsingError("Expression statement missing expression".to_string())
            })?;
            Ok(StatementNode::ExpressionStatement(parse_expression(
                expr_pair,
            )?))
        }
        Rule::if_statement => parse_if_statement(actual_statement_pair),
        Rule::while_statement => {
            let mut inner = actual_statement_pair.into_inner();
            let cond_pair = inner.next().ok_or_else(|| {
                CclError::ParsingError("While statement missing condition".to_string())
            })?;
            let body_pair = inner.next().ok_or_else(|| {
                CclError::ParsingError("While statement missing body".to_string())
            })?;
            Ok(StatementNode::WhileLoop {
                condition: parse_expression(cond_pair)?,
                body: parse_block(body_pair)?,
            })
        }
        Rule::for_statement => {
            let mut inner = actual_statement_pair.into_inner();
            let ident_pair = inner.next().ok_or_else(|| {
                CclError::ParsingError("For statement missing identifier".to_string())
            })?;
            let expr_pair = inner.next().ok_or_else(|| {
                CclError::ParsingError("For statement missing iterable".to_string())
            })?;
            let body_pair = inner
                .next()
                .ok_or_else(|| CclError::ParsingError("For statement missing body".to_string()))?;
            Ok(StatementNode::ForLoop {
                iterator: ident_pair.as_str().to_string(),
                iterable: parse_expression(expr_pair)?,
                body: parse_block(body_pair)?,
            })
        }
        Rule::break_statement => Ok(StatementNode::Break),
        Rule::continue_statement => Ok(StatementNode::Continue),
        _ => Err(CclError::ParsingError(format!(
            "Unsupported statement type: {:?}",
            actual_statement_pair.as_rule()
        ))),
    }
}

fn parse_if_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    let mut inner = pair.into_inner();
    let cond_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("If statement missing condition".to_string()))?;
    let then_block_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("If statement missing then block".to_string()))?;
    let else_pair = inner.next();

    let else_block = if let Some(e) = else_pair {
        let mut e_inner = e.into_inner();
        let next = e_inner
            .next()
            .ok_or_else(|| CclError::ParsingError("Else clause missing body".to_string()))?;
        match next.as_rule() {
            Rule::if_statement => {
                let nested = parse_if_statement(next)?;
                Some(BlockNode {
                    statements: vec![nested],
                })
            }
            Rule::block => Some(parse_block(next)?),
            other => {
                return Err(CclError::ParsingError(format!(
                    "Unexpected rule in else clause: {:?}",
                    other
                )));
            }
        }
    } else {
        None
    };

    Ok(StatementNode::If {
        condition: parse_expression(cond_pair)?,
        then_block: parse_block(then_block_pair)?,
        else_block,
    })
}

pub(crate) fn parse_block(pair: Pair<Rule>) -> Result<BlockNode, CclError> {
    let mut statements = Vec::new();
    for statement_rule_pair in pair.into_inner() {
        if statement_rule_pair.as_rule() == Rule::statement {
            statements.push(parse_statement(statement_rule_pair)?);
        } else {
            return Err(CclError::ParsingError(format!(
                "Unexpected rule directly in block: {:?}, expected a statement",
                statement_rule_pair.as_rule(),
            )));
        }
    }
    Ok(BlockNode { statements })
}
pub(crate) fn parse_type_annotation(pair: Pair<Rule>) -> Result<TypeAnnotationNode, CclError> {
    // `type_annotation` rule in Pest can now be complex (option_type | result_type | array_type | identifier)
    match pair.as_rule() {
        Rule::type_annotation => {
            let inner = pair.into_inner().next().unwrap();
            parse_type_annotation(inner)
        }
        Rule::option_type => {
            let mut inner = pair.into_inner();
            let inner_type = parse_type_annotation(inner.next().unwrap())?;
            Ok(TypeAnnotationNode::Option(Box::new(inner_type)))
        }
        Rule::result_type => {
            let mut inner = pair.into_inner();
            let ok_type = parse_type_annotation(inner.next().unwrap())?;
            let err_type = parse_type_annotation(inner.next().unwrap())?;
            Ok(TypeAnnotationNode::Result {
                ok_type: Box::new(ok_type),
                err_type: Box::new(err_type),
            })
        }
        Rule::array_type => {
            let mut inner = pair.into_inner();
            let element_type = parse_type_annotation(inner.next().unwrap())?;
            Ok(TypeAnnotationNode::Array(Box::new(element_type)))
        }
        Rule::identifier => match pair.as_str() {
            "Integer" => Ok(TypeAnnotationNode::Integer),
            "Bool" => Ok(TypeAnnotationNode::Bool),
            "String" => Ok(TypeAnnotationNode::String),
            "Mana" => Ok(TypeAnnotationNode::Mana),
            "Did" => Ok(TypeAnnotationNode::Did),
            "Proposal" => Ok(TypeAnnotationNode::Proposal),
            "Vote" => Ok(TypeAnnotationNode::Vote),
            other => Ok(TypeAnnotationNode::Custom(other.to_string())),
        },
        _ => {
            // Legacy handling - try to extract identifier from inner
            let type_identifier_pair = pair
                .into_inner()
                .next()
                .ok_or_else(|| CclError::ParsingError("Type annotation missing identifier".to_string()))?;

            if type_identifier_pair.as_rule() != Rule::identifier {
                return Err(CclError::ParsingError(format!(
                    "Expected identifier for type annotation, got {:?}",
                    type_identifier_pair.as_rule()
                )));
            }

            match type_identifier_pair.as_str() {
                "Integer" => Ok(TypeAnnotationNode::Integer),
                "Bool" => Ok(TypeAnnotationNode::Bool),
                "String" => Ok(TypeAnnotationNode::String),
                "Mana" => Ok(TypeAnnotationNode::Mana),
                "DID" => Ok(TypeAnnotationNode::Did),
                other => Ok(TypeAnnotationNode::Custom(other.to_string())),
            }
        }
    }
}

pub(crate) fn parse_function_definition(pair: Pair<Rule>) -> Result<AstNode, CclError> {
    // function_definition = { "fn" ~ identifier ~ "(" ~ (parameter ~ ("," ~ parameter)*)? ~ ")" ~ "->" ~ type_annotation ~ block }
    let mut inner_rules = pair.into_inner();

    let name_token = inner_rules
        .next()
        .ok_or_else(|| CclError::ParsingError("Function definition missing name".to_string()))?;
    assert_eq!(name_token.as_rule(), Rule::identifier);
    let name = name_token.as_str().to_string();

    let mut parameters = Vec::new();
    // Collect all parameter rules until we encounter the return type
    let mut next = inner_rules
        .next()
        .ok_or_else(|| CclError::ParsingError("Function definition truncated".to_string()))?;
    while next.as_rule() == Rule::parameter {
        let mut p_inner = next.into_inner();
        let id_pair = p_inner
            .next()
            .ok_or_else(|| CclError::ParsingError("Parameter missing identifier".to_string()))?;
        let ty_pair = p_inner
            .next()
            .ok_or_else(|| CclError::ParsingError("Parameter missing type".to_string()))?;
        parameters.push(ParameterNode {
            name: id_pair.as_str().to_string(),
            type_ann: parse_type_annotation(ty_pair)?,
        });

        next = inner_rules.next().ok_or_else(|| {
            CclError::ParsingError("Function definition missing return type".to_string())
        })?;
    }

    // `next` now holds the return type rule
    if next.as_rule() != Rule::type_annotation {
        return Err(CclError::ParsingError("Expected return type".to_string()));
    }
    let return_type = parse_type_annotation(next)?;

    let block_pair = inner_rules
        .next()
        .ok_or_else(|| CclError::ParsingError("Function definition missing body".to_string()))?;
    let body = parse_block(block_pair)?;

    Ok(AstNode::FunctionDefinition {
        name,
        parameters,
        return_type,
        body,
    })
}

pub(crate) fn parse_struct_definition(pair: Pair<Rule>) -> Result<AstNode, CclError> {
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Struct missing name".to_string()))?;
    let mut fields = Vec::new();
    for p in inner {
        let mut p_inner = p.into_inner();
        let id_pair = p_inner
            .next()
            .ok_or_else(|| CclError::ParsingError("Field missing name".to_string()))?;
        let ty_pair = p_inner
            .next()
            .ok_or_else(|| CclError::ParsingError("Field missing type".to_string()))?;
        fields.push(ParameterNode {
            name: id_pair.as_str().to_string(),
            type_ann: parse_type_annotation(ty_pair)?,
        });
    }
    Ok(AstNode::StructDefinition {
        name: name_pair.as_str().to_string(),
        fields,
    })
}

pub(crate) fn parse_action(pair: Pair<Rule>) -> Result<ActionNode, CclError> {
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Empty action".to_string()))?;
    match first.as_rule() {
        Rule::ALLOW => Ok(ActionNode::Allow),
        Rule::DENY => Ok(ActionNode::Deny),
        Rule::CHARGE => {
            let expr_pair = inner.next().ok_or_else(|| {
                CclError::ParsingError("Charge action missing expression".to_string())
            })?;
            Ok(ActionNode::Charge(parse_expression(expr_pair)?))
        }
        _ => Err(CclError::ParsingError(format!(
            "Unknown action component: {:?}",
            first.as_rule()
        ))),
    }
}

pub(crate) fn parse_rule_definition(pair: Pair<Rule>) -> Result<AstNode, CclError> {
    let mut inner = pair.into_inner();
    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Rule missing name".to_string()))?;
    let condition_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Rule missing condition".to_string()))?;
    let action_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Rule missing action".to_string()))?;

    Ok(AstNode::RuleDefinition {
        name: name_pair.as_str().to_string(),
        condition: parse_expression(condition_pair)?,
        action: parse_action(action_pair)?,
    })
}

pub(crate) fn parse_policy_statement(pair: Pair<Rule>) -> Result<PolicyStatementNode, CclError> {
    let mut inner = pair.into_inner();
    let stmt_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Empty policy statement".to_string()))?;
    match stmt_pair.as_rule() {
        Rule::rule_definition => Ok(PolicyStatementNode::RuleDef(parse_rule_definition(
            stmt_pair,
        )?)),
        Rule::struct_definition => Ok(PolicyStatementNode::StructDef(parse_struct_definition(
            stmt_pair,
        )?)),
        Rule::import_statement => {
            let mut i = stmt_pair.into_inner();
            let path_pair = i
                .next()
                .ok_or_else(|| CclError::ParsingError("Import missing path".to_string()))?;
            let alias_pair = i
                .next()
                .ok_or_else(|| CclError::ParsingError("Import missing alias".to_string()))?;
            let path = path_pair.as_str().trim_matches('"').to_string();
            let alias = alias_pair.as_str().to_string();
            Ok(PolicyStatementNode::Import { path, alias })
        }
        _ => Err(CclError::ParsingError(format!(
            "Unknown policy statement: {:?}",
            stmt_pair.as_rule()
        ))),
    }
}

pub fn parse_ccl_source(source: &str) -> Result<AstNode, CclError> {
    match CclParser::parse(Rule::policy, source) {
        Ok(mut pairs) => {
            let policy_content = pairs
                .next()
                .ok_or_else(|| CclError::ParsingError("Empty policy source".to_string()))?;

            let mut ast_nodes_for_policy = vec![];

            for pair_in_policy in policy_content.into_inner() {
                match pair_in_policy.as_rule() {
                    Rule::function_definition => {
                        ast_nodes_for_policy.push(PolicyStatementNode::FunctionDef(
                            parse_function_definition(pair_in_policy)?,
                        ));
                    }
                    Rule::struct_definition => {
                        ast_nodes_for_policy.push(PolicyStatementNode::StructDef(
                            parse_struct_definition(pair_in_policy)?,
                        ));
                    }
                    Rule::const_definition => {
                        ast_nodes_for_policy.push(parse_const_definition(pair_in_policy)?);
                    }
                    Rule::macro_definition => {
                        ast_nodes_for_policy.push(parse_macro_definition(pair_in_policy)?);
                    }
                    Rule::policy_statement => {
                        ast_nodes_for_policy.push(parse_policy_statement(pair_in_policy)?);
                    }
                    Rule::EOI => (),
                    _ => {
                        return Err(CclError::ParsingError(format!(
                            "Unexpected rule in policy: {:?}",
                            pair_in_policy.as_rule()
                        )));
                    }
                }
            }
            if ast_nodes_for_policy.is_empty() {
                return Err(CclError::ParsingError(
                    "Policy contained no items".to_string(),
                ));
            }
            Ok(AstNode::Policy(ast_nodes_for_policy))
        }
        Err(e) => Err(CclError::ParsingError(format!("Pest parsing error: {}", e))),
    }
}

/// Parse a CCL file from disk, recursively loading any imported modules.
///
/// Imported modules are merged into the returned [`AstNode::Policy`] with all
/// top-level items prefixed by the provided alias. Paths are resolved relative
/// to the directory of `path`.
pub fn parse_ccl_file(path: &std::path::Path) -> Result<AstNode, CclError> {
    use std::fs;

    let source = fs::read_to_string(path).map_err(|e| {
        CclError::IoError(format!("Failed to read {}: {}", path.display(), e))
    })?;

    let base_dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut ast = parse_ccl_source(&source)?;
    load_imports(&mut ast, base_dir)?;
    Ok(ast)
}

fn alias_ast(ast: AstNode, alias: &str) -> AstNode {
    match ast {
        AstNode::FunctionDefinition { name, parameters, return_type, body } => {
            AstNode::FunctionDefinition {
                name: format!("{}_{}", alias, name),
                parameters,
                return_type,
                body,
            }
        }
        AstNode::StructDefinition { name, fields } => AstNode::StructDefinition {
            name: format!("{}_{}", alias, name),
            fields,
        },
        AstNode::RuleDefinition { name, condition, action } => AstNode::RuleDefinition {
            name: format!("{}_{}", alias, name),
            condition,
            action,
        },
        AstNode::Policy(stmts) => AstNode::Policy(stmts.into_iter().map(|s| alias_stmt(s, alias)).collect()),
        other => other,
    }
}

fn alias_stmt(stmt: PolicyStatementNode, alias: &str) -> PolicyStatementNode {
    match stmt {
        PolicyStatementNode::FunctionDef(f) => PolicyStatementNode::FunctionDef(alias_ast(f, alias)),
        PolicyStatementNode::StructDef(s) => PolicyStatementNode::StructDef(alias_ast(s, alias)),
        PolicyStatementNode::RuleDef(r) => PolicyStatementNode::RuleDef(alias_ast(r, alias)),
        PolicyStatementNode::ConstDef { name, value, type_ann } => PolicyStatementNode::ConstDef {
            name: format!("{}_{}", alias, name),
            value,
            type_ann,
        },
        PolicyStatementNode::MacroDef { name, params, body } => PolicyStatementNode::MacroDef {
            name: format!("{}_{}", alias, name),
            params,
            body,
        },
        PolicyStatementNode::EventDef { name, fields } => PolicyStatementNode::EventDef {
            name: format!("{}_{}", alias, name),
            fields,
        },
        PolicyStatementNode::StateDef { name, type_ann, initial_value } => {
            PolicyStatementNode::StateDef {
                name: format!("{}_{}", alias, name),
                type_ann,
                initial_value,
            }
        }
        PolicyStatementNode::TriggerDef { name, condition, action } => PolicyStatementNode::TriggerDef {
            name: format!("{}_{}", alias, name),
            condition,
            action,
        },
        other => other,
    }
}

fn load_imports(ast: &mut AstNode, base: &std::path::Path) -> Result<(), CclError> {
    if let AstNode::Policy(ref mut items) = ast {
        let mut result = Vec::new();
        for item in std::mem::take(items) {
            match item {
                PolicyStatementNode::Import { path, alias } => {
                    let import_path = base.join(&path);
                    let imported = parse_ccl_file(&import_path).map_err(|e| {
                        CclError::ModuleImportError {
                            module: path.clone(),
                            reason: e.to_string(),
                        }
                    })?;
                    if let AstNode::Policy(mut stmts) = alias_ast(imported, &alias) {
                        result.append(&mut stmts);
                    }
                }
                other => result.push(other),
            }
        }
        *items = result;
    }
    Ok(())
}

pub(crate) fn parse_const_definition(pair: Pair<Rule>) -> Result<PolicyStatementNode, CclError> {
    // const_definition = { "const" ~ identifier ~ ":" ~ type_annotation ~ "=" ~ expression ~ ";" }
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    let type_ann = parse_type_annotation(inner.next().unwrap())?;
    let value = parse_expression(inner.next().unwrap())?;
    
    Ok(PolicyStatementNode::ConstDef { name, value, type_ann })
}

pub(crate) fn parse_macro_definition(pair: Pair<Rule>) -> Result<PolicyStatementNode, CclError> {
    // macro_definition = { "macro" ~ identifier ~ "(" ~ (identifier ~ ("," ~ identifier)*)? ~ ")" ~ "{" ~ (!"}") ~ ANY* ~ "}" }
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();
    
    let mut params = Vec::new();
    let mut body = String::new();
    
    for item in inner {
        if item.as_rule() == Rule::identifier {
            params.push(item.as_str().to_string());
        } else {
            // This is the macro body content
            body = item.as_str().to_string();
        }
    }
    
    Ok(PolicyStatementNode::MacroDef { name, params, body })
}

pub(crate) fn parse_try_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // try_expression = { "try" ~ logical_or ~ ("catch" ~ logical_or)? }
    let mut inner = pair.into_inner();
    let expr = parse_expression(inner.next().unwrap())?;
    let catch_arm = if let Some(catch_pair) = inner.next() {
        Some(Box::new(parse_expression(catch_pair)?))
    } else {
        None
    };
    
    Ok(ExpressionNode::TryExpr {
        expr: Box::new(expr),
        catch_arm,
    })
}

// Example test for the parser
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{
        AstNode, BlockNode, ExpressionNode, PolicyStatementNode, StatementNode, TypeAnnotationNode,
    };

    #[test]
    fn test_parse_simple_function_definition() {
        let source = r#"
            fn get_value() -> Integer {
                return 42;
            }
        "#;
        match parse_ccl_source(source) {
            Ok(ast) => {
                let expected_ast = AstNode::Policy(vec![PolicyStatementNode::FunctionDef(
                    AstNode::FunctionDefinition {
                        name: "get_value".to_string(),
                        parameters: vec![],
                        return_type: TypeAnnotationNode::Integer,
                        body: BlockNode {
                            statements: vec![StatementNode::Return(
                                ExpressionNode::IntegerLiteral(42),
                            )],
                        },
                    },
                )]);
                assert_eq!(ast, expected_ast);
            }
            Err(e) => panic!("Parsing failed: {}", e),
        }
    }

    #[test]
    fn test_parse_invalid_syntax() {
        let source = "fn broken { return 1 }";
        let result = parse_ccl_source(source);
        assert!(result.is_err());
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
        assert!(result.is_ok());
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

    #[test]
    fn test_parse_if_and_let() {
        let source = r#"
            fn check() -> Bool {
                let x = 1 + 2 * 3;
                if x > 5 {
                    return true;
                } else {
                    return false;
                }
            }
        "#;
        let parsed = parse_ccl_source(source).expect("should parse");
        if let AstNode::Policy(items) = parsed {
            assert!(!items.is_empty());
        } else {
            panic!("Expected policy AST");
        }
    }

    #[test]
    fn test_parse_rule_definition() {
        let source = r#"
            rule allow_all when true then allow
        "#;
        let parsed = parse_ccl_source(source).expect("should parse rule");
        if let AstNode::Policy(items) = parsed {
            assert!(matches!(items[0], PolicyStatementNode::RuleDef(_)));
        } else {
            panic!("Expected policy AST");
        }
    }

    #[test]
    #[ignore]
    fn test_parse_struct_and_match() {
        let source = r#"
            struct Point { x: Integer, y: Integer }
            fn run() -> Integer {
                let p = Some(1);
                return match p { 1 => 10, _ => 0 };
            }
        "#;
        let result = parse_ccl_source(source);
        assert!(result.is_ok());
    }
}
