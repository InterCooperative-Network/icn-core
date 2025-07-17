# Federation Identity & Trust CLI Examples

This document demonstrates the new federation identity and trust management capabilities added to the ICN CLI.

## 🏛️ Federation Trust Management

### Configure Federation Trust Policy
```bash
# Configure trust policy for a federation
echo '{
  "contexts": ["governance", "resource_sharing"],
  "min_trust_level": "partial",
  "inheritance": {
    "inheritable": true,
    "max_depth": 3,
    "degradation_factor": 0.8
  }
}' | icn-cli federation trust configure test_federation -
```

### Add Trust Relationship
```bash
# Add a scoped trust relationship
echo '{
  "attestor": "did:key:alice",
  "subject": "did:key:bob", 
  "context": "governance",
  "trust_level": "full",
  "federation": "test_federation"
}' | icn-cli federation trust add -
```

### Validate Trust for Operations
```bash
# Validate if alice can vote in governance context
icn-cli federation trust validate \
  --actor did:key:alice \
  --target did:key:bob \
  --context governance \
  --operation vote \
  --federation test_federation
```

### Create Cross-Federation Bridge
```bash
# Create trust bridge between federations
echo '{
  "from_federation": "coop_federation_a",
  "to_federation": "coop_federation_b",
  "bridge_config": {
    "bidirectional": false,
    "allowed_contexts": ["resource_sharing"],
    "max_bridge_trust": "basic",
    "bridge_degradation": 0.5
  }
}' | icn-cli federation trust bridge -
```

### Bootstrap Trust with Another Federation
```bash
# Bootstrap trust relationship
icn-cli federation trust bootstrap \
  --peer federation_b_peer_id \
  --contexts governance,resource_sharing \
  --trust-level partial
```

## 📊 Federation Metadata Management

### Set Federation Metadata
```bash
# Configure federation scope and policies
echo '{
  "name": "Worker Cooperative Federation",
  "description": "A federation of worker cooperatives focused on sustainable technology",
  "scope": {
    "geographic": {
      "countries": ["US", "CA"],
      "global": false
    },
    "sectoral": ["technology", "renewable_energy"],
    "size_limits": {
      "max_members": 100,
      "min_members": 3
    }
  }
}' | icn-cli federation metadata set test_federation -
```

### Configure Quorum Policies
```bash
# Set voting quorum requirements
echo '{
  "decision_type": "governance",
  "quorum_threshold": 0.6,
  "approval_threshold": 0.67,
  "require_unanimous": false,
  "weight_assignment": "Equal",
  "voting_deadline": 604800
}' | icn-cli federation metadata quorum test_federation -
```

### Manage Federation Members
```bash
# Add a member cooperative
echo '{
  "name": "Green Tech Cooperative",
  "cooperative_type": "Worker",
  "description": "Renewable energy technology development",
  "capabilities": ["software_development", "hardware_design"]
}' | icn-cli federation metadata add-member \
  test_federation \
  did:key:greentech_coop \
  -
```

## 🆔 Federation DID Document Management

### Generate DID Document
```bash
# Generate DID document for federation
icn-cli federation did generate test_federation --output federation_did.json
```

### Verify DID Document
```bash
# Verify a DID document
cat federation_did.json | icn-cli federation did verify -
```

### Publish DID Document
```bash
# Publish DID document to federation network
cat federation_did.json | icn-cli federation did publish test_federation -
```

### Resolve DID Document
```bash
# Resolve DID from federation network
icn-cli federation did resolve did:key:example_federation --federation test_federation
```

## 📈 Query Federation Status

### List Trust Relationships
```bash
# List all trust relationships in federation
icn-cli federation trust list test_federation --context governance
```

### Get Federation Metadata
```bash
# Get complete federation metadata
icn-cli federation metadata get test_federation
```

### List Federation Members
```bash
# List all member cooperatives
icn-cli federation metadata members test_federation
```

## 🔍 Advanced Trust Operations

### Trust Path Finding
```bash
# Find trust paths between entities
icn-cli trust paths did:key:alice did:key:charlie \
  --context governance \
  --max-length 5 \
  --max-paths 3
```

### Federation Trust Statistics
```bash
# Get trust statistics for federation
icn-cli trust federation-stats test_federation
```

### Trust Network Neighbors
```bash
# Find trust network neighbors
icn-cli trust neighbors did:key:alice \
  --max-distance 2 \
  --min-level basic
```

## 🌐 Cross-Federation Operations

These commands enable secure cooperation between multiple federations:

1. **Trust Bootstrapping** - Establish initial trust between federations
2. **Bridge Management** - Control cross-federation trust flow
3. **DID Resolution** - Resolve identities across federation boundaries
4. **Metadata Sharing** - Discover federation capabilities and policies

The enhanced CLI provides comprehensive tools for managing complex federation identity and trust relationships in the InterCooperative Network.