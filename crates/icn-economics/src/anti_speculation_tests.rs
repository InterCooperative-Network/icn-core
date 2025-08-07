//! Tests for anti-speculation mechanisms in the token system
//!
//! This module tests the implementation of:
//! - Demurrage (automatic value decay for hoarded tokens)
//! - Velocity limits (maximum transfers per epoch)
//! - Purpose locks (tokens only redeemable for specified purposes)

use crate::ledger::{
    AntiSpeculationRules, FileResourceLedger, ResourceLedger, TokenClass, TransferTracker,
    VelocityLimits,
};
use icn_common::{Did, SystemTimeProvider, TimeProvider};
use std::str::FromStr;
use tempfile::TempDir;

/// Create a test DID for testing
fn test_did(id: &str) -> Did {
    Did::from_str(&format!("did:icn:cooperative:{}", id)).unwrap()
}

/// Convert string literal to String for testing
fn token_id(id: &str) -> String {
    id.to_string()
}

/// Create a test FileResourceLedger
fn create_test_ledger() -> (FileResourceLedger, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let ledger_path = temp_dir.path().join("test_ledger.json");
    let ledger = FileResourceLedger::new(ledger_path).unwrap();
    (ledger, temp_dir)
}

#[cfg(test)]
mod demurrage_tests {
    use super::*;

    #[test]
    fn test_demurrage_application() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");
        let holder = test_did("holder");

        // Create a token with 1% demurrage per day
        let token_class = TokenClass::new_resource_with_demurrage(
            "Test Token".to_string(),
            "Token with demurrage".to_string(),
            "TEST".to_string(),
            issuer.clone(),
            0.01, // 1% per day
            0,    // No grace period
        );

        // Create the token class
        ledger.create_class(&&token_id("test_token").to_string(), token_class).unwrap();

        // Mint 1000 tokens to the holder
        ledger.mint(&&token_id("test_token").to_string(), &holder, 1000).unwrap();
        assert_eq!(ledger.get_balance(&&token_id("test_token").to_string(), &holder), 1000);

        // Apply demurrage after 1 day (86400 seconds)
        let current_time = SystemTimeProvider.unix_seconds() + 86400;
        let burned_amount = ledger.apply_demurrage(&&token_id("test_token").to_string(), current_time).unwrap();

        // Should have burned ~10 tokens (1% of 1000)
        assert!(burned_amount >= 9 && burned_amount <= 11);
        let remaining_balance = ledger.get_balance(&token_id("test_token"), &holder);
        assert!(remaining_balance >= 989 && remaining_balance <= 991);
    }

    #[test]
    fn test_demurrage_grace_period() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");
        let holder = test_did("holder");

        // Create a token with 7-day grace period
        let token_class = TokenClass::new_resource_with_demurrage(
            "Test Token".to_string(),
            "Token with grace period".to_string(),
            "TEST".to_string(),
            issuer.clone(),
            0.01, // 1% per day
            7,    // 7 day grace period
        );

        ledger.create_class(&token_id("test_token"), token_class).unwrap();
        ledger.mint(&token_id("test_token"), &holder, 1000).unwrap();

        // Apply demurrage after 3 days (within grace period)
        let current_time = SystemTimeProvider.unix_seconds() + (3 * 86400);
        let burned_amount = ledger.apply_demurrage(&token_id("test_token"), current_time).unwrap();

        // Should not burn any tokens during grace period
        assert_eq!(burned_amount, 0);
        assert_eq!(ledger.get_balance(&token_id("test_token"), &holder), 1000);

        // Apply demurrage after 8 days (beyond grace period)
        let current_time = SystemTimeProvider.unix_seconds() + (8 * 86400);
        let burned_amount = ledger.apply_demurrage(&token_id("test_token"), current_time).unwrap();

        // Should now burn tokens
        assert!(burned_amount > 0);
        assert!(ledger.get_balance(&token_id("test_token"), &holder) < 1000);
    }

    #[test]
    fn test_no_demurrage_without_rules() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");
        let holder = test_did("holder");

        // Create a normal token without demurrage
        let token_class = TokenClass::new_fungible(
            "Normal Token".to_string(),
            "Token without demurrage".to_string(),
            "NORMAL".to_string(),
            2,
            issuer.clone(),
        );

        ledger.create_class(&token_id("normal_token"), token_class).unwrap();
        ledger.mint(&token_id("normal_token"), &holder, 1000).unwrap();

        // Apply demurrage after 1 day
        let current_time = SystemTimeProvider.unix_seconds() + 86400;
        let burned_amount = ledger.apply_demurrage(&token_id("normal_token"), current_time).unwrap();

        // Should not burn any tokens
        assert_eq!(burned_amount, 0);
        assert_eq!(ledger.get_balance(&token_id("normal_token"), &holder), 1000);
    }
}

#[cfg(test)]
mod velocity_limit_tests {
    use super::*;

    #[test]
    fn test_velocity_limits_enforcement() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");
        let sender = test_did("sender");
        let receiver = test_did("receiver");

        // Create a token with velocity limits: max 100 tokens per day
        let token_class = TokenClass::new_velocity_limited(
            "Limited Token".to_string(),
            "Token with velocity limits".to_string(),
            "LIMITED".to_string(),
            issuer.clone(),
            100, // Max 100 tokens per day
            Some(5), // Max 5 transfers per day
        );

        ledger.create_class(&token_id("limited_token"), token_class).unwrap();
        ledger.mint(&token_id("limited_token"), &sender, 1000).unwrap();

        let current_time = SystemTimeProvider.unix_seconds();

        // First transfer of 50 tokens should succeed
        assert!(ledger
            .check_velocity_limits(&token_id("limited_token"), &sender, 50, current_time)
            .unwrap());

        // Transfer the 50 tokens
        ledger
            .transfer(&token_id("limited_token"), &sender, &receiver, 50)
            .unwrap();

        // Another transfer of 50 tokens should succeed (total 100)
        assert!(ledger
            .check_velocity_limits(&token_id("limited_token"), &sender, 50, current_time)
            .unwrap());

        ledger
            .transfer(&token_id("limited_token"), &sender, &receiver, 50)
            .unwrap();

        // Another transfer of 1 token should fail (would exceed 100 limit)
        assert!(!ledger
            .check_velocity_limits(&token_id("limited_token"), &sender, 1, current_time)
            .unwrap());

        // Transfer should be rejected
        assert!(ledger
            .transfer(&token_id("limited_token"), &sender, &receiver, 1)
            .is_err());
    }

    #[test]
    fn test_velocity_limits_epoch_reset() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");
        let sender = test_did("sender");
        let receiver = test_did("receiver");

        let token_class = TokenClass::new_velocity_limited(
            "Limited Token".to_string(),
            "Token with velocity limits".to_string(),
            "LIMITED".to_string(),
            issuer.clone(),
            100, // Max 100 tokens per day
            None,
        );

        ledger.create_class(&token_id("limited_token"), token_class).unwrap();
        ledger.mint(&token_id("limited_token"), &sender, 1000).unwrap();

        let current_time = SystemTimeProvider.unix_seconds();

        // Transfer 100 tokens (reaching daily limit)
        ledger
            .transfer(&token_id("limited_token"), &sender, &receiver, 100)
            .unwrap();

        // Another transfer should fail
        assert!(!ledger
            .check_velocity_limits(&token_id("limited_token"), &sender, 1, current_time)
            .unwrap());

        // Move to next day
        let next_day = current_time + 86400;

        // Transfer should now succeed again
        assert!(ledger
            .check_velocity_limits(&token_id("limited_token"), &sender, 50, next_day)
            .unwrap());
    }

    #[test]
    fn test_transfer_count_limits() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");
        let sender = test_did("sender");
        let receiver = test_did("receiver");

        let token_class = TokenClass::new_velocity_limited(
            "Limited Token".to_string(),
            "Token with transfer count limits".to_string(),
            "LIMITED".to_string(),
            issuer.clone(),
            1000, // High amount limit
            Some(2), // Only 2 transfers per day
        );

        ledger.create_class(&token_id("limited_token"), token_class).unwrap();
        ledger.mint(&token_id("limited_token"), &sender, 1000).unwrap();

        // First two transfers should succeed
        ledger
            .transfer(&token_id("limited_token"), &sender, &receiver, 1)
            .unwrap();
        ledger
            .transfer(&token_id("limited_token"), &sender, &receiver, 1)
            .unwrap();

        // Third transfer should fail (exceeds count limit)
        assert!(ledger
            .transfer(&token_id("limited_token"), &sender, &receiver, 1)
            .is_err());
    }
}

#[cfg(test)]
mod purpose_lock_tests {
    use super::*;

    #[test]
    fn test_purpose_lock_validation() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");

        // Create a token that can only be used for "computation" and "storage"
        let token_class = TokenClass::new_purpose_locked(
            "Purpose Token".to_string(),
            "Token with purpose restrictions".to_string(),
            "PURPOSE".to_string(),
            issuer.clone(),
            vec!["computation".to_string(), "storage".to_string()],
        );

        ledger.create_class(&token_id("purpose_token"), token_class).unwrap();

        // Check allowed purposes
        assert!(ledger
            .check_purpose_lock(&token_id("purpose_token"), "computation")
            .unwrap());
        assert!(ledger
            .check_purpose_lock(&token_id("purpose_token"), "storage")
            .unwrap());

        // Check disallowed purposes
        assert!(!ledger
            .check_purpose_lock(&token_id("purpose_token"), "speculation")
            .unwrap());
        assert!(!ledger
            .check_purpose_lock(&token_id("purpose_token"), "trading")
            .unwrap());
    }

    #[test]
    fn test_no_purpose_locks() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");

        // Create a normal token without purpose locks
        let token_class = TokenClass::new_fungible(
            "Normal Token".to_string(),
            "Token without purpose restrictions".to_string(),
            "NORMAL".to_string(),
            2,
            issuer.clone(),
        );

        ledger.create_class(&token_id("normal_token"), token_class).unwrap();

        // All purposes should be allowed
        assert!(ledger
            .check_purpose_lock(&token_id("normal_token"), "anything")
            .unwrap());
        assert!(ledger
            .check_purpose_lock(&token_id("normal_token"), "speculation")
            .unwrap());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_combined_anti_speculation_mechanisms() {
        let (ledger, _temp_dir) = create_test_ledger();
        let issuer = test_did("issuer");
        let holder = test_did("holder");
        let receiver = test_did("receiver");

        // Create a token with all anti-speculation mechanisms
        let velocity_limits = VelocityLimits {
            max_transfer_per_epoch: 50,
            epoch_duration: 86400, // 1 day
            max_transfers_per_epoch: Some(3),
        };

        let anti_speculation = AntiSpeculationRules {
            demurrage_rate: Some(0.01), // 1% per day
            velocity_limits: Some(velocity_limits),
            purpose_locks: Some(vec!["resource".to_string()]),
            demurrage_grace_period: Some(86400), // 1 day grace period
        };

        let mut token_class = TokenClass::new_fungible(
            "Full Protection Token".to_string(),
            "Token with all protections".to_string(),
            "PROTECTED".to_string(),
            0,
            issuer.clone(),
        );
        token_class.anti_speculation = Some(anti_speculation);

        ledger.create_class(&token_id("protected_token"), token_class).unwrap();
        ledger.mint(&token_id("protected_token"), &holder, 1000).unwrap();

        let current_time = SystemTimeProvider.unix_seconds();

        // Test velocity limits
        ledger
            .transfer(&token_id("protected_token"), &holder, &receiver, 50)
            .unwrap();

        // Should fail due to velocity limits
        assert!(ledger
            .transfer(&token_id("protected_token"), &holder, &receiver, 1)
            .is_err());

        // Test purpose locks
        assert!(ledger
            .check_purpose_lock(&token_id("protected_token"), "resource")
            .unwrap());
        assert!(!ledger
            .check_purpose_lock(&token_id("protected_token"), "speculation")
            .unwrap());

        // Test demurrage (should not apply during grace period)
        let burned = ledger
            .apply_demurrage(&token_id("protected_token"), current_time + 3600)
            .unwrap(); // 1 hour later
        assert_eq!(burned, 0); // Still in grace period

        // After grace period, demurrage should apply
        let burned = ledger
            .apply_demurrage(&token_id("protected_token"), current_time + 2 * 86400)
            .unwrap(); // 2 days later
        assert!(burned > 0); // Should have burned some tokens
    }
}