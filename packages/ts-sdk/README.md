# @icn/ts-sdk

Cross-platform TypeScript SDK for ICN applications. Works in React Native, web browsers, and Node.js environments with **full API coverage** and comprehensive functionality.

## Overview

This package provides a unified TypeScript SDK that wraps the core `@icn/client-sdk` with additional functionality for:
- **Complete API Coverage**: All ICN Core APIs including Trust, Tokens, Mutual Aid, Enhanced Credentials
- Cross-platform storage (localStorage, AsyncStorage, memory fallback)
- React/React Native hooks ecosystem
- Enhanced error handling with specific error types
- Type-safe API interfaces
- Connection management
- Comprehensive utilities and examples

## Technology Stack

- **TypeScript**: Full type safety with comprehensive type coverage
- **Platform Agnostic**: Works in React Native, web, and Node.js
- **Storage Abstraction**: Automatic platform detection with encryption support
- **React Integration**: Complete hooks ecosystem for all APIs
- **Enhanced Error Handling**: Specific error types with retry logic
- **Modular Design**: Import only what you need

## Installation

```bash
# Install the SDK
pnpm add @icn/ts-sdk

# For React Native projects, also install AsyncStorage
pnpm add @react-native-async-storage/async-storage
```

## Quick Start

### Basic Usage (Vanilla TypeScript)

```typescript
import { ICNClient, createStorage, ErrorUtils } from '@icn/ts-sdk'

// Create client
const client = new ICNClient({
  nodeEndpoint: 'http://localhost:8080',
  network: 'devnet',
  storage: createStorage(), // Auto-detects platform
  encryptionConfig: {
    enableEncryption: true,
    passphrase: 'your-secure-key'
  }
})

// Connect and use
async function main() {
  try {
    await client.connect()
    
    // Submit a mesh job
    const jobId = await client.submitJob({
      command: 'echo "Hello ICN"',
      resources: { cpu: 1, memory: 512 }
    })
    
    console.log('Job submitted:', jobId)
  } catch (error) {
    console.error('Error:', ErrorUtils.getErrorMessage(error))
  }
}
```

### React/React Native Usage

```tsx
import React from 'react'
import { 
  ICNProvider, 
  useICNClient, 
  useICNJobs, 
  useICNGovernance,
  useICNTrust,
  useICNCredentials,
  useICNTokens,
  useICNMutualAid 
} from '@icn/ts-sdk'

// Wrap your app with the provider
function App() {
  return (
    <ICNProvider options={{
      nodeEndpoint: 'http://localhost:8080',
      network: 'devnet',
      encryptionConfig: { enableEncryption: true }
    }}>
      <JobSubmissionScreen />
    </ICNProvider>
  )
}

// Use the comprehensive hooks in components
function JobSubmissionScreen() {
  const { connected, error } = useICNClient()
  const { jobs, submitJob, loading } = useICNJobs()
  const { proposals, submitProposal } = useICNGovernance()
  const { trustScore, getTrustScore } = useICNTrust()
  
  const handleSubmit = async () => {
    if (!connected) return
    
    await submitJob({
      command: 'echo "Hello from React"',
      resources: { cpu: 1, memory: 256 }
    })
  }
  
  if (error) return <Text>Error: {error}</Text>
  if (!connected) return <Text>Connecting...</Text>
  
  return (
    <View>
      <Button onPress={handleSubmit} disabled={loading}>
        Submit Job
      </Button>
      <Text>Jobs: {jobs.length}</Text>
      <Text>Proposals: {proposals.length}</Text>
    </View>
  )
}
```

## Complete API Coverage

### ✅ Implemented APIs

| API Category | Status | Description |
|--------------|--------|-------------|
| **Governance** | ✅ Complete | Proposals, voting, delegation |
| **Identity & Credentials** | ✅ Enhanced | Issue, verify, present, revoke credentials |
| **Trust Networks** | ✅ Complete | Trust relationships, paths, scores, graph analytics |
| **Mesh Computing** | ✅ Complete | Job submission, status tracking, execution |
| **Federation** | ✅ Complete | Peer management, federation status |
| **Token System** | ✅ Complete | Token classes, minting, burning, transfers |
| **Mutual Aid** | ✅ Complete | Resource sharing, aid coordination |
| **Account Management** | ✅ Complete | Mana balances, keys, reputation |
| **DAG Storage** | ✅ Complete | Block storage, retrieval, metadata |
| **System Information** | ✅ Complete | Node info, status, health, metrics |
| **Executor API** | ✅ Complete | Queue introspection, job management |

### Enhanced Credential Management

```typescript
// Issue enhanced credentials
const credentialResponse = await client.credentials.issueCredential({
  credential_type: 'skill',
  holder: 'did:key:holder123',
  issuer: 'did:key:issuer456',
  claims: {
    skill_name: 'Rust Programming',
    level: 8,
    certification_date: '2024-01-15'
  },
  evidence: ['https://github.com/user/rust-project'],
  validity_period: 365 * 24 * 3600 // 1 year
})

// Present credentials with selective disclosure
const presentation = await client.credentials.presentCredential({
  credential_proof: credentialResponse.credential_proof,
  context: 'job_application',
  disclosed_fields: ['skill_name', 'level'],
  challenge: 'random-challenge-string'
})

// Verify presentations
const verification = await client.credentials.verifyCredential({
  presentation_id: presentation.presentation_id,
  verification_level: 'enhanced',
  required_claims: ['skill_name']
})
```

### Trust Network Operations

```typescript
// Get trust score for an entity
const trustScore = await client.trust.getTrustScore('did:key:entity123')
console.log(`Trust Score: ${trustScore.score * 100}%`)

// Find trust paths between entities
const paths = await client.trust.findTrustPaths({
  from: 'did:key:alice',
  to: 'did:key:bob',
  context: 'financial',
  max_length: 5,
  min_trust_level: 'medium'
})

// Update trust relationships
await client.trust.updateTrustRelationship({
  from: 'did:key:alice',
  to: 'did:key:bob',
  trust_level: 'high',
  context: 'technical',
  metadata: { reason: 'successful_collaboration' }
})

// Get trust graph statistics
const stats = await client.trust.getTrustGraphStats()
console.log(`Total entities: ${stats.total_entities}`)
console.log(`Average trust: ${stats.average_trust_score}`)
```

### Token Operations

```typescript
// Create a new token class
const tokenClass = await client.tokens.createTokenClass({
  id: 'SKILL_TOKEN',
  name: 'Skill Recognition Token',
  symbol: 'SKILL',
  decimals: 2
})

// Mint tokens to a holder
await client.tokens.mintTokens({
  class_id: 'SKILL_TOKEN',
  to_did: 'did:key:holder123',
  amount: 100
})

// Transfer tokens between accounts
await client.tokens.transferTokens({
  class_id: 'SKILL_TOKEN',
  from_did: 'did:key:sender',
  to_did: 'did:key:receiver',
  amount: 50
})

// Check token balances
const balances = await client.tokens.listBalances('did:key:holder123')
balances.forEach(balance => {
  console.log(`${balance.class_id}: ${balance.amount}`)
})
```

### Mutual Aid Resources

```typescript
// Register a resource for mutual aid
await client.mutualAid.registerResource({
  id: 'resource_001',
  name: 'Web Development Skills',
  description: 'Offering React and TypeScript development assistance',
  category: 'technical_skills',
  provider_did: 'did:key:provider123',
  availability: 'available',
  location: 'Remote',
  contact_info: 'contact@example.com',
  metadata: {
    experience_years: '5',
    hourly_rate: 'volunteer'
  }
})

// List available resources
const resources = await client.mutualAid.listResources()
resources.forEach(resource => {
  console.log(`${resource.name} - ${resource.availability}`)
})
```

## Enhanced Error Handling

The SDK provides comprehensive error handling with specific error types:

```typescript
import { 
  ICNError,
  ICNConnectionError,
  ICNAuthError,
  ICNValidationError,
  ICNNetworkError,
  ErrorUtils 
} from '@icn/ts-sdk'

try {
  await client.connect()
} catch (error) {
  if (ErrorUtils.isErrorType(error, ICNConnectionError)) {
    console.error('Connection failed:', error.message)
    // Handle connection issues
  } else if (ErrorUtils.isErrorType(error, ICNAuthError)) {
    console.error('Authentication required:', error.message)
    // Handle auth issues
  } else if (ErrorUtils.isErrorType(error, ICNValidationError)) {
    console.error('Invalid input:', error.message, error.field)
    // Handle validation errors
  } else {
    console.error('Unexpected error:', ErrorUtils.getErrorMessage(error))
  }
  
  // Check if error is retryable
  if (ErrorUtils.isRetryableError(error)) {
    const delay = ErrorUtils.getRetryDelay(error, 1)
    console.log(`Retrying in ${delay}ms...`)
  }
}
```

## React Hooks Ecosystem

### Complete Hook Collection

```typescript
// Connection management
const { client, connected, connecting, error } = useICNClient()
const { connected, connecting, error } = useICNConnection()

// API-specific hooks
const { jobs, submitJob, loading } = useICNJobs()
const { proposals, submitProposal, castVote } = useICNGovernance()
const { trustScore, getTrustScore, updateTrustRelationship } = useICNTrust()
const { credentials, issueCredential, verifyCredential } = useICNCredentials()
const { balances, transferTokens, createTokenClass } = useICNTokens()
const { resources, registerResource, refreshResources } = useICNMutualAid()
```

### Hook Usage Example

```tsx
function TrustManagement() {
  const { trustScore, loading, getTrustScore, updateTrustRelationship } = useICNTrust()
  const [targetDid, setTargetDid] = useState('')

  const handleTrustUpdate = async () => {
    await updateTrustRelationship({
      from: 'did:key:alice',
      to: targetDid,
      trust_level: 'high',
      context: 'technical'
    })
  }

  return (
    <div>
      <input 
        value={targetDid}
        onChange={(e) => setTargetDid(e.target.value)}
        placeholder="Enter DID"
      />
      <button onClick={() => getTrustScore(targetDid)}>
        Get Trust Score
      </button>
      {trustScore && (
        <div>Score: {(trustScore.score * 100).toFixed(1)}%</div>
      )}
    </div>
  )
}
```

## Platform Support

| Feature | Web | React Native | Node.js |
|---------|-----|--------------|---------|
| Core Client | ✅ | ✅ | ✅ |
| All APIs | ✅ | ✅ | ✅ |
| Storage | ✅ (localStorage) | ✅ (AsyncStorage) | ✅ (memory) |
| Encryption | ✅ | ✅ | ✅ |
| React Hooks | ✅ | ✅ | ❌ |
| Error Handling | ✅ | ✅ | ✅ |
| Type Safety | ✅ | ✅ | ✅ |

## Storage & Encryption

```typescript
import { createSecureStorage } from '@icn/ts-sdk'

// Create encrypted storage
const storage = createSecureStorage('@myapp:', {
  enableEncryption: true,
  passphrase: 'user-provided-secret'
})

// Custom storage adapter
const customStorage = {
  async getItem(key: string) { /* your implementation */ },
  async setItem(key: string, value: string) { /* your implementation */ },
  async removeItem(key: string) { /* your implementation */ },
  async clear() { /* your implementation */ }
}

const client = new ICNClient({
  nodeEndpoint: 'http://localhost:8080',
  storage: new ICNStorage(customStorage, { enableEncryption: true })
})
```

## Comprehensive Examples

The SDK includes extensive examples:

- `examples/basic-usage.ts` - Client setup and basic operations
- `examples/governance-example.ts` - Proposal submission and voting
- `examples/credential-management.ts` - Advanced credential workflows
- `examples/trust-networks.ts` - Trust relationship management
- `examples/mesh-computing.ts` - Job submission and monitoring
- `examples/token-operations.ts` - Token creation and transfers
- `examples/mutual-aid.ts` - Resource sharing workflows
- `examples/react-hooks.tsx` - Complete React integration
- `examples/error-handling.ts` - Comprehensive error management

## Development

```bash
# Install dependencies
pnpm install

# Start development
pnpm dev

# Build for production
pnpm build

# Run tests
pnpm test

# Type checking
pnpm type-check

# Lint code
pnpm lint
```

## Contributing

1. Follow TypeScript best practices
2. Ensure cross-platform compatibility
3. Add tests for new functionality
4. Update documentation and examples
5. Test on web, React Native, and Node.js
6. Include comprehensive error handling

## License

Apache-2.0 