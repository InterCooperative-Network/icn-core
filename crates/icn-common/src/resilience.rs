use crate::TimeProvider;
use std::time::Duration;
use thiserror::Error;
use std::future::Future;

#[derive(Debug, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open { opened_at: u64 },
    HalfOpen,
}

#[derive(Debug)]
pub struct CircuitBreaker<T: TimeProvider> {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    open_timeout: Duration,
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
    pub fn new(time_provider: T, failure_threshold: u32, open_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            open_timeout,
            time_provider,
        }
    }

    pub fn state(&self) -> &CircuitState {
        &self.state
    }

    pub async fn call<F, Fut, R, E>(&mut self, operation: F) -> Result<R, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<R, E>>,
    {
        let now = self.time_provider.unix_seconds();
        match &self.state {
            CircuitState::Open { opened_at } => {
                if now - *opened_at >= self.open_timeout.as_secs() {
                    self.state = CircuitState::HalfOpen;
                } else {
                    return Err(CircuitBreakerError::Open);
                }
            }
            _ => {}
        }

        match operation().await {
            Ok(val) => {
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                Ok(val)
            }
            Err(err) => {
                let now = self.time_provider.unix_seconds();
                match self.state {
                    CircuitState::HalfOpen | CircuitState::Closed => {
                        self.failure_count += 1;
                        if self.failure_count >= self.failure_threshold || matches!(self.state, CircuitState::HalfOpen) {
                            self.state = CircuitState::Open { opened_at: now };
                            self.failure_count = 0;
                        }
                    }
                    CircuitState::Open { .. } => {}
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
        let mut cb = CircuitBreaker::new(tp, 2, Duration::from_secs(10));

        // fail twice -> open
        for _ in 0..2 {
            let _ = cb
                .call(|| async { Err::<(), _>("err") })
                .await;
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

