# ICN Core RFC Process

This directory contains Requests for Comments (RFCs) for major design decisions in ICN Core.

## What requires an RFC?

- Major changes to the protocol architecture
- New cryptographic systems or algorithms
- Significant changes to governance mechanisms
- Breaking changes to APIs or data formats
- Cross-cutting concerns affecting multiple crates
- Resolution of open design questions

## RFC Process

1. **Draft**: Create a new RFC document following the template
2. **Discussion**: Open GitHub issue linking to the RFC for community input
3. **Review**: Technical review by maintainers and domain experts
4. **Decision**: RFC is either accepted, rejected, or requires revision
5. **Implementation**: Accepted RFCs guide implementation work

## RFC Status

- **Draft**: Under development, not ready for formal review
- **Proposed**: Ready for community discussion and review
- **Accepted**: Approved for implementation
- **Implemented**: RFC guidance has been implemented
- **Superseded**: Replaced by a newer RFC
- **Rejected**: Not approved for implementation

## Active RFCs

| RFC | Title | Status | Issue |
|-----|-------|--------|-------|
| [RFC-001](rfc-001-governance-scaling.md) | Governance and Federated Scaling Architecture | Proposed | #942 |
| [RFC-002](rfc-002-core-vs-ccl-boundaries.md) | Core Protocol vs CCL Contract Boundaries | Proposed | #942 |
| [RFC-003](rfc-003-tokenomics-design.md) | Tokenomics and Economic System Design | Proposed | #942 |

## RFC Template

See [rfc-template.md](rfc-template.md) for the standard RFC format.