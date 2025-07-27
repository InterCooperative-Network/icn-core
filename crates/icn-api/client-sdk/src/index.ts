// ICN API TypeScript Client SDK
// Auto-generated from icn-core/crates/icn-api

export interface ICNClientConfig {
  baseUrl: string;
  apiKey?: string;
  bearerToken?: string;
  timeout?: number;
}

export interface ErrorResponse {
  error: string;
  details?: any;
  correlation_id?: string;
}

// ============================================================================
// Governance API Types
// ============================================================================

export interface SubmitProposalRequest {
  proposer_did: string;
  proposal: ProposalInputType;
  description: string;
  duration_secs: number;
  quorum?: number;
  threshold?: number;
  body?: Uint8Array | null;
  credential_proof?: ZkCredentialProof | null;
  revocation_proof?: ZkRevocationProof | null;
}

export type ProposalInputType = 
  | { type: "SystemParameterChange"; data: { param: string; value: string } }
  | { type: "MemberAdmission"; data: { did: string } }
  | { type: "RemoveMember"; data: { did: string } }
  | { type: "SoftwareUpgrade"; data: { version: string } }
  | { type: "GenericText"; data: { text: string } }
  | { type: "Resolution"; data: { actions: ResolutionActionInput[] } };

export type ResolutionActionInput =
  | { action: "PauseCredential"; data: { cid: string } }
  | { action: "FreezeReputation"; data: { did: string } };

export interface CastVoteRequest {
  voter_did: string;
  proposal_id: string;
  vote_option: "Yes" | "No" | "Abstain";
  credential_proof?: ZkCredentialProof | null;
  revocation_proof?: ZkRevocationProof | null;
}

export interface DelegateRequest {
  from_did: string;
  to_did: string;
}

export interface RevokeDelegationRequest {
  from_did: string;
}

export interface Proposal {
  id: string;
  proposer: string;
  proposal_type: ProposalInputType;
  description: string;
  status: "Draft" | "Open" | "Closed" | "Executed";
  created_at: string; // ISO 8601 timestamp
  voting_deadline?: string; // ISO 8601 timestamp
  votes: {
    yes: number;
    no: number;
    abstain: number;
  };
  detailed_votes?: {
    voter: string;
    option: "Yes" | "No" | "Abstain";
    timestamp: string;
  }[];
  quorum?: number;
  threshold?: number;
}

// ============================================================================
// Identity API Types
// ============================================================================

export interface IssueCredentialRequest {
  issuer: string;
  holder: string;
  attributes: Record<string, string>;
  schema: string;
  expiration: number;
}

export interface CredentialResponse {
  cid: string;
  credential: VerifiableCredential;
}

export interface VerifiableCredential {
  issuer: string;
  holder: string;
  attributes: Record<string, string>;
  schema: string;
  expiration: number;
  signature: string;
}

export interface VerificationResponse {
  valid: boolean;
}

export interface GenerateProofRequest {
  issuer: string;
  holder: string;
  claim_type: string;
  schema: string;
  backend: string;
  public_inputs?: any;
}

export interface ProofResponse {
  proof: ZkCredentialProof;
}

export interface ZkCredentialProof {
  // ZK proof structure - implementation specific
  proof_data: Uint8Array;
  public_inputs: any[];
}

export interface ZkRevocationProof {
  // ZK revocation proof structure
  proof_data: Uint8Array;
  credential_cid: string;
}

export interface DisclosureRequest {
  credential: VerifiableCredential;
  fields: string[];
}

export interface DisclosureResponse {
  credential: DisclosedCredential;
  proof: ZkCredentialProof;
}

export interface DisclosedCredential {
  // Disclosed credential with selective revelation
  disclosed_attributes: Record<string, string>;
  proof: ZkCredentialProof;
}

// ============================================================================
// Federation API Types
// ============================================================================

export interface FederationPeerRequest {
  peer: string;
}

export interface FederationStatus {
  peer_count: number;
  peers: string[];
}

// ============================================================================
// Mesh Computing API Types
// ============================================================================

export interface MeshJobSubmitRequest {
  job_spec: JobSpecification;
  submitter_did: string;
  max_cost: number;
  timeout_seconds?: number;
}

export interface JobSpecification {
  image: string;
  command: string[];
  resources: ResourceRequirements;
  environment?: Record<string, string>;
}

export interface ResourceRequirements {
  cpu_cores: number;
  memory_mb: number;
  storage_mb: number;
}

export interface MeshJobResponse {
  job_id: string;
}

export interface JobStatus {
  id: string;
  submitter: string;
  status: "Pending" | "Running" | "Completed" | "Failed" | "Cancelled";
  submitted_at: string;
  started_at?: string;
  completed_at?: string;
  executor?: string;
  cost: number;
  progress?: number;
  result?: {
    output: string;
    exit_code: number;
  };
  error?: string;
}

// ============================================================================
// Account & Mana API Types
// ============================================================================

export interface ManaBalance {
  balance: number;
}

export interface AccountKeys {
  did: string;
  public_key_bs58: string;
}

// ============================================================================
// Token API Types
// ============================================================================

export interface CreateTokenClassRequest {
  id: string;
  name: string;
  symbol: string;
  decimals: number;
}

export interface TokenClass {
  id: string;
  name: string;
  symbol: string;
  decimals: number;
}

export interface MintTokensRequest {
  class_id: string;
  to_did: string;
  amount: number;
}

export interface BurnTokensRequest {
  class_id: string;
  from_did: string;
  amount: number;
}

export interface TransferTokensRequest {
  class_id: string;
  from_did: string;
  to_did: string;
  amount: number;
}

export interface TokenBalance {
  class_id: string;
  amount: number;
}

// ============================================================================
// Mutual Aid API Types
// ============================================================================

export interface AidResource {
  id: string;
  name: string;
  description: string;
  category: string;
  provider_did: string;
  availability: 'available' | 'unavailable' | 'limited';
  location?: string;
  contact_info?: string;
  metadata?: Record<string, any>;
  created_at: string;
  updated_at: string;
}

// ============================================================================
// Trust API Types
// ============================================================================

export type TrustLevel = 'none' | 'low' | 'medium' | 'high' | 'absolute';
export type TrustContext = 'general' | 'financial' | 'technical' | 'governance' | 'social';

export interface TrustRelationshipInfo {
  from: string; // DID
  to: string; // DID
  trust_level: TrustLevel;
  context: TrustContext;
  federation?: string;
  created_at: number;
  updated_at: number;
  metadata: Record<string, string>;
}

export interface TrustPath {
  from: string; // DID
  to: string; // DID
  path: string[]; // intermediate DIDs
  effective_trust: TrustLevel;
  contexts: TrustContext[];
  length: number;
  weight: number;
}

export interface TrustScore {
  did: string;
  score: number; // 0.0-1.0
  context_scores: Record<TrustContext, number>;
  incoming_trust_count: number;
  outgoing_trust_count: number;
  federations: string[];
  calculated_at: number;
}

export interface TrustGraphStats {
  total_entities: number;
  total_relationships: number;
  relationships_by_context: Record<TrustContext, number>;
  average_trust_score: number;
  trust_distribution: Record<string, number>;
  connected_components: number;
  federation_stats: Record<string, FederationTrustStats>;
  calculated_at: number;
}

export interface FederationTrustStats {
  member_count: number;
  average_internal_trust: number;
  active_contexts: TrustContext[];
  bridge_count: number;
}

export interface TrustQueryFilter {
  context?: TrustContext;
  min_trust_level?: TrustLevel;
  federation?: string;
  created_after?: number;
  created_before?: number;
  include_inherited?: boolean;
  include_cross_federation?: boolean;
}

export interface TrustPathRequest {
  from: string; // DID
  to: string; // DID
  context: TrustContext;
  max_length?: number;
  max_paths?: number;
  min_trust_level?: TrustLevel;
}

export interface TrustUpdateRequest {
  from: string; // DID
  to: string; // DID
  trust_level: TrustLevel;
  context: TrustContext;
  federation?: string;
  metadata?: Record<string, string>;
}

// ============================================================================
// Enhanced Credential API Types
// ============================================================================

export interface IssueCredentialRequest {
  credential_type: string;
  holder: string; // DID
  issuer: string; // DID
  claims: Record<string, any>;
  evidence?: string[];
  validity_period?: number;
}

export interface IssueCredentialResponse {
  credential_cid: string;
  credential_proof: ZkCredentialProof;
  issued_at: number;
  valid_until?: number;
}

export interface PresentCredentialRequest {
  credential_proof: ZkCredentialProof;
  context: string;
  disclosed_fields: string[];
  challenge?: string;
}

export interface PresentCredentialResponse {
  presentation_id: string;
  verification_result: CredentialVerificationResult;
  timestamp: number;
}

export interface VerifyCredentialRequest {
  presentation_id: string;
  verification_level: string;
  required_claims?: string[];
}

export interface CredentialVerificationResult {
  valid: boolean;
  verification_level: string;
  verified_claims: Record<string, any>;
  warnings: string[];
  errors: string[];
  trust_score?: number;
}

export interface AnchorDisclosureRequest {
  credential_cid: string;
  disclosed_fields: string[];
  presentation_context: string;
  verifier: string; // DID
  metadata?: Record<string, any>;
}

export interface AnchorDisclosureResponse {
  disclosure_cid: string;
  anchored_at: number;
  dag_block_cid: string;
}

export interface RevokeCredentialRequest {
  credential_cid: string;
  reason: string;
  revoked_by: string; // DID
}

export interface RevokeCredentialResponse {
  revoked: boolean;
  revocation_cid: string;
  revoked_at: number;
}

export interface ListCredentialsRequest {
  holder?: string; // DID
  issuer?: string; // DID
  credential_type?: string;
  status?: string;
  limit?: number;
  offset?: number;
}

export interface CredentialMetadata {
  cid: string;
  issuer: string; // DID
  holder: string; // DID
  credential_type: string;
  issued_at: number;
  valid_until?: number;
  status: string;
  revoked: boolean;
  presentation_count: number;
}

export interface ListCredentialsResponse {
  credentials: CredentialMetadata[];
  total_count: number;
  has_more: boolean;
}

export interface CredentialStatus {
  cid: string;
  issuer: string; // DID
  holder: string; // DID
  credential_type: string;
  issued_at: number;
  valid_until?: number;
  revoked: boolean;
  revoked_at?: number;
  revocation_reason?: string;
  presentations: PresentationInfo[];
  anchored_disclosures: string[];
  trust_attestations: TrustAttestationInfo[];
}

export interface PresentationInfo {
  presentation_id: string;
  context: string;
  presented_at: number;
  verifier?: string; // DID
  verification_result?: CredentialVerificationResult;
}

export interface TrustAttestationInfo {
  attestor: string; // DID
  trust_level: string;
  attested_at: number;
  context: string;
}

// ============================================================================
// Executor API Types
// ============================================================================

export interface ExecutorQueueInfo {
  queued: number;
  capacity: number;
}

// ============================================================================
// Reputation API Types
// ============================================================================

export interface ReputationScore {
  score: number;
  frozen?: boolean;
}

// ============================================================================
// DAG Storage API Types
// ============================================================================

export interface DagBlock {
  cid: string;
  data: Uint8Array;
  links: string[];
  author_did: string;
  scope: string;
  timestamp: number;
  signature: string;
}

export interface DagPutRequest {
  data: string; // base64 encoded
  links: string[];
  author_did: string;
  scope: string;
}

export interface DagGetRequest {
  cid: string;
}

// ============================================================================
// System Information API Types
// ============================================================================

export interface NodeInfo {
  name: string;
  version: string;
  did: string;
  capabilities: string[];
}

export interface NodeStatus {
  is_online: boolean;
  peer_count: number;
  current_block_height: number;
  version: string;
}

export interface HealthStatus {
  status: "healthy" | "unhealthy";
  details?: Record<string, any>;
}

// ============================================================================
// ICN Client SDK
// ============================================================================

export class ICNClient {
  private config: ICNClientConfig;
  private baseUrl: string;

  constructor(config: ICNClientConfig) {
    this.config = config;
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // Remove trailing slash
  }

  // Governance API
  get governance() {
    const client = this;
    return {
      async submitProposal(request: SubmitProposalRequest): Promise<string> {
        return client.post<string>('/governance/submit', request);
      },

      async castVote(request: CastVoteRequest): Promise<string> {
        return client.post<string>('/governance/vote', request);
      },

      async listProposals(): Promise<Proposal[]> {
        return client.get<Proposal[]>('/governance/proposals');
      },

      async getProposal(proposalId: string): Promise<Proposal> {
        return client.get<Proposal>(`/governance/proposal/${proposalId}`);
      },

      async delegateVote(request: DelegateRequest): Promise<string> {
        return client.post<string>('/governance/delegate', request);
      },

      async revokeDelegation(request: RevokeDelegationRequest): Promise<string> {
        return client.post<string>('/governance/revoke', request);
      }
    };
  }

  // Identity API
  get identity() {
    const client = this;
    return {
      async issueCredential(request: IssueCredentialRequest): Promise<CredentialResponse> {
        return client.post<CredentialResponse>('/identity/credentials/issue', request);
      },

      async verifyCredential(credential: VerifiableCredential): Promise<VerificationResponse> {
        return client.post<VerificationResponse>('/identity/credentials/verify', { credential });
      },

      async getCredential(cid: string): Promise<CredentialResponse> {
        return client.get<CredentialResponse>(`/identity/credentials/${cid}`);
      },

      async listSchemas(): Promise<string[]> {
        return client.get<string[]>('/identity/credentials/schemas');
      },

      async generateProof(request: GenerateProofRequest): Promise<ProofResponse> {
        return client.post<ProofResponse>('/identity/generate-proof', request);
      },

      async verifyProof(proof: ZkCredentialProof): Promise<VerificationResponse> {
        return client.post<VerificationResponse>('/identity/verify', proof);
      }
    };
  }

  // Federation API
  get federation() {
    const client = this;
    return {
      async listPeers(): Promise<string[]> {
        return client.get<string[]>('/federation/peers');
      },

      async joinFederation(request: FederationPeerRequest): Promise<void> {
        await client.post<void>('/federation/join', request);
      },

      async leaveFederation(request: FederationPeerRequest): Promise<void> {
        await client.post<void>('/federation/leave', request);
      },

      async getStatus(): Promise<FederationStatus> {
        return client.get<FederationStatus>('/federation/status');
      }
    };
  }

  // Mesh Computing API
  get mesh() {
    const client = this;
    return {
      async submitJob(request: MeshJobSubmitRequest): Promise<MeshJobResponse> {
        return client.post<MeshJobResponse>('/mesh/submit', request);
      },

      async listJobs(): Promise<JobStatus[]> {
        return client.get<JobStatus[]>('/mesh/jobs');
      },

      async getJobStatus(jobId: string): Promise<JobStatus> {
        return client.get<JobStatus>(`/mesh/jobs/${jobId}`);
      }
    };
  }

  // Account API
  get account() {
    const client = this;
    return {
      async getManaBalance(did: string): Promise<ManaBalance> {
        return client.get<ManaBalance>(`/account/${did}/mana`);
      },

      async getKeys(): Promise<AccountKeys> {
        return client.get<AccountKeys>('/keys');
      }
    };
  }

  // Reputation API
  get reputation() {
    const client = this;
    return {
      async getScore(did: string): Promise<ReputationScore> {
        return client.get<ReputationScore>(`/reputation/${did}`);
      }
    };
  }

  // DAG API
  get dag() {
    const client = this;
    return {
      async putBlock(request: DagPutRequest): Promise<string> {
        return client.post<string>('/dag/put', request);
      },

      async getBlock(request: DagGetRequest): Promise<DagBlock | null> {
        return client.post<DagBlock | null>('/dag/get', request);
      }
    };
  }

  // System API
  get system() {
    const client = this;
    return {
      async getInfo(): Promise<NodeInfo> {
        return client.get<NodeInfo>('/info');
      },

      async getStatus(): Promise<NodeStatus> {
        return client.get<NodeStatus>('/status');
      },

      async getHealth(): Promise<HealthStatus> {
        return client.get<HealthStatus>('/health');
      },

      async getMetrics(): Promise<string> {
        return client.get<string>('/metrics');
      }
    };
  }

  // Token API
  get tokens() {
    const client = this;
    return {
      async createTokenClass(request: CreateTokenClassRequest): Promise<TokenClass> {
        return client.post<TokenClass>('/tokens/classes', request);
      },

      async getTokenClass(id: string): Promise<TokenClass | null> {
        return client.get<TokenClass | null>(`/tokens/classes/${id}`);
      },

      async mintTokens(request: MintTokensRequest): Promise<void> {
        await client.post<void>('/tokens/mint', request);
      },

      async burnTokens(request: BurnTokensRequest): Promise<void> {
        await client.post<void>('/tokens/burn', request);
      },

      async transferTokens(request: TransferTokensRequest): Promise<void> {
        await client.post<void>('/tokens/transfer', request);
      },

      async listBalances(did: string): Promise<TokenBalance[]> {
        return client.get<TokenBalance[]>(`/tokens/balances/${did}`);
      }
    };
  }

  // Mutual Aid API
  get mutualAid() {
    const client = this;
    return {
      async listResources(): Promise<AidResource[]> {
        return client.get<AidResource[]>('/mutual-aid/resources');
      },

      async registerResource(resource: AidResource): Promise<void> {
        await client.post<void>('/mutual-aid/resources', resource);
      },

      async getResource(id: string): Promise<AidResource | null> {
        return client.get<AidResource | null>(`/mutual-aid/resources/${id}`);
      },

      async updateResource(id: string, resource: Partial<AidResource>): Promise<void> {
        await client.put<void>(`/mutual-aid/resources/${id}`, resource);
      },

      async removeResource(id: string): Promise<void> {
        await client.delete<void>(`/mutual-aid/resources/${id}`);
      }
    };
  }

  // Trust API
  get trust() {
    const client = this;
    return {
      async getTrustRelationship(
        from: string,
        to: string,
        context: TrustContext
      ): Promise<TrustRelationshipInfo | null> {
        return client.get<TrustRelationshipInfo | null>(
          `/trust/relationships?from=${from}&to=${to}&context=${context}`
        );
      },

      async getEntityTrustRelationships(
        entity: string,
        filter?: TrustQueryFilter
      ): Promise<TrustRelationshipInfo[]> {
        const params = new URLSearchParams({ entity });
        if (filter) {
          Object.entries(filter).forEach(([key, value]) => {
            if (value !== undefined) {
              params.append(key, String(value));
            }
          });
        }
        return client.get<TrustRelationshipInfo[]>(`/trust/relationships?${params}`);
      },

      async findTrustPaths(request: TrustPathRequest): Promise<TrustPath[]> {
        return client.post<TrustPath[]>('/trust/paths', request);
      },

      async getTrustScore(entity: string): Promise<TrustScore> {
        return client.get<TrustScore>(`/trust/scores/${entity}`);
      },

      async getTrustScores(entities: string[]): Promise<TrustScore[]> {
        return client.post<TrustScore[]>('/trust/scores', { entities });
      },

      async updateTrustRelationship(request: TrustUpdateRequest): Promise<void> {
        await client.post<void>('/trust/relationships', request);
      },

      async removeTrustRelationship(
        from: string,
        to: string,
        context: TrustContext
      ): Promise<void> {
        await client.delete<void>(
          `/trust/relationships?from=${from}&to=${to}&context=${context}`
        );
      },

      async getTrustGraphStats(): Promise<TrustGraphStats> {
        return client.get<TrustGraphStats>('/trust/stats');
      },

      async getFederationTrustStats(federation: string): Promise<FederationTrustStats> {
        return client.get<FederationTrustStats>(`/trust/federations/${federation}/stats`);
      },

      async searchByTrust(
        filter: TrustQueryFilter,
        limit?: number,
        offset?: number
      ): Promise<TrustScore[]> {
        const params = new URLSearchParams();
        Object.entries(filter).forEach(([key, value]) => {
          if (value !== undefined) {
            params.append(key, String(value));
          }
        });
        if (limit !== undefined) params.append('limit', String(limit));
        if (offset !== undefined) params.append('offset', String(offset));
        return client.get<TrustScore[]>(`/trust/search?${params}`);
      },

      async validateTrustOperation(
        actor: string,
        target: string,
        context: TrustContext,
        operation: string
      ): Promise<boolean> {
        const result = await client.post<{ valid: boolean }>('/trust/validate', {
          actor,
          target,
          context,
          operation
        });
        return result.valid;
      }
    };
  }

  // Enhanced Credential API
  get credentials() {
    const client = this;
    return {
      // Enhanced credential operations
      async issueCredential(request: IssueCredentialRequest): Promise<IssueCredentialResponse> {
        return client.post<IssueCredentialResponse>('/credentials/issue', request);
      },

      async presentCredential(request: PresentCredentialRequest): Promise<PresentCredentialResponse> {
        return client.post<PresentCredentialResponse>('/credentials/present', request);
      },

      async verifyCredential(request: VerifyCredentialRequest): Promise<CredentialVerificationResult> {
        return client.post<CredentialVerificationResult>('/credentials/verify', request);
      },

      async anchorDisclosure(request: AnchorDisclosureRequest): Promise<AnchorDisclosureResponse> {
        return client.post<AnchorDisclosureResponse>('/credentials/anchor', request);
      },

      async revokeCredential(request: RevokeCredentialRequest): Promise<RevokeCredentialResponse> {
        return client.post<RevokeCredentialResponse>('/credentials/revoke', request);
      },

      async getCredentialStatus(cid: string): Promise<CredentialStatus> {
        return client.get<CredentialStatus>(`/credentials/${cid}/status`);
      },

      async listCredentials(request: ListCredentialsRequest): Promise<ListCredentialsResponse> {
        const params = new URLSearchParams();
        Object.entries(request).forEach(([key, value]) => {
          if (value !== undefined) {
            params.append(key, String(value));
          }
        });
        return client.get<ListCredentialsResponse>(`/credentials?${params}`);
      },

      // Legacy identity API compatibility methods
      async issueCredentialLegacy(request: {
        issuer: string;
        holder: string;
        attributes: Record<string, string>;
        schema: string;
        expiration: number;
      }): Promise<{ cid: string; credential: VerifiableCredential }> {
        return client.post<{ cid: string; credential: VerifiableCredential }>('/identity/credentials/issue', request);
      },

      async verifyCredentialLegacy(credential: VerifiableCredential): Promise<{ valid: boolean }> {
        return client.post<{ valid: boolean }>('/identity/credentials/verify', { credential });
      },

      async getCredentialLegacy(cid: string): Promise<{ cid: string; credential: VerifiableCredential }> {
        return client.get<{ cid: string; credential: VerifiableCredential }>(`/identity/credentials/${cid}`);
      },

      async listSchemas(): Promise<string[]> {
        return client.get<string[]>('/identity/credentials/schemas');
      },

      async generateProof(request: {
        issuer: string;
        holder: string;
        claim_type: string;
        schema: string;
        backend: string;
        public_inputs?: any;
      }): Promise<{ proof: ZkCredentialProof }> {
        return client.post<{ proof: ZkCredentialProof }>('/identity/generate-proof', request);
      },

      async verifyProof(proof: ZkCredentialProof): Promise<{ valid: boolean }> {
        return client.post<{ valid: boolean }>('/identity/verify', proof);
      }
    };
  }

  // Executor API
  get executor() {
    const client = this;
    return {
      async getExecutorQueue(did: string): Promise<ExecutorQueueInfo> {
        return client.get<ExecutorQueueInfo>(`/executor/${did}/queue`);
      }
    };
  }

  // Private HTTP methods
  private async get<T>(path: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'GET',
      headers: this.getHeaders(),
      signal: this.getAbortSignal()
    });

    return this.handleResponse<T>(response);
  }

  private async post<T>(path: string, body?: any): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: body ? JSON.stringify(body) : undefined,
      signal: this.getAbortSignal()
    });

    return this.handleResponse<T>(response);
  }

  private async put<T>(path: string, body?: any): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'PUT',
      headers: this.getHeaders(),
      body: body ? JSON.stringify(body) : undefined,
      signal: this.getAbortSignal()
    });

    return this.handleResponse<T>(response);
  }

  private async delete<T>(path: string): Promise<T> {
    const response = await fetch(`${this.baseUrl}${path}`, {
      method: 'DELETE',
      headers: this.getHeaders(),
      signal: this.getAbortSignal()
    });

    return this.handleResponse<T>(response);
  }

  private getHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json'
    };

    if (this.config.apiKey) {
      headers['x-api-key'] = this.config.apiKey;
    }

    if (this.config.bearerToken) {
      headers['Authorization'] = `Bearer ${this.config.bearerToken}`;
    }

    return headers;
  }

  private getAbortSignal(): AbortSignal | undefined {
    if (this.config.timeout) {
      const controller = new AbortController();
      setTimeout(() => controller.abort(), this.config.timeout);
      return controller.signal;
    }
    return undefined;
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const errorText = await response.text();
      let errorData: ErrorResponse;
      
      try {
        errorData = JSON.parse(errorText);
      } catch {
        errorData = { error: errorText };
      }

      throw new ICNApiError(
        response.status,
        errorData.error || 'Unknown error',
        errorData
      );
    }

    // Handle empty responses
    if (response.status === 204 || response.headers.get('content-length') === '0') {
      return undefined as any;
    }

    const text = await response.text();
    if (!text) {
      return undefined as any;
    }

    try {
      return JSON.parse(text);
    } catch {
      // If response is not JSON, return as string
      return text as any;
    }
  }
}

export class ICNApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public details?: ErrorResponse
  ) {
    super(message);
    this.name = 'ICNApiError';
  }
}

// WebSocket Client (planned)
export class ICNWebSocketClient {
  private ws?: WebSocket;
  private config: ICNClientConfig;
  private listeners: Map<string, Set<(data: any) => void>> = new Map();

  constructor(config: ICNClientConfig) {
    this.config = config;
  }

  async connect(): Promise<void> {
    const wsUrl = this.config.baseUrl
      .replace('http://', 'ws://')
      .replace('https://', 'wss://') + '/ws';

    this.ws = new WebSocket(wsUrl);
    
    return new Promise((resolve, reject) => {
      if (!this.ws) return reject(new Error('WebSocket not initialized'));

      this.ws.onopen = () => resolve();
      this.ws.onerror = (error) => reject(error);
      this.ws.onmessage = (event) => this.handleMessage(event);
    });
  }

  subscribe(eventType: string, callback: (data: any) => void): void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }
    this.listeners.get(eventType)!.add(callback);
  }

  unsubscribe(eventType: string, callback: (data: any) => void): void {
    const listeners = this.listeners.get(eventType);
    if (listeners) {
      listeners.delete(callback);
    }
  }

  private handleMessage(event: MessageEvent): void {
    try {
      const message = JSON.parse(event.data);
      const listeners = this.listeners.get(message.type);
      if (listeners) {
        listeners.forEach(callback => callback(message.data));
      }
    } catch (error) {
      console.error('Error parsing WebSocket message:', error);
    }
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = undefined;
    }
  }
}

// Utility functions
export const ICNUtils = {
  /**
   * Validate DID format
   */
  isValidDid(did: string): boolean {
    return /^did:[a-z0-9]+:[a-zA-Z0-9._-]+$/.test(did);
  },

  /**
   * Format mana balance for display
   */
  formatMana(balance: number): string {
    if (balance >= 1000000) {
      return `${(balance / 1000000).toFixed(1)}M`;
    } else if (balance >= 1000) {
      return `${(balance / 1000).toFixed(1)}K`;
    } else {
      return balance.toString();
    }
  },

  /**
   * Calculate time remaining for proposal voting
   */
  getTimeRemaining(deadline: string): string {
    const now = new Date();
    const end = new Date(deadline);
    const diff = end.getTime() - now.getTime();

    if (diff <= 0) return 'Expired';

    const days = Math.floor(diff / (1000 * 60 * 60 * 24));
    const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));

    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${minutes}m`;
    return `${minutes}m`;
  }
};

export default ICNClient; 