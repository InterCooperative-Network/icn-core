import { PaletteComponent } from '../types'

// Governance components based on existing CCL templates and functionality
export const GOVERNANCE_COMPONENTS: PaletteComponent[] = [
  {
    id: 'voting_mechanism',
    category: 'governance',
    name: 'Voting Mechanism',
    description: 'Create voting rules with quorum and threshold requirements',
    icon: '🗳️',
    defaultConfig: {
      quorum: 50,
      threshold: 0.6,
      votingDuration: 7, // days
      allowDelegation: true,
    },
    parameters: [
      {
        name: 'quorum',
        type: 'number',
        description: 'Minimum number of votes required',
        required: true,
        default: 50,
        validation: { min: 1, max: 1000 }
      },
      {
        name: 'threshold',
        type: 'number',
        description: 'Fraction of Yes votes needed to pass (0.0-1.0)',
        required: true,
        default: 0.6,
        validation: { min: 0.5, max: 1.0 }
      },
      {
        name: 'votingDuration',
        type: 'number',
        description: 'Voting period in days',
        required: true,
        default: 7,
        validation: { min: 1, max: 30 }
      },
      {
        name: 'allowDelegation',
        type: 'boolean',
        description: 'Allow members to delegate their votes',
        required: false,
        default: true
      }
    ],
    ports: {
      inputs: [
        { id: 'proposal_in', name: 'Proposal', type: 'data', dataType: 'object' },
        { id: 'voters_in', name: 'Eligible Voters', type: 'data', dataType: 'array' }
      ],
      outputs: [
        { id: 'result_out', name: 'Vote Result', type: 'data', dataType: 'object' },
        { id: 'passed_out', name: 'Proposal Passed', type: 'event' },
        { id: 'failed_out', name: 'Proposal Failed', type: 'event' }
      ]
    }
  },

  {
    id: 'member_role',
    category: 'governance',
    name: 'Member Role',
    description: 'Define member roles and permissions within the cooperative',
    icon: '👥',
    defaultConfig: {
      roleName: 'Member',
      permissions: ['vote', 'propose'],
      inheritance: 'basic',
    },
    parameters: [
      {
        name: 'roleName',
        type: 'string',
        description: 'Name of the role',
        required: true,
        default: 'Member'
      },
      {
        name: 'permissions',
        type: 'select',
        description: 'Permissions for this role',
        required: true,
        default: 'vote',
        validation: {
          options: ['vote', 'propose', 'manage', 'admin', 'delegate', 'audit']
        }
      },
      {
        name: 'inheritance',
        type: 'select',
        description: 'Role inheritance level',
        required: false,
        default: 'basic',
        validation: {
          options: ['basic', 'extended', 'admin']
        }
      }
    ],
    ports: {
      inputs: [
        { id: 'member_in', name: 'Member DID', type: 'data', dataType: 'string' },
        { id: 'requirements_in', name: 'Requirements', type: 'data', dataType: 'object' }
      ],
      outputs: [
        { id: 'role_assigned', name: 'Role Assigned', type: 'event' },
        { id: 'permissions_out', name: 'Permissions', type: 'data', dataType: 'array' }
      ]
    }
  },

  {
    id: 'budget_request',
    category: 'governance',
    name: 'Budget Request',
    description: 'Create budget allocation requests with approval workflows',
    icon: '💰',
    defaultConfig: {
      amount: 1000,
      category: 'operations',
      approvalTier: 'committee',
      deadline: 30,
    },
    parameters: [
      {
        name: 'amount',
        type: 'number',
        description: 'Requested amount in base currency',
        required: true,
        default: 1000,
        validation: { min: 1 }
      },
      {
        name: 'category',
        type: 'select',
        description: 'Budget category',
        required: true,
        default: 'operations',
        validation: {
          options: ['operations', 'infrastructure', 'community', 'emergency', 'development']
        }
      },
      {
        name: 'approvalTier',
        type: 'select',
        description: 'Required approval level',
        required: true,
        default: 'committee',
        validation: {
          options: ['simple', 'committee', 'assembly']
        }
      },
      {
        name: 'deadline',
        type: 'number',
        description: 'Request deadline in days',
        required: false,
        default: 30,
        validation: { min: 1, max: 365 }
      }
    ],
    ports: {
      inputs: [
        { id: 'requester_in', name: 'Requester DID', type: 'data', dataType: 'string' },
        { id: 'justification_in', name: 'Justification', type: 'data', dataType: 'string' }
      ],
      outputs: [
        { id: 'request_created', name: 'Request Created', type: 'event' },
        { id: 'approval_needed', name: 'Approval Required', type: 'control' },
        { id: 'approved_out', name: 'Request Approved', type: 'event' },
        { id: 'rejected_out', name: 'Request Rejected', type: 'event' }
      ]
    }
  },

  {
    id: 'proposal_creation',
    category: 'governance',
    name: 'Proposal Creation',
    description: 'Create and submit governance proposals',
    icon: '📝',
    defaultConfig: {
      proposalType: 'generic',
      requiresDeposit: false,
      depositAmount: 100,
      minimumDiscussion: 3,
    },
    parameters: [
      {
        name: 'proposalType',
        type: 'select',
        description: 'Type of proposal',
        required: true,
        default: 'generic',
        validation: {
          options: ['generic', 'budget', 'membership', 'rule_change', 'emergency']
        }
      },
      {
        name: 'requiresDeposit',
        type: 'boolean',
        description: 'Require mana deposit to submit',
        required: false,
        default: false
      },
      {
        name: 'depositAmount',
        type: 'number',
        description: 'Required deposit amount',
        required: false,
        default: 100,
        validation: { min: 0 }
      },
      {
        name: 'minimumDiscussion',
        type: 'number',
        description: 'Minimum discussion period in days',
        required: false,
        default: 3,
        validation: { min: 0, max: 30 }
      }
    ],
    ports: {
      inputs: [
        { id: 'proposer_in', name: 'Proposer DID', type: 'data', dataType: 'string' },
        { id: 'title_in', name: 'Title', type: 'data', dataType: 'string' },
        { id: 'description_in', name: 'Description', type: 'data', dataType: 'string' }
      ],
      outputs: [
        { id: 'proposal_out', name: 'Created Proposal', type: 'data', dataType: 'object' },
        { id: 'discussion_start', name: 'Discussion Started', type: 'event' },
        { id: 'ready_for_vote', name: 'Ready for Voting', type: 'control' }
      ]
    }
  },

  {
    id: 'reputation_weighting',
    category: 'governance',
    name: 'Reputation Weighting',
    description: 'Apply reputation-based weights to votes and decisions',
    icon: '⭐',
    defaultConfig: {
      weightingType: 'linear',
      minimumReputation: 10,
      maximumWeight: 5.0,
      decayFactor: 0.1,
    },
    parameters: [
      {
        name: 'weightingType',
        type: 'select',
        description: 'How reputation affects weight',
        required: true,
        default: 'linear',
        validation: {
          options: ['linear', 'quadratic', 'logarithmic', 'threshold']
        }
      },
      {
        name: 'minimumReputation',
        type: 'number',
        description: 'Minimum reputation to participate',
        required: false,
        default: 10,
        validation: { min: 0 }
      },
      {
        name: 'maximumWeight',
        type: 'number',
        description: 'Maximum vote weight multiplier',
        required: false,
        default: 5.0,
        validation: { min: 1.0, max: 10.0 }
      },
      {
        name: 'decayFactor',
        type: 'number',
        description: 'Reputation decay factor over time',
        required: false,
        default: 0.1,
        validation: { min: 0.0, max: 1.0 }
      }
    ],
    ports: {
      inputs: [
        { id: 'member_in', name: 'Member DID', type: 'data', dataType: 'string' },
        { id: 'reputation_in', name: 'Reputation Score', type: 'data', dataType: 'number' },
        { id: 'vote_in', name: 'Original Vote', type: 'data', dataType: 'object' }
      ],
      outputs: [
        { id: 'weighted_vote', name: 'Weighted Vote', type: 'data', dataType: 'object' },
        { id: 'weight_applied', name: 'Weight Factor', type: 'data', dataType: 'number' }
      ]
    }
  },

  {
    id: 'assembly_governance',
    category: 'governance',
    name: 'Assembly Governance',
    description: 'Large-scale democratic assembly with delegation support',
    icon: '🏛️',
    defaultConfig: {
      assemblyType: 'general',
      allowDelegation: true,
      maxDelegationDepth: 3,
      quorumPercentage: 10,
    },
    parameters: [
      {
        name: 'assemblyType',
        type: 'select',
        description: 'Type of assembly',
        required: true,
        default: 'general',
        validation: {
          options: ['general', 'emergency', 'budget', 'constitutional']
        }
      },
      {
        name: 'allowDelegation',
        type: 'boolean',
        description: 'Allow liquid democracy delegation',
        required: false,
        default: true
      },
      {
        name: 'maxDelegationDepth',
        type: 'number',
        description: 'Maximum delegation chain length',
        required: false,
        default: 3,
        validation: { min: 1, max: 10 }
      },
      {
        name: 'quorumPercentage',
        type: 'number',
        description: 'Required participation percentage',
        required: true,
        default: 10,
        validation: { min: 1, max: 100 }
      }
    ],
    ports: {
      inputs: [
        { id: 'members_in', name: 'Assembly Members', type: 'data', dataType: 'array' },
        { id: 'agenda_in', name: 'Meeting Agenda', type: 'data', dataType: 'object' }
      ],
      outputs: [
        { id: 'assembly_result', name: 'Assembly Decision', type: 'data', dataType: 'object' },
        { id: 'quorum_met', name: 'Quorum Achieved', type: 'event' },
        { id: 'quorum_failed', name: 'Quorum Failed', type: 'event' }
      ]
    }
  }
]

// Component categories for organization
export const GOVERNANCE_CATEGORIES = {
  voting: ['voting_mechanism', 'reputation_weighting'],
  membership: ['member_role', 'assembly_governance'],
  proposals: ['proposal_creation', 'budget_request'],
  all: GOVERNANCE_COMPONENTS.map(c => c.id)
}

// Helper functions
export function getComponentById(id: string): PaletteComponent | undefined {
  return GOVERNANCE_COMPONENTS.find(c => c.id === id)
}

export function getComponentsByCategory(category: keyof typeof GOVERNANCE_CATEGORIES): PaletteComponent[] {
  const ids = GOVERNANCE_CATEGORIES[category] || []
  return ids.map(id => getComponentById(id)).filter(Boolean) as PaletteComponent[]
}

export function validateComponentConfig(component: PaletteComponent, config: any): { valid: boolean; errors: string[] } {
  const errors: string[] = []
  
  for (const param of component.parameters) {
    const value = config[param.name]
    
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
        }
        break
      case 'boolean':
        if (typeof value !== 'boolean') {
          errors.push(`${param.name} must be true or false`)
        }
        break
      case 'select':
        if (param.validation?.options && !param.validation.options.includes(value)) {
          errors.push(`${param.name} must be one of: ${param.validation.options.join(', ')}`)
        }
        break
    }
  }
  
  return { valid: errors.length === 0, errors }
} 