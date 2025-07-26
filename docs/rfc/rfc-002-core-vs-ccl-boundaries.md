# RFC-002: Core Protocol vs CCL Contract Boundaries

- **RFC Number**: RFC-002
- **Title**: Core Protocol vs CCL Contract Boundaries  
- **Author(s)**: ICN Core Community
- **Status**: Proposed
- **Created**: 2025-01-26
- **Updated**: 2025-01-26
- **Related Issues**: #942, #929

## Summary

This RFC defines clear boundaries between what must be implemented in the ICN Core protocol versus what should be customizable through CCL contracts, with special focus on governance, tokenomics, and mana/reputation systems.

## Motivation

Clear boundaries between core protocol and CCL contracts are essential for:

1. **Security**: Core protocol ensures system integrity and prevents attacks
2. **Flexibility**: CCL contracts enable community customization and experimentation  
3. **Interoperability**: Standard interfaces allow diverse implementations to interact
4. **Maintainability**: Clear separation reduces complexity and maintenance burden
5. **Upgradeability**: Different layers can evolve at different rates

Issue #929 identifies specific areas where these boundaries are unclear, particularly around governance, tokenomics, and mana systems.

## Detailed Design

### Core Protocol: Must Be Built-In

These features require protocol-level implementation for security, interoperability, and system integrity:

#### Identity & Authentication
- **Secure Identity**: Ed25519-based DID management
- **Privacy Controls**: Pseudonymous and multi-identity support
- **Cross-Federation Auth**: Standardized authentication across federations
- **Permission Framework**: Fine-grained access control primitives

#### Federation & Membership
- **Federation Primitives**: Joining, leaving, and membership management
- **Trust Networks**: Web-of-trust and attestation mechanisms
- **Cross-Federation Messaging**: Secure, authenticated inter-federation communication
- **Permissioning Engine**: Role-based access control across all levels

#### Token & Asset Ledger
- **Native Token Support**: Fungible and non-fungible token primitives
- **Multi-Issuer Framework**: Any authorized entity can issue tokens
- **Transfer Mechanisms**: Secure token transfer with cryptographic verification
- **Resource Registry**: Canonical mapping of tokens to real-world assets
- **Atomic Exchange**: Built-in token conversion and trading primitives

#### Mana & Reputation Tracking
- **Core Mana Ledger**: Non-transferable, earned mana tracking per user
- **Reputation Primitives**: Basic reputation scoring and attestation
- **Cross-Federation Mana**: Mana recognition across federation boundaries
- **Anti-Gaming Measures**: Sybil resistance and collusion prevention

#### Voting & Proposal Engine
- **Proposal Lifecycle**: Creation, deliberation, voting, and execution primitives
- **Vote Aggregation**: Secure tallying and result calculation
- **Multi-Level Voting**: Support for local, federation, and global governance
- **Cryptographic Verification**: Tamper-proof voting with ZK privacy options

#### System-Level Features
- **i18n/Localization**: Multi-language support for UIs and contracts
- **Escrow & Arbitration**: Trusted primitives for dispute resolution
- **Upgradability Logic**: Protocol-level support for contract amendments
- **Audit Trails**: Immutable logging of all governance and economic activity

### CCL Contracts: Customizable/Composable

These features should be implemented as customizable CCL contracts:

#### Governance Models
- **Voting Systems**: Direct, delegated, quadratic, reputation-based, sortition
- **Decision Mechanisms**: Consensus, majority, supermajority, consent-based
- **Council Systems**: Specialized working groups and representative bodies
- **Amendment Procedures**: Custom rules for contract and governance changes

#### Economic & Token Logic
- **Minting/Burning**: Token supply management and inflation policies
- **Vesting Schedules**: Time-locked token distribution mechanisms  
- **Reward Systems**: Incentive structures for participation and contribution
- **Economic Primitives**: Mutual credit, time banking, resource sharing models
- **Speculation Resistance**: Transfer restrictions, demurrage, velocity incentives

#### Organizational Structures
- **Treasury Management**: Budget allocation, grants, and funding mechanisms
- **Role Definitions**: Custom roles, responsibilities, and permissions
- **Onboarding Flows**: Member recruitment and integration processes
- **Commons Management**: Collective resource allocation and stewardship rules

#### Dispute & Enforcement
- **Resolution Flows**: Mediation, arbitration, and restorative justice
- **Sanction Systems**: Graduated penalties and enforcement mechanisms
- **Appeal Processes**: Multi-level review and correction procedures
- **Rehabilitation**: Reintegration procedures after sanctions

#### Integration & Bridges
- **External Adapters**: Connections to ActivityPub, other blockchains, legal systems
- **Oracle Systems**: Real-world data integration and verification
- **Legacy Integration**: Bridges to existing cooperative and organizational systems

### Mana System: Hybrid Approach

Mana requires both core protocol and CCL contract components:

#### Core Protocol Responsibilities
- **Mana Tracking**: Secure, tamper-proof mana balance storage
- **Cross-Federation Recognition**: Mana portability across federations
- **Anti-Gaming**: Sybil resistance and manipulation prevention
- **Audit Trails**: Complete history of mana earning and spending

#### CCL Contract Responsibilities  
- **Earning Rules**: How mana is gained through participation and contribution
- **Spending Logic**: What mana can be used for and spending limits
- **Decay Mechanisms**: How mana diminishes over time (if applicable)
- **Transfer Restrictions**: Whether and how mana can be delegated

### Implementation Considerations

#### API Boundaries
- **Core APIs**: Standardized interfaces for all protocol-level features
- **CCL Hooks**: Well-defined extension points for contract customization
- **Event System**: Comprehensive event emission for contract integration
- **State Management**: Clear separation between protocol and contract state

#### Security Model
- **Core Security**: Protocol-level protection against fundamental attacks
- **Contract Validation**: Automated analysis of CCL contracts for security issues
- **Privilege Escalation**: Prevention of contracts gaining unauthorized protocol access
- **Resource Limits**: Compute and storage limits for contract execution

#### Upgrade Mechanisms
- **Protocol Upgrades**: Coordinated upgrade process for core features
- **Contract Evolution**: Independent upgrade cycles for CCL contracts
- **Compatibility**: Backward compatibility guarantees and migration tools
- **Testing Framework**: Comprehensive testing for protocol/contract interactions

### Impact on Existing Code

This RFC will require refactoring existing implementations:

#### Core Crates Changes
- **icn-governance**: Split into core primitives and CCL contract examples
- **icn-economics**: Extract core token mechanics, move policies to CCL
- **icn-reputation**: Keep core tracking, move scoring logic to CCL
- **icn-ccl**: Add governance and economic contract templates

#### New Infrastructure
- **Contract Registry**: System for managing and discovering CCL contracts
- **Validation Framework**: Static analysis and testing tools for contracts
- **Migration Tools**: Utilities for upgrading contracts and migrating state

## Drawbacks

1. **Complexity**: Clear boundaries may increase system complexity initially
2. **Performance**: Additional abstraction layers may impact performance
3. **Development Overhead**: Requires maintaining both protocol and contract APIs
4. **Learning Curve**: Developers must understand both protocol and CCL systems

## Rationale and Alternatives

### Why This Design?

1. **Security**: Critical operations protected at protocol level
2. **Flexibility**: Communities can customize governance and economics
3. **Interoperability**: Standard interfaces enable ecosystem growth
4. **Evolution**: Different layers can upgrade independently

### Alternatives Considered

1. **Everything in Core**: Too rigid, prevents customization
2. **Everything in CCL**: Security risks, compatibility issues
3. **Plugin Architecture**: More complex than CCL contracts
4. **Microservices**: Too complex for current needs

## Prior Art

- **Ethereum**: Core protocol with smart contract customization
- **Substrate**: Framework with runtime modules and pallets  
- **Cosmos SDK**: Modular blockchain application framework
- **Hyperledger Fabric**: Chaincode for business logic customization

## Unresolved Questions

1. **Performance Optimization**: How to minimize overhead of protocol/contract boundaries?
2. **Security Boundaries**: What additional security measures are needed at boundaries?
3. **Contract Standards**: What standard contract interfaces should be defined?
4. **Migration Strategy**: How to migrate existing implementations to new boundaries?
5. **Testing Framework**: How to ensure comprehensive testing across boundaries?

## Future Possibilities

1. **Formal Verification**: Mathematical proof of protocol/contract security properties
2. **Contract Markets**: Ecosystem of tested, audited governance and economic contracts
3. **AI Contract Generation**: Automated creation of contracts from governance requirements
4. **Cross-Protocol Standards**: Interoperability with other blockchain governance systems
5. **Legal Integration**: Formal recognition of CCL contracts in legal jurisdictions