// icn-ccl/src/stdlib.rs
//! CCL Standard Library
//! 
//! This module provides built-in functions and utilities for CCL contracts,
//! including governance primitives, economic operations, and utility functions.

use crate::ast::TypeAnnotationNode;
use std::collections::HashMap;

/// Standard library function signature
#[derive(Debug, Clone)]
pub struct StdFunction {
    pub name: String,
    pub params: Vec<TypeAnnotationNode>,
    pub return_type: TypeAnnotationNode,
    pub description: String,
    pub category: StdCategory,
}

/// Categories of standard library functions
#[derive(Debug, Clone, PartialEq)]
pub enum StdCategory {
    Governance,
    Economics,
    Utility,
    String,
    Array,
    Map, // Add Map category
    Math,
    Crypto,
}

/// CCL Standard Library
pub struct StdLibrary {
    functions: HashMap<String, StdFunction>,
}

impl StdLibrary {
    /// Create a new standard library instance with all built-in functions
    pub fn new() -> Self {
        let mut stdlib = StdLibrary {
            functions: HashMap::new(),
        };
        
        stdlib.register_governance_functions();
        stdlib.register_economic_functions();
        stdlib.register_utility_functions();
        stdlib.register_string_functions();
        stdlib.register_array_functions();
        stdlib.register_map_functions(); // Add map functions
        stdlib.register_math_functions();
        stdlib.register_crypto_functions();
        
        stdlib
    }

    /// Get all function names by category
    pub fn get_functions_by_category(&self, category: StdCategory) -> Vec<&StdFunction> {
        self.functions
            .values()
            .filter(|f| f.category == category)
            .collect()
    }

    /// Look up a function by name
    pub fn get_function(&self, name: &str) -> Option<&StdFunction> {
        self.functions.get(name)
    }

    /// Get all function names
    pub fn get_all_functions(&self) -> Vec<&String> {
        self.functions.keys().collect()
    }

    /// Get all function name-function pairs
    pub fn get_all_function_pairs(&self) -> Vec<(&String, &StdFunction)> {
        self.functions.iter().collect()
    }

    fn register_function(&mut self, func: StdFunction) {
        self.functions.insert(func.name.clone(), func);
    }

    /// Register governance-related functions
    fn register_governance_functions(&mut self) {
        // Proposal management
        self.register_function(StdFunction {
            name: "create_proposal".to_string(),
            params: vec![
                TypeAnnotationNode::String, // title
                TypeAnnotationNode::String, // description
                TypeAnnotationNode::Custom("ProposalType".to_string()),
            ],
            return_type: TypeAnnotationNode::Proposal,
            description: "Create a new governance proposal".to_string(),
            category: StdCategory::Governance,
        });

        self.register_function(StdFunction {
            name: "submit_proposal".to_string(),
            params: vec![TypeAnnotationNode::Proposal],
            return_type: TypeAnnotationNode::Bool,
            description: "Submit a proposal for voting".to_string(),
            category: StdCategory::Governance,
        });

        self.register_function(StdFunction {
            name: "vote_on_proposal".to_string(),
            params: vec![
                TypeAnnotationNode::Proposal,
                TypeAnnotationNode::Vote,
                TypeAnnotationNode::Did, // voter
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Cast a vote on a proposal".to_string(),
            category: StdCategory::Governance,
        });

        self.register_function(StdFunction {
            name: "execute_proposal".to_string(),
            params: vec![TypeAnnotationNode::Proposal],
            return_type: TypeAnnotationNode::Bool,
            description: "Execute a passed proposal".to_string(),
            category: StdCategory::Governance,
        });

        // Role management
        self.register_function(StdFunction {
            name: "has_role".to_string(),
            params: vec![
                TypeAnnotationNode::Did,
                TypeAnnotationNode::String, // role name
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Check if a user has a specific role".to_string(),
            category: StdCategory::Governance,
        });

        self.register_function(StdFunction {
            name: "assign_role".to_string(),
            params: vec![
                TypeAnnotationNode::Did,
                TypeAnnotationNode::String, // role name
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Assign a role to a user".to_string(),
            category: StdCategory::Governance,
        });

        self.register_function(StdFunction {
            name: "revoke_role".to_string(),
            params: vec![
                TypeAnnotationNode::Did,
                TypeAnnotationNode::String, // role name
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Revoke a role from a user".to_string(),
            category: StdCategory::Governance,
        });

        // Permission checking
        self.register_function(StdFunction {
            name: "check_permission".to_string(),
            params: vec![
                TypeAnnotationNode::Did,
                TypeAnnotationNode::String, // permission name
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Check if a user has a specific permission".to_string(),
            category: StdCategory::Governance,
        });
    }

    /// Register economic/financial functions
    fn register_economic_functions(&mut self) {
        // Mana operations
        self.register_function(StdFunction {
            name: "get_balance".to_string(),
            params: vec![TypeAnnotationNode::Did],
            return_type: TypeAnnotationNode::Mana,
            description: "Get the mana balance of an account".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "transfer".to_string(),
            params: vec![
                TypeAnnotationNode::Did, // from
                TypeAnnotationNode::Did, // to
                TypeAnnotationNode::Mana, // amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Transfer mana between accounts".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "mint_mana".to_string(),
            params: vec![
                TypeAnnotationNode::Did, // to
                TypeAnnotationNode::Mana, // amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Mint new mana to an account".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "burn_mana".to_string(),
            params: vec![
                TypeAnnotationNode::Did, // from
                TypeAnnotationNode::Mana, // amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Burn mana from an account".to_string(),
            category: StdCategory::Economics,
        });

        // Economic calculations
        self.register_function(StdFunction {
            name: "calculate_fee".to_string(),
            params: vec![
                TypeAnnotationNode::Mana, // base amount
                TypeAnnotationNode::Integer, // fee percentage (basis points)
            ],
            return_type: TypeAnnotationNode::Mana,
            description: "Calculate transaction fee".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "compound_interest".to_string(),
            params: vec![
                TypeAnnotationNode::Mana, // principal
                TypeAnnotationNode::Integer, // rate (basis points)
                TypeAnnotationNode::Integer, // periods
            ],
            return_type: TypeAnnotationNode::Mana,
            description: "Calculate compound interest".to_string(),
            category: StdCategory::Economics,
        });

        // Reputation and credit
        self.register_function(StdFunction {
            name: "get_reputation".to_string(),
            params: vec![TypeAnnotationNode::Did],
            return_type: TypeAnnotationNode::Integer,
            description: "Get user's reputation score".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "update_reputation".to_string(),
            params: vec![
                TypeAnnotationNode::Did,
                TypeAnnotationNode::Integer, // delta
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Update user's reputation score".to_string(),
            category: StdCategory::Economics,
        });
    }

    /// Register utility functions
    fn register_utility_functions(&mut self) {
        // Time functions
        self.register_function(StdFunction {
            name: "now".to_string(),
            params: vec![],
            return_type: TypeAnnotationNode::Custom("Timestamp".to_string()),
            description: "Get current timestamp".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "add_duration".to_string(),
            params: vec![
                TypeAnnotationNode::Custom("Timestamp".to_string()),
                TypeAnnotationNode::Custom("Duration".to_string()),
            ],
            return_type: TypeAnnotationNode::Custom("Timestamp".to_string()),
            description: "Add duration to timestamp".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "days".to_string(),
            params: vec![TypeAnnotationNode::Integer],
            return_type: TypeAnnotationNode::Custom("Duration".to_string()),
            description: "Create duration from days".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "hours".to_string(),
            params: vec![TypeAnnotationNode::Integer],
            return_type: TypeAnnotationNode::Custom("Duration".to_string()),
            description: "Create duration from hours".to_string(),
            category: StdCategory::Utility,
        });

        // Validation functions
        self.register_function(StdFunction {
            name: "require".to_string(),
            params: vec![
                TypeAnnotationNode::Bool,
                TypeAnnotationNode::String, // error message
            ],
            return_type: TypeAnnotationNode::Custom("void".to_string()),
            description: "Assert condition with custom error message".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "is_valid_did".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::Bool,
            description: "Validate DID format".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "validate_did".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::Bool,
            description: "Validate DID format (alias for is_valid_did)".to_string(),
            category: StdCategory::Utility,
        });

        // Logging and events
        self.register_function(StdFunction {
            name: "log".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::Custom("void".to_string()),
            description: "Log a message".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "emit_event".to_string(),
            params: vec![
                TypeAnnotationNode::String, // event name
                TypeAnnotationNode::String, // event data (JSON)
            ],
            return_type: TypeAnnotationNode::Custom("void".to_string()),
            description: "Emit a custom event".to_string(),
            category: StdCategory::Utility,
        });
    }

    /// Register string manipulation functions
    fn register_string_functions(&mut self) {
        self.register_function(StdFunction {
            name: "string_length".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::Integer,
            description: "Get string length".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_concat".to_string(),
            params: vec![TypeAnnotationNode::String, TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::String,
            description: "Concatenate two strings".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_substring".to_string(),
            params: vec![
                TypeAnnotationNode::String,
                TypeAnnotationNode::Integer, // start
                TypeAnnotationNode::Integer, // length
            ],
            return_type: TypeAnnotationNode::String,
            description: "Extract substring".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_contains".to_string(),
            params: vec![TypeAnnotationNode::String, TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::Bool,
            description: "Check if string contains substring".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_to_upper".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::String,
            description: "Convert string to uppercase".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_to_lower".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::String,
            description: "Convert string to lowercase".to_string(),
            category: StdCategory::String,
        });
    }

    /// Register array manipulation functions
    fn register_array_functions(&mut self) {
        self.register_function(StdFunction {
            name: "array_length".to_string(),
            params: vec![TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer))], // More specific type
            return_type: TypeAnnotationNode::Integer,
            description: "Get array length".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_push".to_string(),
            params: vec![
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)), // More specific type
                TypeAnnotationNode::Integer,
            ],
            return_type: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)),
            description: "Add element to end of array".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_pop".to_string(),
            params: vec![TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer))], // More specific type
            return_type: TypeAnnotationNode::Option(Box::new(TypeAnnotationNode::Integer)),
            description: "Remove and return last element".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_contains".to_string(),
            params: vec![
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)), // More specific type
                TypeAnnotationNode::Integer,
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Check if array contains element".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_slice".to_string(),
            params: vec![
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)), // More specific type
                TypeAnnotationNode::Integer, // start
                TypeAnnotationNode::Integer, // end
            ],
            return_type: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)),
            description: "Extract slice of array".to_string(),
            category: StdCategory::Array,
        });
    }

    /// Register map/dictionary manipulation functions
    fn register_map_functions(&mut self) {
        self.register_function(StdFunction {
            name: "map_new".to_string(),
            params: vec![],
            return_type: TypeAnnotationNode::Map {
                key_type: Box::new(TypeAnnotationNode::String),
                value_type: Box::new(TypeAnnotationNode::Integer),
            },
            description: "Create a new empty map".to_string(),
            category: StdCategory::Map,
        });

        self.register_function(StdFunction {
            name: "map_insert".to_string(),
            params: vec![
                TypeAnnotationNode::Map {
                    key_type: Box::new(TypeAnnotationNode::String), // More specific
                    value_type: Box::new(TypeAnnotationNode::Integer), // More specific
                },
                TypeAnnotationNode::String, // More specific
                TypeAnnotationNode::Integer, // More specific
            ],
            return_type: TypeAnnotationNode::Map {
                key_type: Box::new(TypeAnnotationNode::String),
                value_type: Box::new(TypeAnnotationNode::Integer),
            },
            description: "Insert key-value pair into map".to_string(),
            category: StdCategory::Map,
        });

        self.register_function(StdFunction {
            name: "map_get".to_string(),
            params: vec![
                TypeAnnotationNode::Map {
                    key_type: Box::new(TypeAnnotationNode::String),
                    value_type: Box::new(TypeAnnotationNode::Integer),
                },
                TypeAnnotationNode::String,
            ],
            return_type: TypeAnnotationNode::Option(Box::new(TypeAnnotationNode::Integer)),
            description: "Get value by key from map".to_string(),
            category: StdCategory::Map,
        });

        self.register_function(StdFunction {
            name: "map_contains_key".to_string(),
            params: vec![
                TypeAnnotationNode::Map {
                    key_type: Box::new(TypeAnnotationNode::String),
                    value_type: Box::new(TypeAnnotationNode::Integer),
                },
                TypeAnnotationNode::String,
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Check if map contains key".to_string(),
            category: StdCategory::Map,
        });

        self.register_function(StdFunction {
            name: "map_remove".to_string(),
            params: vec![
                TypeAnnotationNode::Map {
                    key_type: Box::new(TypeAnnotationNode::String),
                    value_type: Box::new(TypeAnnotationNode::Integer),
                },
                TypeAnnotationNode::String,
            ],
            return_type: TypeAnnotationNode::Option(Box::new(TypeAnnotationNode::Integer)),
            description: "Remove key-value pair from map".to_string(),
            category: StdCategory::Map,
        });

        self.register_function(StdFunction {
            name: "map_size".to_string(),
            params: vec![
                TypeAnnotationNode::Map {
                    key_type: Box::new(TypeAnnotationNode::String),
                    value_type: Box::new(TypeAnnotationNode::Integer),
                },
            ],
            return_type: TypeAnnotationNode::Integer,
            description: "Get number of entries in map".to_string(),
            category: StdCategory::Map,
        });
    }

    /// Register mathematical functions
    fn register_math_functions(&mut self) {
        self.register_function(StdFunction {
            name: "abs".to_string(),
            params: vec![TypeAnnotationNode::Integer],
            return_type: TypeAnnotationNode::Integer,
            description: "Absolute value".to_string(),
            category: StdCategory::Math,
        });

        self.register_function(StdFunction {
            name: "min".to_string(),
            params: vec![TypeAnnotationNode::Integer, TypeAnnotationNode::Integer],
            return_type: TypeAnnotationNode::Integer,
            description: "Minimum of two values".to_string(),
            category: StdCategory::Math,
        });

        self.register_function(StdFunction {
            name: "max".to_string(),
            params: vec![TypeAnnotationNode::Integer, TypeAnnotationNode::Integer],
            return_type: TypeAnnotationNode::Integer,
            description: "Maximum of two values".to_string(),
            category: StdCategory::Math,
        });

        self.register_function(StdFunction {
            name: "pow".to_string(),
            params: vec![TypeAnnotationNode::Integer, TypeAnnotationNode::Integer],
            return_type: TypeAnnotationNode::Integer,
            description: "Raise to power".to_string(),
            category: StdCategory::Math,
        });

        self.register_function(StdFunction {
            name: "sqrt".to_string(),
            params: vec![TypeAnnotationNode::Integer],
            return_type: TypeAnnotationNode::Integer,
            description: "Square root (integer)".to_string(),
            category: StdCategory::Math,
        });

        self.register_function(StdFunction {
            name: "sum".to_string(),
            params: vec![TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer))],
            return_type: TypeAnnotationNode::Integer,
            description: "Sum all elements in an integer array".to_string(),
            category: StdCategory::Math,
        });

        // Percentage calculations for governance
        self.register_function(StdFunction {
            name: "percentage".to_string(),
            params: vec![
                TypeAnnotationNode::Integer, // value
                TypeAnnotationNode::Integer, // total
            ],
            return_type: TypeAnnotationNode::Integer,
            description: "Calculate percentage (returns basis points)".to_string(),
            category: StdCategory::Math,
        });

        self.register_function(StdFunction {
            name: "apply_percentage".to_string(),
            params: vec![
                TypeAnnotationNode::Integer, // value
                TypeAnnotationNode::Integer, // percentage (basis points)
            ],
            return_type: TypeAnnotationNode::Integer,
            description: "Apply percentage to value".to_string(),
            category: StdCategory::Math,
        });
    }

    /// Register cryptographic and security functions
    fn register_crypto_functions(&mut self) {
        self.register_function(StdFunction {
            name: "hash".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::String,
            description: "Calculate SHA-256 hash".to_string(),
            category: StdCategory::Crypto,
        });

        self.register_function(StdFunction {
            name: "hash_sha256".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::String,
            description: "Calculate SHA-256 hash (alias for hash)".to_string(),
            category: StdCategory::Crypto,
        });

        self.register_function(StdFunction {
            name: "verify_signature".to_string(),
            params: vec![
                TypeAnnotationNode::String, // message
                TypeAnnotationNode::String, // signature
                TypeAnnotationNode::Did,    // public key/DID
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Verify digital signature".to_string(),
            category: StdCategory::Crypto,
        });

        self.register_function(StdFunction {
            name: "generate_id".to_string(),
            params: vec![],
            return_type: TypeAnnotationNode::String,
            description: "Generate unique identifier".to_string(),
            category: StdCategory::Crypto,
        });

        self.register_function(StdFunction {
            name: "merkle_root".to_string(),
            params: vec![TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::String))],
            return_type: TypeAnnotationNode::String,
            description: "Calculate Merkle tree root".to_string(),
            category: StdCategory::Crypto,
        });
    }
}

impl Default for StdLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Generate documentation for the standard library
pub fn generate_stdlib_docs() -> String {
    let stdlib = StdLibrary::new();
    let mut docs = String::new();
    
    docs.push_str("# CCL Standard Library Functions\n\n");
    
    for category in [
        StdCategory::Governance,
        StdCategory::Economics,
        StdCategory::Utility,
        StdCategory::String,
        StdCategory::Array,
        StdCategory::Math,
        StdCategory::Crypto,
    ] {
        docs.push_str(&format!("## {:?} Functions\n\n", category));
        
        let mut functions = stdlib.get_functions_by_category(category);
        functions.sort_by(|a, b| a.name.cmp(&b.name));
        
        for func in functions {
            docs.push_str(&format!("### `{}`\n\n", func.name));
            docs.push_str(&format!("**Description:** {}\n\n", func.description));
            
            docs.push_str("**Parameters:**\n");
            if func.params.is_empty() {
                docs.push_str("- None\n");
            } else {
                for (i, param) in func.params.iter().enumerate() {
                    docs.push_str(&format!("- `arg{}`: {:?}\n", i + 1, param));
                }
            }
            
            docs.push_str(&format!("\n**Returns:** {:?}\n\n", func.return_type));
            docs.push_str("---\n\n");
        }
    }
    
    docs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_creation() {
        let stdlib = StdLibrary::new();
        
        // Should have functions from all categories
        assert!(stdlib.get_function("transfer").is_some());
        assert!(stdlib.get_function("create_proposal").is_some());
        assert!(stdlib.get_function("string_length").is_some());
        assert!(stdlib.get_function("array_length").is_some());
        assert!(stdlib.get_function("now").is_some());
        assert!(stdlib.get_function("hash").is_some());
        
        // Should not have non-existent functions
        assert!(stdlib.get_function("non_existent").is_none());
    }

    #[test]
    fn test_function_categories() {
        let stdlib = StdLibrary::new();
        
        let governance_funcs = stdlib.get_functions_by_category(StdCategory::Governance);
        assert!(!governance_funcs.is_empty());
        
        let economic_funcs = stdlib.get_functions_by_category(StdCategory::Economics);
        assert!(!economic_funcs.is_empty());
        
        // All governance functions should be in governance category
        for func in governance_funcs {
            assert_eq!(func.category, StdCategory::Governance);
        }
    }

    #[test]
    fn test_transfer_function_signature() {
        let stdlib = StdLibrary::new();
        let transfer = stdlib.get_function("transfer").unwrap();
        
        assert_eq!(transfer.params.len(), 3);
        assert_eq!(transfer.params[0], TypeAnnotationNode::Did);
        assert_eq!(transfer.params[1], TypeAnnotationNode::Did);
        assert_eq!(transfer.params[2], TypeAnnotationNode::Mana);
        assert_eq!(transfer.return_type, TypeAnnotationNode::Bool);
    }

    #[test]
    fn test_docs_generation() {
        let docs = generate_stdlib_docs();
        
        assert!(docs.contains("# CCL Standard Library Functions"));
        assert!(docs.contains("## Governance Functions"));
        assert!(docs.contains("## Economics Functions"));
        assert!(docs.contains("transfer"));
        assert!(docs.contains("create_proposal"));
    }
}