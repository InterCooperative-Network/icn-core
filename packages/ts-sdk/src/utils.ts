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

// Additional utilities for Federation Dashboard and Governance
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