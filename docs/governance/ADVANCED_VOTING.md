# Advanced Voting Mechanisms

> **⚠️ Development Status**: Experimental implementations for advanced voting are available. Interfaces may change.

This guide outlines advanced governance voting systems supported by ICN Core.

## Ranked Choice Voting

Voters rank proposals by preference. The tallying algorithm iteratively eliminates the lowest-ranked option until a majority winner emerges. See `icn-governance/src/ranked_choice.rs` for implementation details.

## Quadratic Voting

Participants allocate voting credits quadratically to express intensity of preference. This mechanism reduces the influence of large stakeholders and encourages consensus.

## Other Mechanisms

Additional strategies such as approval voting and score voting can be implemented via CCL contracts. Experimentation is encouraged to determine what best fits your cooperative.
