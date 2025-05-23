# RFC 0010 – ICN Governance & Deliberation Core

## Status
Draft

## Authors
Matt Faherty, ICN Dev Team

## Abstract
This RFC defines the foundational governance model of the InterCooperative Network (ICN). It outlines how deliberation, proposals, voting, and enforcement occur within scoped identity domains (communities, cooperatives, and federations). Governance is decentralized, DID-based, and enforces policies without reliance on centralized authorities.

## Problem Statement
Current digital governance systems either centralize control (corporate/state) or are too informal (ad hoc Discord groups, mailing lists). ICN aims to implement a structured, transparent, and federated decision-making framework that is enforceable by code and legible to participants.

## Design Goals
- Decentralized and scoped governance (per-DID)
- Federated and interoperable deliberation
- Transparent proposal lifecycle (create → deliberate → vote → execute)
- Enforceable via host ABI and runtime
- Compatible with AgoraNet frontend
- Auditability and participation incentives

## Non-Goals
- Full voting UI/UX (covered in `icn-agoranet`)
- Token-based plutocratic voting (explicitly avoided)
- Governance of real-world physical legal entities

## Architecture

### Governance Domains
- **Individual** – may deliberate, propose, or vote within allowed scopes
- **Community** – civic and cultural governance unit
- **Cooperative** – economic governance unit
- **Federation** – cross-org coordination and protocols

### Proposal Lifecycle
1. `CreateProposal`
2. `DistributeToDeliberation`
3. `OpenForVoting`
4. `CloseAndVerify`
5. `ExecuteOrReject`

### Proposal Types
- Policy change
- Membership admission/removal
- Resource allocation
- Constitution/rule updates
- Mesh job governance

### Enforcement
- All proposals execute via scoped host ABI calls
- Policies are stored on-chain (via DAG + CID anchors)
- Execution receipts are verifiable and reputational

### Integration
- Deliberation UI: `icn-agoranet`
- Runtime Enforcement: `icn-runtime`, `host_abi`
- Reputation Impact: `execution_receipt`, `trust_policy`
- Federation: `federated_quorum` module

## Security Considerations
- Replay attack protection via signed receipts
- Sybil resistance via community/cooperative vetting
- Proposal censorship resistance via DAG anchoring
- Quorum thresholds + timeouts

## Open Questions
- How do emergency overrides work?
- Do federations have veto power?
- How do anonymous proposals or votes interact with transparency?

## References
- [ICN Golden Prompt]
- [Reputation System RFC (TBD)]
- [AgoraNet UI]
- [ICN Host ABI Spec §3.2 – Governance Functions] 