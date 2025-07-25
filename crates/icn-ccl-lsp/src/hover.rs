use icn_ccl::StandardLibrary;
use tower_lsp::lsp_types::*;

pub fn get_hover_info(
    stdlib: &StandardLibrary,
    text: &str,
    position: Position,
) -> Option<Hover> {
    let lines: Vec<&str> = text.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }
    
    let line = lines[position.line as usize];
    let word = extract_word_at_position(line, position.character as usize)?;
    
    // Check if it's a stdlib function
    if let Some(func_def) = stdlib.get_function(&word) {
        let signature = format!("{}{} -> {}",
            word,
            format_params(&func_def.params),
            format_return_type(&func_def.return_type)
        );
        
        let category_emoji = match func_def.category {
            icn_ccl::stdlib::StdCategory::Governance => "🏛️",
            icn_ccl::stdlib::StdCategory::Economics => "💰",
            icn_ccl::stdlib::StdCategory::Identity => "🆔",
            icn_ccl::stdlib::StdCategory::Utility => "🔧",
            icn_ccl::stdlib::StdCategory::String => "📝",
            icn_ccl::stdlib::StdCategory::Array => "📋",
            icn_ccl::stdlib::StdCategory::Map => "🗺️",
            icn_ccl::stdlib::StdCategory::Math => "🧮",
            icn_ccl::stdlib::StdCategory::Crypto => "🔐",
        };
        
        let content = format!(
            "{} **{}**\n\n```ccl\n{}\n```\n\n{}\n\n*Category: {} {:?}*",
            category_emoji,
            word,
            signature,
            func_def.description,
            category_emoji,
            func_def.category
        );
        
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: content,
            }),
            range: Some(Range {
                start: Position {
                    line: position.line,
                    character: position.character.saturating_sub(word.len() as u32),
                },
                end: Position {
                    line: position.line,
                    character: position.character + (word.len() as u32 - (position.character as usize).min(word.len())) as u32,
                },
            }),
        });
    }
    
    // Check for CCL keywords
    let keyword_docs = get_keyword_documentation(&word);
    if let Some(doc) = keyword_docs {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: doc,
            }),
            range: None,
        });
    }
    
    None
}

fn extract_word_at_position(line: &str, character: usize) -> Option<String> {
    if character >= line.len() {
        return None;
    }
    
    let chars: Vec<char> = line.chars().collect();
    
    // Find word boundaries
    let mut start = character;
    let mut end = character;
    
    // Move start backward to find beginning of word
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    
    // Move end forward to find end of word
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    
    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

fn format_params(params: &[icn_ccl::ast::TypeAnnotationNode]) -> String {
    if params.is_empty() {
        "()".to_string()
    } else {
        let param_strings: Vec<String> = params.iter().enumerate().map(|(i, param)| {
            format!("param{}: {}", i + 1, format_type(param))
        }).collect();
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

fn get_keyword_documentation(keyword: &str) -> Option<String> {
    match keyword {
        "fn" => Some("**fn** - Function declaration\n\nDeclares a function with parameters and return type.\n\n```ccl\nfn function_name(param1: Type, param2: Type) -> ReturnType {\n    // function body\n}\n```".to_string()),
        "let" => Some("**let** - Variable binding\n\nBinds a value to a variable name.\n\n```ccl\nlet variable_name = value;\n```".to_string()),
        "if" => Some("**if** - Conditional expression\n\nExecutes code based on a boolean condition.\n\n```ccl\nif condition {\n    // code\n} else {\n    // alternative code\n}\n```".to_string()),
        "while" => Some("**while** - Loop construct\n\nRepeats code while a condition is true.\n\n```ccl\nwhile condition {\n    // loop body\n}\n```".to_string()),
        "return" => Some("**return** - Return statement\n\nReturns a value from a function.\n\n```ccl\nreturn value;\n```".to_string()),
        "state" => Some("**state** - Contract state declaration\n\nDeclares persistent contract state.\n\n```ccl\nstate variable_name: Type;\n```".to_string()),
        _ => None,
    }
}