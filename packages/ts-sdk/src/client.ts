import { ICNClient as BaseICNClient, ICNClientConfig } from '@icn/client-sdk'
import { ICNClientOptions, ICNConnectionState, JobSubmissionOptions } from './types'
import { ICNStorage, createStorage } from './storage'

export class ICNClient {
  private options: ICNClientOptions
  private storage: ICNStorage
  private connectionState: ICNConnectionState
  private baseClient: BaseICNClient

  constructor(options: ICNClientOptions) {
    this.options = options
    this.storage = new ICNStorage(
      options.storage || createStorage()
    )
    this.connectionState = {
      connected: false,
      nodeEndpoint: options.nodeEndpoint,
      network: options.network || 'devnet',
    }

    // Initialize the base client from @icn/client-sdk
    const clientConfig: ICNClientConfig = {
      baseUrl: options.nodeEndpoint,
      timeout: options.timeout || 30000,
    }
    this.baseClient = new BaseICNClient(clientConfig)
  }

  // Connection management
  async connect(): Promise<void> {
    try {
      // Test connection by getting node info
      await this.baseClient.system.getInfo()
      this.connectionState.connected = true
      console.log('Connected to ICN node:', this.options.nodeEndpoint)
    } catch (error) {
      this.connectionState.connected = false
      throw new Error(`Failed to connect: ${error}`)
    }
  }

  async disconnect(): Promise<void> {
    this.connectionState.connected = false
    console.log('Disconnected from ICN node')
  }

  // Job management
  async submitJob(
    jobSpec: any, 
    options: JobSubmissionOptions = {}
  ): Promise<string> {
    if (!this.connectionState.connected) {
      throw new Error('Not connected to ICN node')
    }

    const response = await this.baseClient.mesh.submitJob({
      job_spec: jobSpec,
      submitter_did: this.connectionState.did || '',
      max_cost: options.maxCost || 100,
      timeout_seconds: options.timeout,
    })
    return response.job_id
  }

  async getJob(jobId: string): Promise<any> {
    if (!this.connectionState.connected) {
      throw new Error('Not connected to ICN node')
    }

    return await this.baseClient.mesh.getJobStatus(jobId)
  }

  async listJobs(): Promise<any[]> {
    if (!this.connectionState.connected) {
      throw new Error('Not connected to ICN node')
    }

    return await this.baseClient.mesh.listJobs()
  }

  // Identity management
  async registerDid(didDocument: any): Promise<string> {
    if (!this.connectionState.connected) {
      throw new Error('Not connected to ICN node')
    }

    // This would require a specific endpoint for DID registration
    // For now, we'll generate a placeholder until the API is extended
    const did = 'did:key:' + Date.now().toString(36)
    console.log('Registered DID:', did, didDocument)
    return did
  }

  async resolveDid(did: string): Promise<any> {
    if (!this.connectionState.connected) {
      throw new Error('Not connected to ICN node')
    }

    // This would require a specific endpoint for DID resolution
    return { id: did, document: {} }
  }

  // Governance
  async submitProposal(proposal: any): Promise<string> {
    if (!this.connectionState.connected) {
      throw new Error('Not connected to ICN node')
    }

    return await this.baseClient.governance.submitProposal(proposal)
  }

  async vote(proposalId: string, vote: 'yes' | 'no'): Promise<void> {
    if (!this.connectionState.connected) {
      throw new Error('Not connected to ICN node')
    }

    await this.baseClient.governance.castVote({
      voter_did: this.connectionState.did || '',
      proposal_id: proposalId,
      vote_option: vote === 'yes' ? 'Yes' : 'No',
    })
  }

  // Federation management
  get federation() {
    return this.baseClient.federation
  }

  // Governance with full access
  get governance() {
    return this.baseClient.governance
  }

  // Identity with full access
  get identity() {
    return this.baseClient.identity
  }

  // Mesh computing with full access
  get mesh() {
    return this.baseClient.mesh
  }

  // Account management
  get account() {
    return this.baseClient.account
  }

  // Reputation
  get reputation() {
    return this.baseClient.reputation
  }

  // DAG storage
  get dag() {
    return this.baseClient.dag
  }

  // System information
  get system() {
    return this.baseClient.system
  }

  // Utilities
  getConnectionState(): ICNConnectionState {
    return { ...this.connectionState }
  }

  getStorage(): ICNStorage {
    return this.storage
  }

  getBaseClient(): BaseICNClient {
    return this.baseClient
  }
} 