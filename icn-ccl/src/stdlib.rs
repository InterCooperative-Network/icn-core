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
        
        // Advanced governance constants
        self.constants.insert(
            "QUORUM_THRESHOLD".to_string(),
            (ExpressionNode::IntegerLiteral(50), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "HIGH_QUORUM_THRESHOLD".to_string(),
            (ExpressionNode::IntegerLiteral(75), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "MAX_REPUTATION".to_string(),
            (ExpressionNode::IntegerLiteral(100), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "MIN_REPUTATION".to_string(),
            (ExpressionNode::IntegerLiteral(0), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "DEFAULT_REPUTATION".to_string(),
            (ExpressionNode::IntegerLiteral(50), TypeAnnotationNode::Integer)
        );
        
        // Economic constants
        self.constants.insert(
            "BASIS_POINTS_100_PERCENT".to_string(),
            (ExpressionNode::IntegerLiteral(10000), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "DEFAULT_TRANSACTION_FEE".to_string(),
            (ExpressionNode::IntegerLiteral(10), TypeAnnotationNode::Mana)
        );
        
        self.constants.insert(
            "DEMURRAGE_RATE_DAILY_BASIS_POINTS".to_string(),
            (ExpressionNode::IntegerLiteral(27), TypeAnnotationNode::Integer) // ~1% annual
        );
        
        // Proposal type constants
        self.constants.insert(
            "PROPOSAL_TYPE_SIMPLE".to_string(),
            (ExpressionNode::IntegerLiteral(1), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "PROPOSAL_TYPE_FINANCIAL".to_string(),
            (ExpressionNode::IntegerLiteral(2), TypeAnnotationNode::Integer)
        );
        
        self.constants.insert(
            "PROPOSAL_TYPE_CONSTITUTIONAL".to_string(),
            (ExpressionNode::IntegerLiteral(3), TypeAnnotationNode::Integer)
        );
        
        // Voting period constants
        self.constants.insert(
            "VOTING_PERIOD_SHORT".to_string(),
            (ExpressionNode::IntegerLiteral(259200), TypeAnnotationNode::Integer) // 3 days
        );
        
        self.constants.insert(
            "VOTING_PERIOD_STANDARD".to_string(),
            (ExpressionNode::IntegerLiteral(604800), TypeAnnotationNode::Integer) // 7 days
        );
        
        self.constants.insert(
            "VOTING_PERIOD_EXTENDED".to_string(),
            (ExpressionNode::IntegerLiteral(1209600), TypeAnnotationNode::Integer) // 14 days
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
        
        // Advanced governance macros
        self.macros.insert(
            "calculate_weighted_vote".to_string(),
            "fn calculate_weighted_vote(reputation: Integer, stake: Mana, base_weight: Integer) -> Integer {
                let rep_factor = reputation / 10; // Scale reputation
                let stake_factor = stake / 100; // Scale stake
                return base_weight + rep_factor + stake_factor;
            }".to_string()
        );
        
        self.macros.insert(
            "is_quorum_met".to_string(),
            "fn is_quorum_met(participants: Integer, total_members: Integer, quorum_percent: Integer) -> Bool {
                let required = (total_members * quorum_percent) / 100;
                return participants >= required;
            }".to_string()
        );
        
        self.macros.insert(
            "calculate_quadratic_cost".to_string(),
            "fn calculate_quadratic_cost(votes: Integer) -> Mana {
                return votes * votes;
            }".to_string()
        );
        
        // Resource allocation macros
        self.macros.insert(
            "allocate_proportional".to_string(),
            "fn allocate_proportional(total_budget: Mana, priority_score: Integer, max_priority: Integer) -> Mana {
                if max_priority == 0 { return 0; }
                return (total_budget * priority_score) / max_priority;
            }".to_string()
        );
        
        self.macros.insert(
            "calculate_dividend".to_string(),
            "fn calculate_dividend(total_profit: Mana, member_contribution: Integer, total_contribution: Integer) -> Mana {
                if total_contribution == 0 { return 0; }
                return (total_profit * member_contribution) / total_contribution;
            }".to_string()
        );
        
        // Trust and reputation macros
        self.macros.insert(
            "update_reputation".to_string(),
            "fn update_reputation(current: Integer, feedback: Integer, weight: Integer) -> Integer {
                let weighted_feedback = feedback * weight;
                let new_rep = (current * 9 + weighted_feedback) / 10; // Exponential moving average
                if new_rep > 100 { return 100; }
                if new_rep < 0 { return 0; }
                return new_rep;
            }".to_string()
        );
        
        self.macros.insert(
            "calculate_trust_score".to_string(),
            "fn calculate_trust_score(interactions: Integer, positive_feedback: Integer) -> Integer {
                if interactions == 0 { return 50; } // Neutral score for new members
                let ratio = (positive_feedback * 100) / interactions;
                return ratio;
            }".to_string()
        );
        
        // Economic calculation macros
        self.macros.insert(
            "apply_demurrage".to_string(),
            "fn apply_demurrage(balance: Mana, rate_per_day: Integer, days: Integer) -> Mana {
                let total_rate = rate_per_day * days;
                let reduction = (balance * total_rate) / 10000; // Basis points
                return balance - reduction;
            }".to_string()
        );
        
        self.macros.insert(
            "calculate_interest".to_string(),
            "fn calculate_interest(principal: Mana, rate_percent: Integer, time_days: Integer) -> Mana {
                let daily_rate = rate_percent * 100 / 365; // Convert to daily basis points
                let interest = (principal * daily_rate * time_days) / 1000000;
                return interest;
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
    
    /// Get all available constant names
    pub fn get_constant_names(&self) -> Vec<&str> {
        self.constants.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get all available macro names
    pub fn get_macro_names(&self) -> Vec<&str> {
        self.macros.keys().map(|s| s.as_str()).collect()
    }
    
    /// Check if a name is a standard library constant
    pub fn is_standard_constant(&self, name: &str) -> bool {
        self.constants.contains_key(name)
    }
    
    /// Check if a name is a standard library macro
    pub fn is_standard_macro(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }
    
    /// Add governance helper functions for common patterns
    pub fn add_governance_helpers(&mut self) {
        // Multi-round voting with delegation
        self.macros.insert(
            "conduct_delegated_vote".to_string(),
            "fn conduct_delegated_vote(
                direct_votes: Integer, 
                delegated_votes: Integer, 
                delegation_weight: Integer
            ) -> Integer {
                let weighted_delegated = (delegated_votes * delegation_weight) / 100;
                return direct_votes + weighted_delegated;
            }".to_string()
        );
        
        // Consensus building with graduated penalties
        self.macros.insert(
            "calculate_consensus_penalty".to_string(),
            "fn calculate_consensus_penalty(
                dissent_level: Integer, 
                penalty_rate: Integer
            ) -> Mana {
                if dissent_level <= 10 { return 0; } // No penalty for minor dissent
                let excess_dissent = dissent_level - 10;
                return (excess_dissent * penalty_rate) / 100;
            }".to_string()
        );
        
        // Resource allocation with fairness constraints
        self.macros.insert(
            "enforce_fair_allocation".to_string(),
            "fn enforce_fair_allocation(
                proposed_allocation: Mana,
                member_max_share: Mana,
                total_budget: Mana,
                member_count: Integer
            ) -> Mana {
                let fair_share = total_budget / member_count;
                let max_allowed = if member_max_share < fair_share * 2 { member_max_share } else { fair_share * 2 };
                return if proposed_allocation > max_allowed { max_allowed } else { proposed_allocation };
            }".to_string()
        );
    }
    
    /// Add cooperative economics helpers
    pub fn add_economics_helpers(&mut self) {
        // Mutual aid scoring
        self.macros.insert(
            "calculate_mutual_aid_score".to_string(),
            "fn calculate_mutual_aid_score(
                help_given: Integer,
                help_received: Integer,
                community_benefit: Integer
            ) -> Integer {
                let aid_balance = if help_given >= help_received { 
                    help_given - help_received 
                } else { 
                    0 
                };
                let total_contribution = aid_balance + community_benefit;
                return if total_contribution > 100 { 100 } else { total_contribution };
            }".to_string()
        );
        
        // Solidarity economy calculations
        self.macros.insert(
            "distribute_solidarity_fund".to_string(),
            "fn distribute_solidarity_fund(
                total_fund: Mana,
                member_need_score: Integer,
                total_need_score: Integer
            ) -> Mana {
                if total_need_score == 0 { return 0; }
                let base_share = (total_fund * member_need_score) / total_need_score;
                // Ensure minimum dignity amount
                let min_amount = total_fund / 100; // 1% minimum
                return if base_share < min_amount { min_amount } else { base_share };
            }".to_string()
        );
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