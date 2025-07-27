# Frontend-Backend Integration Guide

> **Comprehensive guide for integrating ICN frontend applications with backend APIs**

This document provides detailed integration instructions for ICN's ~60+ HTTP endpoints, TypeScript SDK usage, and cross-platform development patterns.

## 📖 Overview

The ICN ecosystem provides a comprehensive full-stack development platform with multiple frontend applications connecting to production-grade backend services.

### 🎯 Integration Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                 Frontend Applications                       │
├─────────────┬─────────────┬─────────────┬─────────────────┤
│   Web UI    │  Wallet UI  │  Agoranet   │    Explorer     │
│ (Dashboard) │ (Identity)  │(Governance) │  (DAG/Data)     │
└─────────────┴─────────────┴─────────────┴─────────────────┘
             │               │               │
┌────────────▼───────────────▼───────────────▼─────────────────┐
│                   TypeScript SDK                              │
│              (@icn/ts-sdk - Shared Client)                   │
└─────────────────────────┬─────────────────────────────────────┘
                          │
┌─────────────────────────▼─────────────────────────────────────┐
│                   ICN Node HTTP API                          │
│                    (~60+ Endpoints)                          │
└─────────────────────────┬─────────────────────────────────────┘
                          │
┌─────────────────────────▼─────────────────────────────────────┐
│              Backend Services & Storage                      │
│ (Governance, Mesh, Identity, DAG, Economics, P2P)           │
└───────────────────────────────────────────────────────────────┘
```

### 📱 Frontend Applications

| Application | Technology | Purpose | Integration Level |
|-------------|------------|---------|------------------|
| **Web UI** | React + TypeScript | Federation management dashboard | ~70% complete |
| **Wallet UI** | React Native + Tamagui | Cross-platform DID/asset management | ~60% complete |
| **Agoranet** | React Native + Tamagui | Governance & social platform | ~55% complete |
| **Explorer** | React + TypeScript | DAG and network data browser | ~65% complete |

### 🔧 Backend Integration

- **60+ HTTP Endpoints** - Complete REST API coverage
- **Real-time Updates** - WebSocket support for live data
- **Authentication** - API key and bearer token support
- **Error Handling** - Comprehensive error types and recovery
- **Circuit Breakers** - Built-in resilience patterns

## 🚀 Quick Start Integration

### 1. TypeScript SDK Setup

```bash
# Install the TypeScript SDK
npm install @icn/ts-sdk

# Or if using the monorepo
cd packages/ts-sdk
npm install
npm run build
```

### 2. Basic Client Configuration

```typescript
import { ICNClient } from '@icn/ts-sdk';

// Development configuration
const client = new ICNClient({
  nodeEndpoint: 'http://localhost:7845',
  apiKey: 'dev-key-123',
  network: 'development',
  timeout: 10000,
});

// Production configuration
const prodClient = new ICNClient({
  nodeEndpoint: 'https://node.your-federation.org',
  apiKey: process.env.ICN_API_KEY,
  bearerToken: process.env.ICN_BEARER_TOKEN,
  network: 'production',
  retryConfig: {
    maxAttempts: 3,
    backoffMs: 1000,
  },
});
```

### 3. Connection and Health Check

```typescript
async function initializeConnection() {
  try {
    // Test connection
    await client.connect();
    
    // Get node information
    const nodeInfo = await client.system.getInfo();
    console.log('Connected to ICN node:', nodeInfo.version);
    
    // Check node status
    const status = await client.system.getStatus();
    console.log('Node status:', status.status); // "healthy" | "degraded" | "unhealthy"
    
    return true;
  } catch (error) {
    console.error('Failed to connect to ICN node:', error);
    return false;
  }
}
```

---

## 🔌 Core API Integration Points

### 1. System & Node Information

**Available Endpoints:**
- `GET /system/info` - Node version, build info, and capabilities
- `GET /system/status` - Operational status and health metrics
- `GET /system/metrics` - Prometheus-style metrics (if enabled)

**Frontend Integration:**
```typescript
// Real-time status monitoring component
import React, { useEffect, useState } from 'react';
import { ICNClient } from '@icn/ts-sdk';

interface NodeStatusProps {
  client: ICNClient;
}

export const NodeStatusWidget: React.FC<NodeStatusProps> = ({ client }) => {
  const [status, setStatus] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStatus = async () => {
      try {
        const [info, status] = await Promise.all([
          client.system.getInfo(),
          client.system.getStatus(),
        ]);
        
        setStatus({ info, status });
      } catch (error) {
        console.error('Failed to fetch node status:', error);
      } finally {
        setLoading(false);
      }
    };

    // Initial fetch
    fetchStatus();
    
    // Poll every 30 seconds
    const interval = setInterval(fetchStatus, 30000);
    return () => clearInterval(interval);
  }, [client]);

  if (loading) return <div>Loading node status...</div>;
  if (!status) return <div>Failed to load node status</div>;

  return (
    <div className="node-status">
      <h3>Node Status</h3>
      <div>Version: {status.info.version}</div>
      <div>Status: <span className={`status-${status.status.status}`}>
        {status.status.status}
      </span></div>
      <div>Peers Connected: {status.status.peers_connected}</div>
      <div>Mana Balance: {status.status.mana_balance}</div>
    </div>
  );
};
```

### 2. Identity & DID Management

**Available Endpoints:**
- `POST /identity/did/create` - Create new DID
- `GET /identity/did/{did}` - Resolve DID document
- `POST /identity/credentials/issue` - Issue verifiable credential
- `POST /identity/credentials/verify` - Verify credential
- `GET /identity/credentials/{id}` - Get credential by ID

**Frontend Integration:**

```typescript
// Identity management service
export class IdentityService {
  constructor(private client: ICNClient) {}

  async createNewIdentity(): Promise<{ did: string; document: any }> {
    const response = await this.client.identity.createDid({
      method: 'key',
      keyType: 'Ed25519',
    });
    
    return {
      did: response.did,
      document: response.document,
    };
  }

  async issueCredential(recipientDid: string, credentialData: any) {
    return await this.client.identity.issueCredential({
      recipient: recipientDid,
      type: ['VerifiableCredential'],
      credentialSubject: credentialData,
      expirationDate: new Date(Date.now() + 365 * 24 * 60 * 60 * 1000), // 1 year
    });
  }

  async verifyCredential(credential: any): Promise<boolean> {
    try {
      const result = await this.client.identity.verifyCredential(credential);
      return result.valid;
    } catch (error) {
      console.error('Credential verification failed:', error);
      return false;
    }
  }
}

// React component for DID creation
export const CreateIdentityForm: React.FC = () => {
  const [loading, setLoading] = useState(false);
  const [identity, setIdentity] = useState(null);

  const handleCreateIdentity = async () => {
    setLoading(true);
    try {
      const identityService = new IdentityService(client);
      const newIdentity = await identityService.createNewIdentity();
      setIdentity(newIdentity);
    } catch (error) {
      console.error('Failed to create identity:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <button onClick={handleCreateIdentity} disabled={loading}>
        {loading ? 'Creating...' : 'Create New Identity'}
      </button>
      
      {identity && (
        <div className="identity-result">
          <h3>New Identity Created</h3>
          <div>DID: <code>{identity.did}</code></div>
          <details>
            <summary>DID Document</summary>
            <pre>{JSON.stringify(identity.document, null, 2)}</pre>
          </details>
        </div>
      )}
    </div>
  );
};
```

### 3. Governance & Proposals

**Available Endpoints:**
- `POST /governance/proposals` - Submit new proposal
- `GET /governance/proposals` - List all proposals
- `GET /governance/proposals/{id}` - Get proposal details
- `POST /governance/proposals/{id}/vote` - Cast vote on proposal
- `POST /governance/proposals/{id}/execute` - Execute approved proposal

**Frontend Integration:**

```typescript
// Governance service
export class GovernanceService {
  constructor(private client: ICNClient) {}

  async submitProposal(proposalData: {
    title: string;
    description: string;
    type: 'ParameterChange' | 'TextProposal' | 'BudgetAllocation';
    metadata?: any;
  }) {
    return await this.client.governance.submitProposal({
      title: proposalData.title,
      description: proposalData.description,
      proposal_type: proposalData.type,
      metadata: proposalData.metadata || {},
    });
  }

  async castVote(proposalId: string, vote: 'Yes' | 'No' | 'Abstain') {
    return await this.client.governance.castVote({
      proposal_id: proposalId,
      vote: vote,
      weight: 1.0,
    });
  }

  async getProposals(status?: 'Pending' | 'Active' | 'Passed' | 'Rejected') {
    return await this.client.governance.getProposals({
      status,
      limit: 50,
    });
  }
}

// React component for proposal creation
export const ProposalForm: React.FC = () => {
  const [formData, setFormData] = useState({
    title: '',
    description: '',
    type: 'TextProposal' as const,
  });
  const [submitting, setSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);

    try {
      const governanceService = new GovernanceService(client);
      const proposal = await governanceService.submitProposal(formData);
      
      console.log('Proposal submitted:', proposal.id);
      // Reset form or redirect
      setFormData({ title: '', description: '', type: 'TextProposal' });
    } catch (error) {
      console.error('Failed to submit proposal:', error);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="proposal-form">
      <div>
        <label>Title:</label>
        <input
          type="text"
          value={formData.title}
          onChange={(e) => setFormData(prev => ({...prev, title: e.target.value}))}
          required
        />
      </div>
      
      <div>
        <label>Description:</label>
        <textarea
          value={formData.description}
          onChange={(e) => setFormData(prev => ({...prev, description: e.target.value}))}
          required
        />
      </div>
      
      <div>
        <label>Type:</label>
        <select
          value={formData.type}
          onChange={(e) => setFormData(prev => ({...prev, type: e.target.value as any}))}
        >
          <option value="TextProposal">Text Proposal</option>
          <option value="ParameterChange">Parameter Change</option>
          <option value="BudgetAllocation">Budget Allocation</option>
        </select>
      </div>
      
      <button type="submit" disabled={submitting}>
        {submitting ? 'Submitting...' : 'Submit Proposal'}
      </button>
    </form>
  );
};
```

### 4. Mesh Computing & Job Execution

**Available Endpoints:**
- `POST /mesh/jobs` - Submit computational job
- `GET /mesh/jobs` - List submitted jobs
- `GET /mesh/jobs/{id}` - Get job details and status
- `GET /mesh/jobs/{id}/receipt` - Get execution receipt
- `GET /mesh/executors` - List available executors

**Frontend Integration:**

**Frontend Usage:**
```typescript
// Submit a job
const jobSpec = {
  name: 'data-processing',
  image: 'python:3.9',
  command: ['python', 'process.py'],
  resources: {
    cpu: '500m',
    memory: '1Gi'
  }
};

const jobId = await client.submitJob(jobSpec, {
  maxCost: 500,
  priority: 'normal'
});

// Monitor job status
const status = await client.getJob(jobId);
console.log(`Job ${jobId} status: ${status.status}`);
```

### 3. Identity Management

**Endpoints:**
- `POST /identity/dids` - Register DID documents
- `GET /identity/dids/{did}` - Resolve DID documents
- `POST /identity/credentials` - Issue verifiable credentials

**Frontend Usage:**
```typescript
// Register a new DID
const didDocument = {
  '@context': 'https://www.w3.org/ns/did/v1',
  id: 'did:key:new_cooperative',
  // ... rest of DID document
};

const registeredDid = await client.registerDid(didDocument);

// Resolve existing DID
const resolved = await client.resolveDid('did:key:existing_coop');
```

### 4. Governance

**Endpoints:**
- `POST /governance/proposals` - Submit governance proposals
- `POST /governance/votes` - Cast votes on proposals  
- `GET /governance/proposals` - List active proposals

**Frontend Usage:**
```typescript
// Submit a proposal
const proposal = {
  title: 'Upgrade Network Parameters',
  description: 'Proposal to increase block size limit',
  proposer_did: 'did:key:proposer',
  proposal: {
    type: 'SystemParameterChange',
    param: 'max_block_size',
    value: '2MB'
  },
  duration_secs: 7 * 24 * 3600 // 7 days
};

const proposalId = await client.submitProposal(proposal);

// Cast a vote
await client.vote(proposalId, 'yes');
```

### 5. Federation Management

**Endpoints:**
- `GET /federation/status` - Federation metadata and health
- `GET /federation/peers` - List federation peers
- `POST /federation/peers` - Add new peers

**Frontend Usage:**
```typescript
// Get federation status
const federationStatus = await client.federation.getStatus();
console.log(`Federation: ${federationStatus.name}, Members: ${federationStatus.total_members}`);

// List peers
const peers = await client.federation.listPeers();
```

### 6. Account Management

**Endpoints:**
- `GET /account/{did}/mana` - Get mana balance
- `POST /account/{did}/mana/transfer` - Transfer mana
- `GET /account/{did}/tokens` - List token balances

**Frontend Usage:**
```typescript
// Check mana balance
const balance = await client.account.getManaBalance('did:key:user');
console.log(`Mana balance: ${balance.balance}`);

// List token balances
const tokens = await client.tokens.listBalances('did:key:user');
```

## Configuration

### Environment Variables

Frontend applications use these environment variables for configuration:

```bash
# Required: Backend node endpoint
VITE_ICN_NODE_ENDPOINT=http://localhost:8080

# Optional: Network type
VITE_ICN_NETWORK=devnet

# Optional: API timeout
VITE_ICN_TIMEOUT=30000

# Optional: Enable debug logging
VITE_ICN_DEBUG=true
```

### TypeScript SDK Configuration

```typescript
const client = new ICNClient({
  nodeEndpoint: process.env.VITE_ICN_NODE_ENDPOINT || 'http://localhost:8080',
  network: (process.env.VITE_ICN_NETWORK || 'devnet') as 'mainnet' | 'testnet' | 'devnet',
  timeout: parseInt(process.env.VITE_ICN_TIMEOUT || '30000'),
  storage: createSecureStorage('@icn-app:', {
    enableEncryption: true,
    passphrase: 'user-provided-passphrase'
  })
});
```

## Testing Integration

### Unit Tests

The TypeScript SDK includes comprehensive unit tests:

```bash
cd packages/ts-sdk
npm test
```

### Integration Tests

Full end-to-end integration tests validate frontend-backend flows:

```bash
# Test against local node
npm run test:integration

# Test with offline mocks
npm run test:integration -- --offline

# Test specific category
npm run test:integration -- --category="Mesh Computing"
```

### Manual Testing

1. **Start Backend Node:**
   ```bash
   just build  # Build backend
   cargo run -p icn-node  # Start node
   ```

2. **Start Frontend App:**
   ```bash
   just dev-web-ui  # Start web UI
   # or
   just dev-wallet  # Start wallet UI
   ```

3. **Test Key Flows:**
   - Navigation and UI responsiveness
   - Connection to backend node
   - Job submission and monitoring
   - Proposal creation and voting
   - Account balance checks

## Troubleshooting

### Common Issues

#### 1. Connection Refused
```
Error: Failed to connect: connect ECONNREFUSED 127.0.0.1:8080
```

**Solutions:**
- Ensure ICN node is running: `cargo run -p icn-node`
- Check node endpoint configuration
- Verify network connectivity

#### 2. API Version Mismatch
```
Error: Unsupported API version
```

**Solutions:**
- Rebuild TypeScript SDK: `cd packages/ts-sdk && npm run build`
- Ensure frontend and backend are from same git commit
- Check for breaking API changes in CHANGELOG.md

#### 3. Authentication Errors
```
Error: Unauthorized - Invalid DID or signature
```

**Solutions:**
- Verify DID is properly registered
- Check credential proofs are included
- Ensure cryptographic signatures are valid

#### 4. Timeout Errors
```
Error: Request timeout after 30000ms
```

**Solutions:**
- Increase timeout in client configuration
- Check network latency to node
- Verify node is not under heavy load

### Debug Mode

Enable debug logging for detailed request/response information:

```typescript
const client = new ICNClient({
  nodeEndpoint: 'http://localhost:8080',
  network: 'devnet',
  debug: true  // Enable debug logging
});
```

### Health Checks

Monitor integration health with these endpoints:

- `GET /health` - Basic health check
- `GET /metrics` - Prometheus metrics
- `GET /system/status` - Detailed node status

## API Documentation

### Generated Documentation

The TypeScript client SDK is automatically generated from backend API definitions:

1. **Update API definitions:** Edit trait definitions in `crates/icn-api/src/`
2. **Regenerate client:** Run `cargo run -p icn-api --bin generate-client-sdk`
3. **Rebuild TypeScript:** Run `cd packages/ts-sdk && npm run build`

### OpenAPI Specification

The backend exposes an OpenAPI specification at:
- `GET /api-docs/openapi.json` - OpenAPI 3.0 specification
- `GET /api-docs/` - Swagger UI documentation

## Development Workflow

### Adding New API Endpoints

1. **Define backend trait:** Add method to appropriate trait in `crates/icn-api/src/`
2. **Implement backend logic:** Add implementation in relevant service crate
3. **Regenerate TypeScript client:** Run code generation script
4. **Update frontend code:** Use new client methods in frontend apps
5. **Add integration tests:** Test new endpoints in `integration.test.ts`
6. **Update documentation:** Document new API in this guide

### Testing Changes

1. **Backend tests:** `just test` 
2. **Frontend tests:** `just test-frontend`
3. **Integration tests:** `npm run test:integration`
4. **Manual testing:** Start devnet and test UI flows

## Security Considerations

### Authentication

All API requests should include:
- Valid DID identifier
- Cryptographic signature
- Credential proofs where required

### HTTPS in Production

Always use HTTPS endpoints in production:

```typescript
const client = new ICNClient({
  nodeEndpoint: 'https://api.myicnnode.com',
  network: 'mainnet'
});
```

### Storage Encryption

Enable encryption for sensitive data storage:

```typescript
const storage = createSecureStorage('@icn-app:', {
  enableEncryption: true,
  passphrase: userProvidedPassphrase
});
```

## Performance Optimization

### Caching

The TypeScript SDK includes intelligent caching:

```typescript
// Cached for 5 minutes
const nodeInfo = await client.system.getInfo();

// Force refresh
const freshInfo = await client.system.getInfo({ forceRefresh: true });
```

### Request Batching

Batch multiple requests for better performance:

```typescript
const [nodeInfo, nodeStatus, manaBalance] = await Promise.all([
  client.system.getInfo(),
  client.system.getStatus(), 
  client.account.getManaBalance(userDid)
]);
```

### Connection Pooling

The client automatically manages connection pooling and keep-alive.

## Monitoring and Observability

### Metrics Collection

Monitor integration health with these metrics:

- Request success/failure rates
- Response times
- Connection errors
- API call frequency

### Logging

Enable structured logging in production:

```typescript
import { ICNClient, LogLevel } from '@icn/ts-sdk';

const client = new ICNClient({
  nodeEndpoint: 'https://api.mynode.com',
  logLevel: LogLevel.INFO,
  enableMetrics: true
});
```

### Error Reporting

Implement error reporting for production applications:

## Summary

This integration guide demonstrates how to:

1. **Set up TypeScript SDK** with modern async/await patterns
2. **Integrate core APIs** - System, Identity, Governance, Mesh Computing, Economics
3. **Build cross-platform UIs** using React and React Native + Tamagui
4. **Handle real-time updates** with WebSocket connections
5. **Implement error handling** with comprehensive error types
6. **Optimize performance** with caching and connection pooling
7. **Ensure production readiness** with security and monitoring

The ICN ecosystem provides a complete platform for building sophisticated cooperative applications with ~60+ working endpoints, real backend services, and modern frontend frameworks.

### Next Steps

- **[Complete API Reference](../ICN_API_REFERENCE.md)** - Detailed endpoint documentation
- **[TypeScript SDK Guide](../packages/ts-sdk/README.md)** - SDK usage and examples
- **[Development Setup](DEVELOPER_GUIDE.md)** - Environment setup guide
- **[Production Deployment](deployment-guide.md)** - Production deployment guide

The integration capabilities enable rapid development of decentralized governance tools, cooperative economic systems, and federated computing platforms.