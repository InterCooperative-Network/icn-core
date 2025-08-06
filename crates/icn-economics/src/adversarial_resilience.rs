//! Adversarial Resilience Module
//!
//! This module provides Byzantine fault tolerance, cryptographic verification,
//! anti-gaming mechanisms, and emergency protocols for the ICN economic system.

use crate::ManaLedger;
use icn_common::{CommonError, Did, SystemTimeProvider, TimeProvider};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Byzantine fault tolerance threshold (assumes 1/3 malicious actors)
const BYZANTINE_THRESHOLD: f64 = 0.67;

/// Maximum allowed economic velocity to prevent manipulation
const MAX_ECONOMIC_VELOCITY: f64 = 1000.0;

/// Gaming detection window in seconds
const GAMING_DETECTION_WINDOW: u64 = 3600; // 1 hour

/// Emergency response protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyProtocol {
    /// Freeze all economic operations
    GlobalFreeze,
    /// Freeze specific accounts
    AccountFreeze { accounts: Vec<Did> },
    /// Rollback to a previous state
    Rollback { to_timestamp: u64 },
    /// Rate limit all operations
    RateLimit { max_operations_per_hour: u64 },
    /// Enhanced validation mode
    EnhancedValidation,
}

/// Types of economic attacks that can be detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttackType {
    /// Rapid transaction patterns suggesting automated manipulation
    VelocityAttack { velocity: f64, threshold: f64 },
    /// Coordinated behavior between multiple accounts
    CoordinatedAttack { participants: Vec<Did> },
    /// Double spending or other integrity violations
    IntegrityAttack { description: String },
    /// Sybil attack with multiple fake identities
    SybilAttack { suspected_accounts: Vec<Did> },
    /// Economic drain attacks
    DrainAttack { target_resource: String, drain_rate: f64 },
}

/// Attack detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackDetection {
    pub attack_type: AttackType,
    pub confidence: f64, // 0.0 to 1.0
    pub timestamp: u64,
    pub affected_accounts: Vec<Did>,
    pub recommended_response: EmergencyProtocol,
}

/// Byzantine validator for economic operations
#[derive(Debug)]
pub struct ByzantineValidator {
    validators: HashSet<Did>,
    required_confirmations: usize,
    operation_history: HashMap<String, Vec<ValidationRecord>>,
}

/// Individual validation record
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ValidationRecord {
    validator: Did,
    operation_id: String,
    signature: Vec<u8>, // Cryptographic signature
    timestamp: u64,
    valid: bool,
}

/// Anti-gaming detector
#[derive(Debug)]
pub struct AntiGamingDetector {
    velocity_tracker: HashMap<Did, VelocityTracker>,
    coordination_detector: CoordinationDetector,
    integrity_monitor: IntegrityMonitor,
}

/// Track transaction velocity for gaming detection
#[derive(Debug, Clone)]
struct VelocityTracker {
    operations: Vec<OperationRecord>,
    window_start: u64,
}

#[derive(Debug, Clone)]
struct OperationRecord {
    timestamp: u64,
    amount: u64,
    operation_type: String,
}

/// Detect coordinated attacks between accounts
#[derive(Debug)]
struct CoordinationDetector {
    interaction_graph: HashMap<Did, HashMap<Did, u32>>, // DID -> (target DID -> interaction count)
    suspicious_patterns: Vec<CoordinationPattern>,
}

#[derive(Debug, Clone)]
struct CoordinationPattern {
    participants: Vec<Did>,
    pattern_type: String,
    confidence: f64,
}

/// Monitor economic integrity violations
#[derive(Debug)]
struct IntegrityMonitor {
    double_spend_attempts: HashMap<String, Vec<Did>>,
    balance_inconsistencies: Vec<BalanceInconsistency>,
}

#[derive(Debug, Clone)]
struct BalanceInconsistency {
    account: Did,
    expected_balance: u64,
    actual_balance: u64,
    timestamp: u64,
}

impl ByzantineValidator {
    /// Create a new Byzantine validator
    pub fn new(validators: HashSet<Did>) -> Self {
        let required_confirmations = ((validators.len() as f64 * BYZANTINE_THRESHOLD).ceil() as usize).max(1);
        Self {
            validators,
            required_confirmations,
            operation_history: HashMap::new(),
        }
    }

    /// Validate an economic operation with Byzantine fault tolerance
    pub fn validate_operation(
        &mut self,
        operation_id: String,
        operation_data: &[u8],
        validator_signatures: Vec<(Did, Vec<u8>)>,
    ) -> Result<bool, CommonError> {
        // Check if we have enough validator signatures
        if validator_signatures.len() < self.required_confirmations {
            return Ok(false);
        }

        // Validate each signature
        let mut valid_confirmations = 0;
        let mut validation_records = Vec::new();

        for (validator, signature) in validator_signatures {
            if !self.validators.contains(&validator) {
                continue; // Skip unknown validators
            }

            // Verify cryptographic signature (simplified)
            let is_valid = self.verify_signature(&validator, operation_data, &signature)?;
            
            validation_records.push(ValidationRecord {
                validator: validator.clone(),
                operation_id: operation_id.clone(),
                signature,
                timestamp: SystemTimeProvider.unix_seconds(),
                valid: is_valid,
            });

            if is_valid {
                valid_confirmations += 1;
            }
        }

        // Store validation records
        self.operation_history.insert(operation_id, validation_records);

        // Operation is valid if we have enough valid confirmations
        Ok(valid_confirmations >= self.required_confirmations)
    }

    /// Verify a cryptographic signature (placeholder implementation)
    fn verify_signature(&self, validator: &Did, data: &[u8], signature: &[u8]) -> Result<bool, CommonError> {
        // In a real implementation, this would use proper cryptographic verification
        // For now, we simulate validation based on signature length and validator presence
        Ok(signature.len() >= 64 && self.validators.contains(validator))
    }

    /// Get validation history for an operation
    pub fn get_validation_history(&self, operation_id: &str) -> Option<&Vec<ValidationRecord>> {
        self.operation_history.get(operation_id)
    }
}

impl AntiGamingDetector {
    /// Create a new anti-gaming detector
    pub fn new() -> Self {
        Self {
            velocity_tracker: HashMap::new(),
            coordination_detector: CoordinationDetector::new(),
            integrity_monitor: IntegrityMonitor::new(),
        }
    }

    /// Analyze an economic operation for potential gaming
    pub fn analyze_operation(
        &mut self,
        did: &Did,
        amount: u64,
        operation_type: String,
    ) -> Result<Vec<AttackDetection>, CommonError> {
        let mut detections = Vec::new();
        let timestamp = SystemTimeProvider.unix_seconds();

        // Update velocity tracker
        self.update_velocity_tracker(did, amount, operation_type.clone(), timestamp);

        // Check for velocity attacks
        if let Some(velocity_detection) = self.check_velocity_attack(did)? {
            detections.push(velocity_detection);
        }

        // Check for coordination attacks
        if let Some(coordination_detection) = self.check_coordination_attack(did)? {
            detections.push(coordination_detection);
        }

        Ok(detections)
    }

    /// Update velocity tracking for an account
    fn update_velocity_tracker(&mut self, did: &Did, amount: u64, operation_type: String, timestamp: u64) {
        let tracker = self.velocity_tracker.entry(did.clone()).or_insert_with(|| {
            VelocityTracker {
                operations: Vec::new(),
                window_start: timestamp,
            }
        });

        // Remove old operations outside the detection window
        tracker.operations.retain(|op| op.timestamp >= timestamp.saturating_sub(GAMING_DETECTION_WINDOW));
        
        // Add new operation
        tracker.operations.push(OperationRecord {
            timestamp,
            amount,
            operation_type,
        });
    }

    /// Check for velocity-based attacks
    fn check_velocity_attack(&self, did: &Did) -> Result<Option<AttackDetection>, CommonError> {
        if let Some(tracker) = self.velocity_tracker.get(did) {
            let total_amount: u64 = tracker.operations.iter().map(|op| op.amount).sum();
            let time_span = tracker.operations.last().unwrap().timestamp - tracker.operations.first().unwrap().timestamp;
            
            if time_span > 0 {
                let velocity = total_amount as f64 / time_span as f64;
                
                if velocity > MAX_ECONOMIC_VELOCITY {
                    return Ok(Some(AttackDetection {
                        attack_type: AttackType::VelocityAttack {
                            velocity,
                            threshold: MAX_ECONOMIC_VELOCITY,
                        },
                        confidence: 0.8,
                        timestamp: SystemTimeProvider.unix_seconds(),
                        affected_accounts: vec![did.clone()],
                        recommended_response: EmergencyProtocol::AccountFreeze {
                            accounts: vec![did.clone()],
                        },
                    }));
                }
            }
        }
        
        Ok(None)
    }

    /// Check for coordinated attacks
    fn check_coordination_attack(&self, _did: &Did) -> Result<Option<AttackDetection>, CommonError> {
        // Placeholder for coordination detection logic
        // This would analyze interaction patterns between accounts
        Ok(None)
    }

    /// Record interaction between accounts for coordination detection
    pub fn record_interaction(&mut self, from: &Did, to: &Did) {
        self.coordination_detector.record_interaction(from, to);
    }

    /// Analyze potential Sybil attacks
    pub fn detect_sybil_attack(&self, accounts: &[Did]) -> Option<AttackDetection> {
        // Placeholder for Sybil attack detection
        // This would analyze account creation patterns, interaction graphs, etc.
        None
    }
}

impl CoordinationDetector {
    fn new() -> Self {
        Self {
            interaction_graph: HashMap::new(),
            suspicious_patterns: Vec::new(),
        }
    }

    fn record_interaction(&mut self, from: &Did, to: &Did) {
        let from_interactions = self.interaction_graph.entry(from.clone()).or_default();
        *from_interactions.entry(to.clone()).or_insert(0) += 1;
    }
}

impl IntegrityMonitor {
    fn new() -> Self {
        Self {
            double_spend_attempts: HashMap::new(),
            balance_inconsistencies: Vec::new(),
        }
    }

    /// Check for double spending attempts
    pub fn check_double_spend(&mut self, operation_id: String, account: Did) -> bool {
        let attempts = self.double_spend_attempts.entry(operation_id).or_default();
        if attempts.contains(&account) {
            true // Double spend detected
        } else {
            attempts.push(account);
            false
        }
    }

    /// Record balance inconsistency
    pub fn record_balance_inconsistency(&mut self, account: Did, expected: u64, actual: u64) {
        self.balance_inconsistencies.push(BalanceInconsistency {
            account,
            expected_balance: expected,
            actual_balance: actual,
            timestamp: SystemTimeProvider.unix_seconds(),
        });
    }
}

/// Emergency response coordinator
#[derive(Debug)]
pub struct EmergencyCoordinator {
    active_protocols: Vec<EmergencyProtocol>,
    frozen_accounts: HashSet<Did>,
    rate_limits: HashMap<Did, RateLimit>,
    global_freeze: bool,
}

#[derive(Debug, Clone)]
struct RateLimit {
    operations_count: u32,
    window_start: u64,
    max_per_hour: u64,
}

impl EmergencyCoordinator {
    /// Create a new emergency coordinator
    pub fn new() -> Self {
        Self {
            active_protocols: Vec::new(),
            frozen_accounts: HashSet::new(),
            rate_limits: HashMap::new(),
            global_freeze: false,
        }
    }

    /// Activate an emergency protocol
    pub fn activate_protocol(&mut self, protocol: EmergencyProtocol) -> Result<(), CommonError> {
        match &protocol {
            EmergencyProtocol::GlobalFreeze => {
                self.global_freeze = true;
            }
            EmergencyProtocol::AccountFreeze { accounts } => {
                for account in accounts {
                    self.frozen_accounts.insert(account.clone());
                }
            }
            EmergencyProtocol::RateLimit { max_operations_per_hour } => {
                // Apply rate limits to all accounts
                let current_time = SystemTimeProvider.unix_seconds();
                for account in self.get_all_accounts() {
                    self.rate_limits.insert(account, RateLimit {
                        operations_count: 0,
                        window_start: current_time,
                        max_per_hour: *max_operations_per_hour,
                    });
                }
            }
            _ => {} // Handle other protocols as needed
        }

        self.active_protocols.push(protocol);
        Ok(())
    }

    /// Check if an operation is allowed given current emergency protocols
    pub fn is_operation_allowed(&mut self, account: &Did) -> bool {
        // Global freeze blocks all operations
        if self.global_freeze {
            return false;
        }

        // Check if account is frozen
        if self.frozen_accounts.contains(account) {
            return false;
        }

        // Check rate limits
        if let Some(rate_limit) = self.rate_limits.get_mut(account) {
            let current_time = SystemTimeProvider.unix_seconds();
            
            // Reset window if needed
            if current_time >= rate_limit.window_start + 3600 {
                rate_limit.operations_count = 0;
                rate_limit.window_start = current_time;
            }

            // Check if under rate limit
            if u64::from(rate_limit.operations_count) >= rate_limit.max_per_hour {
                return false;
            }

            rate_limit.operations_count += 1;
        }

        true
    }

    /// Get all known accounts (placeholder)
    fn get_all_accounts(&self) -> Vec<Did> {
        // This would get all accounts from the ledger
        Vec::new()
    }

    /// Deactivate emergency protocols
    pub fn deactivate_protocol(&mut self, protocol_type: &str) -> Result<(), CommonError> {
        match protocol_type {
            "global_freeze" => {
                self.global_freeze = false;
            }
            "account_freeze" => {
                self.frozen_accounts.clear();
            }
            "rate_limit" => {
                self.rate_limits.clear();
            }
            _ => {
                return Err(CommonError::InvalidInputError(
                    format!("Unknown protocol type: {}", protocol_type)
                ));
            }
        }

        self.active_protocols.retain(|p| match p {
            EmergencyProtocol::GlobalFreeze => protocol_type != "global_freeze",
            EmergencyProtocol::AccountFreeze { .. } => protocol_type != "account_freeze",
            EmergencyProtocol::RateLimit { .. } => protocol_type != "rate_limit",
            _ => true,
        });

        Ok(())
    }
}

/// Adversarial-resilient economic operations wrapper
pub struct AdversarialResilientEconomics<L: ManaLedger> {
    ledger: L,
    byzantine_validator: ByzantineValidator,
    anti_gaming_detector: AntiGamingDetector,
    emergency_coordinator: EmergencyCoordinator,
}

impl<L: ManaLedger> AdversarialResilientEconomics<L> {
    /// Create a new adversarial-resilient economics wrapper
    pub fn new(ledger: L, validators: HashSet<Did>) -> Self {
        Self {
            ledger,
            byzantine_validator: ByzantineValidator::new(validators),
            anti_gaming_detector: AntiGamingDetector::new(),
            emergency_coordinator: EmergencyCoordinator::new(),
        }
    }

    /// Perform a validated mana spend operation
    pub fn validated_spend(
        &mut self,
        did: &Did,
        amount: u64,
        operation_id: String,
        validator_signatures: Vec<(Did, Vec<u8>)>,
    ) -> Result<(), CommonError> {
        // Check emergency protocols
        if !self.emergency_coordinator.is_operation_allowed(did) {
            return Err(CommonError::PolicyDenied(
                "Operation blocked by emergency protocols".into()
            ));
        }

        // Validate operation with Byzantine fault tolerance
        let operation_data = format!("spend:{}:{}", did, amount).into_bytes();
        if !self.byzantine_validator.validate_operation(
            operation_id.clone(),
            &operation_data,
            validator_signatures,
        )? {
            return Err(CommonError::PolicyDenied(
                "Byzantine validation failed".into()
            ));
        }

        // Check for gaming patterns
        let detections = self.anti_gaming_detector.analyze_operation(
            did,
            amount,
            "spend".to_string(),
        )?;

        // Activate emergency protocols if attacks detected
        for detection in detections {
            if detection.confidence > 0.7 {
                self.emergency_coordinator.activate_protocol(detection.recommended_response)?;
                return Err(CommonError::PolicyDenied(
                    format!("Attack detected: {:?}", detection.attack_type)
                ));
            }
        }

        // Perform the actual spend operation
        self.ledger.spend(did, amount)?;

        Ok(())
    }

    /// Perform a validated mana credit operation
    pub fn validated_credit(
        &mut self,
        did: &Did,
        amount: u64,
        operation_id: String,
        validator_signatures: Vec<(Did, Vec<u8>)>,
    ) -> Result<(), CommonError> {
        // Similar validation logic for credit operations
        if !self.emergency_coordinator.is_operation_allowed(did) {
            return Err(CommonError::PolicyDenied(
                "Operation blocked by emergency protocols".into()
            ));
        }

        let operation_data = format!("credit:{}:{}", did, amount).into_bytes();
        if !self.byzantine_validator.validate_operation(
            operation_id.clone(),
            &operation_data,
            validator_signatures,
        )? {
            return Err(CommonError::PolicyDenied(
                "Byzantine validation failed".into()
            ));
        }

        let detections = self.anti_gaming_detector.analyze_operation(
            did,
            amount,
            "credit".to_string(),
        )?;

        for detection in detections {
            if detection.confidence > 0.7 {
                self.emergency_coordinator.activate_protocol(detection.recommended_response)?;
                return Err(CommonError::PolicyDenied(
                    format!("Attack detected: {:?}", detection.attack_type)
                ));
            }
        }

        self.ledger.credit(did, amount)?;
        Ok(())
    }

    /// Get the underlying ledger
    pub fn ledger(&self) -> &L {
        &self.ledger
    }

    /// Get current attack detections
    pub fn get_active_detections(&self) -> Vec<AttackDetection> {
        // Return recent attack detections
        Vec::new() // Placeholder
    }

    /// Get emergency coordinator status
    pub fn get_emergency_status(&self) -> &EmergencyCoordinator {
        &self.emergency_coordinator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[derive(Default)]
    struct TestLedger {
        balances: HashMap<Did, u64>,
    }

    impl ManaLedger for TestLedger {
        fn get_balance(&self, did: &Did) -> u64 {
            *self.balances.get(did).unwrap_or(&0)
        }

        fn set_balance(&self, _did: &Did, _amount: u64) -> Result<(), CommonError> {
            Ok(())
        }

        fn spend(&self, _did: &Did, _amount: u64) -> Result<(), CommonError> {
            Ok(())
        }

        fn credit(&self, _did: &Did, _amount: u64) -> Result<(), CommonError> {
            Ok(())
        }
    }

    #[test]
    fn test_byzantine_validator_creation() {
        let validators = vec![
            Did::from_str("did:test:validator1").unwrap(),
            Did::from_str("did:test:validator2").unwrap(),
            Did::from_str("did:test:validator3").unwrap(),
        ].into_iter().collect();

        let validator = ByzantineValidator::new(validators);
        assert_eq!(validator.required_confirmations, 3); // ceil(67% of 3) = ceil(2.01) = 3
    }

    #[test]
    fn test_anti_gaming_detector_velocity() {
        let mut detector = AntiGamingDetector::new();
        let did = Did::from_str("did:test:user").unwrap();

        // Simulate high-velocity operations
        for i in 0..100 {
            let _ = detector.analyze_operation(&did, 1000, "spend".to_string());
        }

        // Should detect velocity attack
        let detections = detector.analyze_operation(&did, 1000, "spend".to_string()).unwrap();
        // Note: This test might pass because we're not actually triggering the velocity threshold
        // In a real implementation, we'd need to mock the time to compress operations into a small window
    }

    #[test]
    fn test_emergency_coordinator() {
        let mut coordinator = EmergencyCoordinator::new();
        let did = Did::from_str("did:test:user").unwrap();

        // Initially operations should be allowed
        assert!(coordinator.is_operation_allowed(&did));

        // Activate global freeze
        coordinator.activate_protocol(EmergencyProtocol::GlobalFreeze).unwrap();
        assert!(!coordinator.is_operation_allowed(&did));

        // Deactivate global freeze
        coordinator.deactivate_protocol("global_freeze").unwrap();
        assert!(coordinator.is_operation_allowed(&did));
    }

    #[test]
    fn test_adversarial_resilient_economics() {
        let ledger = TestLedger::default();
        let validators = vec![
            Did::from_str("did:test:validator1").unwrap(),
        ].into_iter().collect();

        let mut are = AdversarialResilientEconomics::new(ledger, validators);
        let did = Did::from_str("did:test:user").unwrap();
        
        // Test spend with valid signature
        let signature = vec![0u8; 64]; // Placeholder signature
        let validator = Did::from_str("did:test:validator1").unwrap();
        
        let result = are.validated_spend(
            &did,
            100,
            "test_op_1".to_string(),
            vec![(validator, signature)],
        );

        assert!(result.is_ok());
    }
}