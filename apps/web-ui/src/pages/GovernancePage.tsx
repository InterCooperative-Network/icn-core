import React, { useState } from 'react'
import { useGovernance } from '../contexts/GovernanceContext'
import { GovernanceUtils, CCLUtils, ICNUtils } from '@icn/ts-sdk'
import type { CCLTemplate, SubmitProposalRequest, ProposalInputType } from '@icn/ts-sdk'

interface ProposalFormProps {
  templates: CCLTemplate[]
  onSubmit: (request: SubmitProposalRequest) => Promise<void>
  loading: boolean
}

function ProposalCreationForm({ templates, onSubmit, loading }: ProposalFormProps) {
  const [selectedTemplate, setSelectedTemplate] = useState<CCLTemplate | null>(null)
  const [templateParams, setTemplateParams] = useState<Record<string, any>>({})
  const [formData, setFormData] = useState({
    proposer_did: '',
    description: '',
    duration_secs: 7 * 24 * 60 * 60, // 7 days
    quorum: 50,
    threshold: 0.6,
  })
  const [errors, setErrors] = useState<Record<string, string>>({})
  const [showTemplatePreview, setShowTemplatePreview] = useState(false)

  const handleTemplateSelect = (template: CCLTemplate) => {
    setSelectedTemplate(template)
    
    // Initialize template parameters with defaults
    const initialParams: Record<string, any> = {}
    template.parameters.forEach(param => {
      if (param.default !== undefined) {
        initialParams[param.name] = param.default
      }
    })
    setTemplateParams(initialParams)
  }

  const handleParamChange = (paramName: string, value: any) => {
    setTemplateParams(prev => ({
      ...prev,
      [paramName]: value
    }))
  }

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {}

    if (!formData.proposer_did.trim()) {
      newErrors.proposer_did = 'Proposer DID is required'
    } else if (!ICNUtils.isValidDid(formData.proposer_did)) {
      newErrors.proposer_did = 'Invalid DID format'
    }

    if (!formData.description.trim()) {
      newErrors.description = 'Description is required'
    }

    if (formData.duration_secs < 3600) {
      newErrors.duration_secs = 'Duration must be at least 1 hour'
    }

    if (formData.quorum < 1) {
      newErrors.quorum = 'Quorum must be at least 1'
    }

    if (formData.threshold < 0.5 || formData.threshold > 1) {
      newErrors.threshold = 'Threshold must be between 0.5 and 1.0'
    }

    // Validate template parameters if a template is selected
    if (selectedTemplate) {
      const paramValidation = CCLUtils.validateTemplateParameters(selectedTemplate, templateParams)
      if (!paramValidation.valid) {
        paramValidation.errors.forEach(error => {
          newErrors[`template_${error}`] = error
        })
      }
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!validateForm()) return

    let proposalType: ProposalInputType

    if (selectedTemplate) {
      // Generate CCL code from template
      const cclCode = CCLUtils.generateCCLFromTemplate(selectedTemplate, templateParams)
      
      // For now, create a GenericText proposal with the CCL code
      // In a real implementation, we'd have specific proposal types for CCL
      proposalType = {
        type: 'GenericText',
        data: {
          text: `CCL Template: ${selectedTemplate.name}\n\n${cclCode}`
        }
      }
    } else {
      // Default to GenericText
      proposalType = {
        type: 'GenericText',
        data: {
          text: formData.description
        }
      }
    }

    const request: SubmitProposalRequest = {
      proposer_did: formData.proposer_did,
      proposal: proposalType,
      description: formData.description,
      duration_secs: formData.duration_secs,
      quorum: formData.quorum,
      threshold: formData.threshold,
    }

    await onSubmit(request)
  }

  const generatePreview = () => {
    if (!selectedTemplate) return ''
    return CCLUtils.generateCCLFromTemplate(selectedTemplate, templateParams)
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Basic Information */}
      <div className="space-y-4">
        <h3 className="font-medium text-gray-900">Basic Information</h3>
        
        <div>
          <label htmlFor="proposer_did" className="block text-sm font-medium text-gray-700">
            Proposer DID
          </label>
          <input
            type="text"
            id="proposer_did"
            value={formData.proposer_did}
            onChange={(e) => setFormData(prev => ({ ...prev, proposer_did: e.target.value }))}
            className={`mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 ${
              errors.proposer_did ? 'border-red-300' : ''
            }`}
            placeholder="did:key:..."
          />
          {errors.proposer_did && <p className="mt-1 text-sm text-red-600">{errors.proposer_did}</p>}
        </div>

        <div>
          <label htmlFor="description" className="block text-sm font-medium text-gray-700">
            Description
          </label>
          <textarea
            id="description"
            rows={3}
            value={formData.description}
            onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
            className={`mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 ${
              errors.description ? 'border-red-300' : ''
            }`}
            placeholder="Describe the purpose and impact of this proposal"
          />
          {errors.description && <p className="mt-1 text-sm text-red-600">{errors.description}</p>}
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label htmlFor="duration_secs" className="block text-sm font-medium text-gray-700">
              Duration (hours)
            </label>
            <input
              type="number"
              id="duration_secs"
              value={formData.duration_secs / 3600}
              onChange={(e) => setFormData(prev => ({ ...prev, duration_secs: parseInt(e.target.value) * 3600 }))}
              className={`mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 ${
                errors.duration_secs ? 'border-red-300' : ''
              }`}
              min="1"
            />
            {errors.duration_secs && <p className="mt-1 text-sm text-red-600">{errors.duration_secs}</p>}
          </div>

          <div>
            <label htmlFor="quorum" className="block text-sm font-medium text-gray-700">
              Quorum
            </label>
            <input
              type="number"
              id="quorum"
              value={formData.quorum}
              onChange={(e) => setFormData(prev => ({ ...prev, quorum: parseInt(e.target.value) }))}
              className={`mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 ${
                errors.quorum ? 'border-red-300' : ''
              }`}
              min="1"
            />
            {errors.quorum && <p className="mt-1 text-sm text-red-600">{errors.quorum}</p>}
          </div>

          <div>
            <label htmlFor="threshold" className="block text-sm font-medium text-gray-700">
              Threshold
            </label>
            <input
              type="number"
              id="threshold"
              step="0.1"
              value={formData.threshold}
              onChange={(e) => setFormData(prev => ({ ...prev, threshold: parseFloat(e.target.value) }))}
              className={`mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 ${
                errors.threshold ? 'border-red-300' : ''
              }`}
              min="0.5"
              max="1.0"
            />
            {errors.threshold && <p className="mt-1 text-sm text-red-600">{errors.threshold}</p>}
          </div>
        </div>
      </div>

      {/* CCL Template Selection */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h3 className="font-medium text-gray-900">CCL Templates</h3>
          <span className="text-sm text-gray-500">Optional - Use templates for common proposals</span>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {templates.map((template) => (
            <div
              key={template.id}
              className={`border rounded-lg p-4 cursor-pointer transition-all duration-200 ${
                selectedTemplate?.id === template.id
                  ? 'border-blue-500 bg-blue-50 shadow-md'
                  : 'border-gray-200 hover:border-gray-300 hover:shadow-sm'
              }`}
              onClick={() => handleTemplateSelect(template)}
              role="button"
              tabIndex={0}
              aria-pressed={selectedTemplate?.id === template.id}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault()
                  handleTemplateSelect(template)
                }
              }}
            >
              <div className="flex items-start justify-between mb-2">
                <h4 className="font-medium text-gray-900">{template.name}</h4>
                {selectedTemplate?.id === template.id && (
                  <div className="w-5 h-5 bg-blue-500 rounded-full flex items-center justify-center">
                    <svg className="w-3 h-3 text-white" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                    </svg>
                  </div>
                )}
              </div>
              <p className="text-sm text-gray-600 mb-3">{template.description}</p>
              <div className="flex items-center justify-between">
                <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-800">
                  {template.category}
                </span>
                <span className="text-xs text-gray-500">
                  {template.parameters.length} parameters
                </span>
              </div>
            </div>
          ))}
        </div>

        {selectedTemplate && (
          <div className="border border-blue-200 rounded-lg p-6 bg-blue-50">
            <div className="flex justify-between items-center mb-4">
              <h4 className="font-medium text-blue-900">Configure: {selectedTemplate.name}</h4>
              <div className="flex space-x-2">
                <button
                  type="button"
                  onClick={() => setShowTemplatePreview(!showTemplatePreview)}
                  className="text-blue-700 hover:text-blue-900 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-md px-2 py-1"
                >
                  {showTemplatePreview ? 'Hide' : 'Show'} Preview
                </button>
                <button
                  type="button"
                  onClick={() => setSelectedTemplate(null)}
                  className="text-gray-500 hover:text-gray-700 focus:outline-none focus:ring-2 focus:ring-gray-500 focus:ring-offset-2 rounded-md p-1"
                  aria-label="Clear template selection"
                >
                  <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                    <path fillRule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clipRule="evenodd" />
                  </svg>
                </button>
              </div>
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {selectedTemplate.parameters.map((param) => (
                <div key={param.name} className="space-y-1">
                  <label className="block text-sm font-medium text-blue-900">
                    {param.name} {param.required && <span className="text-red-500">*</span>}
                  </label>
                  <p className="text-xs text-blue-700 mb-2">{param.description}</p>
                  
                  {param.type === 'boolean' ? (
                    <select
                      value={templateParams[param.name] !== undefined ? templateParams[param.name].toString() : ''}
                      onChange={(e) => handleParamChange(param.name, e.target.value === 'true')}
                      className="w-full rounded-md border-blue-300 bg-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
                      required={param.required}
                    >
                      <option value="">Select...</option>
                      <option value="true">Yes</option>
                      <option value="false">No</option>
                    </select>
                  ) : param.validation?.options ? (
                    <select
                      value={templateParams[param.name] || ''}
                      onChange={(e) => handleParamChange(param.name, e.target.value)}
                      className="w-full rounded-md border-blue-300 bg-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
                      required={param.required}
                    >
                      <option value="">Select...</option>
                      {param.validation.options.map((option) => (
                        <option key={option} value={option}>{option}</option>
                      ))}
                    </select>
                  ) : param.type === 'date' ? (
                    <input
                      type="date"
                      value={templateParams[param.name] || ''}
                      onChange={(e) => handleParamChange(param.name, e.target.value)}
                      className="w-full rounded-md border-blue-300 bg-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
                      required={param.required}
                    />
                  ) : (
                    <input
                      type={param.type === 'number' ? 'number' : 'text'}
                      value={templateParams[param.name] || ''}
                      onChange={(e) => handleParamChange(
                        param.name, 
                        param.type === 'number' ? parseFloat(e.target.value) : e.target.value
                      )}
                      min={param.validation?.min}
                      max={param.validation?.max}
                      pattern={param.validation?.pattern}
                      className="w-full rounded-md border-blue-300 bg-white shadow-sm focus:border-blue-500 focus:ring-blue-500"
                      placeholder={param.default?.toString() || `Enter ${param.name}`}
                      required={param.required}
                    />
                  )}
                  {errors[`template_${param.name}`] && (
                    <p className="text-sm text-red-600">{errors[`template_${param.name}`]}</p>
                  )}
                </div>
              ))}
            </div>

            {showTemplatePreview && (
              <div className="mt-6 border-t border-blue-200 pt-4">
                <h5 className="font-medium text-blue-900 mb-3">Generated CCL Code</h5>
                <div className="relative">
                  <pre className="bg-white border border-blue-200 rounded-lg p-4 text-sm overflow-x-auto text-gray-900 font-mono">
                    {generatePreview()}
                  </pre>
                  <button
                    type="button"
                    onClick={() => navigator.clipboard?.writeText(generatePreview())}
                    className="absolute top-2 right-2 p-1 text-gray-500 hover:text-gray-700 bg-white rounded border border-gray-200 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    title="Copy to clipboard"
                  >
                    <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                      <path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z" />
                      <path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z" />
                    </svg>
                  </button>
                </div>
              </div>
            )}
          </div>
        )}
      </div>

      <div className="flex justify-end space-x-3">
        <button
          type="button"
          onClick={() => {
            setSelectedTemplate(null)
            setTemplateParams({})
            setFormData({
              proposer_did: '',
              description: '',
              duration_secs: 7 * 24 * 60 * 60,
              quorum: 50,
              threshold: 0.6,
            })
          }}
          className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
        >
          Reset
        </button>
        <button
          type="submit"
          disabled={loading}
          className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
        >
          {loading ? 'Creating...' : 'Create Proposal'}
        </button>
      </div>
    </form>
  )
}

export function GovernancePage() {
  const {
    proposals,
    activeProposals,
    cclTemplates,
    loading,
    submitProposal,
    castVote,
    error
  } = useGovernance()

  const [showCreateForm, setShowCreateForm] = useState(false)
  const [votingLoading, setVotingLoading] = useState<string | null>(null)

  const handleSubmitProposal = async (request: SubmitProposalRequest) => {
    try {
      await submitProposal(request)
      setShowCreateForm(false)
    } catch (err) {
      console.error('Failed to submit proposal:', err)
    }
  }

  const handleVote = async (proposalId: string, vote: 'Yes' | 'No' | 'Abstain') => {
    setVotingLoading(proposalId)
    try {
      await castVote({
        voter_did: 'did:key:example', // In real app, get from auth context
        proposal_id: proposalId,
        vote_option: vote,
      })
    } catch (err) {
      console.error('Failed to cast vote:', err)
    } finally {
      setVotingLoading(null)
    }
  }

  return (
    <div className="space-y-8">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Governance</h1>
          <p className="text-gray-600 mt-2">
            Create proposals and participate in democratic decision-making
          </p>
        </div>
        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
        >
          Create Proposal
        </button>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
          <p className="font-medium">Error</p>
          <p className="text-sm">{error}</p>
        </div>
      )}

      {/* Create Proposal Form */}
      {showCreateForm && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Create New Proposal</h2>
          <ProposalCreationForm
            templates={cclTemplates}
            onSubmit={handleSubmitProposal}
            loading={loading.submitting}
          />
        </div>
      )}

      {/* Active Proposals */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          Active Proposals ({activeProposals.length})
        </h2>
        {loading.proposals ? (
          <div className="space-y-4">
            {[1, 2, 3].map((i) => (
              <div key={i} className="animate-pulse">
                <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
                <div className="h-3 bg-gray-200 rounded w-1/2 mb-2"></div>
                <div className="h-8 bg-gray-200 rounded w-1/4"></div>
              </div>
            ))}
          </div>
        ) : activeProposals.length > 0 ? (
          <div className="space-y-6">
            {activeProposals.map((proposal) => {
              const progress = GovernanceUtils.calculateVotingProgress(proposal)
              const hasQuorum = GovernanceUtils.hasReachedQuorum(proposal)
              const timeRemaining = proposal.voting_deadline ? ICNUtils.getTimeRemaining(proposal.voting_deadline) : 'No deadline'
              
              return (
                <div key={proposal.id} className="border border-gray-200 rounded-lg p-6">
                  <div className="flex justify-between items-start mb-4">
                    <div>
                      <h3 className="font-semibold text-gray-900">
                        {GovernanceUtils.formatProposalType(proposal.proposal_type)}
                      </h3>
                      <p className="text-gray-600 mt-1">{proposal.description}</p>
                      <p className="text-sm text-gray-500 mt-2">
                        By {proposal.proposer} • {timeRemaining} remaining
                      </p>
                    </div>
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                      {proposal.status}
                    </span>
                  </div>

                  {/* Voting Progress */}
                  <div className="mb-4">
                    <div className="flex justify-between text-sm mb-1">
                      <span>Voting Progress</span>
                      <span>{progress.toFixed(1)}% of quorum</span>
                    </div>
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div
                        className={`h-2 rounded-full transition-all duration-300 ${
                          hasQuorum ? 'bg-green-500' : 'bg-blue-500'
                        }`}
                        style={{ width: `${Math.min(progress, 100)}%` }}
                      />
                    </div>
                  </div>

                  {/* Vote Counts */}
                  <div className="flex items-center space-x-6 mb-4 text-sm">
                    <div className="flex items-center space-x-1">
                      <div className="w-3 h-3 bg-green-500 rounded-full"></div>
                      <span>Yes: {proposal.votes.yes}</span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <div className="w-3 h-3 bg-red-500 rounded-full"></div>
                      <span>No: {proposal.votes.no}</span>
                    </div>
                    <div className="flex items-center space-x-1">
                      <div className="w-3 h-3 bg-gray-400 rounded-full"></div>
                      <span>Abstain: {proposal.votes.abstain}</span>
                    </div>
                  </div>

                  {/* Voting Buttons */}
                  <div className="flex space-x-3">
                    {['Yes', 'No', 'Abstain'].map((vote) => (
                      <button
                        key={vote}
                        onClick={() => handleVote(proposal.id, vote as any)}
                        disabled={votingLoading === proposal.id}
                        className={`px-4 py-2 rounded-md text-sm font-medium border transition-colors ${
                          vote === 'Yes'
                            ? 'border-green-500 text-green-700 bg-green-50 hover:bg-green-100'
                            : vote === 'No'
                            ? 'border-red-500 text-red-700 bg-red-50 hover:bg-red-100'
                            : 'border-gray-300 text-gray-700 bg-gray-50 hover:bg-gray-100'
                        } disabled:opacity-50`}
                      >
                        {votingLoading === proposal.id ? 'Voting...' : vote}
                      </button>
                    ))}
                  </div>
                </div>
              )
            })}
          </div>
        ) : (
          <p className="text-gray-500 text-center py-8">No active proposals</p>
        )}
      </div>

      {/* All Proposals */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">
          All Proposals ({proposals.length})
        </h2>
        {proposals.length > 0 ? (
          <div className="space-y-4">
            {proposals.map((proposal) => {
              const outcome = GovernanceUtils.getProposalOutcome(proposal)
              
              return (
                <div key={proposal.id} className="flex items-center justify-between border-b border-gray-200 pb-4">
                  <div>
                    <h4 className="font-medium text-gray-900">
                      {GovernanceUtils.formatProposalType(proposal.proposal_type)}
                    </h4>
                    <p className="text-sm text-gray-600">
                      {GovernanceUtils.generateProposalSummary(proposal.proposal_type)}
                    </p>
                    <p className="text-xs text-gray-500 mt-1">
                      By {proposal.proposer} • {new Date(proposal.created_at).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="flex items-center space-x-2">
                    <span
                      className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        outcome === 'passed'
                          ? 'bg-green-100 text-green-800'
                          : outcome === 'failed'
                          ? 'bg-red-100 text-red-800'
                          : 'bg-blue-100 text-blue-800'
                      }`}
                    >
                      {outcome === 'pending' ? proposal.status : outcome}
                    </span>
                  </div>
                </div>
              )
            })}
          </div>
        ) : (
          <p className="text-gray-500 text-center py-8">No proposals found</p>
        )}
      </div>
    </div>
  )
}