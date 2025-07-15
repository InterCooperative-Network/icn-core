# ICN TypeScript Client SDK

A comprehensive TypeScript client SDK for interacting with InterCooperative Network (ICN) nodes.

## Installation

```bash
npm install @icn/client-sdk
# or
yarn add @icn/client-sdk
```

## Quick Start

```typescript
import { ICNClient } from '@icn/client-sdk';

const client = new ICNClient({
  baseUrl: 'http://localhost:7845',
  apiKey: 'your-api-key-here'
});

// Submit a governance proposal
const proposalId = await client.governance.submitProposal({
  proposer_did: 'did:example:alice',
  proposal: {
    type: 'SystemParameterChange',
    data: {
      param: 'mana_regeneration_rate',
      value: '0.1'
    }
  },
  description: 'Increase mana regeneration rate',
  duration_secs: 604800
});

// Vote on a proposal
await client.governance.castVote({
  voter_did: 'did:example:bob',
  proposal_id: proposalId,
  vote_option: 'Yes'
});

// Submit a mesh job
const jobResult = await client.mesh.submitJob({
  job_spec: {
    image: 'python:3.9',
    command: ['python', '-c', 'print("Hello, ICN!")'],
    resources: {
      cpu_cores: 1,
      memory_mb: 512,
      storage_mb: 1024
    }
  },
  submitter_did: 'did:example:submitter',
  max_cost: 1000
});

console.log('Job submitted:', jobResult.job_id);
```

## Features

- **Full Type Safety**: Complete TypeScript definitions for all API endpoints
- **Comprehensive Coverage**: Support for all ICN APIs (governance, identity, federation, mesh, etc.)
- **Error Handling**: Built-in error handling with detailed error information
- **Authentication**: Support for API key and bearer token authentication
- **WebSocket Support**: Real-time event subscriptions (planned)
- **Utilities**: Helper functions for common operations

## API Reference

### Governance

```typescript
// Submit proposal
const proposalId = await client.governance.submitProposal(request);

// Cast vote
await client.governance.castVote({
  voter_did: 'did:example:voter',
  proposal_id: 'proposal_id',
  vote_option: 'Yes'
});

// List all proposals
const proposals = await client.governance.listProposals();

// Get specific proposal
const proposal = await client.governance.getProposal(proposalId);
```

### Identity & Credentials

```typescript
// Issue credential
const credential = await client.identity.issueCredential({
  issuer: 'did:example:issuer',
  holder: 'did:example:holder',
  attributes: {
    name: 'Alice Smith',
    role: 'member'
  },
  schema: 'QmSchemaHash...',
  expiration: Date.now() + 31536000 // 1 year
});

// Verify credential
const isValid = await client.identity.verifyCredential(credential.credential);
```

### Mesh Computing

```typescript
// Submit job
const job = await client.mesh.submitJob({
  job_spec: {
    image: 'ubuntu:20.04',
    command: ['echo', 'Hello World'],
    resources: {
      cpu_cores: 1,
      memory_mb: 256,
      storage_mb: 512
    }
  },
  submitter_did: 'did:example:submitter',
  max_cost: 500
});

// Monitor job status
const status = await client.mesh.getJobStatus(job.job_id);
```

### Federation Management

```typescript
// Join federation
await client.federation.joinFederation({
  peer: '12D3KooWPeerAddress...'
});

// Get federation status
const status = await client.federation.getStatus();
console.log(`Connected to ${status.peer_count} peers`);
```

## Error Handling

The SDK provides detailed error information:

```typescript
try {
  await client.governance.submitProposal(invalidRequest);
} catch (error) {
  if (error instanceof ICNApiError) {
    console.error(`API Error ${error.status}: ${error.message}`);
    console.error('Details:', error.details);
  }
}
```

## WebSocket Events (Planned)

Real-time event subscriptions:

```typescript
import { ICNWebSocketClient } from '@icn/client-sdk';

const wsClient = new ICNWebSocketClient({
  baseUrl: 'ws://localhost:7845'
});

await wsClient.connect();

// Subscribe to proposal updates
wsClient.subscribe('proposal_status_changed', (data) => {
  console.log('Proposal updated:', data);
});

// Subscribe to job progress
wsClient.subscribe('job_progress_updated', (data) => {
  console.log('Job progress:', data);
});
```

## Utilities

```typescript
import { ICNUtils } from '@icn/client-sdk';

// Validate DID format
const isValid = ICNUtils.isValidDid('did:example:alice');

// Format mana balance
const formatted = ICNUtils.formatMana(1500000); // "1.5M"

// Calculate time remaining
const timeLeft = ICNUtils.getTimeRemaining('2025-01-15T10:00:00Z');
```

## Configuration

```typescript
const client = new ICNClient({
  baseUrl: 'http://localhost:7845',
  apiKey: 'your-api-key',           // Optional
  bearerToken: 'your-bearer-token', // Optional
  timeout: 30000                    // Optional, in milliseconds
});
```

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../../CONTRIBUTING.md) for guidelines.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../../LICENSE). 