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
    Identity,
    Utility,
    String,
    Array,
    Map, // Add Map category
    Math,
    Crypto,
}

/// CCL Standard Library
#[derive(Clone)]
pub struct StdLibrary {
    functions: HashMap<String, StdFunction>,
    macros: HashMap<String, MacroDefinition>,
}

/// Macro definition structure
#[derive(Debug, Clone)]
pub struct MacroDefinition {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<crate::ast::StatementNode>,
}

impl StdLibrary {
    /// Create a new standard library instance with all built-in functions
    pub fn new() -> Self {
        let mut stdlib = StdLibrary {
            functions: HashMap::new(),
            macros: HashMap::new(),
        };

        stdlib.register_governance_functions();
        stdlib.register_economic_functions();
        stdlib.register_identity_functions();
        stdlib.register_utility_functions();
        stdlib.register_string_functions();
        stdlib.register_array_functions();
        stdlib.register_map_functions(); // Add map functions
        stdlib.register_math_functions();
        stdlib.register_dag_functions(); // Add DAG functions
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
                TypeAnnotationNode::Did,  // from
                TypeAnnotationNode::Did,  // to
                TypeAnnotationNode::Mana, // amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Transfer mana between accounts".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "mint_mana".to_string(),
            params: vec![
                TypeAnnotationNode::Did,  // to
                TypeAnnotationNode::Mana, // amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Mint new mana to an account".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "burn_mana".to_string(),
            params: vec![
                TypeAnnotationNode::Did,  // from
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
                TypeAnnotationNode::Mana,    // base amount
                TypeAnnotationNode::Integer, // fee percentage (basis points)
            ],
            return_type: TypeAnnotationNode::Mana,
            description: "Calculate transaction fee".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "compound_interest".to_string(),
            params: vec![
                TypeAnnotationNode::Mana,    // principal
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

        // === CRITICAL ECONOMICS FUNCTIONS - Phase 1 ===

        // Token system operations
        self.register_function(StdFunction {
            name: "create_token_class".to_string(),
            params: vec![
                TypeAnnotationNode::String, // class_id
                TypeAnnotationNode::String, // name
                TypeAnnotationNode::String, // symbol
                TypeAnnotationNode::Did,    // issuer
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Create a new token class with specified properties".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "mint_tokens".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // class_id
                TypeAnnotationNode::Did,     // recipient
                TypeAnnotationNode::Integer, // amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Mint new tokens of specified class to recipient".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "transfer_tokens".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // class_id
                TypeAnnotationNode::Did,     // from
                TypeAnnotationNode::Did,     // to
                TypeAnnotationNode::Integer, // amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Transfer tokens between accounts".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "burn_tokens".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // class_id
                TypeAnnotationNode::Did,     // from
                TypeAnnotationNode::Integer, // amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Burn tokens from an account".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "get_token_balance".to_string(),
            params: vec![
                TypeAnnotationNode::String, // class_id
                TypeAnnotationNode::Did,    // account
            ],
            return_type: TypeAnnotationNode::Integer,
            description: "Get token balance for a specific class and account".to_string(),
            category: StdCategory::Economics,
        });

        // Reputation-linked functions
        self.register_function(StdFunction {
            name: "price_by_reputation".to_string(),
            params: vec![
                TypeAnnotationNode::Integer, // base_price
                TypeAnnotationNode::Integer, // reputation_score
            ],
            return_type: TypeAnnotationNode::Integer,
            description: "Calculate price adjusted by reputation (higher rep = lower price)"
                .to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "credit_by_reputation".to_string(),
            params: vec![
                TypeAnnotationNode::Did,     // account
                TypeAnnotationNode::Integer, // base_amount
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Credit mana to account based on reputation multiplier".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "mint_tokens_with_reputation".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // class_id
                TypeAnnotationNode::Did,     // recipient
                TypeAnnotationNode::Integer, // amount
                TypeAnnotationNode::Did,     // issuer (for reputation check)
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Mint tokens with mana cost adjusted by issuer reputation".to_string(),
            category: StdCategory::Economics,
        });

        // Time banking functions
        self.register_function(StdFunction {
            name: "record_time_work".to_string(),
            params: vec![
                TypeAnnotationNode::Did,     // worker
                TypeAnnotationNode::String,  // work_description
                TypeAnnotationNode::Integer, // hours_worked
                TypeAnnotationNode::Did,     // verifier
            ],
            return_type: TypeAnnotationNode::String, // returns time_record_id
            description: "Record time-based work contribution for time banking".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "mint_time_tokens".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // time_record_id
                TypeAnnotationNode::Did,     // worker
                TypeAnnotationNode::Integer, // hours
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Mint time-based tokens for verified work hours".to_string(),
            category: StdCategory::Economics,
        });

        // Mutual credit functions
        self.register_function(StdFunction {
            name: "create_credit_line".to_string(),
            params: vec![
                TypeAnnotationNode::Did,     // creditor
                TypeAnnotationNode::Did,     // debtor
                TypeAnnotationNode::Integer, // credit_limit
                TypeAnnotationNode::Integer, // interest_rate_bps (basis points)
            ],
            return_type: TypeAnnotationNode::String, // returns credit_line_id
            description: "Establish mutual credit relationship between community members"
                .to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "extend_mutual_credit".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // credit_line_id
                TypeAnnotationNode::Integer, // amount
                TypeAnnotationNode::String,  // purpose
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Extend credit within established mutual credit line".to_string(),
            category: StdCategory::Economics,
        });

        // Marketplace functions
        self.register_function(StdFunction {
            name: "create_marketplace_offer".to_string(),
            params: vec![
                TypeAnnotationNode::Did,     // seller
                TypeAnnotationNode::String,  // item_type
                TypeAnnotationNode::Integer, // quantity
                TypeAnnotationNode::Integer, // price_per_unit
                TypeAnnotationNode::String,  // payment_token_class
            ],
            return_type: TypeAnnotationNode::String, // returns offer_id
            description: "Create marketplace offer for goods or services".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "execute_marketplace_transaction".to_string(),
            params: vec![
                TypeAnnotationNode::String, // offer_id
                TypeAnnotationNode::String, // bid_id
                TypeAnnotationNode::Did,    // executor
            ],
            return_type: TypeAnnotationNode::String, // returns transaction_id
            description: "Execute marketplace transaction between buyer and seller".to_string(),
            category: StdCategory::Economics,
        });

        // === SCOPED TOKEN OPERATIONS ===

        self.register_function(StdFunction {
            name: "create_scoped_token".to_string(),
            params: vec![
                TypeAnnotationNode::String, // class_id
                TypeAnnotationNode::String, // name
                TypeAnnotationNode::String, // symbol
                TypeAnnotationNode::Did,    // issuer
                TypeAnnotationNode::String, // scope_type ("geographic", "community", "time")
                TypeAnnotationNode::String, // scope_value
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Create a token class with scoping constraints (geographic, community, or time-based)".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "transfer_scoped".to_string(),
            params: vec![
                TypeAnnotationNode::String, // class_id
                TypeAnnotationNode::Did,    // from
                TypeAnnotationNode::Did,    // to
                TypeAnnotationNode::Integer, // amount
                TypeAnnotationNode::String, // required_scope
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Transfer scoped tokens with scope validation (validates geographic/community constraints)".to_string(),
            category: StdCategory::Economics,
        });

        self.register_function(StdFunction {
            name: "verify_token_constraints".to_string(),
            params: vec![
                TypeAnnotationNode::String, // class_id
                TypeAnnotationNode::Did,    // actor
                TypeAnnotationNode::String, // operation ("transfer", "mint", "burn")
                TypeAnnotationNode::String, // target_scope
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Verify if token operation is allowed under scoping rules and transferability constraints".to_string(),
            category: StdCategory::Economics,
        });
    }

    /// Register identity and credential functions
    fn register_identity_functions(&mut self) {
        // === CRITICAL IDENTITY FUNCTIONS - Phase 2 ===

        // DID operations
        self.register_function(StdFunction {
            name: "create_did".to_string(),
            params: vec![
                TypeAnnotationNode::String, // method ("key", "web", "peer")
                TypeAnnotationNode::String, // identifier (domain for web, empty for key)
            ],
            return_type: TypeAnnotationNode::Did,
            description: "Create a new decentralized identifier with specified method".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "resolve_did".to_string(),
            params: vec![
                TypeAnnotationNode::Did, // did to resolve
            ],
            return_type: TypeAnnotationNode::String, // DID document JSON
            description: "Resolve a DID to its document containing public keys and services"
                .to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "update_did_document".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // did to update
                TypeAnnotationNode::String, // new document JSON
                TypeAnnotationNode::String, // signature from controller
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Update a DID document with new keys or service endpoints".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "verify_did_signature".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // signer DID
                TypeAnnotationNode::String, // message
                TypeAnnotationNode::String, // signature
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Verify a signature was created by the DID controller".to_string(),
            category: StdCategory::Identity,
        });

        // Credential operations
        self.register_function(StdFunction {
            name: "issue_credential".to_string(),
            params: vec![
                TypeAnnotationNode::Did,     // issuer DID
                TypeAnnotationNode::Did,     // holder DID
                TypeAnnotationNode::String,  // credential type
                TypeAnnotationNode::String,  // claims JSON
                TypeAnnotationNode::Integer, // expiration timestamp (0 for none)
            ],
            return_type: TypeAnnotationNode::String, // credential JSON
            description: "Issue a verifiable credential with specified claims".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "verify_credential".to_string(),
            params: vec![
                TypeAnnotationNode::String, // credential JSON
                TypeAnnotationNode::Did,    // expected issuer (optional check)
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Verify the authenticity and validity of a credential".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "revoke_credential".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // issuer DID
                TypeAnnotationNode::String, // credential ID
                TypeAnnotationNode::String, // revocation reason
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Revoke a previously issued credential".to_string(),
            category: StdCategory::Identity,
        });

        // Enhanced identity functions for cooperative contexts
        self.register_function(StdFunction {
            name: "create_cooperative_membership".to_string(),
            params: vec![
                TypeAnnotationNode::Did,     // member DID
                TypeAnnotationNode::String,  // cooperative ID
                TypeAnnotationNode::String,  // membership type
                TypeAnnotationNode::Integer, // membership level
            ],
            return_type: TypeAnnotationNode::String, // membership credential
            description: "Create cooperative membership credential".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "verify_cooperative_membership".to_string(),
            params: vec![
                TypeAnnotationNode::Did,     // member DID
                TypeAnnotationNode::String,  // cooperative ID
                TypeAnnotationNode::Integer, // required level
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Verify cooperative membership and authorization level".to_string(),
            category: StdCategory::Identity,
        });

        // === FEDERATION DISCOVERY & COORDINATION ===

        self.register_function(StdFunction {
            name: "discover_federations".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // search_criteria
                TypeAnnotationNode::Integer, // max_results
            ],
            return_type: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::String)), // federation IDs
            description: "Discover available federations based on search criteria".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "join_federation".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // member DID
                TypeAnnotationNode::String, // federation ID
                TypeAnnotationNode::String, // application details
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Apply to join a federation with specified member DID".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "leave_federation".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // member DID
                TypeAnnotationNode::String, // federation ID
                TypeAnnotationNode::String, // reason
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Leave a federation and revoke associated credentials".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "verify_cross_federation".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // verifier DID
                TypeAnnotationNode::String, // source federation
                TypeAnnotationNode::String, // target federation
                TypeAnnotationNode::String, // credential type
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Verify credentials are valid across different federations".to_string(),
            category: StdCategory::Identity,
        });

        // === KEY ROTATION & MANAGEMENT ===

        self.register_function(StdFunction {
            name: "rotate_keys".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // did to rotate
                TypeAnnotationNode::String, // new public key
                TypeAnnotationNode::String, // signature from old key
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Rotate DID document keys while maintaining identity continuity"
                .to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "backup_keys".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // did to backup
                TypeAnnotationNode::String, // backup method ("encrypted", "multisig", "social")
                TypeAnnotationNode::String, // backup parameters
            ],
            return_type: TypeAnnotationNode::String, // backup ID
            description: "Create secure backup of DID keys using specified method".to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "recover_keys".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // did to recover
                TypeAnnotationNode::String, // backup ID
                TypeAnnotationNode::String, // recovery proof
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Recover DID keys from backup with verification".to_string(),
            category: StdCategory::Identity,
        });

        // === ADVANCED FEDERATION OPERATIONS ===

        self.register_function(StdFunction {
            name: "get_federation_metadata".to_string(),
            params: vec![
                TypeAnnotationNode::String, // federation ID
            ],
            return_type: TypeAnnotationNode::String, // metadata JSON
            description:
                "Get detailed metadata about a federation including policies and governance"
                    .to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "verify_federation_membership".to_string(),
            params: vec![
                TypeAnnotationNode::Did,    // member DID
                TypeAnnotationNode::String, // federation ID
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Check if a DID is currently a member of the specified federation"
                .to_string(),
            category: StdCategory::Identity,
        });

        self.register_function(StdFunction {
            name: "coordinate_cross_federation_action".to_string(),
            params: vec![
                TypeAnnotationNode::Did, // coordinator DID
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::String)), // federation IDs
                TypeAnnotationNode::String, // action type
                TypeAnnotationNode::String, // action parameters
            ],
            return_type: TypeAnnotationNode::String, // coordination ID
            description: "Coordinate governance actions across multiple federations".to_string(),
            category: StdCategory::Identity,
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

        // Enhanced string operations for CCL WASM Backend
        self.register_function(StdFunction {
            name: "string_format".to_string(),
            params: vec![
                TypeAnnotationNode::String, // format string
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::String)), // arguments
            ],
            return_type: TypeAnnotationNode::String,
            description: "Format string with arguments (simplified version)".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_char_at".to_string(),
            params: vec![
                TypeAnnotationNode::String,
                TypeAnnotationNode::Integer, // index
            ],
            return_type: TypeAnnotationNode::Integer, // character code
            description: "Get character at index (same as string[index])".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_split".to_string(),
            params: vec![
                TypeAnnotationNode::String, // input string
                TypeAnnotationNode::String, // delimiter
            ],
            return_type: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::String)),
            description: "Split string by delimiter".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_trim".to_string(),
            params: vec![TypeAnnotationNode::String],
            return_type: TypeAnnotationNode::String,
            description: "Remove leading and trailing whitespace".to_string(),
            category: StdCategory::String,
        });

        self.register_function(StdFunction {
            name: "string_replace".to_string(),
            params: vec![
                TypeAnnotationNode::String, // input string
                TypeAnnotationNode::String, // search pattern
                TypeAnnotationNode::String, // replacement
            ],
            return_type: TypeAnnotationNode::String,
            description: "Replace all occurrences of pattern with replacement".to_string(),
            category: StdCategory::String,
        });
    }

    /// Register array manipulation functions
    fn register_array_functions(&mut self) {
        // Generic array functions for Integer arrays
        self.register_function(StdFunction {
            name: "array_length".to_string(),
            params: vec![TypeAnnotationNode::Array(Box::new(
                TypeAnnotationNode::Integer,
            ))],
            return_type: TypeAnnotationNode::Integer,
            description: "Get array length".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_len".to_string(),
            params: vec![TypeAnnotationNode::Array(Box::new(
                TypeAnnotationNode::Integer,
            ))],
            return_type: TypeAnnotationNode::Integer,
            description: "Get array length (alias for array_length)".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_push".to_string(),
            params: vec![
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)),
                TypeAnnotationNode::Integer,
            ],
            return_type: TypeAnnotationNode::Integer, // Return new length for simplicity
            description: "Add element to end of array, returns new length".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_pop".to_string(),
            params: vec![TypeAnnotationNode::Array(Box::new(
                TypeAnnotationNode::Integer,
            ))],
            return_type: TypeAnnotationNode::Option(Box::new(TypeAnnotationNode::Integer)),
            description: "Remove and return last element".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_contains".to_string(),
            params: vec![
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)),
                TypeAnnotationNode::Integer,
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Check if array contains element".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_slice".to_string(),
            params: vec![
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)),
                TypeAnnotationNode::Integer, // start
                TypeAnnotationNode::Integer, // end
            ],
            return_type: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Integer)),
            description: "Extract slice of array".to_string(),
            category: StdCategory::Array,
        });

        // DID array functions (needed for governance) - separate names to avoid conflicts
        self.register_function(StdFunction {
            name: "array_len_did".to_string(),
            params: vec![TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Did))],
            return_type: TypeAnnotationNode::Integer,
            description: "Get array length for DID arrays".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_push_did".to_string(),
            params: vec![
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Did)),
                TypeAnnotationNode::Did,
            ],
            return_type: TypeAnnotationNode::Integer,
            description: "Add DID to end of array, returns new length".to_string(),
            category: StdCategory::Array,
        });

        self.register_function(StdFunction {
            name: "array_contains_did".to_string(),
            params: vec![
                TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::Did)),
                TypeAnnotationNode::Did,
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Check if DID array contains element".to_string(),
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
                TypeAnnotationNode::String,  // More specific
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
            params: vec![TypeAnnotationNode::Map {
                key_type: Box::new(TypeAnnotationNode::String),
                value_type: Box::new(TypeAnnotationNode::Integer),
            }],
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
            params: vec![TypeAnnotationNode::Array(Box::new(
                TypeAnnotationNode::Integer,
            ))],
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

    /// Register DAG storage and content addressing functions  
    fn register_dag_functions(&mut self) {
        // Basic DAG operations
        self.register_function(StdFunction {
            name: "dag_put".to_string(),
            params: vec![
                TypeAnnotationNode::String, // data (serialized)
            ],
            return_type: TypeAnnotationNode::String, // CID
            description: "Store data in the DAG and return its Content Identifier (CID)"
                .to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "dag_get".to_string(),
            params: vec![
                TypeAnnotationNode::String, // CID
            ],
            return_type: TypeAnnotationNode::String, // data (serialized)
            description: "Retrieve data from the DAG using its Content Identifier (CID)"
                .to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "dag_pin".to_string(),
            params: vec![
                TypeAnnotationNode::String, // CID
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Pin content in the DAG to prevent garbage collection".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "dag_unpin".to_string(),
            params: vec![
                TypeAnnotationNode::String, // CID
            ],
            return_type: TypeAnnotationNode::Bool,
            description: "Unpin content in the DAG, allowing it to be garbage collected"
                .to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "calculate_cid".to_string(),
            params: vec![
                TypeAnnotationNode::String, // data
            ],
            return_type: TypeAnnotationNode::String, // CID
            description: "Calculate Content Identifier (CID) for given data without storing it"
                .to_string(),
            category: StdCategory::Utility,
        });

        // Contract state persistence
        self.register_function(StdFunction {
            name: "save_contract_state".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // contract_id
                TypeAnnotationNode::String,  // state_data (serialized)
                TypeAnnotationNode::Integer, // version
            ],
            return_type: TypeAnnotationNode::String, // state CID
            description: "Save contract state to DAG with versioning support".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "load_contract_state".to_string(),
            params: vec![
                TypeAnnotationNode::String,  // contract_id
                TypeAnnotationNode::Integer, // version (0 for latest)
            ],
            return_type: TypeAnnotationNode::String, // state_data (serialized)
            description: "Load contract state from DAG by contract ID and version".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "version_contract".to_string(),
            params: vec![
                TypeAnnotationNode::String, // contract_id
                TypeAnnotationNode::String, // new_code_cid
                TypeAnnotationNode::String, // migration_notes
            ],
            return_type: TypeAnnotationNode::Integer, // new version number
            description: "Create a new version of a contract with code and migration information"
                .to_string(),
            category: StdCategory::Utility,
        });

        // Advanced DAG operations
        self.register_function(StdFunction {
            name: "dag_link".to_string(),
            params: vec![
                TypeAnnotationNode::String, // parent_cid
                TypeAnnotationNode::String, // child_cid
                TypeAnnotationNode::String, // link_name
            ],
            return_type: TypeAnnotationNode::String, // new merged CID
            description: "Create a DAG link between two objects, returning new composite CID"
                .to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "dag_resolve_path".to_string(),
            params: vec![
                TypeAnnotationNode::String, // root_cid
                TypeAnnotationNode::String, // path (e.g., "metadata/version")
            ],
            return_type: TypeAnnotationNode::String, // resolved CID or data
            description: "Resolve a path within a DAG structure to find nested content".to_string(),
            category: StdCategory::Utility,
        });

        self.register_function(StdFunction {
            name: "dag_list_links".to_string(),
            params: vec![
                TypeAnnotationNode::String, // cid
            ],
            return_type: TypeAnnotationNode::Array(Box::new(TypeAnnotationNode::String)), // link names
            description: "List all link names in a DAG object".to_string(),
            category: StdCategory::Utility,
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
            params: vec![TypeAnnotationNode::Array(Box::new(
                TypeAnnotationNode::String,
            ))],
            return_type: TypeAnnotationNode::String,
            description: "Calculate Merkle tree root".to_string(),
            category: StdCategory::Crypto,
        });
    }

    /// Register a macro definition
    pub fn register_macro(&mut self, name: String, params: Vec<String>, body: Vec<crate::ast::StatementNode>) {
        self.macros.insert(name.clone(), MacroDefinition {
            name,
            params,
            body,
        });
    }

    /// Get a macro definition by name
    pub fn get_macro(&self, name: &str) -> Option<&MacroDefinition> {
        self.macros.get(name)
    }

    /// Check if a function or macro exists
    pub fn has_function_or_macro(&self, name: &str) -> bool {
        self.functions.contains_key(name) || self.macros.contains_key(name)
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
        StdCategory::Identity,
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

        // Should have new economics functions
        assert!(stdlib.get_function("create_token_class").is_some());
        assert!(stdlib.get_function("mint_tokens").is_some());
        assert!(stdlib.get_function("transfer_tokens").is_some());
        assert!(stdlib.get_function("burn_tokens").is_some());
        assert!(stdlib.get_function("get_token_balance").is_some());
        assert!(stdlib.get_function("price_by_reputation").is_some());
        assert!(stdlib.get_function("credit_by_reputation").is_some());
        assert!(stdlib.get_function("mint_tokens_with_reputation").is_some());
        assert!(stdlib.get_function("record_time_work").is_some());
        assert!(stdlib.get_function("mint_time_tokens").is_some());
        assert!(stdlib.get_function("create_credit_line").is_some());
        assert!(stdlib.get_function("extend_mutual_credit").is_some());
        assert!(stdlib.get_function("create_marketplace_offer").is_some());
        assert!(stdlib
            .get_function("execute_marketplace_transaction")
            .is_some());

        // Should have scoped token functions
        assert!(stdlib.get_function("create_scoped_token").is_some());
        assert!(stdlib.get_function("transfer_scoped").is_some());
        assert!(stdlib.get_function("verify_token_constraints").is_some());

        // Should have new identity functions
        assert!(stdlib.get_function("create_did").is_some());
        assert!(stdlib.get_function("resolve_did").is_some());
        assert!(stdlib.get_function("update_did_document").is_some());
        assert!(stdlib.get_function("verify_did_signature").is_some());
        assert!(stdlib.get_function("issue_credential").is_some());
        assert!(stdlib.get_function("verify_credential").is_some());
        assert!(stdlib.get_function("revoke_credential").is_some());
        assert!(stdlib
            .get_function("create_cooperative_membership")
            .is_some());
        assert!(stdlib
            .get_function("verify_cooperative_membership")
            .is_some());

        // Should have federation and key management functions
        assert!(stdlib.get_function("discover_federations").is_some());
        assert!(stdlib.get_function("join_federation").is_some());
        assert!(stdlib.get_function("leave_federation").is_some());
        assert!(stdlib.get_function("verify_cross_federation").is_some());
        assert!(stdlib.get_function("rotate_keys").is_some());
        assert!(stdlib.get_function("backup_keys").is_some());
        assert!(stdlib.get_function("recover_keys").is_some());
        assert!(stdlib.get_function("get_federation_metadata").is_some());
        assert!(stdlib
            .get_function("verify_federation_membership")
            .is_some());
        assert!(stdlib
            .get_function("coordinate_cross_federation_action")
            .is_some());

        // Should have DAG storage functions
        assert!(stdlib.get_function("dag_put").is_some());
        assert!(stdlib.get_function("dag_get").is_some());
        assert!(stdlib.get_function("dag_pin").is_some());
        assert!(stdlib.get_function("dag_unpin").is_some());
        assert!(stdlib.get_function("calculate_cid").is_some());
        assert!(stdlib.get_function("save_contract_state").is_some());
        assert!(stdlib.get_function("load_contract_state").is_some());
        assert!(stdlib.get_function("version_contract").is_some());
        assert!(stdlib.get_function("dag_link").is_some());
        assert!(stdlib.get_function("dag_resolve_path").is_some());
        assert!(stdlib.get_function("dag_list_links").is_some());

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

        let identity_funcs = stdlib.get_functions_by_category(StdCategory::Identity);
        assert!(!identity_funcs.is_empty());

        // All governance functions should be in governance category
        for func in governance_funcs {
            assert_eq!(func.category, StdCategory::Governance);
        }

        // All economics functions should be in economics category
        for func in economic_funcs {
            assert_eq!(func.category, StdCategory::Economics);
        }

        // All identity functions should be in identity category
        for func in identity_funcs {
            assert_eq!(func.category, StdCategory::Identity);
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
        assert!(docs.contains("## Identity Functions"));
        assert!(docs.contains("transfer"));
        assert!(docs.contains("create_proposal"));
        assert!(docs.contains("create_token_class"));
        assert!(docs.contains("mint_tokens"));
        assert!(docs.contains("create_did"));
        assert!(docs.contains("issue_credential"));
    }

    #[test]
    fn test_new_economics_functions() {
        let stdlib = StdLibrary::new();

        // Test token system functions
        let create_token = stdlib.get_function("create_token_class").unwrap();
        assert_eq!(create_token.params.len(), 4);
        assert_eq!(create_token.return_type, TypeAnnotationNode::Bool);

        let mint_tokens = stdlib.get_function("mint_tokens").unwrap();
        assert_eq!(mint_tokens.params.len(), 3);
        assert_eq!(mint_tokens.return_type, TypeAnnotationNode::Bool);

        // Test reputation functions
        let price_by_rep = stdlib.get_function("price_by_reputation").unwrap();
        assert_eq!(price_by_rep.params.len(), 2);
        assert_eq!(price_by_rep.return_type, TypeAnnotationNode::Integer);

        // Test marketplace functions
        let create_offer = stdlib.get_function("create_marketplace_offer").unwrap();
        assert_eq!(create_offer.params.len(), 5);
        assert_eq!(create_offer.return_type, TypeAnnotationNode::String);
    }

    #[test]
    fn test_new_identity_functions() {
        let stdlib = StdLibrary::new();

        // Test DID functions
        let create_did = stdlib.get_function("create_did").unwrap();
        assert_eq!(create_did.params.len(), 2);
        assert_eq!(create_did.return_type, TypeAnnotationNode::Did);

        let resolve_did = stdlib.get_function("resolve_did").unwrap();
        assert_eq!(resolve_did.params.len(), 1);
        assert_eq!(resolve_did.return_type, TypeAnnotationNode::String);

        // Test credential functions
        let issue_cred = stdlib.get_function("issue_credential").unwrap();
        assert_eq!(issue_cred.params.len(), 5);
        assert_eq!(issue_cred.return_type, TypeAnnotationNode::String);

        let verify_cred = stdlib.get_function("verify_credential").unwrap();
        assert_eq!(verify_cred.params.len(), 2);
        assert_eq!(verify_cred.return_type, TypeAnnotationNode::Bool);
    }
}
