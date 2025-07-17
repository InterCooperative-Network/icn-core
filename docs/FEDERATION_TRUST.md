# Scoped Federation Trust Framework

## Overview

The Scoped Federation Trust Framework provides a comprehensive system for managing trust relationships between cooperatives across different federations within the ICN network. It implements the requirements specified in section 2.2 of the ICN architecture:

1. **Federation Trust Contexts** - Different trust levels for different activities
2. **Trust Inheritance Models** - How cooperatives inherit trust from their federations
3. **Cross-Federation Trust Bridges** - Trust relationships between different federations
4. **Trust Policy Engine** - Configurable rules for trust validation and requirements

## Architecture

### Trust Contexts

The framework defines different trust contexts for various activities:

- **Governance** - Trust for voting, proposals, and decision-making
- **ResourceSharing** - Trust for sharing compute, storage, and network resources
- **MutualCredit** - Trust for economic transactions and credit relationships
- **Identity** - Trust for identity verification and credential validation
- **Infrastructure** - Trust for network infrastructure and routing
- **DataSharing** - Trust for privacy-sensitive data operations
- **General** - Basic cooperation trust
- **Custom** - Application-specific trust contexts

### Core Components

#### 1. ScopedTrustRelationship

```rust
pub struct ScopedTrustRelationship {
    pub base: TrustRelationship,           // Basic trust relationship
    pub context: TrustContext,             // Specific trust context
    pub federation: Option<FederationId>,  // Federation this trust belongs to
    pub inheritance: TrustInheritance,     // Trust inheritance rules
    pub metadata: HashMap<String, String>, // Additional metadata
}
```

#### 2. TrustInheritance

```rust
pub struct TrustInheritance {
    pub inheritable: bool,                    // Whether trust can be inherited
    pub max_depth: Option<u32>,              // Maximum inheritance depth
    pub degradation_factor: f64,             // Trust degradation per level (0.0-1.0)
    pub min_inherited_level: TrustLevel,     // Minimum trust level that can be inherited
}
```

#### 3. FederationTrustBridge

```rust
pub struct FederationTrustBridge {
    pub from_federation: FederationId,       // Source federation
    pub to_federation: FederationId,         // Target federation
    pub trust: ScopedTrustRelationship,      // Trust relationship between federations
    pub bridge_config: BridgeConfig,         // Bridge configuration
    pub established_at: u64,                 // Establishment timestamp
    pub expires_at: Option<u64>,             // Optional expiration
}
```

#### 4. TrustPolicyEngine

The core engine that validates trust relationships according to configurable policies:

```rust
pub struct TrustPolicyEngine {
    rules: HashMap<TrustContext, Vec<TrustPolicyRule>>,
    federation_trusts: HashMap<FederationId, Vec<ScopedTrustRelationship>>,
    bridges: HashMap<(FederationId, FederationId), FederationTrustBridge>,
    memberships: HashMap<Did, HashSet<FederationId>>,
}
```

## Usage Examples

### Setting Up Trust Contexts

```rust
use icn_identity::{TrustPolicyEngine, TrustContext, FederationId, TrustLevel};

let mut engine = TrustPolicyEngine::new();

// Create federations
let housing_federation = FederationId::new("cooperative_housing".to_string());
let tech_federation = FederationId::new("tech_collective".to_string());

// Add cooperative memberships
let alice = Did::new("key", "alice_housing");
let bob = Did::new("key", "bob_tech");

engine.add_federation_membership(alice.clone(), housing_federation.clone());
engine.add_federation_membership(bob.clone(), tech_federation.clone());
```

### Creating Scoped Trust Relationships

```rust
use icn_identity::{ScopedTrustRelationship, TrustInheritance, TrustRelationship};

// Create governance trust with inheritance
let governance_trust = ScopedTrustRelationship {
    base: TrustRelationship::new(
        alice.clone(),
        bob.clone(),
        TrustLevel::Partial,
        vec!["governance".to_string()],
    ),
    context: TrustContext::Governance,
    federation: Some(housing_federation.clone()),
    inheritance: TrustInheritance {
        inheritable: true,
        max_depth: Some(3),
        degradation_factor: 0.8,
        min_inherited_level: TrustLevel::Basic,
    },
    metadata: HashMap::new(),
};

engine.add_federation_trust(housing_federation.clone(), governance_trust);
```

### Configuring Cross-Federation Bridges

```rust
use icn_identity::{FederationTrustBridge, BridgeConfig};

// Configure bridge between federations
let bridge_config = BridgeConfig {
    bidirectional: true,
    allowed_contexts: [TrustContext::ResourceSharing, TrustContext::MutualCredit]
        .into_iter().collect(),
    max_bridge_trust: TrustLevel::Partial,
    bridge_degradation: 0.6,
};

let bridge = FederationTrustBridge {
    from_federation: housing_federation.clone(),
    to_federation: tech_federation.clone(),
    trust: /* bridge trust relationship */,
    bridge_config,
    established_at: chrono::Utc::now().timestamp() as u64,
    expires_at: None,
};

engine.add_bridge(bridge);
```

### Setting Trust Policies

```rust
use icn_identity::TrustPolicyRule;

// Create governance policy
let governance_rule = TrustPolicyRule {
    name: "governance_strict".to_string(),
    applicable_contexts: [TrustContext::Governance].into_iter().collect(),
    min_trust_level: TrustLevel::Partial,
    require_federation_membership: true,
    max_inheritance_depth: Some(2),
    allow_cross_federation: false,
    custom_validator: None,
};

engine.add_rule(governance_rule);
```

### Validating Trust

```rust
// Validate trust for a specific operation
let result = engine.validate_trust(
    &alice,
    &bob,
    &TrustContext::Governance,
    "vote_on_proposal",
);

match result {
    TrustValidationResult::Allowed { effective_trust, trust_path } => {
        println!("Trust validated: {:?}", effective_trust);
        println!("Trust path: {:?}", trust_path);
    }
    TrustValidationResult::Denied { reason } => {
        println!("Trust denied: {}", reason);
    }
}
```

## Trust Inheritance

Trust inheritance allows cooperatives to inherit trust relationships from their federation membership. This enables scalable trust propagation while maintaining security through configurable degradation.

### Inheritance Rules

1. **Inheritable Flag** - Trust relationships can be marked as inheritable
2. **Maximum Depth** - Limits how many levels trust can propagate
3. **Degradation Factor** - Reduces trust level at each inheritance level
4. **Minimum Level** - Prevents trust from degrading below a threshold

### Example Inheritance Calculation

```
Original Trust: Full (level 3)
Degradation Factor: 0.8
Inheritance Depth: 1

Inherited Trust: Full -> Partial (3 * 0.8 = 2.4 -> Partial)
```

## Cross-Federation Bridges

Bridges enable trust relationships to span across different federations, allowing for inter-federation cooperation while maintaining security boundaries.

### Bridge Configuration

- **Bidirectional** - Whether trust flows both ways
- **Allowed Contexts** - Which trust contexts can cross the bridge
- **Maximum Trust** - Maximum trust level allowed across the bridge
- **Degradation** - Trust reduction when crossing the bridge

### Security Considerations

1. Bridges introduce controlled trust degradation
2. Only specific trust contexts can cross bridges
3. Bridge trust levels are capped
4. Bridges can be time-limited with expiration dates

## Integration with Governance

The framework integrates with ICN's governance system through the `FederationGovernanceEngine`:

```rust
use icn_governance::{FederationGovernanceEngine, TrustAwareGovernancePolicy};

let governance = FederationGovernanceEngine::new(trust_engine, Some(federation_id));

// Add trust-aware voting policy
let voting_policy = TrustAwareGovernancePolicy {
    action: GovernanceAction::Vote { /* ... */ },
    required_context: TrustContext::Governance,
    min_trust_level: TrustLevel::Partial,
    require_federation_membership: true,
    voting_threshold: 0.6,
    quorum_requirement: 0.3,
    allow_cross_federation: false,
};

governance.add_policy("vote".to_string(), voting_policy);
```

## Security Model

### Trust Boundaries

1. **Federation Boundaries** - Trust is scoped to specific federations
2. **Context Boundaries** - Trust is limited to specific activity contexts
3. **Time Boundaries** - Trust relationships can have expiration dates
4. **Level Boundaries** - Trust levels provide graduated access control

### Validation Process

1. **Direct Trust Check** - Look for direct trust relationships
2. **Inheritance Check** - Check for inherited trust through federation membership
3. **Bridge Check** - Check for trust through cross-federation bridges
4. **Policy Validation** - Validate against applicable trust policies

### Degradation Mechanisms

- **Inheritance Degradation** - Trust weakens as it propagates through levels
- **Bridge Degradation** - Trust weakens when crossing federation boundaries
- **Time Degradation** - Trust can expire and require renewal
- **Distance Degradation** - Trust weakens with social/organizational distance

## Performance Considerations

- Trust relationships are cached for efficient lookup
- Federation memberships are indexed for fast membership checks
- Bridge configurations are pre-computed for bridge traversal
- Trust validation uses early termination for performance

## Testing

The framework includes comprehensive tests covering:

- Trust context scoping
- Inheritance calculations
- Cross-federation bridges
- Policy validation
- Integration with governance

Run tests with:
```bash
cargo test federation_trust
```

## Examples

See the complete demonstration example at:
```
examples/federation_trust_demo.rs
```

Run the demo with:
```bash
cargo run --example federation_trust_demo
```

## Future Enhancements

1. **Trust Metrics** - Quantitative trust scoring based on interaction history
2. **Reputation Integration** - Integration with the ICN reputation system
3. **Zero-Knowledge Proofs** - Privacy-preserving trust validation
4. **Trust Delegation** - Delegated trust relationships
5. **Trust Revocation** - Mechanisms for revoking trust relationships
6. **Trust Auditing** - Audit trails for trust relationship changes