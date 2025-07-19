export interface ICNClientOptions {
  nodeEndpoint: string
  privateKey?: string
  storage?: StorageAdapter
  network?: 'mainnet' | 'testnet' | 'devnet'
  timeout?: number
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