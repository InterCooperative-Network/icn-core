use icn_runtime::context::RuntimeContext;
use std::time::Duration;

#[tokio::test]
async fn balances_increase_when_regenerator_runs() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:regen", 0).unwrap();
    ctx.reputation_store
        .record_execution(&ctx.current_identity, true, 0);
    ctx.reputation_store
        .record_execution(&ctx.current_identity, true, 0);
    let interval = Duration::from_millis(100);
    ctx.clone().spawn_mana_regenerator(5, interval).await;
    assert_eq!(ctx.mana_ledger.get_balance(&ctx.current_identity), 0);
    tokio::time::sleep(Duration::from_millis(120)).await;
    let bal1 = ctx.mana_ledger.get_balance(&ctx.current_identity);
    // reputation is 2, so at least 10 mana should be credited per cycle
    assert!(bal1 >= 10);
}
