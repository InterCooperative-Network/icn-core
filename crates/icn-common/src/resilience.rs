use std::sync::atomic::{AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::future::Future;

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed = 0,
    Open = 1,
    HalfOpen = 2,
}

/// Errors returned by [`CircuitBreaker::call`]
#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    /// Circuit is open so the operation was not executed
    CircuitOpen,
    /// The wrapped operation failed
    OperationFailed(E),
}

/// Simple circuit breaker for wrapping fallible async operations
#[derive(Debug)]
pub struct CircuitBreaker {
    failure_count: AtomicU32,
    last_failure: AtomicU64,
    failure_threshold: u32,
    timeout: Duration,
    state: AtomicU8,
}

impl CircuitBreaker {
    /// Create a new [`CircuitBreaker`] with the given failure threshold and timeout
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: AtomicU32::new(0),
            last_failure: AtomicU64::new(0),
            failure_threshold,
            timeout,
            state: AtomicU8::new(CircuitState::Closed as u8),
        }
    }

    fn now_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    /// Get the current circuit state
    pub fn get_state(&self) -> CircuitState {
        match self.state.load(Ordering::SeqCst) {
            1 => CircuitState::Open,
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }

    fn set_state(&self, state: CircuitState) {
        self.state.store(state as u8, Ordering::SeqCst);
    }

    fn record_failure(&self) {
        let failures = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        self.last_failure.store(Self::now_secs(), Ordering::SeqCst);
        if failures >= self.failure_threshold {
            self.set_state(CircuitState::Open);
        }
    }

    fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        self.set_state(CircuitState::Closed);
    }

    fn should_attempt_reset(&self) -> bool {
        let last = self.last_failure.load(Ordering::SeqCst);
        Self::now_secs().saturating_sub(last) >= self.timeout.as_secs()
    }

    /// Wrap an asynchronous operation with circuit breaker protection.
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        if self.get_state() == CircuitState::Open {
            if self.should_attempt_reset() {
                self.set_state(CircuitState::HalfOpen);
            } else {
                return Err(CircuitBreakerError::CircuitOpen);
            }
        }

        let result = operation.await;
        match result {
            Ok(v) => {
                self.record_success();
                Ok(v)
            }
            Err(e) => {
                self.record_failure();
                Err(CircuitBreakerError::OperationFailed(e))
            }
        }
    }
}

