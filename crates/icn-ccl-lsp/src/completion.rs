use icn_ccl::StandardLibrary;
use tower_lsp::lsp_types::*;

pub fn get_completions(
    stdlib: &StandardLibrary,
    _text: &str,
    _position: Position,
) -> Vec<CompletionItem> {
    let mut completions = Vec::new();
    
    // Add all stdlib functions
    for (func_name, func_def) in stdlib.get_all_function_pairs() {
        let detail = format!("{} -> {}", 
            format_params(&func_def.params),
            format_return_type(&func_def.return_type)
        );
        
        completions.push(CompletionItem {
            label: func_name.clone(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(detail),
            documentation: Some(Documentation::String(func_def.description.clone())),
            insert_text: Some(format!("{}($0)", func_name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }
    
    // Add CCL keywords
    let keywords = vec![
        "fn", "let", "if", "else", "while", "for", "return", "state", "struct",
        "true", "false", "Integer", "String", "Bool", "Array", "did:key:"
    ];
    
    for keyword in keywords {
        completions.push(CompletionItem {
            label: keyword.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }
    
    // Add common governance patterns
    let governance_snippets = vec![
        ("proposal_template", "Proposal Template", 
         "let proposal = create_proposal(\"$1\", \"$2\", \"$3\");"),
        ("vote_template", "Vote Template",
         "vote_on_proposal($1, \"$2\", $3);"),
        ("delegation_template", "Delegation Template",
         "create_delegation($1, $2, \"$3\", $4);"),
        ("budget_template", "Budget Template",
         "let budget = create_budget(\"$1\", $2, \"$3\", $4, $5);"),
    ];
    
    for (label, detail, snippet) in governance_snippets {
        completions.push(CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some(detail.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        });
    }
    
    completions
}

fn format_params(params: &[icn_ccl::ast::TypeAnnotationNode]) -> String {
    if params.is_empty() {
        "()".to_string()
    } else {
        let param_strings: Vec<String> = params.iter().map(format_type).collect();
        format!("({})", param_strings.join(", "))
    }
}

fn format_return_type(return_type: &icn_ccl::ast::TypeAnnotationNode) -> String {
    format_type(return_type)
}

fn format_type(type_node: &icn_ccl::ast::TypeAnnotationNode) -> String {
    match type_node {
        icn_ccl::ast::TypeAnnotationNode::Integer => "Integer".to_string(),
        icn_ccl::ast::TypeAnnotationNode::String => "String".to_string(),
        icn_ccl::ast::TypeAnnotationNode::Bool => "Bool".to_string(),
        icn_ccl::ast::TypeAnnotationNode::Did => "Did".to_string(),
        icn_ccl::ast::TypeAnnotationNode::Mana => "Mana".to_string(),
        icn_ccl::ast::TypeAnnotationNode::Proposal => "Proposal".to_string(),
        icn_ccl::ast::TypeAnnotationNode::Vote => "Vote".to_string(),
        icn_ccl::ast::TypeAnnotationNode::Array(inner) => {
            format!("Array<{}>", format_type(inner))
        }
        icn_ccl::ast::TypeAnnotationNode::Custom(name) => name.clone(),
    }
}