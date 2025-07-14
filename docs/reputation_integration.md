# Reputation Influence on Mana & Token Issuance

This document summarizes how the ICN core links participant reputation to economic capabilities.

## Mana Regeneration

`icn-runtime` periodically regenerates mana for all accounts. The amount granted is scaled by each account's reputation score.

Relevant code:
- `RuntimeContext` mana regenerator calculates `regeneration_amount` as `base_regeneration * reputation_multiplier` where `reputation_multiplier` is derived from the score.
- The `icn-economics` crate exposes `credit_by_reputation` to credit mana directly based on reputation.

## Scoped Token Issuance

When minting scoped tokens, issuers pay mana. The `price_by_reputation` helper reduces this cost based on the issuer's reputation, enabling trusted contributors to create resources more cheaply.

The new `mint_tokens_with_reputation` function demonstrates this linkage.

## Converting Reputation to Mana or Tokens

Governance policies can directly convert reputation scores into usable mana or
resource tokens. Community members with higher reputation may receive extra
credit via `credit_by_reputation` or earn bonus tokens when contracts call
`mint_tokens_with_reputation`. This allows cooperatives to reward trusted
contributors without relying on speculative markets.
