# ICN Resource Tokens

Resource tokens are purpose-bound digital assets used within the InterCooperative Network. They represent rights to specific resources or services and are tightly coupled with the mana system.

## Design Goals
- **Non-Speculative:** Tokens cannot be freely traded on secondary markets.
- **Capability Bound:** Each token grants access only to a defined resource or service.
- **Transparent Creation:** Token classes are created through governance or trusted contracts.
- **Mana Integration:** Mana is consumed when tokens are minted or transferred, preventing abuse.

## Token Class Creation
A token *class* defines the properties of a resource token. Fields include:
- `id` – unique identifier (e.g. `pantry-credit`)
- `description` – human readable purpose
- `transferable` – if `false`, the token is soul‑bound to the owner

New classes are created via the `/tokens/class` endpoint or a CCL contract. The creator pays a mana cost and specifies whether the tokens are transferable.

## Mint Flow
1. Call `/tokens/mint` with the class id, amount, and recipient DID.
2. Mana is debited from the caller according to the mint cost policy.
3. The ledger records the new balance for the recipient.

## Transfer Flow
1. Call `/tokens/transfer` with class id, amount, sender DID, and recipient DID.
2. If the class is `transferable`, balances are updated and mana fees applied.
3. Soul‑bound tokens (`transferable = false`) will be rejected.

## Burn Flow
1. Call `/tokens/burn` with the class id, amount, and owner DID.
2. Tokens are removed from the owner's balance.
3. Optionally mana can be refunded based on policy.

## Mana Interaction
Mana represents regenerative cooperative capacity. Token operations charge or refund mana so that resource allocation stays fair. A typical policy might charge a small amount of mana when minting or transferring to limit spam but refund a portion when tokens are burned.

## Example
See `icn-ccl/examples/pantry_credit.ccl` for a simple contract that mints a soul‑bound volunteer badge and a transferable pantry credit token.
