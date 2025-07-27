// Re-export types from client SDK
export type {
  Proposal,
  ProposalInputType,
  SubmitProposalRequest,
  CastVoteRequest,
  FederationPeerRequest,
  FederationStatus,
  NodeInfo,
  NodeStatus,
  JobStatus,
  ManaBalance,
  ReputationScore,
  VerifiableCredential,
  ZkCredentialProof,
  ZkRevocationProof,
  // Token types
  CreateTokenClassRequest,
  TokenClass,
  MintTokensRequest,
  BurnTokensRequest,
  TransferTokensRequest,
  TokenBalance,
  // Mutual Aid types
  AidResource,
  // Trust types
  TrustLevel,
  TrustContext,
  TrustRelationshipInfo,
  TrustPath,
  TrustScore,
  TrustGraphStats,
  FederationTrustStats,
  TrustQueryFilter,
  TrustPathRequest,
  TrustUpdateRequest,
  // Enhanced Credential types
  IssueCredentialRequest,
  IssueCredentialResponse,
  PresentCredentialRequest,
  PresentCredentialResponse,
  VerifyCredentialRequest,
  CredentialVerificationResult,
  AnchorDisclosureRequest,
  AnchorDisclosureResponse,
  RevokeCredentialRequest,
  RevokeCredentialResponse,
  ListCredentialsRequest,
  CredentialMetadata,
  ListCredentialsResponse,
  CredentialStatus,
  PresentationInfo,
  TrustAttestationInfo,
  // Executor types
  ExecutorQueueInfo,
} from '@icn/client-sdk'

export interface ICNClientOptions {
  nodeEndpoint: string
  privateKey?: string
  storage?: StorageAdapter
  network?: 'mainnet' | 'testnet' | 'devnet'
  timeout?: number
  encryptionConfig?: EncryptionConfig
}

export interface EncryptionConfig {
  passphrase?: string
  enableEncryption?: boolean
}

export interface StorageAdapter {
  getItem(key: string): Promise<string | null>
  setItem(key: string, value: string): Promise<void>
  removeItem(key: string): Promise<void>
  clear(): Promise<void>
}

export interface ConfigOptions {
  defaultNodeEndpoint?: string
  defaultNetwork?: 'mainnet' | 'testnet' | 'devnet'
  storagePrefix?: string
}

export interface ICNConnectionState {
  connected: boolean
  nodeEndpoint: string
  did?: string
  network: string
  manaBalance?: number
}

export interface JobSubmissionOptions {
  maxCost?: number
  timeout?: number
  priority?: 'low' | 'normal' | 'high'
  metadata?: Record<string, string>
}

export interface ProposalFilter {
  status?: 'active' | 'passed' | 'failed' | 'expired'
  author?: string
  limit?: number
  offset?: number
}

export interface ManaTransferOptions {
  recipient: string
  amount: number
  memo?: string
}

// Additional UI-specific types for Federation Dashboard
export interface FederationCreateRequest {
  name: string
  description: string
  admins: string[]
  metadata?: Record<string, any>
}

export interface CooperativeInfo {
  did: string
  name: string
  description?: string
  status: 'active' | 'inactive' | 'pending'
  memberCount: number
  reputation: number
  capabilities: string[]
  joinedAt: string
}

export interface FederationMetadata {
  name: string
  description: string
  created: string
  admins: string[]
  totalMembers: number
  totalCooperatives: number
  governance: {
    activeProposals: number
    totalProposals: number
  }
  mesh: {
    activeJobs: number
    totalJobs: number
  }
  dag: {
    blockCount: number
    syncStatus: 'synced' | 'syncing' | 'error'
  }
}

// CCL Template types for governance
export interface CCLTemplate {
  id: string
  name: string
  description: string
  category: 'membership' | 'governance' | 'economic' | 'technical'
  template: string
  parameters: CCLParameter[]
}

export interface CCLParameter {
  name: string
  type: 'string' | 'number' | 'boolean' | 'did' | 'duration'
  description: string
  required: boolean
  default?: any
  validation?: {
    min?: number
    max?: number
    pattern?: string
    options?: string[]
  }
} 