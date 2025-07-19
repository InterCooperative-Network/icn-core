# @icn/ts-sdk

Cross-platform TypeScript SDK for ICN applications. Works in React Native, web browsers, and Node.js environments.

## Overview

This package provides a unified TypeScript SDK that wraps the core `@icn/client-sdk` with additional functionality for:
- Cross-platform storage (localStorage, AsyncStorage, memory fallback)
- React/React Native hooks and context
- Type-safe API interfaces
- Connection management
- Error handling and utilities

## Technology Stack

- **TypeScript**: Full type safety
- **Platform Agnostic**: Works in React Native, web, and Node.js
- **Storage Abstraction**: Automatic platform detection for persistence
- **React Integration**: Optional hooks and providers
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
import { ICNClient, createStorage } from '@icn/ts-sdk'

// Create client
const client = new ICNClient({
  nodeEndpoint: 'http://localhost:8080',
  network: 'devnet',
  storage: createStorage() // Auto-detects platform
})

// Connect and use
async function main() {
  await client.connect()
  
  // Submit a mesh job
  const jobId = await client.submitJob({
    command: 'echo "Hello ICN"',
    resources: { cpu: 1, memory: 512 }
  })
  
  console.log('Job submitted:', jobId)
}
```

### React/React Native Usage

```tsx
import React from 'react'
import { ICNProvider, useICNClient, useICNJobs } from '@icn/ts-sdk'

// Wrap your app with the provider
function App() {
  return (
    <ICNProvider options={{
      nodeEndpoint: 'http://localhost:8080',
      network: 'devnet'
    }}>
      <JobSubmissionScreen />
    </ICNProvider>
  )
}

// Use the client in components
function JobSubmissionScreen() {
  const { connected, error } = useICNClient()
  const { jobs, submitJob, loading } = useICNJobs()
  
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
    </View>
  )
}
```

### Storage Configuration

```typescript
import { createStorage, ICNStorage } from '@icn/ts-sdk'
import AsyncStorage from '@react-native-async-storage/async-storage'

// React Native
const storage = createStorage('@myapp:', AsyncStorage)

// Web browser
const storage = createStorage('@myapp:') // Uses localStorage

// Custom storage adapter
const customStorage = {
  async getItem(key: string) { /* ... */ },
  async setItem(key: string, value: string) { /* ... */ },
  async removeItem(key: string) { /* ... */ },
  async clear() { /* ... */ }
}

const client = new ICNClient({
  nodeEndpoint: 'http://localhost:8080',
  storage: new ICNStorage(customStorage)
})
```

## API Reference

### ICNClient

Main client class for interacting with ICN.

```typescript
class ICNClient {
  constructor(options: ICNClientOptions)
  
  // Connection
  async connect(): Promise<void>
  async disconnect(): Promise<void>
  getConnectionState(): ICNConnectionState
  
  // Jobs
  async submitJob(jobSpec: any, options?: JobSubmissionOptions): Promise<string>
  async getJob(jobId: string): Promise<any>
  async listJobs(): Promise<any[]>
  
  // Identity
  async registerDid(didDocument: any): Promise<string>
  async resolveDid(did: string): Promise<any>
  
  // Governance
  async submitProposal(proposal: any): Promise<string>
  async vote(proposalId: string, vote: 'yes' | 'no'): Promise<void>
  
  // Storage
  getStorage(): ICNStorage
}
```

### React Hooks

Available when using React/React Native:

```typescript
// Provider
function ICNProvider(props: { 
  children: ReactNode
  options: ICNClientOptions 
})

// Hooks
function useICNClient(): {
  client: ICNClient | null
  connected: boolean
  connecting: boolean
  error: string | null
}

function useICNConnection(): {
  connected: boolean
  connecting: boolean
  error: string | null
}

function useICNJobs(): {
  jobs: any[]
  loading: boolean
  submitJob: (jobSpec: any, options?: any) => Promise<string>
  refreshJobs: () => Promise<void>
}
```

### Utilities

```typescript
// Validation
validateDid(did: string): boolean
validateAddress(address: string): boolean
validateNetwork(network: string): boolean
validateUrl(url: string): boolean

// Formatting
formatMana(amount: number): string
formatJobId(jobId: string): string
formatRelativeTime(timestamp: number): string
formatBytes(bytes: number): string
formatError(error: unknown): string
```

## Platform Support

| Feature | Web | React Native | Node.js |
|---------|-----|--------------|---------|
| Core Client | ✅ | ✅ | ✅ |
| Storage | ✅ (localStorage) | ✅ (AsyncStorage) | ✅ (memory) |
| React Hooks | ✅ | ✅ | ❌ |
| Type Safety | ✅ | ✅ | ✅ |

## Storage Adapters

The SDK automatically detects the platform and uses appropriate storage:

- **Web**: `localStorage` with fallback to memory
- **React Native**: `AsyncStorage` (requires installation)
- **Node.js**: Memory storage (non-persistent)
- **Custom**: Implement `StorageAdapter` interface

## Error Handling

```typescript
try {
  await client.connect()
} catch (error) {
  console.error('Connection failed:', formatError(error))
}

// Check connection state
const state = client.getConnectionState()
if (!state.connected) {
  console.warn('Not connected to ICN')
}
```

## Configuration

```typescript
import { createConfig } from '@icn/ts-sdk'

const config = createConfig({
  defaultNodeEndpoint: 'https://mainnet.icn.coop',
  defaultNetwork: 'mainnet',
  storagePrefix: '@myapp:'
})
```

## Examples

### Job Submission with Error Handling

```typescript
async function submitJobSafely(client: ICNClient, jobSpec: any) {
  try {
    if (!client.getConnectionState().connected) {
      await client.connect()
    }
    
    const jobId = await client.submitJob(jobSpec, {
      maxCost: 1000,
      timeout: 30000,
      priority: 'normal'
    })
    
    console.log(`Job submitted: ${formatJobId(jobId)}`)
    return jobId
  } catch (error) {
    console.error('Job submission failed:', formatError(error))
    throw error
  }
}
```

### DID Management

```typescript
async function createAndRegisterDid(client: ICNClient) {
  const didDocument = {
    context: ['https://w3id.org/did/v1'],
    id: `did:icn:${generateRandomId()}`,
    publicKey: [{
      id: '#key1',
      type: 'Ed25519VerificationKey2018',
      controller: 'self',
      publicKeyBase58: generatePublicKey()
    }]
  }
  
  const did = await client.registerDid(didDocument)
  console.log(`DID registered: ${did}`)
  return did
}
```

### Governance Participation

```typescript
async function participateInGovernance(client: ICNClient) {
  // Submit a proposal
  const proposalId = await client.submitProposal({
    title: 'Increase Block Size',
    description: 'Proposal to increase max block size to 2MB',
    changes: [{ parameter: 'max_block_size', value: 2048000 }]
  })
  
  // Vote on the proposal
  await client.vote(proposalId, 'yes')
  console.log(`Voted on proposal: ${proposalId}`)
}
```

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
```

## Contributing

1. Follow TypeScript best practices
2. Ensure cross-platform compatibility
3. Add tests for new functionality
4. Update documentation
5. Test on web, React Native, and Node.js

## License

Apache-2.0 