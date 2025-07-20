//! TypeScript Client SDK Generation
//!
//! This module provides utilities to generate TypeScript type definitions
//! and client SDK code for the ICN HTTP API endpoints.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// TypeScript type definition generator for ICN API
pub struct TypeScriptGenerator;

impl TypeScriptGenerator {
    /// Generate TypeScript interfaces for all API request/response types
    pub fn generate_types() -> String {
        format!(
            r#"// ICN API TypeScript Definitions
// Auto-generated from icn-core/crates/icn-api

export interface ICNClientConfig {{
  baseUrl: string;
  apiKey?: string;
  bearerToken?: string;
  timeout?: number;
}}

export interface ErrorResponse {{
  error: string;
  details?: any;
  correlation_id?: string;
}}

// ============================================================================
// Governance API Types
// ============================================================================

export interface SubmitProposalRequest {{
  proposer_did: string;
  proposal: ProposalInputType;
  description: string;
  duration_secs: number;
  quorum?: number;
  threshold?: number;
  body?: Uint8Array | null;
  credential_proof?: ZkCredentialProof | null;
  revocation_proof?: ZkRevocationProof | null;
}}

export type ProposalInputType = 
  | {{ type: "SystemParameterChange"; data: {{ param: string; value: string }} }}
  | {{ type: "MemberAdmission"; data: {{ did: string }} }}
  | {{ type: "RemoveMember"; data: {{ did: string }} }}
  | {{ type: "SoftwareUpgrade"; data: {{ version: string }} }}
  | {{ type: "GenericText"; data: {{ text: string }} }}
  | {{ type: "Resolution"; data: {{ actions: ResolutionActionInput[] }} }};

export type ResolutionActionInput =
  | {{ action: "PauseCredential"; data: {{ cid: string }} }}
  | {{ action: "FreezeReputation"; data: {{ did: string }} }};

export interface CastVoteRequest {{
  voter_did: string;
  proposal_id: string;
  vote_option: "Yes" | "No" | "Abstain";
  credential_proof?: ZkCredentialProof | null;
  revocation_proof?: ZkRevocationProof | null;
}}

export interface DelegateRequest {{
  from_did: string;
  to_did: string;
}}

export interface RevokeDelegationRequest {{
  from_did: string;
}}

export interface Proposal {{
  id: string;
  proposer: string;
  proposal_type: ProposalInputType;
  description: string;
  status: "Draft" | "Open" | "Closed" | "Executed";
  created_at: string; // ISO 8601 timestamp
  voting_deadline?: string; // ISO 8601 timestamp
  votes: {{
    yes: number;
    no: number;
    abstain: number;
  }};
  detailed_votes?: {{
    voter: string;
    option: "Yes" | "No" | "Abstain";
    timestamp: string;
  }}[];
  quorum?: number;
  threshold?: number;
}}

// ============================================================================
// Identity API Types
// ============================================================================

export interface IssueCredentialRequest {{
  issuer: string;
  holder: string;
  attributes: Record<string, string>;
  schema: string;
  expiration: number;
}}

export interface CredentialResponse {{
  cid: string;
  credential: VerifiableCredential;
}}

export interface VerifiableCredential {{
  issuer: string;
  holder: string;
  attributes: Record<string, string>;
  schema: string;
  expiration: number;
  signature: string;
}}

export interface VerificationResponse {{
  valid: boolean;
}}

export interface GenerateProofRequest {{
  issuer: string;
  holder: string;
  claim_type: string;
  schema: string;
  backend: string;
  public_inputs?: any;
}}

export interface ProofResponse {{
  proof: ZkCredentialProof;
}}

export interface ZkCredentialProof {{
  // ZK proof structure - implementation specific
  proof_data: Uint8Array;
  public_inputs: any[];
}}

export interface ZkRevocationProof {{
  // ZK revocation proof structure
  proof_data: Uint8Array;
  credential_cid: string;
}}

export interface DisclosureRequest {{
  credential: VerifiableCredential;
  fields: string[];
}}

export interface DisclosureResponse {{
  credential: DisclosedCredential;
  proof: ZkCredentialProof;
}}

export interface DisclosedCredential {{
  // Disclosed credential with selective revelation
  disclosed_attributes: Record<string, string>;
  proof: ZkCredentialProof;
}}

// ============================================================================
// Federation API Types
// ============================================================================

export interface FederationPeerRequest {{
  peer: string;
}}

export interface FederationStatus {{
  peer_count: number;
  peers: string[];
}}

// ============================================================================
// Mesh Computing API Types
// ============================================================================

export interface MeshJobSubmitRequest {{
  job_spec: JobSpecification;
  submitter_did: string;
  max_cost: number;
  timeout_seconds?: number;
}}

export interface JobSpecification {{
  image: string;
  command: string[];
  resources: ResourceRequirements;
  environment?: Record<string, string>;
}}

export interface ResourceRequirements {{
  cpu_cores: number;
  memory_mb: number;
  storage_mb: number;
}}

export interface MeshJobResponse {{
  job_id: string;
}}

export interface JobStatus {{
  id: string;
  submitter: string;
  status: "Pending" | "Running" | "Completed" | "Failed" | "Cancelled";
  submitted_at: string;
  started_at?: string;
  completed_at?: string;
  executor?: string;
  cost: number;
  progress?: number;
  result?: {{
    output: string;
    exit_code: number;
  }};
  error?: string;
}}

// ============================================================================
// Account & Mana API Types
// ============================================================================

export interface ManaBalance {{
  balance: number;
}}

export interface AccountKeys {{
  did: string;
  public_key_bs58: string;
}}

// ============================================================================
// Reputation API Types
// ============================================================================

export interface ReputationScore {{
  score: number;
  frozen?: boolean;
}}

// ============================================================================
// DAG Storage API Types
// ============================================================================

export interface DagBlock {{
  cid: string;
  data: Uint8Array;
  links: string[];
  author_did: string;
  scope: string;
  timestamp: number;
  signature: string;
}}

export interface DagPutRequest {{
  data: string; // base64 encoded
  links: string[];
  author_did: string;
  scope: string;
}}

export interface DagGetRequest {{
  cid: string;
}}

// ============================================================================
// System Information API Types
// ============================================================================

export interface NodeInfo {{
  name: string;
  version: string;
  did: string;
  capabilities: string[];
}}

export interface NodeStatus {{
  is_online: boolean;
  peer_count: number;
  current_block_height: number;
  version: string;
}}

export interface HealthStatus {{
  status: "healthy" | "unhealthy";
  details?: Record<string, any>;
}}

// ============================================================================
// Contracts & Circuits API Types
// ============================================================================

export interface ContractRequest {{
  source_code: string;
  schema?: string;
}}

export interface ContractResponse {{
  contract_cid: string;
  wasm_cid: string;
}}

export interface CircuitRegisterRequest {{
  slug: string;
  version: string;
  circuit_data: Uint8Array;
  description?: string;
}}

export interface CircuitResponse {{
  slug: string;
  version: string;
  circuit_cid: string;
}}

// ============================================================================
// Cooperative Management API Types
// ============================================================================

export interface CooperativeProfile {{
  did: string;
  name: string;
  description?: string;
  capabilities: string[];
  trust_relationships: TrustRelationship[];
  metadata: Record<string, any>;
}}

export interface TrustRelationship {{
  target_did: string;
  trust_level: number;
  trust_type: string;
  created_at: string;
}}

export interface CooperativeSearchRequest {{
  query?: string;
  capabilities?: string[];
  region?: string;
  trust_level?: number;
}}

export interface CooperativeRegisterRequest {{
  profile: CooperativeProfile;
}}

export interface AddTrustRequest {{
  target_did: string;
  trust_level: number;
  trust_type: string;
}}

export interface RegistryStats {{
  total_cooperatives: number;
  total_trust_relationships: number;
  active_capabilities: string[];
}}

// ============================================================================
// Resource Management API Types
// ============================================================================

export interface ResourceEventRequest {{
  resource_id: string;
  action: "acquire" | "consume";
  scope?: string;
  mana_cost?: number;
}}

export interface ResourceLedgerEntry {{
  resource_id: string;
  action: "acquire" | "consume";
  scope?: string;
  mana_cost: number;
  timestamp: string;
}}

// ============================================================================
// Transaction API Types
// ============================================================================

export interface Transaction {{
  id: string;
  from: string;
  to: string;
  amount: number;
  data?: Uint8Array;
  signature: string;
}}

export interface TransactionResponse {{
  tx_id: string;
}}

export interface DataQueryRequest {{
  cid: string;
}}

// ============================================================================
// Advanced DAG Operations
// ============================================================================

export interface DagPinRequest {{
  cid: string;
  ttl?: number;
}}

export interface DagPruneRequest {{
  max_age_seconds?: number;
  max_blocks?: number;
}}

export interface SyncStatus {{
  is_syncing: boolean;
  peer_count: number;
  blocks_synced: number;
  total_blocks: number;
}}

// ============================================================================
// Advanced Mesh Operations
// ============================================================================

export interface JobProgress {{
  job_id: string;
  progress: number;
  stage: string;
  estimated_completion?: string;
}}

export interface JobStream {{
  job_id: string;
  stream_type: "stdout" | "stderr";
  data: string;
  timestamp: string;
}}

export interface MeshMetrics {{
  total_jobs: number;
  pending_jobs: number;
  running_jobs: number;
  completed_jobs: number;
  failed_jobs: number;
  average_execution_time: number;
}}

// ============================================================================
// ICN Client SDK
// ============================================================================

export class ICNClient {{
  private config: ICNClientConfig;
  private baseUrl: string;

  constructor(config: ICNClientConfig) {{
    this.config = config;
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // Remove trailing slash
  }}

  // Governance API
  governance = {{
    async submitProposal(request: SubmitProposalRequest): Promise<string> {{
      return this.post<string>('/governance/submit', request);
    }},

    async castVote(request: CastVoteRequest): Promise<string> {{
      return this.post<string>('/governance/vote', request);
    }},

    async listProposals(): Promise<Proposal[]> {{
      return this.get<Proposal[]>('/governance/proposals');
    }},

    async getProposal(proposalId: string): Promise<Proposal> {{
      return this.get<Proposal>(`/governance/proposal/${{proposalId}}`);
    }},

    async delegateVote(request: DelegateRequest): Promise<string> {{
      return this.post<string>('/governance/delegate', request);
    }},

    async revokeDelegation(request: RevokeDelegationRequest): Promise<string> {{
      return this.post<string>('/governance/revoke', request);
    }},

    async closeProposal(proposalId: string): Promise<string> {{
      return this.post<string>('/governance/close', {{ proposal_id: proposalId }});
    }},

    async executeProposal(proposalId: string): Promise<string> {{
      return this.post<string>('/governance/execute', {{ proposal_id: proposalId }});
    }}
  }};

  // Identity API
  identity = {{
    async issueCredential(request: IssueCredentialRequest): Promise<CredentialResponse> {{
      return this.post<CredentialResponse>('/identity/credentials/issue', request);
    }},

    async verifyCredential(credential: VerifiableCredential): Promise<VerificationResponse> {{
      return this.post<VerificationResponse>('/identity/credentials/verify', {{ credential }});
    }},

    async getCredential(cid: string): Promise<CredentialResponse> {{
      return this.get<CredentialResponse>(`/identity/credentials/${{cid}}`);
    }},

    async listSchemas(): Promise<string[]> {{
      return this.get<string[]>('/identity/credentials/schemas');
    }},

    async generateProof(request: GenerateProofRequest): Promise<ProofResponse> {{
      return this.post<ProofResponse>('/identity/generate-proof', request);
    }},

    async verifyProof(proof: ZkCredentialProof): Promise<VerificationResponse> {{
      return this.post<VerificationResponse>('/identity/verify', proof);
    }}
  }};

  // Federation API
  federation = {{
    async listPeers(): Promise<string[]> {{
      return this.get<string[]>('/federation/peers');
    }},

    async joinFederation(request: FederationPeerRequest): Promise<void> {{
      await this.post<void>('/federation/join', request);
    }},

    async leaveFederation(request: FederationPeerRequest): Promise<void> {{
      await this.post<void>('/federation/leave', request);
    }},

    async getStatus(): Promise<FederationStatus> {{
      return this.get<FederationStatus>('/federation/status');
    }}
  }};

  // Mesh Computing API
  mesh = {{
    async submitJob(request: MeshJobSubmitRequest): Promise<MeshJobResponse> {{
      return this.post<MeshJobResponse>('/mesh/submit', request);
    }},

    async listJobs(): Promise<JobStatus[]> {{
      return this.get<JobStatus[]>('/mesh/jobs');
    }},

    async getJobStatus(jobId: string): Promise<JobStatus> {{
      return this.get<JobStatus>(`/mesh/jobs/${{jobId}}`);
    }}
  }};

  // Account API
  account = {{
    async getManaBalance(did: string): Promise<ManaBalance> {{
      return this.get<ManaBalance>(`/account/${{did}}/mana`);
    }},

    async getKeys(): Promise<AccountKeys> {{
      return this.get<AccountKeys>('/keys');
    }}
  }};

  // Reputation API
  reputation = {{
    async getScore(did: string): Promise<ReputationScore> {{
      return this.get<ReputationScore>(`/reputation/${{did}}`);
    }}
  }};

  // DAG API
  dag = {{
    async putBlock(request: DagPutRequest): Promise<string> {{
      return this.post<string>('/dag/put', request);
    }},

    async getBlock(request: DagGetRequest): Promise<DagBlock | null> {{
      return this.post<DagBlock | null>('/dag/get', request);
    }}
  }};

  // System API
  system = {{
    async getInfo(): Promise<NodeInfo> {{
      return this.get<NodeInfo>('/info');
    }},

    async getStatus(): Promise<NodeStatus> {{
      return this.get<NodeStatus>('/status');
    }},

    async getHealth(): Promise<HealthStatus> {{
      return this.get<HealthStatus>('/health');
    }},

    async getMetrics(): Promise<string> {{
      return this.get<string>('/metrics');
    }}
  }};

  // Contracts & Circuits API
  contracts = {{
    async compileContract(request: ContractRequest): Promise<ContractResponse> {{
      return this.post<ContractResponse>('/contracts', request);
    }},

    async registerCircuit(request: CircuitRegisterRequest): Promise<CircuitResponse> {{
      return this.post<CircuitResponse>('/circuits/register', request);
    }},

    async getCircuit(slug: string, version: string): Promise<CircuitResponse> {{
      return this.get<CircuitResponse>(`/circuits/${{slug}}/${{version}}`);
    }},

    async getCircuitVersions(slug: string): Promise<string[]> {{
      return this.get<string[]>(`/circuits/${{slug}}`);
    }}
  }};

  // Cooperative Management API
  cooperative = {{
    async register(request: CooperativeRegisterRequest): Promise<string> {{
      return this.post<string>('/cooperative/register', request);
    }},

    async search(request: CooperativeSearchRequest): Promise<CooperativeProfile[]> {{
      return this.post<CooperativeProfile[]>('/cooperative/search', request);
    }},

    async getProfile(did: string): Promise<CooperativeProfile> {{
      return this.get<CooperativeProfile>(`/cooperative/profile/${{did}}`);
    }},

    async addTrust(request: AddTrustRequest): Promise<string> {{
      return this.post<string>('/cooperative/trust', request);
    }},

    async getTrust(did: string): Promise<TrustRelationship[]> {{
      return this.get<TrustRelationship[]>(`/cooperative/trust/${{did}}`);
    }},

    async getCapabilityProviders(capabilityType: string): Promise<CooperativeProfile[]> {{
      return this.get<CooperativeProfile[]>(`/cooperative/capabilities/${{capabilityType}}`);
    }},

    async getRegistryStats(): Promise<RegistryStats> {{
      return this.get<RegistryStats>('/cooperative/registry/stats');
    }}
  }};

  // Resource Management API
  resources = {{
    async recordEvent(request: ResourceEventRequest): Promise<string> {{
      return this.post<string>('/resources/event', request);
    }},

    async getLedger(): Promise<ResourceLedgerEntry[]> {{
      return this.get<ResourceLedgerEntry[]>('/resources/ledger');
    }}
  }};

  // Transaction API
  transactions = {{
    async submit(transaction: Transaction): Promise<TransactionResponse> {{
      return this.post<TransactionResponse>('/transaction/submit', transaction);
    }},

    async queryData(request: DataQueryRequest): Promise<DagBlock | null> {{
      return this.post<DagBlock | null>('/data/query', request);
    }}
  }};

  // Advanced DAG Operations
  dagAdvanced = {{
    async pin(request: DagPinRequest): Promise<string> {{
      return this.post<string>('/dag/pin', request);
    }},

    async unpin(cid: string): Promise<string> {{
      return this.post<string>('/dag/unpin', {{ cid }});
    }},

    async prune(request: DagPruneRequest): Promise<string> {{
      return this.post<string>('/dag/prune', request);
    }},

    async getRoot(): Promise<string> {{
      return this.get<string>('/dag/root');
    }},

    async getStatus(): Promise<any> {{
      return this.get<any>('/dag/status');
    }},

    async getSyncStatus(): Promise<SyncStatus> {{
      return this.get<SyncStatus>('/sync/status');
    }}
  }};

  // Advanced Mesh Operations
  meshAdvanced = {{
    async getJobProgress(jobId: string): Promise<JobProgress> {{
      return this.get<JobProgress>(`/mesh/jobs/${{jobId}}/progress`);
    }},

    async getJobStream(jobId: string): Promise<JobStream[]> {{
      return this.get<JobStream[]>(`/mesh/jobs/${{jobId}}/stream`);
    }},

    async cancelJob(jobId: string): Promise<string> {{
      return this.post<string>(`/mesh/jobs/${{jobId}}/cancel`, {{}});
    }},

    async resumeJob(jobId: string): Promise<string> {{
      return this.post<string>(`/mesh/jobs/${{jobId}}/resume`, {{}});
    }},

    async getMetrics(): Promise<MeshMetrics> {{
      return this.get<MeshMetrics>('/mesh/metrics');
    }},

    async submitReceipt(receipt: any): Promise<string> {{
      return this.post<string>('/mesh/receipt', receipt);
    }}
  }};

  // Network API
  network = {{
    async getLocalPeerId(): Promise<string> {{
      const response = await this.get<{{ peer_id: string }}>('/network/local-peer-id');
      return response.peer_id;
    }},

    async getPeers(): Promise<string[]> {{
      return this.get<string[]>('/network/peers');
    }},

    async connect(address: string): Promise<string> {{
      return this.post<string>('/network/connect', {{ address }});
    }}
  }};

  // Private HTTP methods
  private async get<T>(path: string): Promise<T> {{
    const response = await fetch(`${{this.baseUrl}}${{path}}`, {{
      method: 'GET',
      headers: this.getHeaders(),
      signal: this.getAbortSignal()
    }});

    return this.handleResponse<T>(response);
  }}

  private async post<T>(path: string, body?: any): Promise<T> {{
    const response = await fetch(`${{this.baseUrl}}${{path}}`, {{
      method: 'POST',
      headers: this.getHeaders(),
      body: body ? JSON.stringify(body) : undefined,
      signal: this.getAbortSignal()
    }});

    return this.handleResponse<T>(response);
  }}

  private getHeaders(): Record<string, string> {{
    const headers: Record<string, string> = {{
      'Content-Type': 'application/json'
    }};

    if (this.config.apiKey) {{
      headers['x-api-key'] = this.config.apiKey;
    }}

    if (this.config.bearerToken) {{
      headers['Authorization'] = `Bearer ${{this.config.bearerToken}}`;
    }}

    return headers;
  }}

  private getAbortSignal(): AbortSignal | undefined {{
    if (this.config.timeout) {{
      const controller = new AbortController();
      setTimeout(() => controller.abort(), this.config.timeout);
      return controller.signal;
    }}
    return undefined;
  }}

  private async handleResponse<T>(response: Response): Promise<T> {{
    if (!response.ok) {{
      const errorText = await response.text();
      let errorData: ErrorResponse;
      
      try {{
        errorData = JSON.parse(errorText);
      }} catch {{
        errorData = {{ error: errorText }};
      }}

      throw new ICNApiError(
        response.status,
        errorData.error || 'Unknown error',
        errorData
      );
    }}

    // Handle empty responses
    if (response.status === 204 || response.headers.get('content-length') === '0') {{
      return undefined as any;
    }}

    const text = await response.text();
    if (!text) {{
      return undefined as any;
    }}

    try {{
      return JSON.parse(text);
    }} catch {{
      // If response is not JSON, return as string
      return text as any;
    }}
  }}
}}

export class ICNApiError extends Error {{
  constructor(
    public status: number,
    message: string,
    public details?: ErrorResponse
  ) {{
    super(message);
    this.name = 'ICNApiError';
  }}
}}

// WebSocket Client (planned)
export class ICNWebSocketClient {{
  private ws?: WebSocket;
  private config: ICNClientConfig;
  private listeners: Map<string, Set<(data: any) => void>> = new Map();

  constructor(config: ICNClientConfig) {{
    this.config = config;
  }}

  async connect(): Promise<void> {{
    const wsUrl = this.config.baseUrl
      .replace('http://', 'ws://')
      .replace('https://', 'wss://') + '/ws';

    this.ws = new WebSocket(wsUrl);
    
    return new Promise((resolve, reject) => {{
      if (!this.ws) return reject(new Error('WebSocket not initialized'));

      this.ws.onopen = () => resolve();
      this.ws.onerror = (error) => reject(error);
      this.ws.onmessage = (event) => this.handleMessage(event);
    }});
  }}

  subscribe(eventType: string, callback: (data: any) => void): void {{
    if (!this.listeners.has(eventType)) {{
      this.listeners.set(eventType, new Set());
    }}
    this.listeners.get(eventType)!.add(callback);
  }}

  unsubscribe(eventType: string, callback: (data: any) => void): void {{
    const listeners = this.listeners.get(eventType);
    if (listeners) {{
      listeners.delete(callback);
    }}
  }}

  private handleMessage(event: MessageEvent): void {{
    try {{
      const message = JSON.parse(event.data);
      const listeners = this.listeners.get(message.type);
      if (listeners) {{
        listeners.forEach(callback => callback(message.data));
      }}
    }} catch (error) {{
      console.error('Error parsing WebSocket message:', error);
    }}
  }}

  disconnect(): void {{
    if (this.ws) {{
      this.ws.close();
      this.ws = undefined;
    }}
  }}
}}

// Utility functions
export const ICNUtils = {{
  /**
   * Validate DID format
   */
  isValidDid(did: string): boolean {{
    return /^did:[a-z0-9]+:[a-zA-Z0-9._-]+$/.test(did);
  }},

  /**
   * Format mana balance for display
   */
  formatMana(balance: number): string {{
    if (balance >= 1000000) {{
      return `${{(balance / 1000000).toFixed(1)}}M`;
    }} else if (balance >= 1000) {{
      return `${{(balance / 1000).toFixed(1)}}K`;
    }} else {{
      return balance.toString();
    }}
  }},

  /**
   * Calculate time remaining for proposal voting
   */
  getTimeRemaining(deadline: string): string {{
    const now = new Date();
    const end = new Date(deadline);
    const diff = end.getTime() - now.getTime();

    if (diff <= 0) return 'Expired';

    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

    if (days > 0) return `${{days}}d ${{hours}}h`;
    if (hours > 0) return `${{hours}}h ${{minutes}}m`;
    return `${{minutes}}m`;
  }}
}};

export default ICNClient;"#
        )
    }

    /// Generate package.json for the TypeScript SDK
    pub fn generate_package_json() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "name": "@icn/client-sdk",
            "version": "0.1.0",
            "description": "TypeScript client SDK for InterCooperative Network (ICN) APIs",
            "main": "dist/index.js",
            "types": "dist/index.d.ts",
            "files": [
                "dist/**/*",
                "README.md"
            ],
            "scripts": {
                "build": "tsc",
                "dev": "tsc --watch",
                "test": "jest",
                "lint": "eslint src/**/*.ts",
                "format": "prettier --write src/**/*.ts"
            },
            "keywords": [
                "icn",
                "intercooperative",
                "network",
                "api",
                "client",
                "sdk",
                "governance",
                "mesh",
                "federation",
                "did",
                "credentials"
            ],
            "author": "ICN Core Contributors",
            "license": "Apache-2.0",
            "devDependencies": {
                "@types/jest": "^29.0.0",
                "@typescript-eslint/eslint-plugin": "^6.0.0",
                "@typescript-eslint/parser": "^6.0.0",
                "eslint": "^8.0.0",
                "jest": "^29.0.0",
                "prettier": "^3.0.0",
                "typescript": "^5.0.0"
            },
            "peerDependencies": {
                "typescript": ">=4.0.0"
            },
            "repository": {
                "type": "git",
                "url": "https://github.com/InterCooperative-Network/icn-core.git",
                "directory": "crates/icn-api/client-sdk"
            },
            "bugs": {
                "url": "https://github.com/InterCooperative-Network/icn-core/issues"
            },
            "homepage": "https://intercooperative.network"
        }))
        .unwrap()
    }

    /// Generate TypeScript configuration
    pub fn generate_tsconfig() -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "compilerOptions": {
                "target": "ES2020",
                "module": "commonjs",
                "lib": ["ES2020", "DOM"],
                "declaration": true,
                "outDir": "./dist",
                "rootDir": "./src",
                "strict": true,
                "esModuleInterop": true,
                "skipLibCheck": true,
                "forceConsistentCasingInFileNames": true,
                "moduleResolution": "node",
                "resolveJsonModule": true,
                "allowSyntheticDefaultImports": true
            },
            "include": [
                "src/**/*"
            ],
            "exclude": [
                "node_modules",
                "dist",
                "**/*.test.ts"
            ]
        }))
        .unwrap()
    }

    /// Generate README for the TypeScript SDK
    pub fn generate_readme() -> String {
        r#"# ICN TypeScript Client SDK

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
"#.to_string()
    }
}

/// Utilities for generating client SDK files
pub mod sdk_files {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Generate all TypeScript SDK files in the specified directory
    pub fn generate_sdk_files(output_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        // Create directories
        let src_dir = output_dir.join("src");
        fs::create_dir_all(&src_dir)?;

        // Generate main TypeScript file
        let ts_content = TypeScriptGenerator::generate_types();
        fs::write(src_dir.join("index.ts"), ts_content)?;

        // Generate package.json
        let package_json = TypeScriptGenerator::generate_package_json();
        fs::write(output_dir.join("package.json"), package_json)?;

        // Generate tsconfig.json
        let tsconfig = TypeScriptGenerator::generate_tsconfig();
        fs::write(output_dir.join("tsconfig.json"), tsconfig)?;

        // Generate README.md
        let readme = TypeScriptGenerator::generate_readme();
        fs::write(output_dir.join("README.md"), readme)?;

        // Generate .gitignore
        let gitignore = r#"node_modules/
dist/
*.log
.DS_Store
*.tsbuildinfo
"#;
        fs::write(output_dir.join(".gitignore"), gitignore)?;

        // Generate .npmignore
        let npmignore = r#"src/
tsconfig.json
*.log
.DS_Store
node_modules/
"#;
        fs::write(output_dir.join(".npmignore"), npmignore)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typescript_generation() {
        let ts_content = TypeScriptGenerator::generate_types();
        assert!(ts_content.contains("export interface SubmitProposalRequest"));
        assert!(ts_content.contains("export class ICNClient"));
        assert!(ts_content.contains("governance ="));
        assert!(ts_content.contains("identity ="));
        assert!(ts_content.contains("federation ="));
    }

    #[test]
    fn test_package_json_generation() {
        let package_json = TypeScriptGenerator::generate_package_json();
        assert!(package_json.contains("@icn/client-sdk"));
        assert!(package_json.contains("typescript"));
    }
}
