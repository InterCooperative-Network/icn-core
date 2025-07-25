// icn-ccl/src/lsp/hover.rs
//! Hover information provider for CCL LSP

use tower_lsp::lsp_types::*;
use super::server::DocumentState;

/// Provide hover information for symbols at a given position
pub fn provide_hover(doc_state: &DocumentState, position: Position) -> Option<Hover> {
    // Get the word at the cursor position
    let word = get_word_at_position(&doc_state.text, position)?;
    
    // Try to provide hover information
    if let Some(info) = get_keyword_hover(&word) {
        return Some(info);
    }
    
    if let Some(info) = get_builtin_type_hover(&word) {
        return Some(info);
    }
    
    if let Some(info) = get_stdlib_function_hover(&word) {
        return Some(info);
    }
    
    // TODO: Add hover for user-defined symbols by analyzing the AST
    
    None
}

/// Extract the word at a given position in the text
fn get_word_at_position(text: &str, position: Position) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    if position.line as usize >= lines.len() {
        return None;
    }
    
    let line = lines[position.line as usize];
    let char_pos = position.character as usize;
    
    // Find word boundaries
    let chars: Vec<char> = line.chars().collect();
    
    // Check bounds using character count, not byte count
    if char_pos >= chars.len() {
        return None;
    }
    
    // Find start of word
    let mut start = char_pos;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }
    
    // Find end of word
    let mut end = char_pos;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }
    
    if start < end {
        Some(chars[start..end].iter().collect())
    } else {
        None
    }
}

/// Get hover information for CCL keywords
fn get_keyword_hover(word: &str) -> Option<Hover> {
    let info = match word {
        "contract" => ("Contract Declaration", "Defines a smart contract with state, functions, and governance rules. Contracts are the main building blocks of CCL applications."),
        "proposal" => ("Proposal Declaration", "Defines a governance proposal that stakeholders can vote on. Proposals can modify contract state, parameters, or execute arbitrary logic."),
        "role" => ("Role Declaration", "Defines a role with specific permissions and responsibilities within the contract. Roles determine what actions different parties can perform."),
        "policy" => ("Policy Statement", "Defines governance policies and rules that control how the contract operates and how decisions are made."),
        "state" => ("State Declaration", "Declares persistent state variables that are stored on the blockchain and maintained across contract executions."),
        "require" => ("Requirement Statement", "Asserts that a condition must be true. If the condition is false, the contract execution fails with an error message."),
        "function" => ("Function Declaration", "Defines a callable function that can be invoked by external callers or other functions within the contract."),
        "vote" => ("Vote Operation", "Casts a vote on a governance proposal. Votes can be in favor, against, or abstain."),
        "execute" => ("Execute Operation", "Executes a governance proposal that has passed voting requirements."),
        "delegate" => ("Delegate Operation", "Delegates voting power to another entity, allowing them to vote on your behalf."),
        "quorum" => ("Quorum Requirement", "Specifies the minimum number or percentage of participants required for a vote to be valid."),
        "threshold" => ("Threshold Requirement", "Specifies the minimum approval percentage required for a proposal to pass."),
        "deadline" => ("Deadline Specification", "Sets a time limit for voting or proposal execution."),
        "if" => ("Conditional Statement", "Executes code conditionally based on a boolean expression."),
        "else" => ("Else Clause", "Specifies alternative code to execute when an if condition is false."),
        "match" => ("Pattern Matching", "Performs pattern matching on values, similar to switch statements in other languages."),
        "for" => ("For Loop", "Iterates over collections or ranges."),
        "while" => ("While Loop", "Repeatedly executes code while a condition remains true."),
        "let" => ("Variable Declaration", "Declares an immutable variable."),
        "mut" => ("Mutable Modifier", "Makes a variable mutable, allowing its value to be changed after declaration."),
        "const" => ("Constant Declaration", "Declares a compile-time constant value."),
        "import" => ("Import Statement", "Imports functions, types, or modules from external CCL libraries."),
        "export" => ("Export Statement", "Exports functions or types to make them available to other contracts."),
        _ => return None,
    };
    
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("## {}\n\n{}", info.0, info.1),
        }),
        range: None,
    })
}

/// Get hover information for built-in types
fn get_builtin_type_hover(word: &str) -> Option<Hover> {
    let info = match word {
        "u32" => ("32-bit Unsigned Integer", "A 32-bit unsigned integer type, capable of storing values from 0 to 4,294,967,295."),
        "u64" => ("64-bit Unsigned Integer", "A 64-bit unsigned integer type, capable of storing values from 0 to 18,446,744,073,709,551,615."),
        "i32" => ("32-bit Signed Integer", "A 32-bit signed integer type, capable of storing values from -2,147,483,648 to 2,147,483,647."),
        "i64" => ("64-bit Signed Integer", "A 64-bit signed integer type, capable of storing values from -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807."),
        "f32" => ("32-bit Floating Point", "A 32-bit floating-point number type for decimal values."),
        "f64" => ("64-bit Floating Point", "A 64-bit floating-point number type for high-precision decimal values."),
        "bool" => ("Boolean", "A boolean type that can be either `true` or `false`."),
        "string" => ("String", "A UTF-8 encoded string type for text data."),
        "address" => ("Address", "A blockchain address type representing a unique identifier for accounts or contracts."),
        "duration" => ("Duration", "A type representing a span of time, typically specified with units like `1h`, `30m`, `7d`."),
        "timestamp" => ("Timestamp", "A type representing a specific point in time."),
        "bytes" => ("Byte Array", "A variable-length array of bytes for storing arbitrary binary data."),
        "did" => ("Decentralized Identifier", "A W3C standard decentralized identifier for identifying entities in a trustless manner."),
        "cid" => ("Content Identifier", "An IPFS content identifier for referencing immutable data."),
        "mana" => ("Mana Token", "The native token type used for resource allocation and governance in ICN."),
        _ => return None,
    };
    
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("## {}\n\n{}", info.0, info.1),
        }),
        range: None,
    })
}

/// Get hover information for standard library functions
fn get_stdlib_function_hover(word: &str) -> Option<Hover> {
    let info = match word {
        "log" => ("log(message: string)", "Logs a message for debugging purposes. The message will be recorded in the contract execution logs."),
        "require" => ("require(condition: bool, message: string)", "Asserts that a condition is true. If false, halts execution and displays the error message."),
        "transfer" => ("transfer(to: address, amount: mana)", "Transfers the specified amount of mana tokens to the target address."),
        "balance" => ("balance(account: address) -> mana", "Returns the current mana balance of the specified account."),
        "now" => ("now() -> timestamp", "Returns the current timestamp when the contract is being executed."),
        "hash" => ("hash(data: bytes) -> bytes", "Computes a cryptographic hash (SHA-256) of the input data."),
        "verify_signature" => ("verify_signature(message: bytes, signature: bytes, pubkey: bytes) -> bool", "Verifies a cryptographic signature against a message and public key."),
        "encode_json" => ("encode_json(data: any) -> string", "Encodes arbitrary data as a JSON string."),
        "decode_json" => ("decode_json(json: string) -> any", "Decodes a JSON string into structured data."),
        "get_caller" => ("get_caller() -> did", "Returns the decentralized identifier (DID) of the entity that called this contract."),
        "get_timestamp" => ("get_timestamp() -> timestamp", "Returns the timestamp of the current block or transaction."),
        _ => return None,
    };
    
    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: format!("## {}\n\n{}", info.0, info.1),
        }),
        range: None,
    })
}