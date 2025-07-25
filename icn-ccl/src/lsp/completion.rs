// icn-ccl/src/lsp/completion.rs
//! Autocompletion support for CCL LSP

use tower_lsp::lsp_types::*;
use super::server::DocumentState;

/// Provide autocompletion items for CCL
pub fn provide_completions(doc_state: &DocumentState, position: Position) -> Vec<CompletionItem> {
    let mut completions = Vec::new();

    // Add CCL keywords
    completions.extend(get_keyword_completions());
    
    // Add built-in types
    completions.extend(get_builtin_type_completions());
    
    // Add standard library functions
    completions.extend(get_stdlib_completions());

    // Add context-aware completions based on current scope
    if let Some(ref ast) = doc_state.ast {
        completions.extend(get_scope_completions(ast, position));
    }

    completions
}

/// Get completion items for CCL keywords
fn get_keyword_completions() -> Vec<CompletionItem> {
    let keywords = vec![
        "contract", "proposal", "role", "policy", "state", "require", "enum",
        "struct", "function", "if", "else", "match", "for", "while", "let", "mut",
        "const", "import", "export", "as", "true", "false", "null", "this",
        "vote", "execute", "delegate", "withdraw", "deposit", "transfer",
        "approve", "reject", "abstain", "quorum", "threshold", "deadline",
    ];

    keywords
        .into_iter()
        .map(|keyword| CompletionItem {
            label: keyword.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(format!("CCL keyword: {}", keyword)),
            documentation: Some(Documentation::String(get_keyword_documentation(keyword))),
            insert_text: Some(keyword.to_string()),
            ..Default::default()
        })
        .collect()
}

/// Get completion items for built-in types
fn get_builtin_type_completions() -> Vec<CompletionItem> {
    let types = vec![
        ("u32", "32-bit unsigned integer"),
        ("u64", "64-bit unsigned integer"),
        ("i32", "32-bit signed integer"),
        ("i64", "64-bit signed integer"),
        ("f32", "32-bit floating point"),
        ("f64", "64-bit floating point"),
        ("bool", "Boolean type"),
        ("string", "String type"),
        ("address", "Blockchain address"),
        ("duration", "Time duration"),
        ("timestamp", "Point in time"),
        ("bytes", "Byte array"),
        ("did", "Decentralized identifier"),
        ("cid", "Content identifier"),
        ("mana", "Mana token amount"),
    ];

    types
        .into_iter()
        .map(|(type_name, description)| CompletionItem {
            label: type_name.to_string(),
            kind: Some(CompletionItemKind::TYPE_PARAMETER),
            detail: Some(format!("Built-in type: {}", type_name)),
            documentation: Some(Documentation::String(description.to_string())),
            insert_text: Some(type_name.to_string()),
            ..Default::default()
        })
        .collect()
}

/// Get completion items for standard library functions
fn get_stdlib_completions() -> Vec<CompletionItem> {
    let functions = vec![
        ("log", "log(message: string)", "Log a message"),
        ("require", "require(condition: bool, message: string)", "Assert a condition"),
        ("transfer", "transfer(to: address, amount: mana)", "Transfer mana"),
        ("balance", "balance(account: address) -> mana", "Get account balance"),
        ("now", "now() -> timestamp", "Get current timestamp"),
        ("hash", "hash(data: bytes) -> bytes", "Compute hash of data"),
        ("verify_signature", "verify_signature(message: bytes, signature: bytes, pubkey: bytes) -> bool", "Verify cryptographic signature"),
        ("encode_json", "encode_json(data: any) -> string", "Encode data as JSON"),
        ("decode_json", "decode_json(json: string) -> any", "Decode JSON data"),
        ("get_caller", "get_caller() -> did", "Get the caller's DID"),
        ("get_timestamp", "get_timestamp() -> timestamp", "Get current block timestamp"),
    ];

    functions
        .into_iter()
        .map(|(name, signature, description)| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(signature.to_string()),
            documentation: Some(Documentation::String(description.to_string())),
            insert_text: Some(format!("{}(${{1}})", name)),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        })
        .collect()
}

/// Get scope-aware completions based on AST context
fn get_scope_completions(_ast: &crate::ast::AstNode, _position: Position) -> Vec<CompletionItem> {
    // TODO: Implement scope-aware completions by analyzing the AST
    // This would include:
    // - Local variables in scope
    // - Function parameters
    // - Struct fields
    // - Contract members
    // - Imported symbols
    
    Vec::new()
}

/// Get documentation for a keyword
fn get_keyword_documentation(keyword: &str) -> String {
    match keyword {
        "contract" => "Defines a contract with governance rules and state management".to_string(),
        "proposal" => "Defines a governance proposal that can be voted on".to_string(),
        "role" => "Defines a role with specific permissions and responsibilities".to_string(),
        "policy" => "Defines a governance policy or rule".to_string(),
        "state" => "Declares contract state variables".to_string(),
        "require" => "Asserts a condition, failing if false".to_string(),
        "function" => "Defines a callable function".to_string(),
        "vote" => "Cast a vote on a proposal".to_string(),
        "execute" => "Execute a passed proposal".to_string(),
        "delegate" => "Delegate voting power to another entity".to_string(),
        "quorum" => "Minimum participation required for a vote".to_string(),
        "threshold" => "Minimum approval required for passage".to_string(),
        "deadline" => "Time limit for voting or execution".to_string(),
        _ => format!("CCL keyword: {}", keyword),
    }
}