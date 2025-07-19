import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react'
import { useICNClient } from '@icn/ts-sdk'
import type { 
  Proposal,
  SubmitProposalRequest,
  CastVoteRequest,
  CCLTemplate,
} from '@icn/ts-sdk'

interface GovernanceContextType {
  // Proposals
  proposals: Proposal[]
  activeProposals: Proposal[]
  
  // CCL Templates
  cclTemplates: CCLTemplate[]
  
  // Loading states
  loading: {
    proposals: boolean
    templates: boolean
    submitting: boolean
    voting: boolean
  }
  
  // Actions
  submitProposal: (request: SubmitProposalRequest) => Promise<string>
  castVote: (request: CastVoteRequest) => Promise<void>
  refreshProposals: () => Promise<void>
  loadCCLTemplates: () => Promise<void>
  
  // Error state
  error: string | null
}

const GovernanceContext = createContext<GovernanceContextType | undefined>(undefined)

interface GovernanceProviderProps {
  children: ReactNode
}

export function GovernanceProvider({ children }: GovernanceProviderProps) {
  const icnClient = useICNClient()
  
  // State
  const [proposals, setProposals] = useState<Proposal[]>([])
  const [cclTemplates, setCclTemplates] = useState<CCLTemplate[]>([])
  const [error, setError] = useState<string | null>(null)
  
  const [loading, setLoading] = useState({
    proposals: false,
    templates: false,
    submitting: false,
    voting: false,
  })

  // Computed values
  const activeProposals = proposals.filter(p => p.status === 'Open')

  // Load proposals
  const loadProposals = async () => {
    try {
      setLoading(prev => ({ ...prev, proposals: true }))
      setError(null)
      const proposalList = await icnClient.governance.listProposals()
      setProposals(proposalList)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load proposals')
    } finally {
      setLoading(prev => ({ ...prev, proposals: false }))
    }
  }

  // Load CCL templates (mock data for now)
  const loadCCLTemplates = async () => {
    try {
      setLoading(prev => ({ ...prev, templates: true }))
      
      // Mock CCL templates - in real implementation, these would come from a templates API
      const mockTemplates: CCLTemplate[] = [
        {
          id: 'member-admission',
          name: 'Member Admission',
          description: 'Template for admitting new members to the cooperative',
          category: 'membership',
          template: `
// Member Admission Proposal
cooperative "{{cooperative_name}}" {
  propose admission {
    candidate: "{{candidate_did}}"
    sponsor: "{{sponsor_did}}"
    
    requirements {
      background_check: {{require_background_check}}
      skills: [{{required_skills}}]
      commitment_period: {{commitment_months}} months
    }
    
    voting {
      threshold: {{approval_threshold}}
      quorum: {{quorum_percentage}}
    }
  }
}`,
          parameters: [
            {
              name: 'cooperative_name',
              type: 'string',
              description: 'Name of the cooperative',
              required: true,
            },
            {
              name: 'candidate_did',
              type: 'did',
              description: 'DID of the candidate member',
              required: true,
            },
            {
              name: 'sponsor_did',
              type: 'did',
              description: 'DID of the sponsoring member',
              required: true,
            },
            {
              name: 'require_background_check',
              type: 'boolean',
              description: 'Whether a background check is required',
              required: false,
              default: false,
            },
            {
              name: 'required_skills',
              type: 'string',
              description: 'Comma-separated list of required skills',
              required: false,
              default: '',
            },
            {
              name: 'commitment_months',
              type: 'number',
              description: 'Minimum commitment period in months',
              required: true,
              validation: { min: 1, max: 60 }
            },
            {
              name: 'approval_threshold',
              type: 'number',
              description: 'Approval threshold (0.0 to 1.0)',
              required: true,
              default: 0.6,
              validation: { min: 0.5, max: 1.0 }
            },
            {
              name: 'quorum_percentage',
              type: 'number',
              description: 'Quorum requirement as percentage',
              required: true,
              default: 50,
              validation: { min: 25, max: 100 }
            }
          ]
        },
        {
          id: 'budget-allocation',
          name: 'Budget Allocation',
          description: 'Template for budget allocation proposals',
          category: 'economic',
          template: `
// Budget Allocation Proposal
cooperative "{{cooperative_name}}" {
  propose budget_allocation {
    purpose: "{{purpose}}"
    amount: {{amount}} mana
    category: "{{category}}"
    
    timeline {
      start_date: "{{start_date}}"
      end_date: "{{end_date}}"
    }
    
    accountability {
      responsible_member: "{{responsible_did}}"
      reporting_frequency: "{{reporting_frequency}}"
    }
    
    voting {
      threshold: {{approval_threshold}}
      quorum: {{quorum_percentage}}
    }
  }
}`,
          parameters: [
            {
              name: 'cooperative_name',
              type: 'string',
              description: 'Name of the cooperative',
              required: true,
            },
            {
              name: 'purpose',
              type: 'string',
              description: 'Purpose of the budget allocation',
              required: true,
            },
            {
              name: 'amount',
              type: 'number',
              description: 'Amount in mana',
              required: true,
              validation: { min: 1 }
            },
            {
              name: 'category',
              type: 'string',
              description: 'Budget category',
              required: true,
              validation: {
                options: ['operations', 'development', 'marketing', 'infrastructure', 'community', 'emergency']
              }
            },
            {
              name: 'start_date',
              type: 'string',
              description: 'Start date (YYYY-MM-DD)',
              required: true,
              validation: { pattern: '^\\d{4}-\\d{2}-\\d{2}$' }
            },
            {
              name: 'end_date',
              type: 'string',
              description: 'End date (YYYY-MM-DD)',
              required: true,
              validation: { pattern: '^\\d{4}-\\d{2}-\\d{2}$' }
            },
            {
              name: 'responsible_did',
              type: 'did',
              description: 'DID of responsible member',
              required: true,
            },
            {
              name: 'reporting_frequency',
              type: 'string',
              description: 'Reporting frequency',
              required: true,
              validation: {
                options: ['weekly', 'monthly', 'quarterly']
              }
            },
            {
              name: 'approval_threshold',
              type: 'number',
              description: 'Approval threshold (0.0 to 1.0)',
              required: true,
              default: 0.75,
              validation: { min: 0.5, max: 1.0 }
            },
            {
              name: 'quorum_percentage',
              type: 'number',
              description: 'Quorum requirement as percentage',
              required: true,
              default: 60,
              validation: { min: 25, max: 100 }
            }
          ]
        },
        {
          id: 'governance-change',
          name: 'Governance Rule Change',
          description: 'Template for changing governance rules',
          category: 'governance',
          template: `
// Governance Rule Change Proposal
cooperative "{{cooperative_name}}" {
  propose governance_change {
    rule_name: "{{rule_name}}"
    current_value: "{{current_value}}"
    proposed_value: "{{proposed_value}}"
    
    rationale: "{{rationale}}"
    
    impact_assessment {
      affected_members: {{affected_members}}
      implementation_complexity: "{{complexity}}"
      reversible: {{reversible}}
    }
    
    voting {
      threshold: {{approval_threshold}}
      quorum: {{quorum_percentage}}
      super_majority: true
    }
  }
}`,
          parameters: [
            {
              name: 'cooperative_name',
              type: 'string',
              description: 'Name of the cooperative',
              required: true,
            },
            {
              name: 'rule_name',
              type: 'string',
              description: 'Name of the governance rule to change',
              required: true,
            },
            {
              name: 'current_value',
              type: 'string',
              description: 'Current value of the rule',
              required: true,
            },
            {
              name: 'proposed_value',
              type: 'string',
              description: 'Proposed new value',
              required: true,
            },
            {
              name: 'rationale',
              type: 'string',
              description: 'Rationale for the change',
              required: true,
            },
            {
              name: 'affected_members',
              type: 'number',
              description: 'Number of members affected',
              required: true,
              validation: { min: 0 }
            },
            {
              name: 'complexity',
              type: 'string',
              description: 'Implementation complexity',
              required: true,
              validation: {
                options: ['low', 'medium', 'high']
              }
            },
            {
              name: 'reversible',
              type: 'boolean',
              description: 'Whether the change is easily reversible',
              required: true,
              default: false,
            },
            {
              name: 'approval_threshold',
              type: 'number',
              description: 'Approval threshold (0.0 to 1.0)',
              required: true,
              default: 0.8,
              validation: { min: 0.67, max: 1.0 }
            },
            {
              name: 'quorum_percentage',
              type: 'number',
              description: 'Quorum requirement as percentage',
              required: true,
              default: 75,
              validation: { min: 50, max: 100 }
            }
          ]
        }
      ]
      
      setCclTemplates(mockTemplates)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load CCL templates')
    } finally {
      setLoading(prev => ({ ...prev, templates: false }))
    }
  }

  // Actions
  const submitProposal = async (request: SubmitProposalRequest): Promise<string> => {
    try {
      setLoading(prev => ({ ...prev, submitting: true }))
      setError(null)
      const proposalId = await icnClient.governance.submitProposal(request)
      await loadProposals() // Refresh proposals list
      return proposalId
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to submit proposal'
      setError(errorMsg)
      throw new Error(errorMsg)
    } finally {
      setLoading(prev => ({ ...prev, submitting: false }))
    }
  }

  const castVote = async (request: CastVoteRequest): Promise<void> => {
    try {
      setLoading(prev => ({ ...prev, voting: true }))
      setError(null)
      await icnClient.governance.castVote(request)
      await loadProposals() // Refresh proposals list
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to cast vote'
      setError(errorMsg)
      throw new Error(errorMsg)
    } finally {
      setLoading(prev => ({ ...prev, voting: false }))
    }
  }

  const refreshProposals = async () => {
    await loadProposals()
  }

  // Effects
  useEffect(() => {
    if (icnClient.getConnectionState().connected) {
      loadProposals()
      loadCCLTemplates()
    }
  }, [icnClient.getConnectionState().connected])

  const contextValue: GovernanceContextType = {
    proposals,
    activeProposals,
    cclTemplates,
    loading,
    submitProposal,
    castVote,
    refreshProposals,
    loadCCLTemplates,
    error,
  }

  return (
    <GovernanceContext.Provider value={contextValue}>
      {children}
    </GovernanceContext.Provider>
  )
}

export function useGovernance() {
  const context = useContext(GovernanceContext)
  if (context === undefined) {
    throw new Error('useGovernance must be used within a GovernanceProvider')
  }
  return context
}