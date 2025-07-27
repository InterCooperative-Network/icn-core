// Re-export the core client SDK
export * from '@icn/client-sdk'

// Platform-specific storage and utilities
export { ICNClient } from './client'
export { ICNStorage, createStorage, createSecureStorage } from './storage'
export type { EncryptionConfig } from './storage'
export { createConfig } from './config'
export type { ICNConfig } from './config'

// Enhanced error handling
export {
  ICNError,
  ICNConnectionError,
  ICNAuthError,
  ICNValidationError,
  ICNNetworkError,
  ICNGovernanceError,
  ICNCredentialError,
  ICNTrustError,
  ICNMeshError,
  ICNStorageError,
  ICNTokenError,
  ICNTimeoutError,
  ICNRateLimitError,
  ErrorFactory,
  ErrorUtils,
} from './errors'

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
  EnhancedUtils,
  formatError,
  validateUrl,
  formatRelativeTime,
  formatBytes,
} from './utils'

// React Native specific exports (optional)
export { 
  useICNClient, 
  ICNProvider,
  useICNConnection,
  useICNJobs,
  useICNGovernance,
  useICNTrust,
  useICNCredentials,
  useICNTokens,
  useICNMutualAid,
} from './react' 