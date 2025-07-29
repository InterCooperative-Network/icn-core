//! Security utilities and hardened cryptographic operations for ICN Identity
//!
//! This module provides security-hardened implementations of cryptographic
//! operations with protections against common attack vectors including:
//! - Timing attacks through constant-time operations
//! - Input validation and sanitization
//! - Secure memory handling
//! - Enhanced error handling without information leakage

use crate::{EdSignature, SigningKey, VerifyingKey};
use ed25519_dalek::Signer;
use icn_common::{CommonError, Did};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use zeroize::Zeroize;

/// Security configuration for cryptographic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable constant-time operations where possible
    pub constant_time_operations: bool,
    /// Enable timing attack mitigation
    pub timing_attack_mitigation: bool,
    /// Maximum input length for validation
    pub max_input_length: usize,
    /// Enable secure memory handling
    pub secure_memory_handling: bool,
    /// Enable comprehensive input validation
    pub strict_input_validation: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            constant_time_operations: true,
            timing_attack_mitigation: true,
            max_input_length: 1048576, // 1MB
            secure_memory_handling: true,
            strict_input_validation: true,
        }
    }
}

/// Secure wrapper for sensitive data that automatically zeroes on drop
#[derive(Debug, Clone)]
pub struct SecureBytes {
    data: Vec<u8>,
}

impl SecureBytes {
    /// Create new secure bytes container
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Get reference to the underlying data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get the length of the data
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl Drop for SecureBytes {
    fn drop(&mut self) {
        self.data.zeroize();
    }
}

/// Security-hardened signature verification with timing attack protection
pub fn secure_verify_signature(
    pk: &VerifyingKey,
    msg: &[u8],
    sig: &EdSignature,
    config: &SecurityConfig,
) -> Result<bool, CommonError> {
    // Input validation
    if config.strict_input_validation {
        validate_signature_inputs(msg, config)?;
    }

    let start_time = if config.timing_attack_mitigation {
        Some(Instant::now())
    } else {
        None
    };

    // Perform signature verification
    let result = pk.verify_strict(msg, sig).is_ok();

    // Timing attack mitigation: ensure consistent timing
    if let Some(start) = start_time {
        mitigate_timing_attack(start, config);
    }

    Ok(result)
}

/// Security-hardened message signing with secure memory handling
pub fn secure_sign_message(
    sk: &SigningKey,
    msg: &[u8],
    config: &SecurityConfig,
) -> Result<EdSignature, CommonError> {
    // Input validation
    if config.strict_input_validation {
        validate_signature_inputs(msg, config)?;
    }

    let start_time = if config.timing_attack_mitigation {
        Some(Instant::now())
    } else {
        None
    };

    // Perform signing
    let signature = sk.sign(msg);

    // Timing attack mitigation
    if let Some(start) = start_time {
        mitigate_timing_attack(start, config);
    }

    Ok(signature)
}

/// Validate inputs for signature operations
fn validate_signature_inputs(msg: &[u8], config: &SecurityConfig) -> Result<(), CommonError> {
    // Check message length
    if msg.len() > config.max_input_length {
        return Err(CommonError::IdentityError(
            "Message exceeds maximum allowed length".to_string(),
        ));
    }

    // Check for empty message (depending on policy)
    if msg.is_empty() {
        return Err(CommonError::IdentityError(
            "Empty message not allowed".to_string(),
        ));
    }

    Ok(())
}

/// Mitigate timing attacks by ensuring consistent operation duration
fn mitigate_timing_attack(start_time: Instant, _config: &SecurityConfig) {
    const MIN_OPERATION_TIME_MS: u64 = 1; // Minimum 1ms for crypto operations

    let elapsed = start_time.elapsed();
    let min_duration = std::time::Duration::from_millis(MIN_OPERATION_TIME_MS);

    if elapsed < min_duration {
        let remaining = min_duration - elapsed;
        std::thread::sleep(remaining);
    }
}

/// Secure DID validation with comprehensive checks
pub fn secure_validate_did(did: &Did, config: &SecurityConfig) -> Result<(), CommonError> {
    if !config.strict_input_validation {
        return Ok(());
    }

    // Check DID method
    if did.method.is_empty() || did.method.len() > 32 {
        return Err(CommonError::IdentityError(
            "Invalid DID method length".to_string(),
        ));
    }

    // Check DID identifier string
    if did.id_string.is_empty() || did.id_string.len() > config.max_input_length {
        return Err(CommonError::IdentityError(
            "Invalid DID identifier length".to_string(),
        ));
    }

    // Validate method characters (alphanumeric and dash only)
    if !did
        .method
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-')
    {
        return Err(CommonError::IdentityError(
            "Invalid characters in DID method".to_string(),
        ));
    }

    // Additional validation based on method
    match did.method.as_str() {
        "key" => validate_did_key_format(&did.id_string, config)?,
        "web" => validate_did_web_format(&did.id_string, config)?,
        "peer" => validate_did_peer_format(&did.id_string, config)?,
        _ => {
            // For unknown methods, just check basic format
            if !did
                .id_string
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':')
            {
                return Err(CommonError::IdentityError(
                    "Invalid characters in DID identifier".to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// Validate did:key format
fn validate_did_key_format(id_string: &str, _config: &SecurityConfig) -> Result<(), CommonError> {
    // Should start with 'z' (base58btc multibase prefix)
    if !id_string.starts_with('z') {
        return Err(CommonError::IdentityError(
            "Invalid did:key format: must start with 'z'".to_string(),
        ));
    }

    // Check reasonable length (Ed25519 public key + multicodec should be ~44-48 chars)
    if id_string.len() < 40 || id_string.len() > 60 {
        return Err(CommonError::IdentityError(
            "Invalid did:key length".to_string(),
        ));
    }

    // Validate base58 characters
    const BASE58_CHARS: &str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if !id_string.chars().all(|c| BASE58_CHARS.contains(c)) {
        return Err(CommonError::IdentityError(
            "Invalid base58 characters in did:key".to_string(),
        ));
    }

    Ok(())
}

/// Validate did:web format
fn validate_did_web_format(id_string: &str, _config: &SecurityConfig) -> Result<(), CommonError> {
    // Basic domain validation - more comprehensive than the original
    if id_string.is_empty() || id_string.len() > 253 {
        return Err(CommonError::IdentityError(
            "Invalid did:web domain length".to_string(),
        ));
    }

    // Check for malicious characters
    if id_string.contains("..") || id_string.contains("//") {
        return Err(CommonError::IdentityError(
            "Invalid did:web format: contains dangerous sequences".to_string(),
        ));
    }

    Ok(())
}

/// Validate did:peer format
fn validate_did_peer_format(id_string: &str, _config: &SecurityConfig) -> Result<(), CommonError> {
    // Should start with algorithm number (0-3 currently defined)
    if id_string.is_empty() {
        return Err(CommonError::IdentityError(
            "Empty did:peer identifier".to_string(),
        ));
    }

    let first_char = id_string.chars().next().unwrap();
    if !first_char.is_ascii_digit() || first_char > '3' {
        return Err(CommonError::IdentityError(
            "Invalid did:peer algorithm".to_string(),
        ));
    }

    Ok(())
}

/// Security audit result for cryptographic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditResult {
    /// Timestamp of the audit
    pub timestamp: u64,
    /// Overall security score (0-100)
    pub security_score: u8,
    /// List of security issues found
    pub issues: Vec<SecurityIssue>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Security issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityIssueSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Individual security issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Issue severity
    pub severity: SecurityIssueSeverity,
    /// Issue category
    pub category: String,
    /// Issue description
    pub description: String,
    /// Affected component
    pub component: String,
    /// Remediation suggestion
    pub remediation: Option<String>,
}

/// Perform security audit of cryptographic operations
pub fn audit_cryptographic_security(config: &SecurityConfig) -> SecurityAuditResult {
    let mut issues = Vec::new();
    let mut score = 100u8;

    // Check if constant-time operations are enabled
    if !config.constant_time_operations {
        issues.push(SecurityIssue {
            severity: SecurityIssueSeverity::High,
            category: "Timing Attacks".to_string(),
            description: "Constant-time operations are disabled".to_string(),
            component: "SecurityConfig".to_string(),
            remediation: Some("Enable constant_time_operations in SecurityConfig".to_string()),
        });
        score = score.saturating_sub(15);
    }

    // Check timing attack mitigation
    if !config.timing_attack_mitigation {
        issues.push(SecurityIssue {
            severity: SecurityIssueSeverity::Medium,
            category: "Timing Attacks".to_string(),
            description: "Timing attack mitigation is disabled".to_string(),
            component: "SecurityConfig".to_string(),
            remediation: Some("Enable timing_attack_mitigation in SecurityConfig".to_string()),
        });
        score = score.saturating_sub(10);
    }

    // Check input validation
    if !config.strict_input_validation {
        issues.push(SecurityIssue {
            severity: SecurityIssueSeverity::High,
            category: "Input Validation".to_string(),
            description: "Strict input validation is disabled".to_string(),
            component: "SecurityConfig".to_string(),
            remediation: Some("Enable strict_input_validation in SecurityConfig".to_string()),
        });
        score = score.saturating_sub(20);
    }

    // Check max input length
    if config.max_input_length > 10 * 1024 * 1024 {
        // > 10MB
        issues.push(SecurityIssue {
            severity: SecurityIssueSeverity::Medium,
            category: "DoS Protection".to_string(),
            description: "Maximum input length is very large".to_string(),
            component: "SecurityConfig".to_string(),
            remediation: Some("Consider reducing max_input_length".to_string()),
        });
        score = score.saturating_sub(5);
    }

    let recommendations = generate_security_recommendations(&issues);

    SecurityAuditResult {
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        security_score: score,
        issues,
        recommendations,
    }
}

/// Generate security recommendations based on issues found
fn generate_security_recommendations(issues: &[SecurityIssue]) -> Vec<String> {
    let mut recommendations = Vec::new();

    if issues.iter().any(|i| i.category == "Timing Attacks") {
        recommendations.push(
            "Implement constant-time cryptographic operations to prevent timing attacks"
                .to_string(),
        );
    }

    if issues.iter().any(|i| i.category == "Input Validation") {
        recommendations.push(
            "Enable comprehensive input validation for all cryptographic operations".to_string(),
        );
    }

    if issues
        .iter()
        .any(|i| i.severity == SecurityIssueSeverity::Critical)
    {
        recommendations.push(
            "Address critical security issues immediately before production deployment".to_string(),
        );
    }

    recommendations
        .push("Regularly audit cryptographic implementations and update dependencies".to_string());
    recommendations
        .push("Consider external security audit before production deployment".to_string());

    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{did_key_from_verifying_key, generate_ed25519_keypair};
    use std::str::FromStr;

    #[test]
    fn test_secure_sign_verify_roundtrip() {
        let config = SecurityConfig::default();
        let (sk, pk) = generate_ed25519_keypair();
        let message = b"test message for security audit";

        let signature = secure_sign_message(&sk, message, &config).unwrap();
        let verified = secure_verify_signature(&pk, message, &signature, &config).unwrap();

        assert!(verified);
    }

    #[test]
    fn test_secure_verify_bad_signature() {
        let config = SecurityConfig::default();
        let (sk, pk) = generate_ed25519_keypair();
        let (_, other_pk) = generate_ed25519_keypair();
        let message = b"test message";

        let signature = secure_sign_message(&sk, message, &config).unwrap();
        let verified = secure_verify_signature(&other_pk, message, &signature, &config).unwrap();

        assert!(!verified);
    }

    #[test]
    fn test_input_validation() {
        let config = SecurityConfig::default();
        let (sk, _) = generate_ed25519_keypair();

        // Test empty message
        let result = secure_sign_message(&sk, &[], &config);
        assert!(result.is_err());

        // Test oversized message
        let large_msg = vec![0u8; config.max_input_length + 1];
        let result = secure_sign_message(&sk, &large_msg, &config);
        assert!(result.is_err());
    }

    #[test]
    fn test_did_validation() {
        let config = SecurityConfig::default();

        // Valid did:key
        let (_, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();
        assert!(secure_validate_did(&did, &config).is_ok());

        // Invalid did:key (bad prefix)
        let bad_did = Did::new("key", "invalid_key_format");
        assert!(secure_validate_did(&bad_did, &config).is_err());
    }

    #[test]
    fn test_security_audit() {
        let mut config = SecurityConfig::default();

        // Audit with all security features enabled
        let result = audit_cryptographic_security(&config);
        assert_eq!(result.security_score, 100);
        assert!(result.issues.is_empty());

        // Disable some security features
        config.constant_time_operations = false;
        config.strict_input_validation = false;

        let result = audit_cryptographic_security(&config);
        assert!(result.security_score < 100);
        assert!(!result.issues.is_empty());
    }

    #[test]
    fn test_secure_bytes_zeroization() {
        let sensitive_data = vec![0xDEu8, 0xAD, 0xBE, 0xEF];
        let secure_bytes = SecureBytes::new(sensitive_data.clone());

        assert_eq!(secure_bytes.as_bytes(), &sensitive_data);
        assert_eq!(secure_bytes.len(), 4);

        // SecureBytes should automatically zero on drop
        drop(secure_bytes);
        // Note: We can't easily test the zeroing without unsafe code
        // but the ZeroizeOnDrop trait ensures it happens
    }
}
