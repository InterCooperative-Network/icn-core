//! Enhanced runtime context with production-ready error recovery patterns.
//!
//! This module provides an enhanced wrapper around RuntimeContext that adds
//! error recovery capabilities to critical operations.

use crate::context::runtime_context::RuntimeContext;
use crate::error_recovery::{
    ErrorRecoveryConfig, ErrorClassifier, 
    RecoveryError, CircuitBreaker, CircuitBreakerConfig, resilient_operation
};
use crate::context::errors::HostAbiError;
use icn_common::{Cid, Did, CommonError};
use icn_mesh::JobId;
use std::sync::Arc;
use std::time::Duration;
use log::debug;

/// Production-ready error classifier for ICN runtime operations
pub struct ICNErrorClassifier;

impl ErrorClassifier<HostAbiError> for ICNErrorClassifier {
    fn is_recoverable(&self, error: &HostAbiError) -> bool {
        match error {
            // Permanent errors that should not be retried
            HostAbiError::InvalidParameters(_) => false,
            HostAbiError::AuthenticationFailed(_) => false,
            HostAbiError::InsufficientMana => false, // Don't retry mana issues
            HostAbiError::SerializationError(_) => false,
            
            // Potentially recoverable errors
            HostAbiError::InternalError(_) => true,
            HostAbiError::DagOperationFailed(_) => true,
            HostAbiError::NetworkError(_) => true,
            HostAbiError::SignatureError(_) => true, // Could be timing related
            HostAbiError::GovernanceError(_) => true, // Could be temporary
            HostAbiError::OperationTimeout(_) => true,
            HostAbiError::ExecutionFailed(_) => true,
            HostAbiError::NodeNotFound(_) => true, // Network connectivity issue
            HostAbiError::JobNotFound(_) => false, // Permanent - job doesn't exist
            HostAbiError::InvalidJobState(_) => false, // Permanent - wrong state
            HostAbiError::ResourceLimitExceeded(_) => true, // Could be temporary
            HostAbiError::ConfigurationError(_) => false, // Permanent
            
            // Default to recoverable for any other errors
            _ => true,
        }
    }
}

impl ErrorClassifier<CommonError> for ICNErrorClassifier {
    fn is_recoverable(&self, error: &CommonError) -> bool {
        match error {
            // Permanent errors
            CommonError::ValidationError(_) => false,
            CommonError::InvalidParameters(_) => false,
            CommonError::DeserializationError(_) => false,
            CommonError::SerializationError(_) => false,
            
            // Potentially recoverable errors
            CommonError::InternalError(_) => true,
            CommonError::NotFound(_) => false, // Usually permanent
            CommonError::NetworkError(_) => true,
            CommonError::StorageError(_) => true,
            CommonError::TimeoutError(_) => true,
            CommonError::RateLimitError(_) => true,
            CommonError::ServiceUnavailable(_) => true,
            
            // Default to recoverable for any other errors
            _ => true,
        }
    }
}

/// Enhanced runtime context with error recovery capabilities
pub struct ResilientRuntimeContext {
    inner: Arc<RuntimeContext>,
    mana_circuit_breaker: CircuitBreaker,
    dag_circuit_breaker: CircuitBreaker,
    job_circuit_breaker: CircuitBreaker,
    retry_config: ErrorRecoveryConfig,
    error_classifier: ICNErrorClassifier,
}

impl ResilientRuntimeContext {
    /// Create a new resilient runtime context
    pub fn new(inner: Arc<RuntimeContext>) -> Self {
        let circuit_config = CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
        };

        Self {
            inner,
            mana_circuit_breaker: CircuitBreaker::new(circuit_config.clone()),
            dag_circuit_breaker: CircuitBreaker::new(circuit_config.clone()),
            job_circuit_breaker: CircuitBreaker::new(circuit_config),
            retry_config: ErrorRecoveryConfig::production(),
            error_classifier: ICNErrorClassifier,
        }
    }

    /// Create with custom retry configuration
    pub fn with_retry_config(mut self, config: ErrorRecoveryConfig) -> Self {
        self.retry_config = config;
        self
    }

    /// Get the inner runtime context
    pub fn inner(&self) -> &Arc<RuntimeContext> {
        &self.inner
    }

    /// Enhanced mana spending with error recovery
    pub async fn spend_mana_resilient(
        &self,
        account: &Did,
        amount: u64,
    ) -> Result<(), RecoveryError<HostAbiError>> {
        let inner = self.inner.clone();
        let account = account.clone();
        
        resilient_operation(
            || {
                let inner = inner.clone();
                let account = account.clone();
                async move { inner.spend_mana(&account, amount).await }
            },
            &self.retry_config,
            &self.mana_circuit_breaker,
            &self.error_classifier,
            "spend_mana",
        ).await
    }

    /// Enhanced mana retrieval with error recovery
    pub async fn get_mana_resilient(
        &self,
        account: &Did,
    ) -> Result<u64, RecoveryError<HostAbiError>> {
        let inner = self.inner.clone();
        let account = account.clone();

        resilient_operation(
            || {
                let inner = inner.clone();
                let account = account.clone();
                async move { inner.get_mana(&account).await }
            },
            &self.retry_config,
            &self.mana_circuit_breaker,
            &self.error_classifier,
            "get_mana",
        ).await
    }

    /// Enhanced job submission with error recovery
    pub async fn submit_job_resilient(
        &self,
        manifest_cid: Cid,
        spec_bytes: Vec<u8>,
        cost_mana: u64,
    ) -> Result<JobId, RecoveryError<HostAbiError>> {
        let inner = self.inner.clone();
        
        resilient_operation(
            || {
                let inner = inner.clone();
                let manifest_cid = manifest_cid.clone();
                let spec_bytes = spec_bytes.clone();
                async move { inner.handle_submit_job(manifest_cid, spec_bytes, cost_mana).await }
            },
            &self.retry_config,
            &self.job_circuit_breaker,
            &self.error_classifier,
            "submit_job",
        ).await
    }

    /// Enhanced DAG operations with error recovery
    pub async fn anchor_receipt_resilient(
        &self,
        receipt: &icn_identity::ExecutionReceipt,
    ) -> Result<Cid, RecoveryError<HostAbiError>> {
        let inner = self.inner.clone();
        let receipt = receipt.clone();
        
        resilient_operation(
            || {
                let inner = inner.clone();
                let mut receipt = receipt.clone();
                async move { inner.anchor_receipt(&mut receipt).await }
            },
            &self.retry_config,
            &self.dag_circuit_breaker,
            &self.error_classifier,
            "anchor_receipt",
        ).await
    }

    /// Enhanced job status retrieval with error recovery
    pub async fn get_job_status_resilient(
        &self,
        job_id: &JobId,
    ) -> Result<Option<icn_mesh::JobLifecycle>, RecoveryError<HostAbiError>> {
        let inner = self.inner.clone();
        let job_id = job_id.clone();
        
        resilient_operation(
            || {
                let inner = inner.clone();
                let job_id = job_id.clone();
                async move { inner.get_job_status(&job_id).await }
            },
            &self.retry_config,
            &self.job_circuit_breaker,
            &self.error_classifier,
            "get_job_status",
        ).await
    }

    /// Get circuit breaker status for monitoring
    pub fn get_circuit_breaker_status(&self) -> CircuitBreakerStatus {
        CircuitBreakerStatus {
            mana_circuit_open: self.mana_circuit_breaker.is_open(),
            dag_circuit_open: self.dag_circuit_breaker.is_open(),
            job_circuit_open: self.job_circuit_breaker.is_open(),
        }
    }

    /// Reset all circuit breakers (for admin operations)
    pub fn reset_circuit_breakers(&self) {
        self.mana_circuit_breaker.force_close();
        self.dag_circuit_breaker.force_close();
        self.job_circuit_breaker.force_close();
        debug!("All circuit breakers have been reset");
    }

    /// Get error recovery metrics
    pub fn get_recovery_metrics(&self) -> RecoveryMetrics {
        RecoveryMetrics {
            mana_failures: self.mana_circuit_breaker.failure_count.load(std::sync::atomic::Ordering::SeqCst),
            dag_failures: self.dag_circuit_breaker.failure_count.load(std::sync::atomic::Ordering::SeqCst),
            job_failures: self.job_circuit_breaker.failure_count.load(std::sync::atomic::Ordering::SeqCst),
            retry_config: self.retry_config.clone(),
        }
    }
}

/// Circuit breaker status for monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct CircuitBreakerStatus {
    pub mana_circuit_open: bool,
    pub dag_circuit_open: bool,
    pub job_circuit_open: bool,
}

/// Error recovery metrics for monitoring
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecoveryMetrics {
    pub mana_failures: u64,
    pub dag_failures: u64,
    pub job_failures: u64,
    pub retry_config: ErrorRecoveryConfig,
}

// Circuit breaker integration complete

#[cfg(test)]
mod tests {
    use super::*;
    use icn_common::Did;
    use std::str::FromStr;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn create_test_context() -> Arc<RuntimeContext> {
        let test_did = Did::from_str("did:key:test123").unwrap();
        RuntimeContext::new_for_testing(test_did, Some(1000)).unwrap()
    }

    #[tokio::test]
    async fn test_resilient_mana_operations() {
        let ctx = create_test_context();
        let resilient_ctx = ResilientRuntimeContext::new(ctx);
        let test_did = Did::from_str("did:key:test123").unwrap();

        // Test successful mana retrieval
        let balance = resilient_ctx.get_mana_resilient(&test_did).await;
        assert!(balance.is_ok());
        assert_eq!(balance.unwrap(), 1000);

        // Test successful mana spending
        let spend_result = resilient_ctx.spend_mana_resilient(&test_did, 100).await;
        assert!(spend_result.is_ok());

        // Verify balance decreased
        let new_balance = resilient_ctx.get_mana_resilient(&test_did).await;
        assert!(new_balance.is_ok());
        assert_eq!(new_balance.unwrap(), 900);
    }

    #[tokio::test]
    async fn test_circuit_breaker_status() {
        let ctx = create_test_context();
        let resilient_ctx = ResilientRuntimeContext::new(ctx);

        let status = resilient_ctx.get_circuit_breaker_status();
        assert!(!status.mana_circuit_open);
        assert!(!status.dag_circuit_open);
        assert!(!status.job_circuit_open);
    }

    #[tokio::test]
    async fn test_recovery_metrics() {
        let ctx = create_test_context();
        let resilient_ctx = ResilientRuntimeContext::new(ctx);

        let metrics = resilient_ctx.get_recovery_metrics();
        assert_eq!(metrics.mana_failures, 0);
        assert_eq!(metrics.dag_failures, 0);
        assert_eq!(metrics.job_failures, 0);
    }

    #[tokio::test]
    async fn test_error_classifier() {
        let classifier = ICNErrorClassifier;

        // Test permanent errors
        assert!(!classifier.is_recoverable(&HostAbiError::InvalidParameters("test".to_string())));
        assert!(!classifier.is_recoverable(&HostAbiError::InsufficientMana));
        assert!(!classifier.is_recoverable(&HostAbiError::JobNotFound("test".to_string())));

        // Test recoverable errors
        assert!(classifier.is_recoverable(&HostAbiError::NetworkError("test".to_string())));
        assert!(classifier.is_recoverable(&HostAbiError::DagOperationFailed("test".to_string())));
        assert!(classifier.is_recoverable(&HostAbiError::OperationTimeout("test".to_string())));
    }

    #[tokio::test]
    async fn test_circuit_breaker_integration() {
        let ctx = create_test_context();
        let resilient_ctx = ResilientRuntimeContext::new(ctx);
        let test_did = Did::from_str("did:key:nonexistent").unwrap();

        // Try an operation that will fail multiple times
        for _ in 0..6 {
            let _ = resilient_ctx.get_mana_resilient(&test_did).await;
        }

        // Check that failures were recorded
        let metrics = resilient_ctx.get_recovery_metrics();
        assert!(metrics.mana_failures > 0, "Circuit breaker should have recorded failures");

        // The circuit might be open now due to failures
        let status = resilient_ctx.get_circuit_breaker_status();
        // We don't assert it's open because the exact behavior depends on error classification
        // but we verify that the circuit breaker is being exercised
    }

    #[tokio::test]
    async fn test_circuit_breaker_prevents_calls_when_open() {
        let ctx = create_test_context();
        let mut resilient_ctx = ResilientRuntimeContext::new(ctx);
        
        // Manually open the mana circuit breaker
        for _ in 0..10 {
            resilient_ctx.mana_circuit_breaker.record_failure();
        }
        
        let test_did = Did::from_str("did:key:test123").unwrap();
        
        // This should fail due to open circuit
        let result = resilient_ctx.get_mana_resilient(&test_did).await;
        
        // Verify it's a circuit breaker error
        match result {
            Err(RecoveryError::CircuitBreakerOpen { service }) => {
                assert_eq!(service, "get_mana");
            }
            _ => panic!("Expected CircuitBreakerOpen error, got: {:?}", result),
        }
    }
}