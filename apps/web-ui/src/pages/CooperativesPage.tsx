import React, { useState } from 'react'
import { useFederation } from '../contexts/FederationContext'
import { formatRelativeTime, FederationUtils } from '@icn/ts-sdk'
import type { CooperativeInfo } from '@icn/ts-sdk'

interface CooperativeFormData {
  name: string
  description: string
  capabilities: string[]
  adminDid: string
}

function CreateCooperativeForm({ onSubmit, loading }: {
  onSubmit: (data: CooperativeFormData) => Promise<void>
  loading: boolean
}) {
  const [formData, setFormData] = useState<CooperativeFormData>({
    name: '',
    description: '',
    capabilities: [],
    adminDid: ''
  })
  const [newCapability, setNewCapability] = useState('')
  const [errors, setErrors] = useState<Record<string, string>>({})

  const availableCapabilities = [
    'energy-trading',
    'grid-balancing', 
    'food-distribution',
    'supply-chain',
    'housing-management',
    'transportation',
    'education',
    'healthcare',
    'financial-services',
    'waste-management',
    'data-processing',
    'governance-services'
  ]

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {}

    if (!formData.name.trim()) {
      newErrors.name = 'Cooperative name is required'
    } else if (!FederationUtils.isValidFederationName(formData.name)) {
      newErrors.name = 'Invalid name format (3-50 characters, alphanumeric with spaces/hyphens)'
    }

    if (!formData.description.trim()) {
      newErrors.description = 'Description is required'
    }

    if (!formData.adminDid.trim()) {
      newErrors.adminDid = 'Admin DID is required'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (validateForm()) {
      await onSubmit(formData)
    }
  }

  const addCapability = (capability: string) => {
    if (!formData.capabilities.includes(capability)) {
      setFormData(prev => ({
        ...prev,
        capabilities: [...prev.capabilities, capability]
      }))
    }
  }

  const removeCapability = (capability: string) => {
    setFormData(prev => ({
      ...prev,
      capabilities: prev.capabilities.filter(c => c !== capability)
    }))
  }

  const addCustomCapability = () => {
    if (newCapability.trim() && !formData.capabilities.includes(newCapability.trim())) {
      addCapability(newCapability.trim())
      setNewCapability('')
    }
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div>
          <label htmlFor="name" className="form-label">
            Cooperative Name
          </label>
          <input
            type="text"
            id="name"
            value={formData.name}
            onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
            className={errors.name ? 'form-input-error' : 'form-input'}
            placeholder="Enter cooperative name"
          />
          {errors.name && <p className="form-error">{errors.name}</p>}
        </div>

        <div>
          <label htmlFor="adminDid" className="form-label">
            Admin DID
          </label>
          <input
            type="text"
            id="adminDid"
            value={formData.adminDid}
            onChange={(e) => setFormData(prev => ({ ...prev, adminDid: e.target.value }))}
            className={errors.adminDid ? 'form-input-error' : 'form-input'}
            placeholder="did:key:..."
          />
          {errors.adminDid && <p className="form-error">{errors.adminDid}</p>}
        </div>
      </div>

      <div>
        <label htmlFor="description" className="form-label">
          Description
        </label>
        <textarea
          id="description"
          rows={3}
          value={formData.description}
          onChange={(e) => setFormData(prev => ({ ...prev, description: e.target.value }))}
          className={errors.description ? 'form-input-error' : 'form-input'}
          placeholder="Describe the cooperative's purpose and mission"
        />
        {errors.description && <p className="form-error">{errors.description}</p>}
      </div>

      <div>
        <label className="form-label">Capabilities</label>
        <div className="mt-2">
          <div className="flex flex-wrap gap-2 mb-4">
            {formData.capabilities.map((capability) => (
              <span
                key={capability}
                className="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-blue-100 text-blue-800"
              >
                {capability}
                <button
                  type="button"
                  onClick={() => removeCapability(capability)}
                  className="ml-2 text-blue-600 hover:text-blue-800"
                >
                  ×
                </button>
              </span>
            ))}
          </div>
          
          <div className="space-y-3">
            <div>
              <p className="text-sm text-gray-600 mb-2">Select from predefined capabilities:</p>
              <div className="flex flex-wrap gap-2">
                {availableCapabilities
                  .filter(cap => !formData.capabilities.includes(cap))
                  .map((capability) => (
                    <button
                      key={capability}
                      type="button"
                      onClick={() => addCapability(capability)}
                      className="px-3 py-1 border border-gray-300 rounded-full text-sm text-gray-700 hover:bg-gray-50"
                    >
                      + {capability}
                    </button>
                  ))}
              </div>
            </div>
            
            <div className="flex space-x-2">
              <input
                type="text"
                value={newCapability}
                onChange={(e) => setNewCapability(e.target.value)}
                className="form-input flex-1"
                placeholder="Add custom capability"
                onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), addCustomCapability())}
              />
              <button
                type="button"
                onClick={addCustomCapability}
                className="btn-secondary"
              >
                Add
              </button>
            </div>
          </div>
        </div>
      </div>

      <div className="flex justify-end space-x-3">
        <button
          type="button"
          onClick={() => setFormData({ name: '', description: '', capabilities: [], adminDid: '' })}
          className="btn-secondary"
        >
          Reset
        </button>
        <button
          type="submit"
          disabled={loading}
          className="btn-primary"
        >
          {loading ? 'Creating...' : 'Create Cooperative'}
        </button>
      </div>
    </form>
  )
}

interface CooperativeCardProps {
  cooperative: CooperativeInfo
  onEdit?: (cooperative: CooperativeInfo) => void
  onRemove?: (cooperative: CooperativeInfo) => void
}

function CooperativeCard({ cooperative, onEdit, onRemove }: CooperativeCardProps) {
  const healthScore = FederationUtils.calculateHealthScore({
    totalMembers: cooperative.memberCount,
    governance: { activeProposals: 2 },
    mesh: { activeJobs: 5 },
    dag: { syncStatus: 'synced' }
  })

  const getHealthColor = (score: number) => {
    if (score >= 80) return 'text-green-600'
    if (score >= 60) return 'text-yellow-600'
    return 'text-red-600'
  }

  return (
    <div className="card">
      <div className="card-body">
        <div className="flex items-start justify-between mb-4">
          <div className="flex-1">
            <h3 className="text-lg font-semibold text-gray-900">{cooperative.name}</h3>
            <p className="text-gray-600 text-sm mt-1">{cooperative.description}</p>
            <p className="text-xs text-gray-500 mt-2 font-mono">{cooperative.did}</p>
          </div>
          <div className="flex items-center space-x-2">
            <span
              className={`status-badge ${
                cooperative.status === 'active'
                  ? 'status-badge-success'
                  : cooperative.status === 'pending'
                  ? 'status-badge-warning'
                  : 'status-badge-error'
              }`}
            >
              {cooperative.status}
            </span>
          </div>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
          <div>
            <p className="text-sm font-medium text-gray-500">Members</p>
            <p className="text-lg font-semibold text-gray-900">{cooperative.memberCount}</p>
          </div>
          <div>
            <p className="text-sm font-medium text-gray-500">Reputation</p>
            <p className="text-lg font-semibold text-gray-900">{cooperative.reputation}%</p>
          </div>
          <div>
            <p className="text-sm font-medium text-gray-500">Health</p>
            <p className={`text-lg font-semibold ${getHealthColor(healthScore)}`}>
              {healthScore}%
            </p>
          </div>
          <div>
            <p className="text-sm font-medium text-gray-500">Joined</p>
            <p className="text-sm text-gray-900">
              {formatRelativeTime(new Date(cooperative.joinedAt).getTime())}
            </p>
          </div>
        </div>

        <div className="mb-4">
          <p className="text-sm font-medium text-gray-500 mb-2">Capabilities</p>
          <div className="flex flex-wrap gap-1">
            {cooperative.capabilities.map((capability) => (
              <span
                key={capability}
                className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
              >
                {capability}
              </span>
            ))}
          </div>
        </div>

        {(onEdit || onRemove) && (
          <div className="flex justify-end space-x-2 pt-4 border-t border-gray-200">
            {onEdit && (
              <button
                onClick={() => onEdit(cooperative)}
                className="text-blue-600 hover:text-blue-800 text-sm font-medium"
              >
                Edit
              </button>
            )}
            {onRemove && (
              <button
                onClick={() => onRemove(cooperative)}
                className="text-red-600 hover:text-red-800 text-sm font-medium"
              >
                Remove
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  )
}

export function CooperativesPage() {
  const { cooperatives, loading, error } = useFederation()
  const [showCreateForm, setShowCreateForm] = useState(false)
  const [actionLoading, setActionLoading] = useState(false)
  const [searchTerm, setSearchTerm] = useState('')
  const [filterStatus, setFilterStatus] = useState<'all' | 'active' | 'pending' | 'inactive'>('all')
  const [filterCapability, setFilterCapability] = useState('')

  const handleCreateCooperative = async (data: CooperativeFormData) => {
    setActionLoading(true)
    try {
      // Generate cooperative DID
      const did = `did:coop:${data.name.toLowerCase().replace(/\s+/g, '-')}`
      
      // In a real implementation, this would call the API
      console.log('Creating cooperative:', { ...data, did })
      
      // Mock success
      setShowCreateForm(false)
    } catch (err) {
      console.error('Failed to create cooperative:', err)
    } finally {
      setActionLoading(false)
    }
  }

  const handleEditCooperative = (cooperative: CooperativeInfo) => {
    console.log('Edit cooperative:', cooperative)
    // In a real implementation, open edit modal
  }

  const handleRemoveCooperative = (cooperative: CooperativeInfo) => {
    if (confirm(`Are you sure you want to remove ${cooperative.name} from the federation?`)) {
      console.log('Remove cooperative:', cooperative)
      // In a real implementation, call API to remove
    }
  }

  // Filter cooperatives
  const filteredCooperatives = cooperatives.filter(coop => {
    const matchesSearch = !searchTerm || 
      coop.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      coop.description?.toLowerCase().includes(searchTerm.toLowerCase())
    
    const matchesStatus = filterStatus === 'all' || coop.status === filterStatus
    
    const matchesCapability = !filterCapability || 
      coop.capabilities.some(cap => cap.includes(filterCapability))
    
    return matchesSearch && matchesStatus && matchesCapability
  })

  // Get unique capabilities for filter
  const allCapabilities = Array.from(
    new Set(cooperatives.flatMap(coop => coop.capabilities))
  ).sort()

  return (
    <div className="space-y-8">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Cooperatives</h1>
          <p className="text-gray-600 mt-2">
            Manage cooperatives within the federation
          </p>
        </div>
        <button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="btn-primary"
        >
          Add Cooperative
        </button>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
          <p className="font-medium">Error</p>
          <p className="text-sm">{error}</p>
        </div>
      )}

      {/* Create Cooperative Form */}
      {showCreateForm && (
        <div className="card">
          <div className="card-header">
            <h2 className="text-lg font-semibold text-gray-900">Add New Cooperative</h2>
          </div>
          <div className="card-body">
            <CreateCooperativeForm
              onSubmit={handleCreateCooperative}
              loading={actionLoading}
            />
          </div>
        </div>
      )}

      {/* Filters */}
      <div className="card">
        <div className="card-body">
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
            <div>
              <label htmlFor="search" className="form-label">Search</label>
              <input
                type="text"
                id="search"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                className="form-input"
                placeholder="Search by name or description"
              />
            </div>
            
            <div>
              <label htmlFor="status" className="form-label">Status</label>
              <select
                id="status"
                value={filterStatus}
                onChange={(e) => setFilterStatus(e.target.value as any)}
                className="form-input"
              >
                <option value="all">All Statuses</option>
                <option value="active">Active</option>
                <option value="pending">Pending</option>
                <option value="inactive">Inactive</option>
              </select>
            </div>
            
            <div>
              <label htmlFor="capability" className="form-label">Capability</label>
              <select
                id="capability"
                value={filterCapability}
                onChange={(e) => setFilterCapability(e.target.value)}
                className="form-input"
              >
                <option value="">All Capabilities</option>
                {allCapabilities.map((capability) => (
                  <option key={capability} value={capability}>
                    {capability}
                  </option>
                ))}
              </select>
            </div>
            
            <div className="flex items-end">
              <button
                onClick={() => {
                  setSearchTerm('')
                  setFilterStatus('all')
                  setFilterCapability('')
                }}
                className="btn-secondary w-full"
              >
                Clear Filters
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Cooperatives List */}
      <div>
        <div className="flex justify-between items-center mb-6">
          <h2 className="text-xl font-semibold text-gray-900">
            Cooperatives ({filteredCooperatives.length})
          </h2>
        </div>

        {loading.cooperatives ? (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {[1, 2, 3, 4].map((i) => (
              <div key={i} className="card">
                <div className="card-body">
                  <div className="animate-pulse space-y-4">
                    <div className="h-6 bg-gray-200 rounded w-3/4"></div>
                    <div className="h-4 bg-gray-200 rounded w-full"></div>
                    <div className="h-4 bg-gray-200 rounded w-2/3"></div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="h-8 bg-gray-200 rounded"></div>
                      <div className="h-8 bg-gray-200 rounded"></div>
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        ) : filteredCooperatives.length > 0 ? (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {filteredCooperatives.map((cooperative) => (
              <CooperativeCard
                key={cooperative.did}
                cooperative={cooperative}
                onEdit={handleEditCooperative}
                onRemove={handleRemoveCooperative}
              />
            ))}
          </div>
        ) : (
          <div className="text-center py-12">
            <div className="text-gray-500 text-lg mb-2">No cooperatives found</div>
            <p className="text-gray-400">
              {searchTerm || filterStatus !== 'all' || filterCapability
                ? 'Try adjusting your filters'
                : 'Add the first cooperative to get started'}
            </p>
          </div>
        )}
      </div>
    </div>
  )
}