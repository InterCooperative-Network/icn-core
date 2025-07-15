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