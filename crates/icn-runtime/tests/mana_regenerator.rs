use icn_runtime::context::RuntimeContext;
use std::time::Duration;

#[tokio::test]
async fn balances_increase_when_regenerator_runs() {
    let ctx = RuntimeContext::new_with_stubs_and_mana("did:icn:test:regen", 0).unwrap();
    let interval = Duration::from_millis(100);
    ctx.clone().spawn_mana_regenerator(5, interval).await;
    assert_eq!(ctx.mana_ledger.get_balance(&ctx.current_identity), 0);
    tokio::time::sleep(Duration::from_millis(120)).await;
    let bal1 = ctx.mana_ledger.get_balance(&ctx.current_identity);
    assert!(bal1 >= 5);
}
