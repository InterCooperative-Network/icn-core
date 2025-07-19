// Re-export the core client SDK
export * from '@icn/client-sdk'

// Platform-specific storage and utilities
export { ICNClient } from './client'
export { ICNStorage, createStorage } from './storage'
export { ICNConfig, createConfig } from './config'

// Types
export type {
  ICNClientOptions,
  StorageAdapter,
  ConfigOptions,
  FederationCreateRequest,
  CooperativeInfo,
  FederationMetadata,
  CCLTemplate,
  CCLParameter,
} from './types'

// Utilities
export { 
  validateDid, 
  formatMana, 
  formatJobId,
  FederationUtils,
  GovernanceUtils,
  CCLUtils,
  ICNUtils,
  formatError,
  validateUrl,
  formatRelativeTime,
  formatBytes,
} from './utils'

// React Native specific exports (optional)
export { useICNClient, ICNProvider } from './react' 