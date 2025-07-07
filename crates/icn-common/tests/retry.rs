use icn_common::retry_with_backoff;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

#[tokio::test]
async fn retry_succeeds_after_failures() {
    let attempts = AtomicU32::new(0);
    let result = retry_with_backoff(
        || {
            let n = attempts.fetch_add(1, Ordering::SeqCst);
            async move {
                if n < 2 {
                    Err("fail")
                } else {
                    Ok(42)
                }
            }
        },
        5,
        Duration::from_millis(10),
        Duration::from_millis(50),
    )
    .await;
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn retry_respects_max_retries() {
    let attempts = AtomicU32::new(0);
    let result: Result<(), &str> = retry_with_backoff(
        || {
            attempts.fetch_add(1, Ordering::SeqCst);
            async { Err("fail") }
        },
        3,
        Duration::from_millis(5),
        Duration::from_millis(20),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}
