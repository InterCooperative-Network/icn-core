# Mana Policy Implementation and Deployment

This guide demonstrates how to create and deploy Cooperative Contract Language (CCL) policies that manipulate mana in the InterCooperative Network.

## 1. Write a mana cost contract

You can implement arbitrary cost logic in CCL. The snippet below adapts the working example from `icn-ccl/ACCOMPLISHMENTS.md`:

```ccl
fn calculate_mana_cost(cores: Integer, memory: Integer, rep: Integer) -> Mana {
    let base = calculate_base_cost(cores, memory);
    let final_cost = apply_reputation_modifier(base, rep);
    return final_cost;
}
```

Use this function inside your `run` entry point to charge or refund mana depending on job parameters.

## 2. Compile the contract

Compile the `.ccl` file using the CLI:

```bash
icn-cli ccl compile policy.ccl
```

This produces `policy.wasm` and accompanying metadata.

## 3. Upload the compiled module

Send the contract to a running node and retrieve its content identifier (CID):

```bash
icn-cli --api-url http://localhost:7845 compile-ccl policy.ccl
```

## 4. Submit a mesh job

Reference the returned CID when creating a job:

```bash
icn-cli --api-url http://localhost:7845 submit-job \
  '{"manifest_cid":"<CID>","spec_json":{},"cost_mana":0}'
```

The mesh executor will invoke the contract's `run` function when processing the job.

## 5. Regenerate mana

To keep accounts funded, start a background regenerator with `spawn_mana_regenerator`:

```rust
ctx.clone().spawn_mana_regenerator(5, std::time::Duration::from_secs(60)).await;
```

This method is described in the runtime documentation and periodically credits all accounts【F:crates/icn-runtime/README.md†L98-L102】.
You can also credit individual accounts directly through the ledger API using `icn_economics::credit_mana`:

```rust
icn_economics::credit_mana(ledger, &did, 10)?;
```

See the ledger helper implementation for details【F:crates/icn-economics/src/lib.rs†L128-L134】.
