// DID validation
export function validateDid(did: string): boolean {
  if (!did || typeof did !== 'string') return false
  
  // Basic DID format validation
  const didRegex = /^did:[a-z0-9]+:[a-zA-Z0-9._-]+$/
  return didRegex.test(did)
}

// Mana formatting
export function formatMana(amount: number): string {
  if (amount >= 1000000) {
    return `${(amount / 1000000).toFixed(1)}M`
  } else if (amount >= 1000) {
    return `${(amount / 1000).toFixed(1)}K`
  }
  return amount.toString()
}

// Job ID formatting
export function formatJobId(jobId: string): string {
  if (!jobId) return ''
  
  // Show first 8 and last 4 characters for readability
  if (jobId.length > 12) {
    return `${jobId.slice(0, 8)}...${jobId.slice(-4)}`
  }
  return jobId
}

// Address validation
export function validateAddress(address: string): boolean {
  if (!address || typeof address !== 'string') return false
  
  // Basic address format validation (adjust based on ICN address format)
  return address.length >= 10 && address.length <= 100
}

// Network validation
export function validateNetwork(network: string): boolean {
  const validNetworks = ['mainnet', 'testnet', 'devnet']
  return validNetworks.includes(network)
}

// Error formatting
export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }
  if (typeof error === 'string') {
    return error
  }
  return 'An unknown error occurred'
}

// URL validation
export function validateUrl(url: string): boolean {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

// Time formatting
export function formatRelativeTime(timestamp: number): string {
  const now = Date.now()
  const diff = now - timestamp
  
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (days > 0) return `${days}d ago`
  if (hours > 0) return `${hours}h ago`
  if (minutes > 0) return `${minutes}m ago`
  return 'just now'
}

// Data size formatting
export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
} 