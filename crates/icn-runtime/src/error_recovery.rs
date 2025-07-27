//! Production-ready error recovery patterns for ICN runtime operations.
//!
//! This module provides robust error handling patterns including retry mechanisms,
//! circuit breakers, and graceful degradation for critical operations.

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use log::{debug, warn, error};
use thiserror::Error;

/// Error recovery configuration for production operations
#[derive(Debug, Clone)]
pub struct ErrorRecoveryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Jitter factor to avoid thundering herd (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for ErrorRecoveryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }
}

impl ErrorRecoveryConfig {
    /// Production configuration with conservative retry settings
    pub fn production() -> Self {
        Self {
            max_retries: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 1.5,
            jitter_factor: 0.2,
        }
    }

    /// Development configuration with faster retries
    pub fn development() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
        }
    }

    /// Testing configuration with minimal delays
    pub fn testing() -> Self {
        Self {
            max_retries: 2,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            backoff_multiplier: 2.0,
            jitter_factor: 0.0,
        }
    }
}

/// Errors that can occur during error recovery operations
#[derive(Debug, Error)]
pub enum RecoveryError<E> {
    #[error("Operation failed after {attempts} attempts: {last_error}")]
    ExhaustedRetries { attempts: u32, last_error: E },
    #[error("Circuit breaker is open for {service}")]
    CircuitBreakerOpen { service: String },
    #[error("Operation timeout after {duration:?}")]
    Timeout { duration: Duration },
}

/// Trait for classifying errors as recoverable or not
pub trait ErrorClassifier<E> {
    /// Returns true if the error is recoverable and operation should be retried
    fn is_recoverable(&self, error: &E) -> bool;
    
    /// Returns true if the error indicates a permanent failure
    fn is_permanent(&self, error: &E) -> bool {
        !self.is_recoverable(error)
    }
}

/// Default error classifier that treats most errors as recoverable
pub struct DefaultErrorClassifier;

impl<E> ErrorClassifier<E> for DefaultErrorClassifier {
    fn is_recoverable(&self, _error: &E) -> bool {
        true // By default, assume errors are recoverable
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for preventing cascade failures
pub struct CircuitBreaker {
    state: Arc<std::sync::Mutex<CircuitState>>,
    failure_count: AtomicU64,
    success_count: AtomicU64,
    last_failure_time: Arc<std::sync::Mutex<Option<Instant>>>,
    config: CircuitBreakerConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures to trigger circuit opening
    pub failure_threshold: u64,
    /// Time to wait before attempting to close circuit
    pub recovery_timeout: Duration,
    /// Number of successful requests needed to close circuit in half-open state
    pub success_threshold: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 3,
        }
    }
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(std::sync::Mutex::new(CircuitState::Closed)),
            failure_count: AtomicU64::new(0),
            success_count: AtomicU64::new(0),
            last_failure_time: Arc::new(std::sync::Mutex::new(None)),
            config,
        }
    }

    /// Execute operation with circuit breaker protection
    pub async fn execute<F, Fut, T, E>(&self, service_name: &str, operation: F) -> Result<T, RecoveryError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        // Check if circuit is open
        if self.is_open() {
            return Err(RecoveryError::CircuitBreakerOpen {
                service: service_name.to_string(),
            });
        }

        match operation().await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(error) => {
                self.record_failure();
                Err(RecoveryError::ExhaustedRetries {
                    attempts: 1,
                    last_error: error,
                })
            }
        }
    }

    fn is_open(&self) -> bool {
        let state = self.state.lock().unwrap();
        match *state {
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() > self.config.recovery_timeout {
                        drop(state);
                        self.transition_to_half_open();
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => false,
            CircuitState::Closed => false,
        }
    }

    fn record_success(&self) {
        let prev_success = self.success_count.fetch_add(1, Ordering::SeqCst);
        let state = self.state.lock().unwrap();
        
        if *state == CircuitState::HalfOpen && prev_success + 1 >= self.config.success_threshold {
            drop(state);
            self.transition_to_closed();
        }
    }

    fn record_failure(&self) {
        let prev_failure = self.failure_count.fetch_add(1, Ordering::SeqCst);
        *self.last_failure_time.lock().unwrap() = Some(Instant::now());

        if prev_failure + 1 >= self.config.failure_threshold {
            self.transition_to_open();
        }
    }

    fn transition_to_open(&self) {
        let mut state = self.state.lock().unwrap();
        if *state != CircuitState::Open {
            *state = CircuitState::Open;
            warn!("Circuit breaker opened due to {} consecutive failures", self.failure_count.load(Ordering::SeqCst));
        }
    }

    fn transition_to_half_open(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::HalfOpen;
        self.success_count.store(0, Ordering::SeqCst);
        debug!("Circuit breaker transitioned to half-open state");
    }

    fn transition_to_closed(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        debug!("Circuit breaker closed after successful recovery");
    }

    /// Check if circuit breaker is currently open
    pub fn is_open(&self) -> bool {
        let state = self.state.lock().unwrap();
        match *state {
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
                    if last_failure.elapsed() > self.config.recovery_timeout {
                        drop(state);
                        self.transition_to_half_open();
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            CircuitState::HalfOpen => false,
            CircuitState::Closed => false,
        }
    }

    /// Manually close the circuit breaker (for admin operations)
    pub fn force_close(&self) {
        let mut state = self.state.lock().unwrap();
        *state = CircuitState::Closed;
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
        debug!("Circuit breaker manually closed");
    }
}

/// Retry operation with exponential backoff and jitter
pub async fn retry_with_backoff<F, Fut, T, E, C>(
    operation: F,
    config: &ErrorRecoveryConfig,
    classifier: &C,
    service_name: &str,
) -> Result<T, RecoveryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    C: ErrorClassifier<E>,
    E: std::fmt::Debug,
{
    let mut last_error = None;
    let mut delay = config.initial_delay;

    for attempt in 0..=config.max_retries {
        debug!("Attempting operation {} (attempt {}/{})", service_name, attempt + 1, config.max_retries + 1);

        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation {} succeeded after {} retries", service_name, attempt);
                }
                return Ok(result);
            }
            Err(error) => {
                debug!("Operation {} failed on attempt {}: {:?}", service_name, attempt + 1, error);

                // Check if error is recoverable
                if classifier.is_permanent(&error) {
                    warn!("Operation {} failed with permanent error: {:?}", service_name, error);
                    return Err(RecoveryError::ExhaustedRetries {
                        attempts: attempt + 1,
                        last_error: error,
                    });
                }

                last_error = Some(error);

                // Don't delay after the last attempt
                if attempt < config.max_retries {
                    // Add jitter to prevent thundering herd
                    let jitter = if config.jitter_factor > 0.0 {
                        let jitter_amount = delay.as_millis() as f64 * config.jitter_factor;
                        Duration::from_millis((fastrand::f64() * jitter_amount) as u64)
                    } else {
                        Duration::ZERO
                    };

                    let total_delay = delay + jitter;
                    debug!("Retrying operation {} in {:?}", service_name, total_delay);
                    sleep(total_delay).await;

                    // Calculate next delay with exponential backoff
                    let next_delay_ms = (delay.as_millis() as f64 * config.backoff_multiplier) as u64;
                    delay = Duration::from_millis(next_delay_ms).min(config.max_delay);
                }
            }
        }
    }

    error!("Operation {} exhausted all {} retry attempts", service_name, config.max_retries + 1);
    Err(RecoveryError::ExhaustedRetries {
        attempts: config.max_retries + 1,
        last_error: last_error.unwrap(), // Safe unwrap - we had at least one attempt
    })
}

/// Timeout wrapper for operations
pub async fn with_timeout<F, Fut, T>(
    operation: F,
    timeout_duration: Duration,
    service_name: &str,
) -> Result<T, RecoveryError<tokio::time::error::Elapsed>>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = T>,
{
    match tokio::time::timeout(timeout_duration, operation()).await {
        Ok(result) => Ok(result),
        Err(elapsed) => {
            warn!("Operation {} timed out after {:?}", service_name, timeout_duration);
            Err(RecoveryError::Timeout {
                duration: timeout_duration,
            })
        }
    }
}

/// Combined retry with circuit breaker protection
pub async fn resilient_operation<F, Fut, T, E, C>(
    operation: F,
    retry_config: &ErrorRecoveryConfig,
    circuit_breaker: &CircuitBreaker,
    classifier: &C,
    service_name: &str,
) -> Result<T, RecoveryError<E>>
where
    F: Fn() -> Fut + Clone,
    Fut: Future<Output = Result<T, E>>,
    C: ErrorClassifier<E>,
    E: std::fmt::Debug,
{
    circuit_breaker
        .execute(service_name, || {
            let op = operation.clone();
            async move {
                retry_with_backoff(
                    || op(),
                    retry_config,
                    classifier,
                    service_name,
                ).await
            }
        })
        .await
        .and_then(|result| result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    
    #[derive(Debug, PartialEq)]
    struct TestError {
        recoverable: bool,
    }

    struct TestErrorClassifier;

    impl ErrorClassifier<TestError> for TestErrorClassifier {
        fn is_recoverable(&self, error: &TestError) -> bool {
            error.recoverable
        }
    }

    #[tokio::test]
    async fn test_retry_eventually_succeeds() {
        let attempt_count = Arc::new(AtomicU32::new(0));
        let attempt_count_clone = attempt_count.clone();
        
        let operation = move || {
            let count = attempt_count_clone.clone();
            async move {
                let current = count.fetch_add(1, Ordering::SeqCst);
                if current < 2 {
                    Err(TestError { recoverable: true })
                } else {
                    Ok("success")
                }
            }
        };

        let config = ErrorRecoveryConfig::testing();
        let classifier = TestErrorClassifier;
        
        let result = retry_with_backoff(operation, &config, &classifier, "test").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_with_permanent_error() {
        let operation = || async { Err(TestError { recoverable: false }) };
        let config = ErrorRecoveryConfig::testing();
        let classifier = TestErrorClassifier;
        
        let result = retry_with_backoff(operation, &config, &classifier, "test").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RecoveryError::ExhaustedRetries { attempts, .. } => {
                assert_eq!(attempts, 1); // Should fail immediately for permanent errors
            }
            _ => panic!("Expected ExhaustedRetries error"),
        }
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 1,
        };
        let circuit_breaker = CircuitBreaker::new(config);
        
        // First failure
        let result1 = circuit_breaker.execute("test", || async {
            Err::<(), TestError>(TestError { recoverable: true })
        }).await;
        assert!(result1.is_err());
        
        // Second failure - should open circuit
        let result2 = circuit_breaker.execute("test", || async {
            Err::<(), TestError>(TestError { recoverable: true })
        }).await;
        assert!(result2.is_err());
        
        // Third attempt - should be rejected due to open circuit
        let result3 = circuit_breaker.execute("test", || async {
            Ok::<(), TestError>(())
        }).await;
        
        match result3.unwrap_err() {
            RecoveryError::CircuitBreakerOpen { .. } => {
                // Expected
            }
            _ => panic!("Expected CircuitBreakerOpen error"),
        }
    }

    #[tokio::test]
    async fn test_timeout_wrapper() {
        let slow_operation = || async {
            sleep(Duration::from_millis(200)).await;
            "result"
        };
        
        let result = with_timeout(slow_operation, Duration::from_millis(100), "test").await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            RecoveryError::Timeout { .. } => {
                // Expected
            }
            _ => panic!("Expected Timeout error"),
        }
    }
}