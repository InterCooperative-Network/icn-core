// icn-ccl/src/stdlib.rs
//! CCL Standard Library
//! 
//! Provides constants, macros, and helper functions for CCL contracts.

use crate::ast::{ExpressionNode, TypeAnnotationNode, PolicyStatementNode};
use crate::error::CclError;
use std::collections::HashMap;

/// Standard constants available in CCL
pub struct StandardLibrary {
    pub constants: HashMap<String, (ExpressionNode, TypeAnnotationNode)>,
    pub macros: HashMap<String, String>,
}

impl StandardLibrary {
    pub fn new() -> Self {
        let mut stdlib = StandardLibrary {
            constants: HashMap::new(),
            macros: HashMap::new(),
        };
        
        // Add standard constants
        stdlib.add_constants();
        stdlib.add_macros();
        
        stdlib
    }
    
    fn add_constants(&mut self) {
        // Mathematical constants
        self.constants.insert(
            "MAX_MANA".to_string(),
            (ExpressionNode::IntegerLiteral(1_000_000), TypeAnnotationNode::Mana)
        );
        
        self.constants.insert(
            "MIN_MANA".to_string(),
            (ExpressionNode::IntegerLiteral(0), TypeAnnotationNode::Mana)
        );
        
        // Governance constants
        self.constants.insert(
            "MAJORITY_THRESHOLD".to_string(),
            (ExpressionNode::IntegerLiteral(51), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "SUPERMAJORITY_THRESHOLD".to_string(),
            (ExpressionNode::IntegerLiteral(67), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "CONSENSUS_THRESHOLD".to_string(),
            (ExpressionNode::IntegerLiteral(100), TypeAnnotationNode::Integer)
        );
        
        // Time constants (in seconds)
        self.constants.insert(
            "MINUTE".to_string(),
            (ExpressionNode::IntegerLiteral(60), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "HOUR".to_string(),
            (ExpressionNode::IntegerLiteral(3600), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "DAY".to_string(),
            (ExpressionNode::IntegerLiteral(86400), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "WEEK".to_string(),
            (ExpressionNode::IntegerLiteral(604800), TypeAnnotationNode::Integer)
        );
        
        // Boolean constants
        self.constants.insert(
            "TRUE".to_string(),
            (ExpressionNode::BooleanLiteral(true), TypeAnnotationNode::Bool)
        );
        
        self.constants.insert(
            "FALSE".to_string(),
            (ExpressionNode::BooleanLiteral(false), TypeAnnotationNode::Bool)
        );
    }
    
    fn add_macros(&mut self) {
        // Voting macros
        self.macros.insert(
            "is_majority".to_string(),
            "fn is_majority(yes_votes: Integer, total_votes: Integer) -> Bool {
                return (yes_votes * 100) / total_votes >= MAJORITY_THRESHOLD;
            }".to_string()
        );
        
        self.macros.insert(
            "is_supermajority".to_string(),
            "fn is_supermajority(yes_votes: Integer, total_votes: Integer) -> Bool {
                return (yes_votes * 100) / total_votes >= SUPERMAJORITY_THRESHOLD;
            }".to_string()
        );
        
        self.macros.insert(
            "is_consensus".to_string(),
            "fn is_consensus(yes_votes: Integer, total_votes: Integer) -> Bool {
                return (yes_votes * 100) / total_votes >= CONSENSUS_THRESHOLD;
            }".to_string()
        );
        
        // Mana management macros
        self.macros.insert(
            "has_sufficient_mana".to_string(),
            "fn has_sufficient_mana(available: Mana, required: Mana) -> Bool {
                return available >= required;
            }".to_string()
        );
        
        self.macros.insert(
            "calculate_fee".to_string(),
            "fn calculate_fee(base_cost: Mana, complexity: Integer) -> Mana {
                return base_cost * complexity;
            }".to_string()
        );
        
        // Array helper macros
        self.macros.insert(
            "array_is_empty".to_string(),
            "fn array_is_empty(arr: Array<Integer>) -> Bool {
                return array_len(arr) == 0;
            }".to_string()
        );
        
        self.macros.insert(
            "array_contains".to_string(),
            "fn array_contains(arr: Array<Integer>, item: Integer) -> Bool {
                let i = 0;
                while i < array_len(arr) {
                    if arr[i] == item {
                        return true;
                    }
                    i = i + 1;
                }
                return false;
            }".to_string()
        );

        // Math utility macros
        self.macros.insert(
            "array_sum".to_string(),
            "fn array_sum(arr: Array<Integer>) -> Integer {
                let sum = 0;
                let i = 0;
                while i < array_len(arr) {
                    sum = sum + arr[i];
                    i = i + 1;
                }
                return sum;
            }".to_string()
        );

        self.macros.insert(
            "array_max".to_string(),
            "fn array_max(arr: Array<Integer>) -> Integer {
                if array_is_empty(arr) {
                    panic(\"Cannot find max of empty array\");
                }
                let max = arr[0];
                let i = 1;
                while i < array_len(arr) {
                    if arr[i] > max {
                        max = arr[i];
                    }
                    i = i + 1;
                }
                return max;
            }".to_string()
        );

        self.macros.insert(
            "array_min".to_string(),
            "fn array_min(arr: Array<Integer>) -> Integer {
                if array_is_empty(arr) {
                    panic(\"Cannot find min of empty array\");
                }
                let min = arr[0];
                let i = 1;
                while i < array_len(arr) {
                    if arr[i] < min {
                        min = arr[i];
                    }
                    i = i + 1;
                }
                return min;
            }".to_string()
        );

        self.macros.insert(
            "array_average".to_string(),
            "fn array_average(arr: Array<Integer>) -> Integer {
                if array_is_empty(arr) {
                    panic(\"Cannot find average of empty array\");
                }
                return array_sum(arr) / array_len(arr);
            }".to_string()
        );

        // Map utility macros
        self.macros.insert(
            "map_contains_key".to_string(),
            "fn map_contains_key(map: Map<String, Integer>, key: String) -> Bool {
                try {
                    let _ = map[key];
                    return true;
                } catch {
                    return false;
                }
            }".to_string()
        );

        self.macros.insert(
            "map_get_or_default".to_string(),
            "fn map_get_or_default(map: Map<String, Integer>, key: String, default: Integer) -> Integer {
                try {
                    return map[key];
                } catch {
                    return default;
                }
            }".to_string()
        );

        // String utility macros
        self.macros.insert(
            "string_is_empty".to_string(),
            "fn string_is_empty(str: String) -> Bool {
                return string_len(str) == 0;
            }".to_string()
        );

        self.macros.insert(
            "string_contains".to_string(),
            "fn string_contains(haystack: String, needle: String) -> Bool {
                return string_find(haystack, needle) >= 0;
            }".to_string()
        );

        // Error handling macros
        self.macros.insert(
            "require".to_string(),
            "fn require(condition: Bool, message: String) {
                if !condition {
                    panic(message);
                }
            }".to_string()
        );

        self.macros.insert(
            "safe_divide".to_string(),
            "fn safe_divide(a: Integer, b: Integer) -> Result<Integer, String> {
                if b == 0 {
                    return Err(\"Division by zero\");
                }
                return Ok(a / b);
            }".to_string()
        );

        // Governance utility macros
        self.macros.insert(
            "calculate_weighted_vote".to_string(),
            "fn calculate_weighted_vote(votes: Array<Integer>, weights: Array<Integer>) -> Integer {
                require(array_len(votes) == array_len(weights), \"Votes and weights arrays must have same length\");
                let weighted_sum = 0;
                let i = 0;
                while i < array_len(votes) {
                    weighted_sum = weighted_sum + (votes[i] * weights[i]);
                    i = i + 1;
                }
                return weighted_sum;
            }".to_string()
        );

        self.macros.insert(
            "is_quorum_met".to_string(),
            "fn is_quorum_met(participant_count: Integer, total_members: Integer, quorum_percent: Integer) -> Bool {
                let required = (total_members * quorum_percent) / 100;
                return participant_count >= required;
            }".to_string()
        );
    }
    
    /// Get a constant value by name
    pub fn get_constant(&self, name: &str) -> Option<&(ExpressionNode, TypeAnnotationNode)> {
        self.constants.get(name)
    }
    
    /// Get a macro definition by name
    pub fn get_macro(&self, name: &str) -> Option<&String> {
        self.macros.get(name)
    }
    
    /// Expand a macro call by replacing parameters
    pub fn expand_macro(&self, name: &str, params: &[String], args: &[String]) -> Result<String, CclError> {
        if let Some(macro_body) = self.get_macro(name) {
            if params.len() != args.len() {
                return Err(CclError::SemanticError(format!(
                    "Macro {} expects {} parameters, got {}",
                    name, params.len(), args.len()
                )));
            }
            
            let mut expanded = macro_body.clone();
            for (param, arg) in params.iter().zip(args.iter()) {
                expanded = expanded.replace(param, arg);
            }
            
            Ok(expanded)
        } else {
            Err(CclError::SemanticError(format!("Unknown macro: {}", name)))
        }
    }
    
    /// Generate policy statements for all constants and macros
    pub fn generate_statements(&self) -> Vec<PolicyStatementNode> {
        let mut statements = Vec::new();
        
        // Add constants
        for (name, (value, type_ann)) in &self.constants {
            statements.push(PolicyStatementNode::ConstDef {
                name: name.clone(),
                value: value.clone(),
                type_ann: type_ann.clone(),
            });
        }
        
        statements
    }
}

impl Default for StandardLibrary {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_stdlib_constants() {
        let stdlib = StandardLibrary::new();
        
        assert!(stdlib.get_constant("MAX_MANA").is_some());
        assert!(stdlib.get_constant("MAJORITY_THRESHOLD").is_some());
        assert!(stdlib.get_constant("NONEXISTENT").is_none());
    }
    
    #[test]
    fn test_stdlib_macros() {
        let stdlib = StandardLibrary::new();
        
        assert!(stdlib.get_macro("is_majority").is_some());
        assert!(stdlib.get_macro("has_sufficient_mana").is_some());
        assert!(stdlib.get_macro("nonexistent").is_none());
    }
    
    #[test]
    fn test_macro_expansion() {
        let stdlib = StandardLibrary::new();
        
        let result = stdlib.expand_macro(
            "is_majority",
            &["yes_votes".to_string(), "total_votes".to_string()],
            &["10".to_string(), "20".to_string()]
        );
        
        assert!(result.is_ok());
        let expanded = result.unwrap();
        assert!(expanded.contains("10"));
        assert!(expanded.contains("20"));
    }
}