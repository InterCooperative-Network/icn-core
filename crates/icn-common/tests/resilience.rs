use icn_common::resilience::{CircuitBreaker, CircuitBreakerError};
use std::time::Duration;

#[tokio::test]
async fn circuit_breaker_state_transitions() {
    let cb = CircuitBreaker::new(2, Duration::from_millis(50));

    // first failure
    let _ = cb
        .call(async { Err::<(), _>("err1") })
        .await
        .unwrap_err();
    assert_eq!(cb.get_state(), icn_common::resilience::CircuitState::Closed);

    // second failure should open
    let _ = cb
        .call(async { Err::<(), _>("err2") })
        .await
        .unwrap_err();
    assert_eq!(cb.get_state(), icn_common::resilience::CircuitState::Open);

    // call while open before timeout should error
    match cb.call(async { Ok::<(), &'static str>(()) }).await {
        Err(CircuitBreakerError::CircuitOpen) => {}
        _ => panic!("expected CircuitOpen"),
    }

    // wait for timeout, next call moves to half-open
    tokio::time::sleep(Duration::from_millis(60)).await;
    let _ = cb.call(async { Err::<(), _>("err3") }).await;
    assert_eq!(cb.get_state(), icn_common::resilience::CircuitState::Open);
}

#[tokio::test]
async fn circuit_breaker_recovers_after_success() {
    let cb = CircuitBreaker::new(1, Duration::from_millis(20));
    let _ = cb.call(async { Err::<(), _>("e") }).await;
    assert_eq!(cb.get_state(), icn_common::resilience::CircuitState::Open);

    tokio::time::sleep(Duration::from_millis(25)).await;
    let res = cb.call(async { Ok::<_, ()>(42) }).await;
    assert!(res.is_ok());
    assert_eq!(cb.get_state(), icn_common::resilience::CircuitState::Closed);
}
