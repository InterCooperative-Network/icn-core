import { ICNUtils } from '@icn/client-sdk'

// Re-export utilities from client SDK
export { ICNUtils }

// Legacy utilities (keeping for compatibility)
export function validateDid(did: string): boolean {
  return ICNUtils.isValidDid(did)
}

export function formatMana(amount: number): string {
  return ICNUtils.formatMana(amount)
}

export function formatJobId(jobId: string): string {
  if (!jobId) return ''
  
  // Show first 8 and last 4 characters for readability
  if (jobId.length > 12) {
    return `${jobId.slice(0, 8)}...${jobId.slice(-4)}`
  }
  return jobId
}

// Enhanced utilities for comprehensive SDK functionality
export const EnhancedUtils = {
  /**
   * Format credential ID for display
   */
  formatCredentialId(cid: string): string {
    if (!cid) return ''
    return cid.length > 16 ? `${cid.slice(0, 8)}...${cid.slice(-8)}` : cid
  },

  /**
   * Format token amount with proper decimals
   */
  formatTokenAmount(amount: number, decimals: number = 0, symbol?: string): string {
    const divisor = Math.pow(10, decimals)
    const formatted = (amount / divisor).toLocaleString(undefined, {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals
    })
    return symbol ? `${formatted} ${symbol}` : formatted
  },

  /**
   * Format trust score as percentage
   */
  formatTrustScore(score: number): string {
    return `${(score * 100).toFixed(1)}%`
  },

  /**
   * Format trust level for display
   */
  formatTrustLevel(level: string): string {
    const levelMap: Record<string, string> = {
      'none': '❌ None',
      'low': '🟡 Low',
      'medium': '🟠 Medium',
      'high': '🟢 High',
      'absolute': '💎 Absolute'
    }
    return levelMap[level] || level
  },

  /**
   * Calculate time until expiration
   */
  getExpirationStatus(expirationTimestamp: number): {
    status: 'expired' | 'expiring_soon' | 'current'
    daysRemaining: number
    message: string
  } {
    const now = Date.now() / 1000
    const timeRemaining = expirationTimestamp - now
    const daysRemaining = Math.floor(timeRemaining / (24 * 3600))

    if (daysRemaining < 0) {
      return {
        status: 'expired',
        daysRemaining: 0,
        message: `Expired ${Math.abs(daysRemaining)} days ago`
      }
    } else if (daysRemaining < 30) {
      return {
        status: 'expiring_soon',
        daysRemaining,
        message: `Expires in ${daysRemaining} days`
      }
    } else {
      return {
        status: 'current',
        daysRemaining,
        message: `Valid for ${daysRemaining} days`
      }
    }
  },

  /**
   * Validate token amount format
   */
  validateTokenAmount(amount: string, decimals: number = 0): {
    valid: boolean
    error?: string
    parsedAmount?: number
  } {
    if (!amount || amount.trim() === '') {
      return { valid: false, error: 'Amount is required' }
    }

    const num = parseFloat(amount)
    if (isNaN(num)) {
      return { valid: false, error: 'Invalid number format' }
    }

    if (num < 0) {
      return { valid: false, error: 'Amount must be positive' }
    }

    const multiplier = Math.pow(10, decimals)
    const integerAmount = Math.round(num * multiplier)
    
    // Check if we lost precision
    if (Math.abs((integerAmount / multiplier) - num) > 1e-10) {
      return { 
        valid: false, 
        error: `Too many decimal places (max ${decimals})` 
      }
    }

    return { valid: true, parsedAmount: integerAmount }
  },

  /**
   * Generate secure random string for challenges
   */
  generateChallenge(length: number = 32): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
    let result = ''
    
    // Use crypto.getRandomValues if available, fallback to Math.random
    if (typeof crypto !== 'undefined' && crypto.getRandomValues) {
      const array = new Uint8Array(length)
      crypto.getRandomValues(array)
      for (let i = 0; i < length; i++) {
        result += chars[array[i] % chars.length]
      }
    } else {
      for (let i = 0; i < length; i++) {
        result += chars[Math.floor(Math.random() * chars.length)]
      }
    }
    
    return result
  },

  /**
   * Format proposal status for display
   */
  formatProposalStatus(status: string): string {
    const statusMap: Record<string, string> = {
      'Draft': '📝 Draft',
      'Open': '🗳️  Open',
      'Closed': '🔒 Closed',
      'Executed': '✅ Executed'
    }
    return statusMap[status] || status
  },

  /**
   * Calculate voting progress
   */
  calculateVotingProgress(proposal: any): {
    totalVotes: number
    yesPercentage: number
    noPercentage: number
    abstainPercentage: number
    quorumReached: boolean
    progressPercentage: number
  } {
    const yes = proposal.votes?.yes || 0
    const no = proposal.votes?.no || 0
    const abstain = proposal.votes?.abstain || 0
    const totalVotes = yes + no + abstain
    const quorum = proposal.quorum || 50

    return {
      totalVotes,
      yesPercentage: totalVotes > 0 ? (yes / totalVotes) * 100 : 0,
      noPercentage: totalVotes > 0 ? (no / totalVotes) * 100 : 0,
      abstainPercentage: totalVotes > 0 ? (abstain / totalVotes) * 100 : 0,
      quorumReached: totalVotes >= quorum,
      progressPercentage: Math.min((totalVotes / quorum) * 100, 100)
    }
  },

  /**
   * Format resource availability status
   */
  formatResourceAvailability(availability: string): string {
    const availabilityMap: Record<string, string> = {
      'available': '✅ Available',
      'unavailable': '❌ Unavailable',
      'limited': '⚠️  Limited'
    }
    return availabilityMap[availability] || availability
  },

  /**
   * Validate DID format with enhanced error messages
   */
  validateDidFormat(did: string): { valid: boolean; error?: string } {
    if (!did || typeof did !== 'string') {
      return { valid: false, error: 'DID is required and must be a string' }
    }

    if (!ICNUtils.isValidDid(did)) {
      return { 
        valid: false, 
        error: 'Invalid DID format. Must be in format: did:method:identifier' 
      }
    }

    return { valid: true }
  },

  /**
   * Sanitize user input for safe display
   */
  sanitizeInput(input: string, maxLength: number = 256): string {
    if (!input || typeof input !== 'string') return ''
    
    return input
      .trim()
      .slice(0, maxLength)
      .replace(/[<>]/g, '') // Remove potential HTML tags
      .replace(/javascript:/gi, '') // Remove javascript: protocols
  },

  /**
   * Format job execution duration
   */
  formatJobDuration(startedAt?: string, completedAt?: string): string {
    if (!startedAt) return 'Not started'
    if (!completedAt) return 'Running...'

    const start = new Date(startedAt).getTime()
    const end = new Date(completedAt).getTime()
    const duration = (end - start) / 1000 // seconds

    if (duration < 60) {
      return `${Math.round(duration)}s`
    } else if (duration < 3600) {
      return `${Math.round(duration / 60)}m ${Math.round(duration % 60)}s`
    } else {
      const hours = Math.floor(duration / 3600)
      const minutes = Math.round((duration % 3600) / 60)
      return `${hours}h ${minutes}m`
    }
  },

  /**
   * Calculate trust path strength
   */
  calculatePathStrength(path: any): {
    strength: 'weak' | 'moderate' | 'strong' | 'very_strong'
    score: number
    factors: string[]
  } {
    const factors: string[] = []
    let score = 0

    // Length factor (shorter is better)
    if (path.length <= 2) {
      score += 40
      factors.push('Short path')
    } else if (path.length <= 4) {
      score += 20
      factors.push('Moderate path')
    } else {
      score += 5
      factors.push('Long path')
    }

    // Weight factor
    if (path.weight > 0.8) {
      score += 40
      factors.push('High weight')
    } else if (path.weight > 0.6) {
      score += 25
      factors.push('Good weight')
    } else if (path.weight > 0.4) {
      score += 15
      factors.push('Fair weight')
    } else {
      score += 5
      factors.push('Low weight')
    }

    // Trust level factor
    const trustLevelScore: Record<string, number> = {
      'absolute': 20,
      'high': 15,
      'medium': 10,
      'low': 5,
      'none': 0
    }
    const levelScore = trustLevelScore[path.effective_trust] || 0
    score += levelScore
    factors.push(`${path.effective_trust} effective trust`)

    let strength: 'weak' | 'moderate' | 'strong' | 'very_strong'
    if (score >= 80) strength = 'very_strong'
    else if (score >= 60) strength = 'strong'
    else if (score >= 40) strength = 'moderate'
    else strength = 'weak'

    return { strength, score, factors }
  }
}

// Additional UI-specific utilities for Federation Dashboard and Governance
export const FederationUtils = {
  /**
   * Generate a federation DID based on metadata
   */
  generateFederationDid(name: string): string {
    const timestamp = Date.now()
    const hash = btoa(name + timestamp).replace(/[^a-zA-Z0-9]/g, '').substring(0, 20)
    return `did:federation:${hash}`
  },

  /**
   * Validate federation name
   */
  isValidFederationName(name: string): boolean {
    return /^[a-zA-Z0-9][a-zA-Z0-9_\-\s]{2,49}$/.test(name)
  },

  /**
   * Calculate federation health score based on various metrics
   */
  calculateHealthScore(metadata: any): number {
    let score = 0
    const factors = {
      memberCount: metadata.totalMembers || 0,
      activeProposals: metadata.governance?.activeProposals || 0,
      activeJobs: metadata.mesh?.activeJobs || 0,
      syncStatus: metadata.dag?.syncStatus === 'synced' ? 1 : 0,
    }

    // Member count (0-40 points)
    score += Math.min(factors.memberCount * 2, 40)
    
    // Active governance (0-20 points)
    score += Math.min(factors.activeProposals * 5, 20)
    
    // Mesh activity (0-20 points)
    score += Math.min(factors.activeJobs * 2, 20)
    
    // DAG sync status (0-20 points)
    score += factors.syncStatus * 20

    return Math.min(score, 100)
  }
}

export const GovernanceUtils = {
  /**
   * Calculate voting progress percentage
   */
  calculateVotingProgress(proposal: any): number {
    const totalVotes = (proposal.votes?.yes || 0) + (proposal.votes?.no || 0) + (proposal.votes?.abstain || 0)
    const quorum = proposal.quorum || 50
    return Math.min((totalVotes / quorum) * 100, 100)
  },

  /**
   * Determine if proposal has reached quorum
   */
  hasReachedQuorum(proposal: any): boolean {
    const totalVotes = (proposal.votes?.yes || 0) + (proposal.votes?.no || 0) + (proposal.votes?.abstain || 0)
    const quorum = proposal.quorum || 50
    return totalVotes >= quorum
  },

  /**
   * Determine proposal outcome
   */
  getProposalOutcome(proposal: any): 'passed' | 'failed' | 'pending' {
    if (proposal.status === 'Closed' || proposal.status === 'Executed') {
      const threshold = proposal.threshold || 0.5
      const totalVotes = (proposal.votes?.yes || 0) + (proposal.votes?.no || 0)
      const yesPercentage = totalVotes > 0 ? (proposal.votes?.yes || 0) / totalVotes : 0
      return yesPercentage >= threshold ? 'passed' : 'failed'
    }
    return 'pending'
  },

  /**
   * Format proposal type for display
   */
  formatProposalType(proposalType: any): string {
    if (typeof proposalType === 'object' && proposalType.type) {
      switch (proposalType.type) {
        case 'SystemParameterChange':
          return 'System Parameter Change'
        case 'MemberAdmission':
          return 'Member Admission'
        case 'RemoveMember':
          return 'Remove Member'
        case 'SoftwareUpgrade':
          return 'Software Upgrade'
        case 'GenericText':
          return 'Text Proposal'
        case 'Resolution':
          return 'Resolution'
        default:
          return proposalType.type
      }
    }
    return 'Unknown'
  },

  /**
   * Generate proposal summary based on type and data
   */
  generateProposalSummary(proposalType: any): string {
    if (typeof proposalType === 'object' && proposalType.type) {
      switch (proposalType.type) {
        case 'SystemParameterChange':
          return `Change ${proposalType.data?.param} to ${proposalType.data?.value}`
        case 'MemberAdmission':
          return `Admit ${proposalType.data?.did} as member`
        case 'RemoveMember':
          return `Remove ${proposalType.data?.did} from membership`
        case 'SoftwareUpgrade':
          return `Upgrade to version ${proposalType.data?.version}`
        case 'GenericText':
          return proposalType.data?.text?.substring(0, 100) + (proposalType.data?.text?.length > 100 ? '...' : '')
        case 'Resolution':
          return `Resolution with ${proposalType.data?.actions?.length || 0} action(s)`
        default:
          return 'Unknown proposal type'
      }
    }
    return 'Unknown proposal'
  }
}

export const CCLUtils = {
  /**
   * Validate CCL template parameters
   */
  validateTemplateParameters(template: any, parameters: Record<string, any>): { valid: boolean; errors: string[] } {
    const errors: string[] = []
    
    if (!template.parameters) {
      return { valid: true, errors: [] }
    }

    for (const param of template.parameters) {
      const value = parameters[param.name]
      
      // Check required parameters
      if (param.required && (value === undefined || value === null || value === '')) {
        errors.push(`${param.name} is required`)
        continue
      }

      // Skip validation if value is not provided and not required
      if (value === undefined || value === null || value === '') {
        continue
      }

      // Type validation
      switch (param.type) {
        case 'number':
          if (typeof value !== 'number' && isNaN(Number(value))) {
            errors.push(`${param.name} must be a number`)
          } else {
            const numValue = Number(value)
            if (param.validation?.min !== undefined && numValue < param.validation.min) {
              errors.push(`${param.name} must be at least ${param.validation.min}`)
            }
            if (param.validation?.max !== undefined && numValue > param.validation.max) {
              errors.push(`${param.name} must be at most ${param.validation.max}`)
            }
          }
          break
        case 'string':
          if (typeof value !== 'string') {
            errors.push(`${param.name} must be a string`)
          } else {
            if (param.validation?.pattern && !new RegExp(param.validation.pattern).test(value)) {
              errors.push(`${param.name} format is invalid`)
            }
            if (param.validation?.options && !param.validation.options.includes(value)) {
              errors.push(`${param.name} must be one of: ${param.validation.options.join(', ')}`)
            }
          }
          break
        case 'boolean':
          if (typeof value !== 'boolean') {
            errors.push(`${param.name} must be true or false`)
          }
          break
        case 'did':
          if (!ICNUtils.isValidDid(value)) {
            errors.push(`${param.name} must be a valid DID`)
          }
          break
      }
    }

    return { valid: errors.length === 0, errors }
  },

  /**
   * Generate CCL code from template and parameters
   */
  generateCCLFromTemplate(template: any, parameters: Record<string, any>): string {
    let ccl = template.template
    
    // Replace parameter placeholders
    for (const [key, value] of Object.entries(parameters)) {
      const placeholder = new RegExp(`\\{\\{${key}\\}\\}`, 'g')
      ccl = ccl.replace(placeholder, String(value))
    }
    
    return ccl
  }
}

// Keep existing utilities for compatibility
export function validateAddress(address: string): boolean {
  if (!address || typeof address !== 'string') return false
  return address.length >= 10 && address.length <= 100
}

export function validateNetwork(network: string): boolean {
  const validNetworks = ['mainnet', 'testnet', 'devnet']
  return validNetworks.includes(network)
}

export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message
  }
  if (typeof error === 'string') {
    return error
  }
  return 'An unknown error occurred'
}

export function validateUrl(url: string): boolean {
  try {
    new URL(url)
    return true
  } catch {
    return false
  }
}

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

export function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
} 