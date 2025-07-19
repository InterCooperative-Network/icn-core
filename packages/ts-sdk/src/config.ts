import { ConfigOptions } from './types'

export interface ICNConfig {
  defaultNodeEndpoint: string
  defaultNetwork: 'mainnet' | 'testnet' | 'devnet'
  storagePrefix: string
  endpoints: {
    mainnet: string
    testnet: string
    devnet: string
  }
}

export function createConfig(options: ConfigOptions = {}): ICNConfig {
  return {
    defaultNodeEndpoint: options.defaultNodeEndpoint || 'http://localhost:8080',
    defaultNetwork: options.defaultNetwork || 'devnet',
    storagePrefix: options.storagePrefix || '@icn:',
    endpoints: {
      mainnet: 'https://mainnet.icn.coop',
      testnet: 'https://testnet.icn.coop', 
      devnet: 'http://localhost:8080',
    }
  }
} 