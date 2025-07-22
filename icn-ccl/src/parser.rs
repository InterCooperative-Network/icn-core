// icn-ccl/src/parser.rs
#![allow(clippy::while_let_on_iterator)]
use crate::ast::{
    AstNode, BinaryOperator, BlockNode, ConstDeclarationNode, ContractBodyNode,
    ContractDeclarationNode, ContractMetaNode, DurationExprNode, DurationUnitNode,
    EnumDefinitionNode, EnumVariantNode, ExpressionNode, FieldInitNode, FieldNode,
    FunctionDefinitionNode, ImportStatementNode, LValueNode, LiteralNode, MatchArmNode,
    ParameterNode, PatternNode, PolicyStatementNode, ProposalDeclarationNode, ProposalFieldNode,
    RequirementNode, RoleBodyNode, RoleDeclarationNode, StateDeclarationNode, StatementNode,
    StructDefinitionNode, StructFieldPattern, ThresholdTypeNode, TypeExprNode, TypeParameterNode,
    UnaryOperator,
};
use crate::error::CclError;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/ccl.pest"] // Now uses the updated CCL 0.1 grammar
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

/// Parse an import statement
pub fn parse_import_statement(pair: Pair<Rule>) -> Result<ImportStatementNode, CclError> {
    // import_stmt = { "import" ~ string ~ ("as" ~ identifier)? ~ ";" }
    let mut inner = pair.into_inner();

    let path_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Import missing path".to_string()))?;
    let path = path_pair.as_str().trim_matches('"').to_string();

    let alias = inner
        .next()
        .map(|alias_pair| alias_pair.as_str().to_string());

    Ok(ImportStatementNode { path, alias })
}

/// Parse a contract declaration
pub fn parse_contract_declaration(pair: Pair<Rule>) -> Result<ContractDeclarationNode, CclError> {
    // contract_decl = { "contract" ~ identifier ~ "{" ~ contract_meta* ~ contract_body* ~ "}" }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Contract missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let mut metadata = Vec::new();
    let mut body = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::contract_meta => {
                // Handle nested metadata rules
                for meta_item in item.into_inner() {
                    match meta_item.as_rule() {
                        Rule::scope_meta => {
                            let meta = parse_scope_meta(meta_item)?;
                            metadata.push(meta);
                        }
                        Rule::version_meta => {
                            let meta = parse_version_meta(meta_item)?;
                            metadata.push(meta);
                        }
                        Rule::extends_meta => {
                            let meta = parse_extends_meta(meta_item)?;
                            metadata.push(meta);
                        }
                        _ => {
                            return Err(CclError::ParsingError(format!(
                                "Unexpected metadata rule: {:?}",
                                meta_item.as_rule()
                            )));
                        }
                    }
                }
            }
            Rule::scope_meta => {
                let meta = parse_scope_meta(item)?;
                metadata.push(meta);
            }
            Rule::version_meta => {
                let meta = parse_version_meta(item)?;
                metadata.push(meta);
            }
            Rule::extends_meta => {
                let meta = parse_extends_meta(item)?;
                metadata.push(meta);
            }
            Rule::role_decl => {
                let role = parse_role_declaration(item)?;
                body.push(ContractBodyNode::Role(role));
            }
            Rule::proposal_decl => {
                let proposal = parse_proposal_declaration(item)?;
                body.push(ContractBodyNode::Proposal(proposal));
            }
            Rule::fn_decl => {
                let function = parse_function_declaration(item)?;
                body.push(ContractBodyNode::Function(function));
            }
            Rule::state_decl => {
                let state = parse_state_declaration(item)?;
                body.push(ContractBodyNode::State(state));
            }
            Rule::struct_decl => {
                let struct_def = parse_struct_declaration(item)?;
                body.push(ContractBodyNode::Struct(struct_def));
            }
            Rule::enum_decl => {
                let enum_def = parse_enum_declaration(item)?;
                body.push(ContractBodyNode::Enum(enum_def));
            }
            Rule::const_decl => {
                let const_def = parse_const_declaration(item)?;
                body.push(ContractBodyNode::Const(const_def));
            }
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Unexpected rule in contract body: {:?}",
                    item.as_rule()
                )));
            }
        }
    }

    Ok(ContractDeclarationNode {
        name,
        metadata,
        body,
    })
}

/// Parse contract metadata
fn parse_scope_meta(pair: Pair<Rule>) -> Result<ContractMetaNode, CclError> {
    // scope_meta = { "scope:" ~ scope_literal ~ ";" }
    let mut inner = pair.into_inner();
    let scope_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Scope meta missing value".to_string()))?;
    let scope = scope_pair.as_str().trim_matches('"').to_string();
    Ok(ContractMetaNode::Scope(scope))
}

fn parse_version_meta(pair: Pair<Rule>) -> Result<ContractMetaNode, CclError> {
    // version_meta = { "version:" ~ string ~ ";" }
    let mut inner = pair.into_inner();
    let version_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Version meta missing value".to_string()))?;
    let version = version_pair.as_str().trim_matches('"').to_string();
    Ok(ContractMetaNode::Version(version))
}

fn parse_extends_meta(pair: Pair<Rule>) -> Result<ContractMetaNode, CclError> {
    // extends_meta = { "extends:" ~ identifier ~ ";" }
    let mut inner = pair.into_inner();
    let extends_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Extends meta missing value".to_string()))?;
    let extends = extends_pair.as_str().to_string();
    Ok(ContractMetaNode::Extends(extends))
}

/// Parse a role declaration
fn parse_role_declaration(pair: Pair<Rule>) -> Result<RoleDeclarationNode, CclError> {
    // role_decl = { "role" ~ identifier ~ ("extends" ~ identifier)? ~ "{" ~ role_body* ~ "}" }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Role missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let mut extends = None;
    let mut body = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::identifier => {
                // This is the extends identifier
                extends = Some(item.as_str().to_string());
            }
            Rule::can_clause => {
                let permissions = parse_can_clause(item)?;
                body.push(RoleBodyNode::Can(permissions));
            }
            Rule::requires_clause => {
                let requirements = parse_requires_clause(item)?;
                body.push(RoleBodyNode::Requires(requirements));
            }
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Unexpected rule in role body: {:?}",
                    item.as_rule()
                )));
            }
        }
    }

    Ok(RoleDeclarationNode {
        name,
        extends,
        body,
    })
}

fn parse_can_clause(pair: Pair<Rule>) -> Result<Vec<String>, CclError> {
    // can_clause = { "can:" ~ "[" ~ permission_list? ~ "]" ~ ";" }
    let mut inner = pair.into_inner();
    let mut permissions = Vec::new();

    if let Some(permission_list) = inner.next() {
        for permission in permission_list.into_inner() {
            if permission.as_rule() == Rule::identifier {
                permissions.push(permission.as_str().to_string());
            }
        }
    }

    Ok(permissions)
}

fn parse_requires_clause(pair: Pair<Rule>) -> Result<Vec<RequirementNode>, CclError> {
    // requires_clause = { "requires:" ~ "[" ~ requirement_list? ~ "]" ~ ";" }
    let mut inner = pair.into_inner();
    let mut requirements = Vec::new();

    if let Some(requirement_list) = inner.next() {
        for requirement in requirement_list.into_inner() {
            if requirement.as_rule() == Rule::requirement {
                let req = parse_requirement(requirement)?;
                requirements.push(req);
            }
        }
    }

    Ok(requirements)
}

fn parse_requirement(pair: Pair<Rule>) -> Result<RequirementNode, CclError> {
    // requirement = { identifier ~ ":" ~ expr }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Requirement missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let expr_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Requirement missing expression".to_string()))?;
    let expr = parse_expression(expr_pair)?;

    Ok(RequirementNode { name, expr })
}

/// Parse a proposal declaration
fn parse_proposal_declaration(pair: Pair<Rule>) -> Result<ProposalDeclarationNode, CclError> {
    // proposal_decl = { "proposal" ~ identifier ~ "{" ~ proposal_field* ~ "}" }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Proposal missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let mut fields = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::description_meta => {
                let field = parse_description_meta(item)?;
                fields.push(field);
            }
            Rule::eligible_meta => {
                let field = parse_eligible_meta(item)?;
                fields.push(field);
            }
            Rule::duration_meta => {
                let field = parse_duration_meta(item)?;
                fields.push(field);
            }
            Rule::quorum_config => {
                let field = parse_quorum_config(item)?;
                fields.push(field);
            }
            Rule::threshold_config => {
                let field = parse_threshold_config(item)?;
                fields.push(field);
            }
            Rule::execution_block => {
                let field = parse_execution_block(item)?;
                fields.push(field);
            }
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Unexpected rule in proposal: {:?}",
                    item.as_rule()
                )));
            }
        }
    }

    Ok(ProposalDeclarationNode { name, fields })
}

fn parse_description_meta(pair: Pair<Rule>) -> Result<ProposalFieldNode, CclError> {
    // description_meta = { "description:" ~ string ~ ";" }
    let mut inner = pair.into_inner();
    let desc_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Description meta missing value".to_string()))?;
    let description = desc_pair.as_str().trim_matches('"').to_string();
    Ok(ProposalFieldNode::Description(description))
}

fn parse_eligible_meta(pair: Pair<Rule>) -> Result<ProposalFieldNode, CclError> {
    // eligible_meta = { "eligible:" ~ identifier ~ ";" }
    let mut inner = pair.into_inner();
    let eligible_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Eligible meta missing value".to_string()))?;
    let eligible = eligible_pair.as_str().to_string();
    Ok(ProposalFieldNode::Eligible(eligible))
}

fn parse_duration_meta(pair: Pair<Rule>) -> Result<ProposalFieldNode, CclError> {
    // duration_meta = { "duration:" ~ duration_expr ~ ";" }
    let mut inner = pair.into_inner();
    let duration_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Duration meta missing value".to_string()))?;
    let duration = parse_duration_expr(duration_pair)?;
    Ok(ProposalFieldNode::Duration(duration))
}

fn parse_duration_expr(pair: Pair<Rule>) -> Result<DurationExprNode, CclError> {
    // duration_expr = { integer ~ duration_unit }
    let mut inner = pair.into_inner();

    let value_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Duration missing value".to_string()))?;
    let value = value_pair
        .as_str()
        .parse::<i64>()
        .map_err(|e| CclError::ParsingError(format!("Invalid duration value: {}", e)))?;

    let unit_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Duration missing unit".to_string()))?;
    let unit = match unit_pair.as_str() {
        "days" => DurationUnitNode::Days,
        "hours" => DurationUnitNode::Hours,
        "minutes" => DurationUnitNode::Minutes,
        "seconds" => DurationUnitNode::Seconds,
        _ => return Err(CclError::ParsingError("Invalid duration unit".to_string())),
    };

    Ok(DurationExprNode { value, unit })
}

fn parse_quorum_config(pair: Pair<Rule>) -> Result<ProposalFieldNode, CclError> {
    // quorum_config = { "quorum:" ~ percentage ~ ";" }
    let mut inner = pair.into_inner();
    let percentage_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Quorum missing percentage".to_string()))?;
    let percentage_str = percentage_pair.as_str().trim_end_matches('%');
    let percentage = percentage_str
        .parse::<u32>()
        .map_err(|e| CclError::ParsingError(format!("Invalid percentage: {}", e)))?;
    Ok(ProposalFieldNode::Quorum(percentage))
}

fn parse_threshold_config(pair: Pair<Rule>) -> Result<ProposalFieldNode, CclError> {
    // threshold_config = { "threshold:" ~ threshold_type ~ ";" }
    let mut inner = pair.into_inner();
    let threshold_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Threshold missing type".to_string()))?;
    let threshold = parse_threshold_type(threshold_pair)?;
    Ok(ProposalFieldNode::Threshold(threshold))
}

fn parse_threshold_type(pair: Pair<Rule>) -> Result<ThresholdTypeNode, CclError> {
    // threshold_type = { "majority" | "supermajority" ~ "(" ~ fraction ~ ")" | "consensus" | "unanimous" }
    let mut inner = pair.into_inner();
    let first = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Threshold type missing".to_string()))?;

    match first.as_str() {
        "majority" => Ok(ThresholdTypeNode::Majority),
        "consensus" => Ok(ThresholdTypeNode::Consensus),
        "unanimous" => Ok(ThresholdTypeNode::Unanimous),
        "supermajority" => {
            let fraction_pair = inner.next().ok_or_else(|| {
                CclError::ParsingError("Supermajority missing fraction".to_string())
            })?;
            let fraction_str = fraction_pair.as_str();
            let parts: Vec<&str> = fraction_str.split('/').collect();
            if parts.len() != 2 {
                return Err(CclError::ParsingError(
                    "Invalid fraction format".to_string(),
                ));
            }
            let numerator = parts[0]
                .parse::<u32>()
                .map_err(|e| CclError::ParsingError(format!("Invalid numerator: {}", e)))?;
            let denominator = parts[1]
                .parse::<u32>()
                .map_err(|e| CclError::ParsingError(format!("Invalid denominator: {}", e)))?;
            Ok(ThresholdTypeNode::Supermajority {
                numerator,
                denominator,
            })
        }
        _ => Err(CclError::ParsingError("Unknown threshold type".to_string())),
    }
}

fn parse_execution_block(pair: Pair<Rule>) -> Result<ProposalFieldNode, CclError> {
    // execution_block = { "execution:" ~ "{" ~ statement* ~ "}" }
    let inner = pair.into_inner();
    let mut statements = Vec::new();

    for stmt_pair in inner {
        if stmt_pair.as_rule() == Rule::statement {
            let stmt = parse_statement(stmt_pair)?;
            statements.push(stmt);
        }
    }

    Ok(ProposalFieldNode::Execution(BlockNode { statements }))
}

/// Parse a function declaration (CCL 0.1 style)
pub fn parse_function_declaration(pair: Pair<Rule>) -> Result<FunctionDefinitionNode, CclError> {
    // fn_decl = { "fn" ~ identifier ~ type_parameters? ~ "(" ~ parameter_list? ~ ")" ~ return_type? ~ block }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Function missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let mut type_parameters = Vec::new();
    let mut parameters = Vec::new();
    let mut return_type = None;
    let mut body = None;

    for item in inner {
        match item.as_rule() {
            Rule::type_parameters => {
                type_parameters = parse_type_parameters(item)?;
            }
            Rule::parameter_list => {
                parameters = parse_parameter_list(item)?;
            }
            Rule::return_type => {
                return_type = Some(parse_return_type(item)?);
            }
            Rule::block => {
                body = Some(parse_block(item)?);
            }
            Rule::parameter => {
                // Single parameter without parameter_list wrapper
                let param = parse_parameter(item)?;
                parameters.push(param);
            }
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Unexpected rule in function: {:?}",
                    item.as_rule()
                )));
            }
        }
    }

    let body = body.ok_or_else(|| CclError::ParsingError("Function missing body".to_string()))?;

    Ok(FunctionDefinitionNode {
        name,
        type_parameters,
        parameters,
        return_type,
        body,
    })
}

/// Parse function for new CCL 0.1 functions (wrapper for AST compatibility)
pub fn parse_function_definition_new(pair: Pair<Rule>) -> Result<FunctionDefinitionNode, CclError> {
    parse_function_declaration(pair)
}

fn parse_parameter_list(pair: Pair<Rule>) -> Result<Vec<ParameterNode>, CclError> {
    // parameter_list = { parameter ~ ("," ~ parameter)* }
    let mut parameters = Vec::new();

    for param_pair in pair.into_inner() {
        if param_pair.as_rule() == Rule::parameter {
            let param = parse_parameter(param_pair)?;
            parameters.push(param);
        }
    }

    Ok(parameters)
}

fn parse_type_parameters(pair: Pair<Rule>) -> Result<Vec<TypeParameterNode>, CclError> {
    // type_parameters = { "<" ~ type_parameter_list ~ ">" }
    let inner = pair.into_inner();

    for item in inner {
        if item.as_rule() == Rule::type_parameter_list {
            return parse_type_parameter_list(item);
        }
    }

    Ok(Vec::new())
}

fn parse_type_parameter_list(pair: Pair<Rule>) -> Result<Vec<TypeParameterNode>, CclError> {
    // type_parameter_list = { type_parameter ~ ("," ~ type_parameter)* }
    let mut type_params = Vec::new();

    for param_pair in pair.into_inner() {
        if param_pair.as_rule() == Rule::type_parameter {
            let param = parse_type_parameter(param_pair)?;
            type_params.push(param);
        }
    }

    Ok(type_params)
}

fn parse_type_parameter(pair: Pair<Rule>) -> Result<TypeParameterNode, CclError> {
    // type_parameter = { identifier ~ (":" ~ type_bounds)? }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Type parameter missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let mut bounds = Vec::new();

    // Parse optional type bounds
    for item in inner {
        if item.as_rule() == Rule::type_bounds {
            bounds = parse_type_bounds(item)?;
        }
    }

    Ok(TypeParameterNode { name, bounds })
}

fn parse_type_bounds(pair: Pair<Rule>) -> Result<Vec<String>, CclError> {
    // type_bounds = { type_bound ~ ("+" ~ type_bound)* }
    let mut bounds = Vec::new();

    for bound_pair in pair.into_inner() {
        if bound_pair.as_rule() == Rule::type_bound {
            bounds.push(bound_pair.as_str().to_string());
        }
    }

    Ok(bounds)
}

fn parse_type_expr_list(pair: Pair<Rule>) -> Result<Vec<TypeExprNode>, CclError> {
    // type_expr_list = { type_expr ~ ("," ~ type_expr)* }
    let mut type_exprs = Vec::new();

    for type_pair in pair.into_inner() {
        if type_pair.as_rule() == Rule::type_expr {
            let type_expr = parse_type_expr(type_pair)?;
            type_exprs.push(type_expr);
        }
    }

    Ok(type_exprs)
}

fn parse_parameter(pair: Pair<Rule>) -> Result<ParameterNode, CclError> {
    // parameter = { identifier ~ ":" ~ type_expr }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Parameter missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let type_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Parameter missing type".to_string()))?;
    let type_expr = parse_type_expr(type_pair)?;

    Ok(ParameterNode { name, type_expr })
}

fn parse_return_type(pair: Pair<Rule>) -> Result<TypeExprNode, CclError> {
    // return_type = { "->" ~ type_expr }
    let mut inner = pair.into_inner();
    let type_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Return type missing type expression".to_string()))?;
    parse_type_expr(type_pair)
}

/// Parse type expressions (new CCL 0.1 type system)
pub fn parse_type_expr(pair: Pair<Rule>) -> Result<TypeExprNode, CclError> {
    match pair.as_rule() {
        Rule::type_expr => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| CclError::ParsingError("Empty type expression".to_string()))?;
            parse_type_expr(inner)
        }
        Rule::basic_type => match pair.as_str() {
            "Integer" => Ok(TypeExprNode::Integer),
            "String" => Ok(TypeExprNode::String),
            "Boolean" | "Bool" => Ok(TypeExprNode::Boolean),
            "Mana" => Ok(TypeExprNode::Mana),
            "Did" => Ok(TypeExprNode::Did),
            "Timestamp" => Ok(TypeExprNode::Timestamp),
            "Duration" => Ok(TypeExprNode::Duration),
            _ => Err(CclError::ParsingError(format!(
                "Unknown basic type: {}",
                pair.as_str()
            ))),
        },
        Rule::array_type => {
            // array_type = { "[" ~ type_expr ~ "]" }
            let mut inner = pair.into_inner();
            let element_type = inner.next().ok_or_else(|| {
                CclError::ParsingError("Array type missing element type".to_string())
            })?;
            let element_type_expr = parse_type_expr(element_type)?;
            Ok(TypeExprNode::Array(Box::new(element_type_expr)))
        }
        Rule::map_type => {
            // map_type = { "map" ~ "<" ~ type_expr ~ "," ~ type_expr ~ ">" }
            let mut inner = pair.into_inner();
            let key_type = inner
                .next()
                .ok_or_else(|| CclError::ParsingError("Map type missing key type".to_string()))?;
            let value_type = inner
                .next()
                .ok_or_else(|| CclError::ParsingError("Map type missing value type".to_string()))?;
            let key_type_expr = parse_type_expr(key_type)?;
            let value_type_expr = parse_type_expr(value_type)?;
            Ok(TypeExprNode::Map {
                key_type: Box::new(key_type_expr),
                value_type: Box::new(value_type_expr),
            })
        }
        Rule::option_type => {
            // option_type = { "Option" ~ "<" ~ type_expr ~ ">" }
            let mut inner = pair.into_inner();
            let inner_type = inner.next().ok_or_else(|| {
                CclError::ParsingError("Option type missing inner type".to_string())
            })?;
            let inner_type_expr = parse_type_expr(inner_type)?;
            Ok(TypeExprNode::Option(Box::new(inner_type_expr)))
        }
        Rule::result_type => {
            // result_type = { "Result" ~ "<" ~ type_expr ~ "," ~ type_expr ~ ">" }
            let mut inner = pair.into_inner();
            let ok_type = inner
                .next()
                .ok_or_else(|| CclError::ParsingError("Result type missing ok type".to_string()))?;
            let err_type = inner.next().ok_or_else(|| {
                CclError::ParsingError("Result type missing error type".to_string())
            })?;
            let ok_type_expr = parse_type_expr(ok_type)?;
            let err_type_expr = parse_type_expr(err_type)?;
            Ok(TypeExprNode::Result {
                ok_type: Box::new(ok_type_expr),
                err_type: Box::new(err_type_expr),
            })
        }
        Rule::custom_type => Ok(TypeExprNode::Custom(pair.as_str().to_string())),
        Rule::identifier => Ok(TypeExprNode::Custom(pair.as_str().to_string())),
        Rule::generic_instantiation => {
            // generic_instantiation = { identifier ~ "<" ~ type_expr_list ~ ">" }
            let mut inner = pair.into_inner();
            let base_type = inner.next().ok_or_else(|| {
                CclError::ParsingError("Generic instantiation missing base type".to_string())
            })?;
            let type_args_list = inner.next().ok_or_else(|| {
                CclError::ParsingError("Generic instantiation missing type arguments".to_string())
            })?;

            let type_args = parse_type_expr_list(type_args_list)?;

            Ok(TypeExprNode::GenericInstantiation {
                base_type: base_type.as_str().to_string(),
                type_args,
            })
        }
        Rule::type_parameter_ref => Ok(TypeExprNode::TypeParameter(pair.as_str().to_string())),
        _ => Err(CclError::ParsingError(format!(
            "Unexpected type expression rule: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_state_declaration(pair: Pair<Rule>) -> Result<StateDeclarationNode, CclError> {
    // state_decl = { "state" ~ identifier ~ ":" ~ type_expr ~ ("=" ~ expr)? ~ ";" }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("State declaration missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let type_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("State declaration missing type".to_string()))?;
    let type_expr = parse_type_expr(type_pair)?;

    let initial_value = if let Some(expr_pair) = inner.next() {
        Some(parse_expression(expr_pair)?)
    } else {
        None
    };

    Ok(StateDeclarationNode {
        name,
        type_expr,
        initial_value,
    })
}

pub fn parse_struct_declaration(pair: Pair<Rule>) -> Result<StructDefinitionNode, CclError> {
    // struct_decl = { "struct" ~ identifier ~ type_parameters? ~ "{" ~ field_list? ~ "}" }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Struct missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let mut type_parameters = Vec::new();
    let mut fields = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::type_parameters => {
                type_parameters = parse_type_parameters(item)?;
            }
            Rule::field_list => {
                fields = parse_field_list(item)?;
            }
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Unexpected rule in struct: {:?}",
                    item.as_rule()
                )));
            }
        }
    }

    Ok(StructDefinitionNode {
        name,
        type_parameters,
        fields,
    })
}

fn parse_field_list(pair: Pair<Rule>) -> Result<Vec<FieldNode>, CclError> {
    // field_list = { field ~ ("," ~ field)* }
    let mut fields = Vec::new();

    for field_pair in pair.into_inner() {
        if field_pair.as_rule() == Rule::field {
            let field = parse_field(field_pair)?;
            fields.push(field);
        }
    }

    Ok(fields)
}

fn parse_field(pair: Pair<Rule>) -> Result<FieldNode, CclError> {
    // field = { identifier ~ ":" ~ type_expr }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Field missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let type_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Field missing type".to_string()))?;
    let type_expr = parse_type_expr(type_pair)?;

    Ok(FieldNode { name, type_expr })
}

pub fn parse_enum_declaration(pair: Pair<Rule>) -> Result<EnumDefinitionNode, CclError> {
    // enum_decl = { "enum" ~ identifier ~ type_parameters? ~ "{" ~ variant_list? ~ "}" }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Enum missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let mut type_parameters = Vec::new();
    let mut variants = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::type_parameters => {
                type_parameters = parse_type_parameters(item)?;
            }
            Rule::variant_list => {
                variants = parse_variant_list(item)?;
            }
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Unexpected rule in enum: {:?}",
                    item.as_rule()
                )));
            }
        }
    }

    Ok(EnumDefinitionNode {
        name,
        type_parameters,
        variants,
    })
}

fn parse_variant_list(pair: Pair<Rule>) -> Result<Vec<EnumVariantNode>, CclError> {
    // variant_list = { variant ~ ("," ~ variant)* }
    let mut variants = Vec::new();

    for variant_pair in pair.into_inner() {
        if variant_pair.as_rule() == Rule::variant {
            let variant = parse_variant(variant_pair)?;
            variants.push(variant);
        }
    }

    Ok(variants)
}

fn parse_variant(pair: Pair<Rule>) -> Result<EnumVariantNode, CclError> {
    // variant = { identifier ~ ("(" ~ type_expr ~ ")")? }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Variant missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let type_expr = if let Some(type_pair) = inner.next() {
        Some(parse_type_expr(type_pair)?)
    } else {
        None
    };

    Ok(EnumVariantNode { name, type_expr })
}

pub fn parse_const_declaration(pair: Pair<Rule>) -> Result<ConstDeclarationNode, CclError> {
    // const_decl = { "const" ~ identifier ~ ":" ~ type_expr ~ "=" ~ expr ~ ";" }
    let mut inner = pair.into_inner();

    let name_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Const declaration missing name".to_string()))?;
    let name = name_pair.as_str().to_string();

    let type_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Const declaration missing type".to_string()))?;
    let type_expr = parse_type_expr(type_pair)?;

    let value_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Const declaration missing value".to_string()))?;
    let value = parse_expression(value_pair)?;

    Ok(ConstDeclarationNode {
        name,
        type_expr,
        value,
    })
}

// Continue with updated parsing functions for expressions and statements...

pub(crate) fn parse_literal_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    match pair.as_rule() {
        Rule::literal => {
            // Handle the wrapper literal rule by extracting the inner literal type
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| CclError::ParsingError("Empty literal rule".to_string()))?;
            parse_literal_expression(inner)
        }
        Rule::integer => {
            let value = pair
                .as_str()
                .parse::<i64>()
                .map_err(|e| CclError::ParsingError(format!("Invalid integer: {}", e)))?;
            Ok(ExpressionNode::Literal(LiteralNode::Integer(value)))
        }
        Rule::boolean => match pair.as_str() {
            "true" => Ok(ExpressionNode::Literal(LiteralNode::Boolean(true))),
            "false" => Ok(ExpressionNode::Literal(LiteralNode::Boolean(false))),
            _ => Err(CclError::ParsingError("Invalid boolean value".to_string())),
        },
        Rule::string => {
            let raw_string = pair.as_str();
            let trimmed = raw_string.trim_matches('"');
            let unescaped = unescape_string(trimmed)?;
            Ok(ExpressionNode::Literal(LiteralNode::String(unescaped)))
        }
        Rule::float => {
            let value = pair
                .as_str()
                .parse::<f64>()
                .map_err(|e| CclError::ParsingError(format!("Invalid float: {}", e)))?;
            Ok(ExpressionNode::Literal(LiteralNode::Float(value)))
        }
        Rule::did_literal => {
            let did_str = pair.as_str().to_string();
            Ok(ExpressionNode::Literal(LiteralNode::Did(did_str)))
        }
        Rule::timestamp_literal => {
            let timestamp_str = pair.as_str().to_string();
            Ok(ExpressionNode::Literal(LiteralNode::Timestamp(
                timestamp_str,
            )))
        }
        Rule::identifier => Ok(ExpressionNode::Identifier(pair.as_str().to_string())),
        _ => Err(CclError::ParsingError(format!(
            "Unexpected literal rule: {:?}",
            pair.as_rule()
        ))),
    }
}

/// Parse expressions with the new CCL 0.1 grammar
pub(crate) fn parse_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    match pair.as_rule() {
        Rule::expr => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| CclError::ParsingError("Empty expression".to_string()))?;
            parse_expression(inner)
        }
        Rule::logical_or => parse_logical_or(pair),
        Rule::logical_and => parse_logical_and(pair),
        Rule::equality => parse_equality(pair),
        Rule::comparison => parse_comparison(pair),
        Rule::addition => parse_addition(pair),
        Rule::multiplication => parse_multiplication(pair),
        Rule::unary => parse_unary(pair),
        Rule::postfix => parse_postfix(pair),
        Rule::primary => parse_primary(pair),
        Rule::literal => parse_literal_expression(pair),
        Rule::identifier => Ok(ExpressionNode::Identifier(pair.as_str().to_string())),
        Rule::integer => parse_literal_expression(pair),
        Rule::float => parse_literal_expression(pair),
        Rule::string => parse_literal_expression(pair),
        Rule::boolean => parse_literal_expression(pair),
        Rule::did_literal => parse_literal_expression(pair),
        Rule::timestamp_literal => parse_literal_expression(pair),
        Rule::array_literal => parse_array_literal(pair),
        Rule::struct_literal => parse_struct_literal(pair),
        Rule::enum_value => parse_enum_value(pair),
        Rule::some_expr => parse_some_expr(pair),
        Rule::none_expr => Ok(ExpressionNode::None),
        Rule::ok_expr => parse_ok_expr(pair),
        Rule::err_expr => parse_err_expr(pair),
        Rule::transfer_expr => parse_transfer_expr(pair),
        Rule::mint_expr => parse_mint_expr(pair),
        Rule::burn_expr => parse_burn_expr(pair),
        _ => Err(CclError::ParsingError(format!(
            "Unsupported expression rule: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_logical_or(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // logical_or = { logical_and ~ ("||" ~ logical_and)* }
    let mut pairs = pair.into_inner();
    let mut left = parse_expression(pairs.next().unwrap())?;

    while let Some(op_pair) = pairs.next() {
        if op_pair.as_str() == "||" {
            if let Some(right_pair) = pairs.next() {
                let right = parse_expression(right_pair)?;
                left = ExpressionNode::BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOperator::Or,
                    right: Box::new(right),
                };
            } else {
                return Err(CclError::ParsingError(
                    "Missing right operand in logical OR".to_string(),
                ));
            }
        } else {
            // This is an expression, not an operator
            let right = parse_expression(op_pair)?;
            left = ExpressionNode::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
    }

    Ok(left)
}

fn parse_logical_and(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // logical_and = { equality ~ ("&&" ~ equality)* }
    let mut pairs = pair.into_inner();
    let mut left = parse_expression(pairs.next().unwrap())?;

    while let Some(op_pair) = pairs.next() {
        if op_pair.as_str() == "&&" {
            if let Some(right_pair) = pairs.next() {
                let right = parse_expression(right_pair)?;
                left = ExpressionNode::BinaryOp {
                    left: Box::new(left),
                    operator: BinaryOperator::And,
                    right: Box::new(right),
                };
            } else {
                return Err(CclError::ParsingError(
                    "Missing right operand in logical AND".to_string(),
                ));
            }
        } else {
            // This is an expression, not an operator
            let right = parse_expression(op_pair)?;
            left = ExpressionNode::BinaryOp {
                left: Box::new(left),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }
    }

    Ok(left)
}

fn parse_equality(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // equality = { comparison ~ (("==" | "!=") ~ comparison)* }
    let inner_pairs: Vec<_> = pair.into_inner().collect();

    if inner_pairs.len() == 1 {
        // No operators, just a single comparison
        return parse_expression(inner_pairs[0].clone());
    }

    // Build left-associative binary operations
    // Format: [operand, operator, operand, operator, operand, ...]
    let mut result = parse_expression(inner_pairs[0].clone())?;

    let mut i = 1;
    while i < inner_pairs.len() {
        // Parse operator token
        if inner_pairs[i].as_rule() != Rule::equality_op {
            return Err(CclError::ParsingError(format!(
                "Expected equality_op rule, got: {:?}",
                inner_pairs[i].as_rule()
            )));
        }

        let operator_str = inner_pairs[i].as_str();
        let operator = match operator_str {
            "==" => BinaryOperator::Eq,
            "!=" => BinaryOperator::Neq,
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Expected equality operator, got: {}",
                    operator_str
                )));
            }
        };

        // Parse right operand
        if i + 1 >= inner_pairs.len() {
            return Err(CclError::ParsingError(
                "Missing right operand in equality".to_string(),
            ));
        }

        let right = parse_expression(inner_pairs[i + 1].clone())?;

        result = ExpressionNode::BinaryOp {
            left: Box::new(result),
            operator,
            right: Box::new(right),
        };

        i += 2; // Skip to next operator
    }

    Ok(result)
}

fn parse_comparison(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // comparison = { addition ~ (comparison_op ~ addition)* }
    let inner_pairs: Vec<_> = pair.into_inner().collect();

    if inner_pairs.len() == 1 {
        // No operators, just a single addition
        return parse_expression(inner_pairs[0].clone());
    }

    // Build left-associative binary operations
    // Format: [operand, operator, operand, operator, operand, ...]
    let mut result = parse_expression(inner_pairs[0].clone())?;

    let mut i = 1;
    while i < inner_pairs.len() {
        // Parse operator token
        if inner_pairs[i].as_rule() != Rule::comparison_op {
            return Err(CclError::ParsingError(format!(
                "Expected comparison_op rule, got: {:?}",
                inner_pairs[i].as_rule()
            )));
        }

        let operator_str = inner_pairs[i].as_str();
        let operator = match operator_str {
            "<=" => BinaryOperator::Lte,
            ">=" => BinaryOperator::Gte,
            "<" => BinaryOperator::Lt,
            ">" => BinaryOperator::Gt,
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Expected comparison operator, got: {}",
                    operator_str
                )));
            }
        };

        // Parse right operand
        if i + 1 >= inner_pairs.len() {
            return Err(CclError::ParsingError(
                "Missing right operand in comparison".to_string(),
            ));
        }

        let right = parse_expression(inner_pairs[i + 1].clone())?;

        result = ExpressionNode::BinaryOp {
            left: Box::new(result),
            operator,
            right: Box::new(right),
        };

        i += 2; // Skip to next operator
    }

    Ok(result)
}

fn parse_addition(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // addition = { multiplication ~ (addition_op ~ multiplication)* }
    let inner_pairs: Vec<_> = pair.into_inner().collect();

    if inner_pairs.len() == 1 {
        // No operators, just a single multiplication
        return parse_expression(inner_pairs[0].clone());
    }

    // Build left-associative binary operations
    // Format: [operand, operator, operand, operator, operand, ...]
    let mut result = parse_expression(inner_pairs[0].clone())?;

    let mut i = 1;
    while i < inner_pairs.len() {
        // Parse operator token
        if inner_pairs[i].as_rule() != Rule::addition_op {
            return Err(CclError::ParsingError(format!(
                "Expected addition_op rule, got: {:?}",
                inner_pairs[i].as_rule()
            )));
        }

        let operator_str = inner_pairs[i].as_str();
        let operator = match operator_str {
            "+" => BinaryOperator::Add,
            "-" => BinaryOperator::Sub,
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Expected addition operator, got: {}",
                    operator_str
                )));
            }
        };

        // Parse right operand
        if i + 1 >= inner_pairs.len() {
            return Err(CclError::ParsingError(
                "Missing right operand in addition".to_string(),
            ));
        }

        let right = parse_expression(inner_pairs[i + 1].clone())?;

        result = ExpressionNode::BinaryOp {
            left: Box::new(result),
            operator,
            right: Box::new(right),
        };

        i += 2; // Skip to next operator
    }

    Ok(result)
}

fn parse_multiplication(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // multiplication = { unary ~ (multiplication_op ~ unary)* }
    let inner_pairs: Vec<_> = pair.into_inner().collect();

    if inner_pairs.len() == 1 {
        // No operators, just a single unary
        return parse_expression(inner_pairs[0].clone());
    }

    // Build left-associative binary operations
    // Format: [operand, operator, operand, operator, operand, ...]
    let mut result = parse_expression(inner_pairs[0].clone())?;

    let mut i = 1;
    while i < inner_pairs.len() {
        // Parse operator token
        if inner_pairs[i].as_rule() != Rule::multiplication_op {
            return Err(CclError::ParsingError(format!(
                "Expected multiplication_op rule, got: {:?}",
                inner_pairs[i].as_rule()
            )));
        }

        let operator_str = inner_pairs[i].as_str();
        let operator = match operator_str {
            "*" => BinaryOperator::Mul,
            "/" => BinaryOperator::Div,
            "%" => BinaryOperator::Mod,
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Expected multiplication operator, got: {}",
                    operator_str
                )));
            }
        };

        // Parse right operand
        if i + 1 >= inner_pairs.len() {
            return Err(CclError::ParsingError(
                "Missing right operand in multiplication".to_string(),
            ));
        }

        let right = parse_expression(inner_pairs[i + 1].clone())?;

        result = ExpressionNode::BinaryOp {
            left: Box::new(result),
            operator,
            right: Box::new(right),
        };

        i += 2; // Skip to next operator
    }

    Ok(result)
}

fn parse_unary(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // unary = { ("!" | "-") ~ unary | postfix }
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_str() {
        "!" => {
            let operand = parse_expression(inner.next().unwrap())?;
            Ok(ExpressionNode::UnaryOp {
                operator: UnaryOperator::Not,
                operand: Box::new(operand),
            })
        }
        "-" => {
            let operand = parse_expression(inner.next().unwrap())?;
            Ok(ExpressionNode::UnaryOp {
                operator: UnaryOperator::Neg,
                operand: Box::new(operand),
            })
        }
        _ => {
            // This is a postfix expression
            parse_expression(first)
        }
    }
}

fn parse_postfix(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // postfix = { primary ~ (call_suffix | member_suffix | index_suffix)* }
    let mut inner = pair.into_inner();
    let mut expr = parse_expression(inner.next().unwrap())?;

    for suffix in inner {
        match suffix.as_rule() {
            Rule::call_suffix => {
                expr = parse_function_call_with_expr(expr, suffix)?;
            }
            Rule::member_suffix => {
                expr = parse_member_access_with_expr(expr, suffix)?;
            }
            Rule::index_suffix => {
                expr = parse_index_access_with_expr(expr, suffix)?;
            }
            _ => {
                return Err(CclError::ParsingError(format!(
                    "Unknown postfix rule: {:?}",
                    suffix.as_rule()
                )));
            }
        }
    }

    Ok(expr)
}

fn parse_primary(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // primary = { literal | identifier | "(" ~ expr ~ ")" | array_literal | struct_literal | some_expr | none_expr | ok_expr | err_expr }
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();

    match first.as_rule() {
        Rule::literal => parse_literal_expression(first),
        Rule::identifier => Ok(ExpressionNode::Identifier(first.as_str().to_string())),
        Rule::expr => parse_expression(first), // Parenthesized expression
        Rule::array_literal => parse_array_literal(first),
        Rule::struct_literal => parse_struct_literal(first),
        Rule::some_expr => parse_some_expr(first),
        Rule::none_expr => Ok(ExpressionNode::None),
        Rule::ok_expr => parse_ok_expr(first),
        Rule::err_expr => parse_err_expr(first),
        Rule::match_expr => parse_match_expression(first),
        _ => parse_expression(first),
    }
}

fn parse_array_literal(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // array_literal = { "[" ~ (expr ~ ("," ~ expr)*)? ~ "]" }
    let mut elements = Vec::new();

    for expr_pair in pair.into_inner() {
        if expr_pair.as_rule() == Rule::expr {
            let expr = parse_expression(expr_pair)?;
            elements.push(expr);
        }
    }

    Ok(ExpressionNode::ArrayLiteral(elements))
}

fn parse_enum_value(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // enum_value = { identifier ~ "::" ~ identifier }
    let mut inner = pair.into_inner();
    let enum_name = inner.next().unwrap().as_str().to_string();
    let variant = inner.next().unwrap().as_str().to_string();

    Ok(ExpressionNode::EnumValue { enum_name, variant })
}

fn parse_struct_literal(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // struct_literal = { identifier ~ "{" ~ field_init_list? ~ "}" }
    let mut inner = pair.into_inner();

    let type_name = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Struct literal missing type name".to_string()))?
        .as_str()
        .to_string();

    let mut fields = Vec::new();

    if let Some(field_list) = inner.next() {
        fields = parse_field_init_list(field_list)?;
    }

    Ok(ExpressionNode::StructLiteral { type_name, fields })
}

fn parse_field_init_list(pair: Pair<Rule>) -> Result<Vec<FieldInitNode>, CclError> {
    // field_init_list = { field_init ~ ("," ~ field_init)* }
    let mut fields = Vec::new();

    for field_pair in pair.into_inner() {
        if field_pair.as_rule() == Rule::field_init {
            let field = parse_field_init(field_pair)?;
            fields.push(field);
        }
    }

    Ok(fields)
}

fn parse_field_init(pair: Pair<Rule>) -> Result<FieldInitNode, CclError> {
    // field_init = { identifier ~ ":" ~ expr }
    let mut inner = pair.into_inner();

    let name = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Field init missing name".to_string()))?
        .as_str()
        .to_string();

    let value_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Field init missing value".to_string()))?;
    let value = parse_expression(value_pair)?;

    Ok(FieldInitNode { name, value })
}

fn parse_some_expr(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // some_expr = { "Some" ~ "(" ~ expr ~ ")" }
    let mut inner = pair.into_inner();
    let expr_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Some expression missing value".to_string()))?;
    let expr = parse_expression(expr_pair)?;
    Ok(ExpressionNode::Some(Box::new(expr)))
}

fn parse_ok_expr(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // ok_expr = { "Ok" ~ "(" ~ expr ~ ")" }
    let mut inner = pair.into_inner();
    let expr_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Ok expression missing value".to_string()))?;
    let expr = parse_expression(expr_pair)?;
    Ok(ExpressionNode::Ok(Box::new(expr)))
}

fn parse_err_expr(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // err_expr = { "Err" ~ "(" ~ expr ~ ")" }
    let mut inner = pair.into_inner();
    let expr_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Err expression missing value".to_string()))?;
    let expr = parse_expression(expr_pair)?;
    Ok(ExpressionNode::Err(Box::new(expr)))
}

fn parse_transfer_expr(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // transfer_expr = { "transfer" ~ "(" ~ expr ~ "," ~ expr ~ "," ~ expr ~ ")" }
    let mut inner = pair.into_inner();

    let from = parse_expression(inner.next().unwrap())?;
    let to = parse_expression(inner.next().unwrap())?;
    let amount = parse_expression(inner.next().unwrap())?;

    Ok(ExpressionNode::Transfer {
        from: Box::new(from),
        to: Box::new(to),
        amount: Box::new(amount),
    })
}

fn parse_mint_expr(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // mint_expr = { "mint" ~ "(" ~ expr ~ "," ~ expr ~ ")" }
    let mut inner = pair.into_inner();

    let to = parse_expression(inner.next().unwrap())?;
    let amount = parse_expression(inner.next().unwrap())?;

    Ok(ExpressionNode::Mint {
        to: Box::new(to),
        amount: Box::new(amount),
    })
}

fn parse_burn_expr(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // burn_expr = { "burn" ~ "(" ~ expr ~ "," ~ expr ~ ")" }
    let mut inner = pair.into_inner();

    let from = parse_expression(inner.next().unwrap())?;
    let amount = parse_expression(inner.next().unwrap())?;

    Ok(ExpressionNode::Burn {
        from: Box::new(from),
        amount: Box::new(amount),
    })
}

fn parse_function_call_with_expr(
    expr: ExpressionNode,
    suffix: Pair<Rule>,
) -> Result<ExpressionNode, CclError> {
    // call_suffix = { "(" ~ arg_list? ~ ")" }
    let mut args = Vec::new();

    for arg_pair in suffix.into_inner() {
        if arg_pair.as_rule() == Rule::arg_list {
            for arg in arg_pair.into_inner() {
                args.push(parse_expression(arg)?);
            }
        }
    }

    match expr {
        ExpressionNode::Identifier(name) => Ok(ExpressionNode::FunctionCall { name, args }),
        ExpressionNode::MemberAccess { object, member } => {
            // Handle method calls like numbers.length()
            Ok(ExpressionNode::MethodCall {
                object,
                method: member,
                args,
            })
        }
        _ => Err(CclError::ParsingError(
            "Function call only supported on identifiers and member access".to_string(),
        )),
    }
}

fn parse_member_access_with_expr(
    expr: ExpressionNode,
    suffix: Pair<Rule>,
) -> Result<ExpressionNode, CclError> {
    // member_suffix = { "." ~ identifier }
    let mut inner = suffix.into_inner();
    let member = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Member access missing member name".to_string()))?
        .as_str()
        .to_string();

    Ok(ExpressionNode::MemberAccess {
        object: Box::new(expr),
        member,
    })
}

fn parse_index_access_with_expr(
    expr: ExpressionNode,
    suffix: Pair<Rule>,
) -> Result<ExpressionNode, CclError> {
    // index_suffix = { "[" ~ expr ~ "]" }
    let mut inner = suffix.into_inner();
    let index_pair = inner
        .next()
        .ok_or_else(|| CclError::ParsingError("Index access missing index".to_string()))?;
    let index = parse_expression(index_pair)?;

    Ok(ExpressionNode::IndexAccess {
        object: Box::new(expr),
        index: Box::new(index),
    })
}

/// Parse statements with the new CCL 0.1 grammar
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
        Rule::let_stmt => parse_let_statement(actual_statement_pair),
        Rule::assignment_stmt => parse_assignment_statement(actual_statement_pair),
        Rule::if_stmt => parse_if_statement(actual_statement_pair),
        Rule::while_stmt => parse_while_statement(actual_statement_pair),
        Rule::for_stmt => parse_for_statement(actual_statement_pair),
        Rule::match_stmt => parse_match_statement(actual_statement_pair),
        Rule::return_stmt => parse_return_statement(actual_statement_pair),
        Rule::break_stmt => Ok(StatementNode::Break),
        Rule::continue_stmt => Ok(StatementNode::Continue),
        Rule::emit_stmt => parse_emit_statement(actual_statement_pair),
        Rule::require_stmt => parse_require_statement(actual_statement_pair),
        Rule::expr_stmt => parse_expr_statement(actual_statement_pair),
        _ => Err(CclError::ParsingError(format!(
            "Unsupported statement type: {:?}",
            actual_statement_pair.as_rule()
        ))),
    }
}

fn parse_let_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // let_stmt = { "let" ~ ("mut")? ~ identifier ~ (":" ~ type_expr)? ~ "=" ~ expr ~ ";" }
    let full_text = pair.as_str();
    let inner_pairs: Vec<_> = pair.into_inner().collect();

    // Check if the source text contains "mut" keyword (similar to binary operator fix)
    let mutable = full_text.contains(" mut ");

    // Get identifier (always the first inner pair)
    let name = inner_pairs[0].as_str().to_string();

    // Check for optional type annotation
    let mut type_expr = None;
    let mut value_index = 1;

    // If next item is a type_expr, parse it and skip to the next index
    if value_index < inner_pairs.len() && inner_pairs[value_index].as_rule() == Rule::type_expr {
        type_expr = Some(parse_type_expr(inner_pairs[value_index].clone())?);
        value_index += 1;
    }

    let value = if value_index < inner_pairs.len() {
        parse_expression(inner_pairs[value_index].clone())?
    } else {
        return Err(CclError::ParsingError(
            "Let statement missing value".to_string(),
        ));
    };

    Ok(StatementNode::Let {
        mutable,
        name,
        type_expr,
        value,
    })
}

fn parse_assignment_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // assignment_stmt = { identifier ~ "=" ~ expr ~ ";" | identifier ~ "[" ~ expr ~ "]" ~ "=" ~ expr ~ ";" | identifier ~ "." ~ identifier ~ "=" ~ expr ~ ";" }
    let inner = pair.into_inner();
    let tokens: Vec<_> = inner.collect();

    match tokens.len() {
        2 => {
            // Simple assignment: identifier ~ "=" ~ expr
            let identifier = tokens[0].as_str().to_string();
            let value = parse_expression(tokens[1].clone())?;

            let lvalue = LValueNode::Identifier(identifier);
            Ok(StatementNode::Assignment { lvalue, value })
        }
        3 => {
            // Check if this is member access or index access by looking at the pattern
            if tokens[1].as_rule() == Rule::expr {
                // Index assignment: identifier ~ "[" ~ expr ~ "]" ~ "=" ~ expr
                let identifier = tokens[0].as_str().to_string();
                let index = parse_expression(tokens[1].clone())?;
                let value = parse_expression(tokens[2].clone())?;

                let lvalue = LValueNode::IndexAccess {
                    object: Box::new(ExpressionNode::Identifier(identifier)),
                    index: Box::new(index),
                };
                Ok(StatementNode::Assignment { lvalue, value })
            } else {
                // Member access assignment: identifier ~ "." ~ identifier ~ "=" ~ expr
                let object_name = tokens[0].as_str().to_string();
                let member_name = tokens[1].as_str().to_string();
                let value = parse_expression(tokens[2].clone())?;

                let lvalue = LValueNode::MemberAccess {
                    object: Box::new(ExpressionNode::Identifier(object_name)),
                    member: member_name,
                };
                Ok(StatementNode::Assignment { lvalue, value })
            }
        }
        _ => Err(CclError::ParsingError(format!(
            "Unexpected assignment statement structure with {} tokens",
            tokens.len()
        ))),
    }
}

#[allow(dead_code)]
fn parse_lvalue(pair: Pair<Rule>) -> Result<LValueNode, CclError> {
    match pair.as_rule() {
        Rule::lvalue => {
            // Handle the wrapper lvalue rule by extracting the inner lvalue type
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| CclError::ParsingError("Empty lvalue rule".to_string()))?;
            parse_lvalue(inner)
        }
        Rule::identifier => Ok(LValueNode::Identifier(pair.as_str().to_string())),
        Rule::member_access => {
            let mut inner = pair.into_inner();
            let object_pair = inner.next().unwrap();
            let member_pair = inner.next().unwrap();

            let object = parse_expression(object_pair)?;
            let member = member_pair.as_str().to_string();

            Ok(LValueNode::MemberAccess {
                object: Box::new(object),
                member,
            })
        }
        Rule::index_access => {
            let mut inner = pair.into_inner();
            let object_pair = inner.next().unwrap();
            let index_pair = inner.next().unwrap();

            let object = parse_expression(object_pair)?;
            let index = parse_expression(index_pair)?;

            Ok(LValueNode::IndexAccess {
                object: Box::new(object),
                index: Box::new(index),
            })
        }
        _ => Err(CclError::ParsingError(format!(
            "Unexpected lvalue rule: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_if_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // if_stmt = { "if" ~ expr ~ block ~ ("else" ~ "if" ~ expr ~ block)* ~ ("else" ~ block)? }
    let mut inner = pair.into_inner();

    let condition = parse_expression(inner.next().unwrap())?;
    let then_block = parse_block(inner.next().unwrap())?;

    let mut else_ifs = Vec::new();
    let mut else_block = None;

    while let Some(item) = inner.next() {
        if item.as_str() == "else" {
            if let Some(next_item) = inner.next() {
                if next_item.as_str() == "if" {
                    // else if
                    let elif_condition = parse_expression(inner.next().unwrap())?;
                    let elif_block = parse_block(inner.next().unwrap())?;
                    else_ifs.push((elif_condition, elif_block));
                } else {
                    // else block
                    else_block = Some(parse_block(next_item)?);
                }
            }
        }
    }

    Ok(StatementNode::If {
        condition,
        then_block,
        else_ifs,
        else_block,
    })
}

fn parse_while_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // while_stmt = { "while" ~ expr ~ block }
    let mut inner = pair.into_inner();

    let condition = parse_expression(inner.next().unwrap())?;
    let body = parse_block(inner.next().unwrap())?;

    Ok(StatementNode::While { condition, body })
}

fn parse_for_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // for_stmt = { "for" ~ identifier ~ "in" ~ expr ~ block }
    let mut inner = pair.into_inner();

    let iterator = inner.next().unwrap().as_str().to_string();
    let iterable = parse_expression(inner.next().unwrap())?;
    let body = parse_block(inner.next().unwrap())?;

    Ok(StatementNode::For {
        iterator,
        iterable,
        body,
    })
}

fn parse_match_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // match_stmt = { "match" ~ expr ~ "{" ~ match_arm* ~ "}" }
    let mut inner = pair.into_inner();

    let expr = parse_expression(inner.next().unwrap())?;
    let mut arms = Vec::new();

    for arm_pair in inner {
        if arm_pair.as_rule() == Rule::match_arm {
            let arm = parse_match_arm(arm_pair)?;
            arms.push(arm);
        }
    }

    Ok(StatementNode::Match { expr, arms })
}

fn parse_match_arm(pair: Pair<Rule>) -> Result<MatchArmNode, CclError> {
    // For the statement context, we reuse the new match arm structure
    // but convert blocks to expressions as needed
    let mut inner = pair.into_inner();

    let pattern_pair = inner.next().unwrap();
    let pattern = parse_pattern(pattern_pair)?;

    let body_pair = inner.next().unwrap();
    let body = match body_pair.as_rule() {
        Rule::expr => parse_expression(body_pair)?,
        Rule::block => {
            // Convert block to a block expression (we'll need to handle this)
            // For now, we'll create a simple expression that represents the block
            ExpressionNode::Identifier("block_placeholder".to_string())
        }
        _ => return Err(CclError::ParsingError("Invalid match arm body".to_string())),
    };

    Ok(MatchArmNode {
        pattern,
        guard: None, // No guard support in old syntax
        body,
    })
}

fn parse_match_expression(pair: Pair<Rule>) -> Result<ExpressionNode, CclError> {
    // match_expr = { "match" ~ expr ~ "{" ~ match_arm+ ~ "}" }
    let mut inner = pair.into_inner();

    let expr = parse_expression(inner.next().unwrap())?;
    let mut arms = Vec::new();

    for arm_pair in inner {
        if arm_pair.as_rule() == Rule::match_arm {
            let arm = parse_match_arm_expr(arm_pair)?;
            arms.push(arm);
        }
    }

    Ok(ExpressionNode::Match {
        expr: Box::new(expr),
        arms,
    })
}

fn parse_match_arm_expr(pair: Pair<Rule>) -> Result<MatchArmNode, CclError> {
    // match_arm = { pattern ~ ("if" ~ expr)? ~ "=>" ~ expr ~ ","? }
    let mut inner = pair.into_inner();

    let pattern_pair = inner.next().unwrap();
    let pattern = parse_pattern(pattern_pair)?;

    // Check for optional guard
    let mut guard = None;
    let mut body_pair = inner.next().unwrap();

    if body_pair.as_str() == "if" {
        // There's a guard condition
        guard = Some(parse_expression(inner.next().unwrap())?);
        body_pair = inner.next().unwrap();
    }

    let body = parse_expression(body_pair)?;

    Ok(MatchArmNode {
        pattern,
        guard,
        body,
    })
}

fn parse_pattern(pair: Pair<Rule>) -> Result<PatternNode, CclError> {
    match pair.as_rule() {
        Rule::pattern => {
            // Unwrap the inner pattern
            let inner = pair.into_inner().next().unwrap();
            parse_pattern(inner)
        }
        Rule::literal_pattern => {
            let inner = pair.into_inner().next().unwrap();
            let literal = parse_literal_node(inner)?;
            Ok(PatternNode::Literal(literal))
        }
        Rule::variable_pattern => Ok(PatternNode::Variable(pair.as_str().to_string())),
        Rule::wildcard_pattern => Ok(PatternNode::Wildcard),
        Rule::struct_pattern => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let mut fields = Vec::new();

            if let Some(field_list) = inner.next() {
                fields = parse_struct_field_pattern_list(field_list)?;
            }

            Ok(PatternNode::Struct { name, fields })
        }
        Rule::enum_pattern => {
            let mut inner = pair.into_inner();
            let type_name = inner.next().unwrap().as_str().to_string();
            let variant = inner.next().unwrap().as_str().to_string();
            let inner_pattern = if let Some(pattern_list) = inner.next() {
                // Parse the pattern list and convert to single pattern if needed
                let patterns = parse_pattern_list(pattern_list)?;
                if patterns.len() == 1 {
                    Some(Box::new(patterns.into_iter().next().unwrap()))
                } else if patterns.is_empty() {
                    None
                } else {
                    Some(Box::new(PatternNode::Tuple(patterns)))
                }
            } else {
                None
            };
            Ok(PatternNode::Enum {
                type_name,
                variant,
                inner: inner_pattern,
            })
        }
        Rule::tuple_pattern => {
            let mut inner = pair.into_inner();
            let patterns = if let Some(pattern_list) = inner.next() {
                parse_pattern_list(pattern_list)?
            } else {
                Vec::new()
            };
            Ok(PatternNode::Tuple(patterns))
        }
        Rule::array_pattern => {
            let mut inner = pair.into_inner();
            let patterns = if let Some(pattern_list) = inner.next() {
                parse_pattern_list(pattern_list)?
            } else {
                Vec::new()
            };
            Ok(PatternNode::Array(patterns))
        }
        _ => Err(CclError::ParsingError(format!(
            "Unsupported pattern rule: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_pattern_list(pair: Pair<Rule>) -> Result<Vec<PatternNode>, CclError> {
    // pattern_list = { pattern ~ ("," ~ pattern)* }
    let mut patterns = Vec::new();

    for pattern_pair in pair.into_inner() {
        if pattern_pair.as_rule() == Rule::pattern {
            patterns.push(parse_pattern(pattern_pair)?);
        }
    }

    Ok(patterns)
}

fn parse_struct_field_pattern_list(pair: Pair<Rule>) -> Result<Vec<StructFieldPattern>, CclError> {
    // struct_field_pattern_list = { struct_field_pattern ~ ("," ~ struct_field_pattern)* }
    let mut fields = Vec::new();

    for field_pair in pair.into_inner() {
        if field_pair.as_rule() == Rule::struct_field_pattern {
            fields.push(parse_struct_field_pattern(field_pair)?);
        }
    }

    Ok(fields)
}

fn parse_struct_field_pattern(pair: Pair<Rule>) -> Result<StructFieldPattern, CclError> {
    // struct_field_pattern = { identifier ~ ":" ~ pattern | identifier }
    let mut inner = pair.into_inner();
    let name = inner.next().unwrap().as_str().to_string();

    if let Some(pattern_pair) = inner.next() {
        // Long form: field: pattern
        let pattern = parse_pattern(pattern_pair)?;
        Ok(StructFieldPattern {
            name,
            pattern: Some(pattern),
        })
    } else {
        // Short form: just field name
        Ok(StructFieldPattern {
            name,
            pattern: None,
        })
    }
}

fn parse_literal_node(pair: Pair<Rule>) -> Result<LiteralNode, CclError> {
    match pair.as_rule() {
        Rule::integer => Ok(LiteralNode::Integer(pair.as_str().parse().unwrap())),
        Rule::float => Ok(LiteralNode::Float(pair.as_str().parse().unwrap())),
        Rule::string => Ok(LiteralNode::String(unescape_string(
            pair.as_str().trim_matches('"'),
        )?)),
        Rule::boolean => Ok(LiteralNode::Boolean(pair.as_str() == "true")),
        Rule::did_literal => Ok(LiteralNode::Did(pair.as_str().to_string())),
        Rule::timestamp_literal => Ok(LiteralNode::Timestamp(pair.as_str().to_string())),
        _ => Err(CclError::ParsingError(format!(
            "Unsupported literal type: {:?}",
            pair.as_rule()
        ))),
    }
}

fn parse_return_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // return_stmt = { "return" ~ expr? ~ ";" }
    let mut inner = pair.into_inner();

    let expr = if let Some(expr_pair) = inner.next() {
        Some(parse_expression(expr_pair)?)
    } else {
        None
    };

    Ok(StatementNode::Return(expr))
}

fn parse_emit_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // emit_stmt = { "emit" ~ identifier ~ "{" ~ field_init_list? ~ "}" ~ ";" }
    let mut inner = pair.into_inner();

    let event_name = inner.next().unwrap().as_str().to_string();

    let mut fields = Vec::new();
    if let Some(field_list) = inner.next() {
        fields = parse_field_init_list(field_list)?;
    }

    Ok(StatementNode::Emit { event_name, fields })
}

fn parse_require_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // require_stmt = { "require" ~ "(" ~ expr ~ ")" ~ ";" }
    let mut inner = pair.into_inner();
    let expr_pair = inner.next().unwrap();
    let expr = parse_expression(expr_pair)?;
    Ok(StatementNode::Require(expr))
}

fn parse_expr_statement(pair: Pair<Rule>) -> Result<StatementNode, CclError> {
    // expr_stmt = { expr ~ ";" }
    let mut inner = pair.into_inner();
    let expr_pair = inner.next().unwrap();
    let expr = parse_expression(expr_pair)?;
    Ok(StatementNode::ExpressionStatement(expr))
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

/// Main parsing function for CCL 0.1 programs
pub fn parse_ccl_source(source: &str) -> Result<AstNode, CclError> {
    match CclParser::parse(Rule::program, source) {
        Ok(mut pairs) => {
            let program_content = pairs
                .next()
                .ok_or_else(|| CclError::ParsingError("Empty program source".to_string()))?;

            let mut top_level_nodes = Vec::new();

            for pair_in_program in program_content.into_inner() {
                match pair_in_program.as_rule() {
                    Rule::import_stmt => {
                        let import = parse_import_statement(pair_in_program)?;
                        top_level_nodes.push(crate::ast::TopLevelNode::Import(import));
                    }
                    Rule::contract_decl => {
                        let contract = parse_contract_declaration(pair_in_program)?;
                        top_level_nodes.push(crate::ast::TopLevelNode::Contract(contract));
                    }
                    Rule::fn_decl => {
                        let function = parse_function_definition_new(pair_in_program)?;
                        top_level_nodes.push(crate::ast::TopLevelNode::Function(function));
                    }
                    Rule::struct_decl => {
                        let struct_def = parse_struct_declaration(pair_in_program)?;
                        top_level_nodes.push(crate::ast::TopLevelNode::Struct(struct_def));
                    }
                    Rule::enum_decl => {
                        let enum_def = parse_enum_declaration(pair_in_program)?;
                        top_level_nodes.push(crate::ast::TopLevelNode::Enum(enum_def));
                    }
                    Rule::const_decl => {
                        let const_def = parse_const_declaration(pair_in_program)?;
                        top_level_nodes.push(crate::ast::TopLevelNode::Const(const_def));
                    }
                    Rule::EOI => (),
                    _ => {
                        return Err(CclError::ParsingError(format!(
                            "Unexpected rule in program: {:?}",
                            pair_in_program.as_rule()
                        )));
                    }
                }
            }

            if top_level_nodes.is_empty() {
                return Err(CclError::ParsingError(
                    "Program contained no items".to_string(),
                ));
            }

            Ok(AstNode::Program(top_level_nodes))
        }
        Err(e) => Err(CclError::ParsingError(format!("Pest parsing error: {}", e))),
    }
}

/// Parse a CCL file from disk, recursively loading any imported modules.
pub fn parse_ccl_file(path: &std::path::Path) -> Result<AstNode, CclError> {
    use std::fs;

    let source = fs::read_to_string(path)
        .map_err(|e| CclError::IoError(format!("Failed to read {}: {}", path.display(), e)))?;

    let base_dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let mut ast = parse_ccl_source(&source)?;
    load_imports(&mut ast, base_dir)?;
    Ok(ast)
}

fn alias_ast(ast: AstNode, alias: &str) -> AstNode {
    match ast {
        AstNode::FunctionDefinition {
            name,
            parameters,
            return_type,
            body,
            type_parameters,
        } => AstNode::FunctionDefinition {
            name: format!("{}_{}", alias, name),
            type_parameters,
            parameters,
            return_type,
            body,
        },
        AstNode::StructDefinition {
            name,
            fields,
            type_parameters,
        } => AstNode::StructDefinition {
            name: format!("{}_{}", alias, name),
            type_parameters,
            fields,
        },
        AstNode::Program(nodes) => {
            let aliased_nodes = nodes
                .into_iter()
                .map(|node| {
                    match node {
                        crate::ast::TopLevelNode::Import(import) => {
                            crate::ast::TopLevelNode::Import(import)
                        }
                        crate::ast::TopLevelNode::Contract(contract) => {
                            let aliased_contract = crate::ast::ContractDeclarationNode {
                                name: format!("{}_{}", alias, contract.name),
                                metadata: contract.metadata,
                                body: contract.body,
                            };
                            crate::ast::TopLevelNode::Contract(aliased_contract)
                        }
                        crate::ast::TopLevelNode::Function(function) => {
                            // TODO: Alias standalone functions if needed
                            crate::ast::TopLevelNode::Function(function)
                        }
                        crate::ast::TopLevelNode::Struct(struct_def) => {
                            // TODO: Alias standalone structs if needed
                            crate::ast::TopLevelNode::Struct(struct_def)
                        }
                        crate::ast::TopLevelNode::Enum(enum_def) => {
                            // TODO: Alias standalone enums if needed
                            crate::ast::TopLevelNode::Enum(enum_def)
                        }
                        crate::ast::TopLevelNode::Const(const_def) => {
                            // TODO: Alias standalone constants if needed
                            crate::ast::TopLevelNode::Const(const_def)
                        }
                    }
                })
                .collect();
            AstNode::Program(aliased_nodes)
        }
        other => other,
    }
}

fn load_imports(ast: &mut AstNode, base: &std::path::Path) -> Result<(), CclError> {
    match ast {
        AstNode::Program(ref mut nodes) => {
            let mut result = Vec::new();
            let mut imports_to_process = Vec::new();

            // Extract imports and other nodes
            for node in std::mem::take(nodes) {
                match node {
                    crate::ast::TopLevelNode::Import(import) => {
                        imports_to_process.push(import);
                    }
                    other => result.push(other),
                }
            }

            // Process imports
            for import in imports_to_process {
                let import_path = base.join(&import.path);
                let imported =
                    parse_ccl_file(&import_path).map_err(|e| CclError::ModuleImportError {
                        module: import.path.clone(),
                        reason: e.to_string(),
                    })?;

                let alias = import.alias.unwrap_or_else(|| {
                    // Generate default alias from filename
                    import_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("imported")
                        .to_string()
                });

                if let AstNode::Program(mut imported_nodes) = alias_ast(imported, &alias) {
                    result.append(&mut imported_nodes);
                }
            }

            *nodes = result;
        }
        AstNode::Policy(ref mut items) => {
            // Legacy policy handling
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
        _ => {}
    }
    Ok(())
}

// Note: TryExpr removed in CCL 0.1 - try/catch replaced with Result type

// Example test for the parser
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{AstNode, BlockNode, ExpressionNode, PolicyStatementNode, StatementNode};

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
                        type_parameters: vec![],
                        parameters: vec![],
                        return_type: Some(TypeExprNode::Integer),
                        body: BlockNode {
                            statements: vec![StatementNode::Return(Some(
                                ExpressionNode::IntegerLiteral(42),
                            ))],
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
