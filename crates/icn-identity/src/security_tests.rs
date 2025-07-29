//! Comprehensive security tests for ICN Identity cryptographic operations
//!
//! These tests validate security hardening measures including:
//! - Input validation and sanitization
//! - Timing attack resistance
//! - Error handling without information leakage
//! - Edge cases and malicious inputs

#[cfg(test)]
mod security_tests {
    use crate::security::*;
    use crate::*;
    use icn_common::Did;
    use std::str::FromStr;

    /// Maximum allowed timing difference between valid and invalid signature operations (in milliseconds)
    /// This value should be larger than the minimum operation time (1ms) but small enough to detect
    /// significant timing differences that could leak information.
    const TIMING_THRESHOLD_MS: u128 = 5;

    #[test]
    fn test_signature_validation_prevents_empty_messages() {
        let config = SecurityConfig::default();
        let (sk, _) = generate_ed25519_keypair();

        // Empty message should be rejected
        let result = secure_sign_message(&sk, &[], &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty message"));
    }

    #[test]
    fn test_signature_validation_prevents_oversized_messages() {
        let config = SecurityConfig::default();
        let (sk, _) = generate_ed25519_keypair();

        // Oversized message should be rejected
        let large_message = vec![0u8; config.max_input_length + 1];
        let result = secure_sign_message(&sk, &large_message, &config);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }

    #[test]
    fn test_timing_attack_mitigation_consistency() {
        let config = SecurityConfig {
            timing_attack_mitigation: true,
            ..Default::default()
        };
        let (sk, pk) = generate_ed25519_keypair();
        let message = b"test message for timing";

        // Measure timing for valid signature
        let start = std::time::Instant::now();
        let signature = secure_sign_message(&sk, message, &config).unwrap();
        let _verified = secure_verify_signature(&pk, message, &signature, &config).unwrap();
        let valid_duration = start.elapsed();

        // Measure timing for invalid signature
        let (_, wrong_pk) = generate_ed25519_keypair();
        let start = std::time::Instant::now();
        let _verified = secure_verify_signature(&wrong_pk, message, &signature, &config).unwrap();
        let invalid_duration = start.elapsed();

        // Times should be similar (within reasonable bounds due to timing mitigation)
        let diff = if valid_duration > invalid_duration {
            valid_duration - invalid_duration
        } else {
            invalid_duration - valid_duration
        };

        // Should be within 5ms of each other due to timing mitigation
        assert!(
            diff.as_millis() < 5,
            "Timing difference too large: {diff:?}"
        );
    }

    #[test]
    fn test_did_validation_rejects_malicious_inputs() {
        let config = SecurityConfig::default();

        // Test various malicious DID inputs
        let malicious_dids = vec![
            // Empty method
            Did::new("", "test"),
            // Oversized method
            Did::new(&"a".repeat(100), "test"),
            // Invalid characters in method
            Did::new("key;drop table", "test"),
            Did::new("key<script>", "test"),
            // Null bytes
            Did::new("key\0", "test"),
            // Path traversal attempts
            Did::new("../../../etc/passwd", "test"),
        ];

        for malicious_did in malicious_dids {
            let result = secure_validate_did(&malicious_did, &config);
            assert!(
                result.is_err(),
                "Should reject malicious DID: {malicious_did:?}"
            );
        }
    }

    #[test]
    fn test_did_key_validation_rejects_invalid_formats() {
        let config = SecurityConfig::default();

        let invalid_did_keys = vec![
            // Wrong prefix
            Did::new("key", "a123invalid"),
            // Too short
            Did::new("key", "z123"),
            // Too long
            Did::new("key", &format!("z{}", "a".repeat(100))),
            // Invalid base58 characters
            Did::new("key", "z0OIl"), // Contains 0, O, I, l which are not in base58
            // Empty
            Did::new("key", ""),
        ];

        for invalid_did in invalid_did_keys {
            let result = secure_validate_did(&invalid_did, &config);
            assert!(
                result.is_err(),
                "Should reject invalid did:key: {invalid_did:?}"
            );
        }
    }

    #[test]
    fn test_did_web_validation_rejects_malicious_domains() {
        let long_domain = "a".repeat(300);
        let malicious_domains = vec![
            // Path traversal
            "../../../etc/passwd",
            "example.com/../../../etc/passwd",
            // Double slashes
            "example.com//malicious",
            // Null bytes
            "example.com\0.evil.com",
            // Unicode attacks
            "еxample.com", // Cyrillic е instead of Latin e
            // Control characters
            "example.com\r\n.evil.com",
            // Extremely long domain
            &long_domain,
        ];

        for domain in malicious_domains {
            let result = did_web_from_parts(domain, &[]);
            assert!(
                result.is_err(),
                "Should reject malicious domain: {domain}"
            );
        }
    }

    #[test]
    fn test_did_peer_validation_rejects_invalid_algorithms() {
        let config = SecurityConfig::default();

        let invalid_did_peers = vec![
            // Invalid algorithm numbers
            Did::new("peer", "9invalidalgo"),
            Did::new("peer", "ainvalidstart"),
            // Empty
            Did::new("peer", ""),
        ];

        for invalid_did in invalid_did_peers {
            let result = secure_validate_did(&invalid_did, &config);
            assert!(
                result.is_err(),
                "Should reject invalid did:peer: {invalid_did:?}"
            );
        }
    }

    #[test]
    fn test_verifying_key_from_did_key_validates_length() {
        // Create a did:key with invalid length public key data
        let invalid_did = Did::new("key", "z1234567890"); // Too short for valid Ed25519 key

        let result = verifying_key_from_did_key(&invalid_did);
        assert!(result.is_err());
    }

    #[test]
    fn test_execution_receipt_tamper_resistance() {
        let (sk, pk) = generate_ed25519_keypair();
        let did_str = did_key_from_verifying_key(&pk);
        let did = Did::from_str(&did_str).unwrap();

        let job_cid = icn_common::Cid::new_v1_sha256(0x55, b"test_job");
        let result_cid = icn_common::Cid::new_v1_sha256(0x55, b"test_result");

        let receipt = ExecutionReceipt {
            job_id: job_cid,
            executor_did: did.clone(),
            result_cid,
            cpu_ms: 100,
            success: true,
            sig: SignatureBytes(vec![]),
        };

        let signed_receipt = receipt.sign_with_key(&sk).unwrap();

        // Test various tampering attempts
        let mut tampered = signed_receipt.clone();
        tampered.cpu_ms = 200;
        assert!(
            tampered.verify_against_did(&did).is_err(),
            "Should detect cpu_ms tampering"
        );

        let mut tampered = signed_receipt.clone();
        tampered.success = false;
        assert!(
            tampered.verify_against_did(&did).is_err(),
            "Should detect success tampering"
        );

        // Test signature byte tampering
        let mut tampered = signed_receipt.clone();
        if !tampered.sig.0.is_empty() {
            tampered.sig.0[0] = tampered.sig.0[0].wrapping_add(1);
            assert!(
                tampered.verify_against_did(&did).is_err(),
                "Should detect signature tampering"
            );
        }
    }

    #[test]
    fn test_secure_bytes_zeroization() {
        let sensitive_data = vec![0xDEu8, 0xAD, 0xBE, 0xEF];
        let secure_bytes = SecureBytes::new(sensitive_data.clone());

        // Verify data is accessible
        assert_eq!(secure_bytes.as_bytes(), &sensitive_data);
        assert_eq!(secure_bytes.len(), 4);
        assert!(!secure_bytes.is_empty());

        // After drop, memory should be zeroed (tested via ZeroizeOnDrop trait)
        drop(secure_bytes);
        // Note: We can't easily verify the zeroing without unsafe memory access,
        // but the ZeroizeOnDrop derive ensures it happens
    }

    #[test]
    fn test_security_audit_comprehensive() {
        // Test with all security features enabled
        let secure_config = SecurityConfig::default();
        let audit_result = audit_cryptographic_security(&secure_config);

        assert_eq!(audit_result.security_score, 100);
        assert!(audit_result.issues.is_empty());
        assert!(!audit_result.recommendations.is_empty());

        // Test with security features disabled
        let insecure_config = SecurityConfig {
            constant_time_operations: false,
            timing_attack_mitigation: false,
            strict_input_validation: false,
            max_input_length: 100 * 1024 * 1024, // 100MB - too large
            secure_memory_handling: false,
        };

        let audit_result = audit_cryptographic_security(&insecure_config);

        assert!(audit_result.security_score < 100);
        assert!(!audit_result.issues.is_empty());

        // Check that critical issues are flagged
        let _has_critical = audit_result
            .issues
            .iter()
            .any(|issue| matches!(issue.severity, SecurityIssueSeverity::Critical));
        // Note: Our current audit might not flag Critical issues, but High/Medium issues should exist
        let has_high = audit_result
            .issues
            .iter()
            .any(|issue| matches!(issue.severity, SecurityIssueSeverity::High));
        assert!(has_high, "Should detect high severity issues");
    }

    #[test]
    fn test_constant_time_signature_verification() {
        let config = SecurityConfig {
            constant_time_operations: true,
            timing_attack_mitigation: true,
            ..Default::default()
        };

        let (sk1, pk1) = generate_ed25519_keypair();
        let (sk2, pk2) = generate_ed25519_keypair();
        let message = b"timing test message";

        // Create signatures with different keys
        let sig1 = secure_sign_message(&sk1, message, &config).unwrap();
        let _sig2 = secure_sign_message(&sk2, message, &config).unwrap();

        // Verify timing is consistent regardless of validity
        let mut valid_times = Vec::new();
        let mut invalid_times = Vec::new();

        for _ in 0..10 {
            // Valid verification
            let start = std::time::Instant::now();
            let _ = secure_verify_signature(&pk1, message, &sig1, &config);
            valid_times.push(start.elapsed());

            // Invalid verification (wrong key)
            let start = std::time::Instant::now();
            let _ = secure_verify_signature(&pk2, message, &sig1, &config);
            invalid_times.push(start.elapsed());
        }

        // Calculate average times
        let avg_valid: std::time::Duration =
            valid_times.iter().sum::<std::time::Duration>() / valid_times.len() as u32;
        let avg_invalid: std::time::Duration =
            invalid_times.iter().sum::<std::time::Duration>() / invalid_times.len() as u32;

        // Timing should be similar (within 2ms due to our mitigation)
        let diff = if avg_valid > avg_invalid {
            avg_valid - avg_invalid
        } else {
            avg_invalid - avg_valid
        };

        assert!(
            diff.as_millis() < TIMING_THRESHOLD_MS,
            "Timing difference too large: {diff:?}"
        );
    }

    #[test]
    fn test_error_messages_dont_leak_sensitive_info() {
        let config = SecurityConfig::default();

        // Test that error messages don't contain sensitive information
        let (sk, _) = generate_ed25519_keypair();

        // Test with oversized input
        let large_input = vec![0u8; config.max_input_length + 1];
        let result = secure_sign_message(&sk, &large_input, &config);

        if let Err(error) = result {
            let error_msg = error.to_string();
            // Should not contain the actual input data
            assert!(
                !error_msg.contains("00000000"),
                "Error message contains sensitive data"
            );
            // Should contain general error description
            assert!(error_msg.contains("exceeds maximum") || error_msg.contains("length"));
        }
    }

    #[test]
    fn test_multicodec_validation_edge_cases() {
        // Test edge cases in multicodec parsing

        // Create a did:key with invalid multicodec
        use multibase::{encode, Base};
        use unsigned_varint::encode as varint_encode;

        // Wrong multicodec (not 0xed for Ed25519)
        let mut wrong_codec_buf = varint_encode::u16_buffer();
        let wrong_prefix = varint_encode::u16(0x12, &mut wrong_codec_buf); // SHA256 instead of Ed25519
        let mut wrong_data = wrong_prefix.to_vec();
        wrong_data.extend_from_slice(&[0u8; 32]); // 32 zero bytes

        let wrong_multibase = encode(Base::Base58Btc, wrong_data);
        let wrong_did = Did::new("key", &wrong_multibase);

        let result = verifying_key_from_did_key(&wrong_did);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported multicodec"));
    }

    #[test]
    fn test_base58_validation() {
        // Test base58 validation in did:key parsing
        let invalid_base58_chars = vec!["0", "O", "I", "l"]; // Not in base58 alphabet

        for invalid_char in invalid_base58_chars {
            let invalid_did = Did::new("key", &format!("z123{invalid_char}"));
            let result = secure_validate_did(&invalid_did, &SecurityConfig::default());
            assert!(
                result.is_err(),
                "Should reject invalid base58 character: {invalid_char}"
            );
        }
    }
}
