# Frontend-Backend Integration Guide

This document outlines the integration between ICN frontend applications and backend APIs, including testing procedures and troubleshooting.

## Overview

The ICN ecosystem consists of multiple frontend applications that communicate with backend node APIs through a standardized TypeScript SDK.

### Frontend Applications

1. **Web UI** (`apps/web-ui`) - Federation and cooperative management dashboard
2. **Wallet UI** (`apps/wallet-ui`) - Personal DID and asset management
3. **Agoranet** (`apps/agoranet`) - Decentralized social and economic platform  
4. **Explorer** (`apps/explorer`) - Blockchain and DAG explorer

### Backend APIs

- **ICN Node** (`crates/icn-node`) - Main node server with REST APIs
- **API Crate** (`crates/icn-api`) - Service traits and implementations
- **Client SDK** (`crates/icn-api/client-sdk`) - Generated TypeScript client

## API Integration Points

### 1. System Information

**Endpoints:**
- `GET /system/info` - Node version and basic information
- `GET /system/status` - Node operational status and metrics

**Frontend Usage:**
```typescript
import { ICNClient } from '@icn/ts-sdk';

const client = new ICNClient({
  nodeEndpoint: 'http://localhost:8080',
  network: 'devnet'
});

await client.connect();
const nodeInfo = await client.system.getInfo();
const nodeStatus = await client.system.getStatus();
```

### 2. Mesh Computing

**Endpoints:**
- `POST /mesh/jobs` - Submit computational jobs
- `GET /mesh/jobs/{id}` - Get job status
- `GET /mesh/jobs` - List jobs

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

```typescript
client.onError((error, context) => {
  // Send to error reporting service
  errorReportingService.captureException(error, {
    context,
    user: userDid,
    endpoint: context.endpoint
  });
});
```