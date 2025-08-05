# RFC-003: Tokenomics and Economic System Design

- **RFC Number**: RFC-003
- **Title**: Tokenomics and Economic System Design
- **Author(s)**: ICN Core Community
- **Status**: Proposed
- **Created**: 2025-01-26
- **Updated**: 2025-01-26
- **Related Issues**: #942, #929

## Summary

This RFC proposes a comprehensive tokenomics design for ICN Core that supports multi-asset, multi-issuer economies with built-in speculation resistance and seamless token exchange capabilities.

## Motivation

ICN Core needs an economic system that:

1. **Supports Real Value**: Tokens represent actual goods, services, and resources
2. **Prevents Speculation**: Design discourages hoarding and financialization
3. **Enables Exchange**: Seamless conversion between local, federation, and global tokens
4. **Maintains Sovereignty**: Communities control their economic policies
5. **Scales Globally**: Local economies can participate in global coordination

The current system lacks clear principles for token design, exchange mechanisms, and speculation resistance outlined in issue #929.

## Detailed Design

### Token Principles

#### Multi-Asset, Multi-Issuer Framework
- **Authorized Issuers**: Co-ops, communities, and federations can issue tokens
- **Resource Backing**: All tokens linked to real goods, services, or access rights
- **Transparent Registry**: Public mapping of tokens to underlying resources
- **Democratic Control**: Token policies governed by issuing community

#### Speculation Resistance
- **Transfer Restrictions**: Limits on who can hold and trade tokens
- **Demurrage**: Optional holding costs to encourage circulation
- **Velocity Incentives**: Rewards for active use rather than accumulation
- **Community Oversight**: Democratic control over token listing and trading

### Mana System Enhancements

#### Capacity-Aware Regeneration

The mana system now incorporates capacity-based regeneration to reward nodes that contribute more resources to the network:

**Regeneration Formula**: `base_rate × capacity_factor × reputation_factor × time_elapsed`

Where:
- **base_rate**: Minimum regeneration for all nodes (e.g., 10 mana/hour)
- **capacity_factor**: 0.1 to 3.0 based on resource contribution metrics
- **reputation_factor**: 0.5 to 2.0 based on community standing
- **time_elapsed**: Hours since last regeneration

#### Capacity Metrics

```rust
pub struct CapacityMetrics {
    pub compute_contribution: f64,    // CPU/GPU resources provided
    pub storage_contribution: f64,    // Storage space provided
    pub bandwidth_contribution: f64,  // Network bandwidth provided
    pub uptime_score: f64,           // Reliability and availability
    pub quality_score: f64,          // Performance and reliability
}
```

The capacity factor is calculated as a weighted average:
- Compute: 30%
- Storage: 25% 
- Bandwidth: 25%
- Uptime: 15%
- Quality: 5%

#### Capacity-Based Spending Limits

Higher capacity nodes can spend more freely, with spending limits calculated as:
`max_spendable = max_capacity × (0.5 + capacity_factor × 0.3)`

This prevents low-capacity nodes from draining the system while allowing productive nodes greater flexibility.

### Token Types

#### Resource Tokens
- **Physical Goods**: Food, materials, manufactured products
- **Services**: Labor hours, professional services, maintenance
- **Access Rights**: Facility use, event attendance, membership benefits
- **Infrastructure**: Energy, transportation, communication services

#### Community Currencies
- **Local Exchange**: Facilitate trade within communities
- **Mutual Credit**: Community-backed credit systems
- **Time Banking**: Labor hour tracking and exchange
- **Contribution Tracking**: Recognition of community participation

#### Federation Tokens
- **Inter-Community Trade**: Exchange between federation members
- **Shared Resources**: Access to federation-wide infrastructure
- **Coordination**: Voting and governance participation tokens
- **Emergency Reserves**: Crisis response and mutual aid

#### Global Coordination Tokens
- **Planetary Commons**: Global resource management (climate, oceans, etc.)
- **Inter-Federation Exchange**: Trade between different federations
- **Infrastructure**: Global communication and coordination systems
- **Knowledge Sharing**: Research, education, and information exchange

### Architecture: Core Protocol vs CCL Governance

#### Core Protocol Primitives (Built-in)

These economic components are embedded in the ICN protocol and provide the foundation for all economic activity:

**Identity & Authentication**
- DID management and credential verification
- Cryptographic signing and verification
- Identity reputation tracking

**Core Ledgers**
- ManaLedger: Regenerating resource credits
- ResourceLedger: Multi-asset token management  
- CapacityLedger: Node contribution tracking
- Transaction history and audit trails

**Voting Engine**
- Ranked choice voting implementation
- Proposal creation and management
- Vote tallying and result calculation

**Resource Registry**
- Physical and digital resource cataloging
- Availability and capacity tracking
- Resource allocation and scheduling

#### CCL-Governed Policies (Programmable)

These economic behaviors can be customized by communities through CCL contracts:

**Governance Models**
- Voting rules and quorum requirements
- Decision-making processes
- Leadership selection mechanisms

**Tokenomics Policies**
- Custom regeneration formulas
- Demurrage and velocity incentives
- Transfer restrictions and permissions
- Token issuance policies

**Economic Primitives**
- Mutual credit systems
- Time banking mechanisms
- Local currencies
- Bulk purchasing cooperatives

**Dispute Resolution**
- Mediation workflows
- Arbitration processes  
- Penalty and compensation mechanisms

#### Integration Boundaries

```rust
/// Core protocol provides stable interfaces
pub trait CoreEconomicInterface {
    // Core primitives that CCL can use but not override
    fn get_mana_balance(&self, did: &Did) -> u64;
    fn spend_mana(&self, did: &Did, amount: u64) -> Result<(), CommonError>;
    fn transfer_tokens(&self, class_id: &str, from: &Did, to: &Did, amount: u64) -> Result<(), CommonError>;
}

/// CCL policies can customize economic behavior
pub trait CCLEconomicPolicy {
    // Policies that communities can customize
    fn calculate_regeneration_rate(&self, did: &Did, base_rate: f64) -> f64;
    fn validate_transfer(&self, class_id: &str, from: &Did, to: &Did, amount: u64) -> Result<bool, String>;
    fn apply_economic_penalties(&self, violation: EconomicViolation) -> Vec<EconomicAction>;
}
```

This separation ensures:
- **Stability**: Core economic functions remain reliable and interoperable
- **Flexibility**: Communities can customize economic policies to their needs
- **Security**: Critical economic infrastructure cannot be compromised by policy changes
- **Upgradability**: Policies can evolve without breaking the core protocol

### Seamless Exchange System

#### Automated Conversion
- **Background Processing**: Token conversion happens transparently
- **User Choice**: Users can transact in preferred token types
- **Real-Time Rates**: Dynamic exchange rates based on resource availability
- **Minimal Friction**: Conversion doesn't disrupt user experience

#### Exchange Mechanisms
- **Resource-Based Rates**: Exchange rates reflect actual resource values
- **Democratic Price Discovery**: Communities participate in rate setting
- **Anti-Manipulation**: Safeguards against rate manipulation
- **Transparency**: All exchange rates and mechanisms publicly auditable

#### Cross-Federation Trade
- **Standardized Interfaces**: Common protocols for inter-federation exchange
- **Trust Networks**: Reputation-based trading relationships
- **Dispute Resolution**: Multi-level arbitration for trade disputes
- **Settlement**: Automated settlement of cross-federation trades

### Implementation Architecture

#### Core Protocol Components
- **Token Registry**: Canonical database of all tokens and their properties
- **Ledger System**: Secure tracking of token balances and transfers
- **Exchange Engine**: Automated conversion and trading mechanisms
- **Rate Oracle**: Dynamic exchange rate calculation and distribution

#### CCL Contract Components
- **Issuance Policies**: Community-defined rules for token creation
- **Transfer Rules**: Restrictions and permissions for token movement
- **Economic Policies**: Demurrage, velocity incentives, and other mechanisms
- **Governance Integration**: Voting and decision-making using tokens

#### Integration Points
- **Resource Mapping**: Link tokens to real-world assets and services
- **Exchange Interfaces**: APIs for external systems and applications
- **Monitoring**: Real-time tracking of economic activity and health
- **Analytics**: Tools for understanding economic flows and impacts

### Speculation Resistance Mechanisms

#### Design-Level Resistance
- **Purpose Binding**: Tokens tied to specific uses and communities
- **Transfer Limitations**: Restrictions on secondary markets
- **Holding Costs**: Demurrage or other costs for non-productive holding
- **Community Control**: Democratic oversight of token economics

#### Technical Safeguards
- **Trading Limits**: Maximum positions and transaction sizes
- **Velocity Monitoring**: Detection of speculation patterns
- **Circuit Breakers**: Automatic suspension of suspicious trading
- **Audit Trails**: Complete tracking of all token movements

#### Governance Controls
- **Listing Requirements**: Community approval for new tokens
- **Policy Updates**: Democratic changes to economic rules
- **Enforcement**: Community-based enforcement of economic policies
- **Appeals Process**: Fair resolution of policy disputes

### Implementation Considerations

#### Performance Requirements
- **High Throughput**: Support for large-scale economic activity
- **Low Latency**: Real-time transaction processing
- **Scalability**: Growth from local to global scale
- **Reliability**: High availability for critical economic functions

#### Security Considerations
- **Cryptographic Security**: Strong protection for token transfers
- **Economic Attacks**: Prevention of manipulation and gaming
- **Privacy Protection**: Selective disclosure of economic activity
- **Audit Capabilities**: Comprehensive tracking without compromising privacy

#### User Experience
- **Simplicity**: Complex economics hidden behind simple interfaces
- **Transparency**: Users understand what they're trading and why
- **Control**: Users maintain sovereignty over their economic choices
- **Education**: Built-in tools for understanding the economic system

### Impact on Existing Code

This RFC will require significant enhancements to economic systems:

#### Core Crates
- **icn-economics**: Major expansion of token and exchange capabilities
- **icn-api**: New endpoints for token management and trading
- **icn-governance**: Integration with token-based governance
- **icn-identity**: Link economic activity to identity systems

#### New Components
- **Exchange Engine**: Automated token conversion system
- **Rate Oracle**: Dynamic exchange rate calculation
- **Economic Analytics**: Tools for monitoring economic health
- **Resource Registry**: Mapping between tokens and real assets

## Drawbacks

1. **Complexity**: Multi-token economy increases system complexity
2. **Performance**: Real-time exchange may impact system performance
3. **Governance Overhead**: Economic decisions require community involvement
4. **Market Dynamics**: Complex interactions between different token systems

## Rationale and Alternatives

### Why This Design?

1. **Real Value**: Focus on actual resources prevents speculative bubbles
2. **Community Control**: Democratic governance of economic policies
3. **Flexibility**: Support for diverse economic models and experiments
4. **Seamless Integration**: Users don't need to understand token complexity

### Alternatives Considered

1. **Single Global Currency**: Too centralized, reduces local sovereignty
2. **Pure Barter**: Insufficient for complex modern economies
3. **Traditional Banking**: Extractive, not aligned with cooperative values
4. **Existing Cryptocurrencies**: Speculation-focused, not resource-backed

## Prior Art

- **Ithaca Hours**: Local currency for community exchange
- **Community Exchange System (CES)**: Mutual credit networks
- **Sardex**: Regional currency for businesses in Sardinia
- **Complementary Currencies**: Thousands of local economic experiments
- **Resource-Based Economy**: Theoretical frameworks for post-scarcity economics

## Unresolved Questions

1. **Rate Calculation**: What algorithms best determine fair exchange rates?
2. **Economic Incentives**: How to incentivize participation in economic governance?
3. **Crisis Management**: How to handle economic shocks and emergencies?
4. **Legal Integration**: How to integrate with existing legal and tax systems?
5. **Measurement**: What metrics indicate a healthy cooperative economy?

## Future Possibilities

1. **AI Economic Agents**: Automated economic decision-making and optimization
2. **Ecological Integration**: Direct integration with ecological and environmental systems
3. **Post-Scarcity Transition**: Evolution toward abundance-based economic models
4. **Interplanetary Economics**: Extension to space-based cooperative communities
5. **Time-Based Economics**: Integration of time and attention as core economic resources