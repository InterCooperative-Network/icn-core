use crate::TimeProvider;
use std::future::Future;
use std::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open { opened_at: u64 },
    HalfOpen,
}

#[derive(Debug)]
pub struct CircuitBreaker<T: TimeProvider> {
    failure_threshold: usize,
    timeout: Duration,
    current_failures: AtomicUsize,
    last_failure_time: AtomicU64,
    state: AtomicU8, // 0 = Closed, 1 = Open, 2 = Half-Open
    time_provider: T,
}

#[derive(Debug, Error)]
pub enum CircuitBreakerError<E> {
    #[error("circuit breaker open")]
    Open,
    #[error(transparent)]
    Inner(#[from] E),
}

impl<T: TimeProvider> CircuitBreaker<T> {
    pub fn new(time_provider: T, failure_threshold: usize, timeout: Duration) -> Self {
        Self {
            failure_threshold,
            timeout,
            current_failures: AtomicUsize::new(0),
            last_failure_time: AtomicU64::new(0),
            state: AtomicU8::new(0),
            time_provider,
        }
    }

    pub fn state(&self) -> CircuitState {
        match self.state.load(Ordering::SeqCst) {
            0 => CircuitState::Closed,
            1 => CircuitState::Open {
                opened_at: self.last_failure_time.load(Ordering::SeqCst),
            },
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }

    pub async fn call<F, Fut, R, E>(&self, operation: F) -> Result<R, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<R, E>>,
    {
        let now = self.time_provider.unix_seconds();
        let state = self.state.load(Ordering::SeqCst);
        if state == 1 {
            let opened = self.last_failure_time.load(Ordering::SeqCst);
            if now - opened < self.timeout.as_secs() {
                return Err(CircuitBreakerError::Open);
            }
            self.state.store(2, Ordering::SeqCst);
        }

        match operation().await {
            Ok(val) => {
                self.state.store(0, Ordering::SeqCst);
                self.current_failures.store(0, Ordering::SeqCst);
                Ok(val)
            }
            Err(err) => {
                self.current_failures.fetch_add(1, Ordering::SeqCst);
                self.last_failure_time.store(now, Ordering::SeqCst);
                let failures = self.current_failures.load(Ordering::SeqCst);
                let state = self.state.load(Ordering::SeqCst);
                if failures >= self.failure_threshold || state == 2 {
                    self.state.store(1, Ordering::SeqCst);
                    self.current_failures.store(0, Ordering::SeqCst);
                }
                Err(CircuitBreakerError::Inner(err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FixedTimeProvider;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_state_transitions() {
        let provider = Arc::new(Mutex::new(FixedTimeProvider::new(0)));
        struct P(Arc<Mutex<FixedTimeProvider>>);
        impl TimeProvider for P {
            fn unix_seconds(&self) -> u64 {
                self.0.lock().unwrap().unix_seconds()
            }
        }
        let tp = P(provider.clone());
        let cb = CircuitBreaker::new(tp, 2, Duration::from_secs(10));

        // fail twice -> open
        for _ in 0..2 {
            let _ = cb.call(|| async { Err::<(), _>("err") }).await;
        }
        assert!(matches!(cb.state(), CircuitState::Open { .. }));

        // advance time so breaker allows a trial call
        provider.lock().unwrap().0 += 11;
        let _ = cb.call(|| async { Err::<(), &str>("fail") }).await;
        // failure in half-open should reopen circuit
        assert!(matches!(cb.state(), CircuitState::Open { .. }));

        // advance time again and succeed
        provider.lock().unwrap().0 += 11;
        cb.call(|| async { Ok::<_, &str>(()) }).await.unwrap();
        assert!(matches!(cb.state(), CircuitState::Closed));
    }
}
