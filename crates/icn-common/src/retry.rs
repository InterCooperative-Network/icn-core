use log::{error, warn};
use std::future::Future;
use std::time::Duration;

/// Retry an asynchronous operation with jittered exponential backoff.
///
/// The `operation` closure is executed until it succeeds or `max_retries`
/// attempts have been made. The delay between attempts starts at
/// `initial_delay` and doubles each time up to `max_delay`, with a small
/// random jitter added to avoid thundering herd issues.
pub async fn retry_with_backoff<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
    initial_delay: Duration,
    max_delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    let mut delay = initial_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                attempts += 1;
                if attempts >= max_retries {
                    error!("Operation failed after {} attempts: {:?}", attempts, error);
                    return Err(error);
                }
                warn!(
                    "Operation failed (attempt {}), retrying in {:?}: {:?}",
                    attempts, delay, error
                );
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, max_delay);
                let jitter =
                    Duration::from_millis(fastrand::u64(0..=delay.as_millis() as u64 / 10));
                delay += jitter;
            }
        }
    }
}
