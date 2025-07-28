// icn-ccl/src/lsp/completion.rs
//! Autocompletion support for CCL LSP

use super::server::DocumentState;
use tower_lsp::lsp_types::*;

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
        // Core language keywords
        ("contract", "Define a governance contract"),
        ("proposal", "Define a governance proposal type"),
        ("role", "Define a role with permissions"),
        ("policy", "Define a governance policy"),
        ("state", "Declare contract state variables"),
        ("function", "Define a callable function"),
        ("fn", "Short form of function"),
        // Control flow
        ("if", "Conditional statement"),
        ("else", "Alternative branch"),
        ("while", "Loop while condition is true"),
        ("for", "Iterate over a collection"),
        ("match", "Pattern matching"),
        ("return", "Return from function"),
        ("break", "Exit loop"),
        ("continue", "Skip to next iteration"),
        // Variable declarations
        ("let", "Declare a variable"),
        ("mut", "Mutable variable modifier"),
        ("const", "Declare a constant"),
        ("static", "Static variable"),
        // Module system
        ("import", "Import external module"),
        ("export", "Export for external use"),
        ("use", "Use items from module"),
        ("as", "Alias imported item"),
        ("mod", "Module declaration"),
        ("pub", "Public visibility"),
        // Type system
        ("enum", "Enumeration type"),
        ("struct", "Structure type"),
        ("trait", "Trait definition"),
        ("impl", "Implementation block"),
        // Governance actions
        ("vote", "Cast a vote"),
        ("execute", "Execute a proposal"),
        ("delegate", "Delegate voting power"),
        ("withdraw", "Withdraw funds"),
        ("deposit", "Deposit funds"),
        ("transfer", "Transfer tokens"),
        // Voting keywords
        ("approve", "Vote to approve"),
        ("reject", "Vote to reject"),
        ("abstain", "Abstain from voting"),
        ("quorum", "Minimum participation"),
        ("threshold", "Approval threshold"),
        ("deadline", "Time limit"),
        // Special keywords
        ("require", "Assert condition"),
        ("true", "Boolean true"),
        ("false", "Boolean false"),
        ("null", "Null value"),
        ("this", "Current contract"),
        ("self", "Self reference"),
    ];

    keywords
        .into_iter()
        .map(|(keyword, description)| CompletionItem {
            label: keyword.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(format!("CCL keyword: {}", keyword)),
            documentation: Some(Documentation::String(description.to_string())),
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
        // Core functions
        (
            "log",
            "log(message: string)",
            "Log a message to the console",
        ),
        (
            "require",
            "require(condition: bool, message: string)",
            "Assert a condition, panic with message if false",
        ),
        (
            "assert",
            "assert(condition: bool)",
            "Assert a condition, panic if false",
        ),
        ("panic", "panic(message: string)", "Panic with a message"),
        // Economic functions
        (
            "transfer",
            "transfer(to: address, amount: mana)",
            "Transfer mana tokens",
        ),
        (
            "balance",
            "balance(account: address) -> mana",
            "Get account balance",
        ),
        ("mint", "mint(to: address, amount: mana)", "Mint new tokens"),
        ("burn", "burn(amount: mana)", "Burn tokens"),
        (
            "total_supply",
            "total_supply() -> mana",
            "Get total token supply",
        ),
        // Time functions
        ("now", "now() -> timestamp", "Get current timestamp"),
        (
            "current_timestamp",
            "current_timestamp() -> timestamp",
            "Get current block timestamp",
        ),
        (
            "days",
            "days(n: u32) -> duration",
            "Create duration of n days",
        ),
        (
            "hours",
            "hours(n: u32) -> duration",
            "Create duration of n hours",
        ),
        (
            "minutes",
            "minutes(n: u32) -> duration",
            "Create duration of n minutes",
        ),
        (
            "seconds",
            "seconds(n: u32) -> duration",
            "Create duration of n seconds",
        ),
        // Cryptographic functions
        (
            "hash",
            "hash(data: bytes) -> bytes",
            "Compute SHA-256 hash of data",
        ),
        (
            "keccak256",
            "keccak256(data: bytes) -> bytes",
            "Compute Keccak-256 hash",
        ),
        (
            "verify_signature",
            "verify_signature(message: bytes, signature: bytes, pubkey: bytes) -> bool",
            "Verify cryptographic signature",
        ),
        (
            "recover_pubkey",
            "recover_pubkey(message: bytes, signature: bytes) -> bytes",
            "Recover public key from signature",
        ),
        // Data encoding/decoding
        (
            "encode_json",
            "encode_json(data: any) -> string",
            "Encode data as JSON",
        ),
        (
            "decode_json",
            "decode_json(json: string) -> any",
            "Decode JSON data",
        ),
        (
            "encode_hex",
            "encode_hex(data: bytes) -> string",
            "Encode bytes as hex string",
        ),
        (
            "decode_hex",
            "decode_hex(hex: string) -> bytes",
            "Decode hex string to bytes",
        ),
        // Context functions
        ("get_caller", "get_caller() -> did", "Get the caller's DID"),
        (
            "get_contract",
            "get_contract() -> address",
            "Get current contract address",
        ),
        (
            "get_block_number",
            "get_block_number() -> u64",
            "Get current block number",
        ),
        (
            "get_block_hash",
            "get_block_hash(number: u64) -> bytes",
            "Get block hash",
        ),
        // Collection functions
        (
            "len",
            "len(collection: any) -> u32",
            "Get length of collection",
        ),
        (
            "is_empty",
            "is_empty(collection: any) -> bool",
            "Check if collection is empty",
        ),
        (
            "contains",
            "contains(collection: any, item: any) -> bool",
            "Check if collection contains item",
        ),
        (
            "push",
            "push(collection: any, item: any)",
            "Add item to collection",
        ),
        (
            "pop",
            "pop(collection: any) -> any",
            "Remove and return last item",
        ),
        // String functions
        (
            "to_upper",
            "to_upper(text: string) -> string",
            "Convert to uppercase",
        ),
        (
            "to_lower",
            "to_lower(text: string) -> string",
            "Convert to lowercase",
        ),
        ("trim", "trim(text: string) -> string", "Remove whitespace"),
        (
            "substring",
            "substring(text: string, start: u32, end: u32) -> string",
            "Extract substring",
        ),
        (
            "concat",
            "concat(a: string, b: string) -> string",
            "Concatenate strings",
        ),
        // Governance functions
        (
            "create_proposal",
            "create_proposal(title: string, description: string) -> proposal_id",
            "Create a new proposal",
        ),
        (
            "cast_vote",
            "cast_vote(proposal_id: string, vote: vote_type)",
            "Cast a vote on proposal",
        ),
        (
            "tally_votes",
            "tally_votes(proposal_id: string) -> vote_result",
            "Count votes for proposal",
        ),
        (
            "check_quorum",
            "check_quorum(proposal_id: string) -> bool",
            "Check if quorum is met",
        ),
        (
            "execute_proposal",
            "execute_proposal(proposal_id: string)",
            "Execute approved proposal",
        ),
        // DAG storage functions
        (
            "dag_put",
            "dag_put(data: bytes) -> cid",
            "Store data in DAG",
        ),
        (
            "dag_get",
            "dag_get(cid: cid) -> bytes",
            "Retrieve data from DAG",
        ),
        ("dag_pin", "dag_pin(cid: cid)", "Pin content in DAG"),
        ("dag_unpin", "dag_unpin(cid: cid)", "Unpin content from DAG"),
        (
            "calculate_cid",
            "calculate_cid(data: bytes) -> cid",
            "Calculate CID for data",
        ),
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
