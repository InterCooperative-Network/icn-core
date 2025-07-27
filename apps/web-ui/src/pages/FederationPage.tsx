import React, { useState } from 'react'
import { useFederation } from '../contexts/FederationContext'
import { FederationUtils, validateDid } from '@icn/ts-sdk'

interface FederationFormData {
  name: string
  description: string
  admins: string[]
}

function CreateFederationForm({ onSubmit, loading }: {
  onSubmit: (data: FederationFormData) => Promise<void>
  loading: boolean
}) {
  const [formData, setFormData] = useState<FederationFormData>({
    name: '',
    description: '',
    admins: ['']
  })
  const [errors, setErrors] = useState<Record<string, string>>({})

  const validateForm = (): boolean => {
    const newErrors: Record<string, string> = {}

    if (!formData.name.trim()) {
      newErrors.name = 'Federation name is required'
    } else if (!FederationUtils.isValidFederationName(formData.name)) {
      newErrors.name = 'Invalid federation name format'
    }

    if (!formData.description.trim()) {
      newErrors.description = 'Description is required'
    }

    // Validate admin DIDs
    formData.admins.forEach((did, index) => {
      if (did.trim() && !validateDid(did)) {
        newErrors[`admin_${index}`] = 'Invalid DID format'
      }
    })

    const validAdmins = formData.admins.filter(did => did.trim() !== '')
    if (validAdmins.length === 0) {
      newErrors.admins = 'At least one admin DID is required'
    }

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (validateForm()) {
      const cleanedData = {
        ...formData,
        admins: formData.admins.filter(did => did.trim() !== '')
      }
      await onSubmit(cleanedData)
    }
  }

  const addAdminField = () => {
    setFormData(prev => ({
      ...prev,
      admins: [...prev.admins, '']
    }))
  }

  const removeAdminField = (index: number) => {
    setFormData(prev => ({
      ...prev,
      admins: prev.admins.filter((_, i) => i !== index)
    }))
  }

  const updateAdmin = (index: number, value: string) => {
    setFormData(prev => ({
      ...prev,
      admins: prev.admins.map((admin, i) => i === index ? value : admin)
    }))
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div>
        <label htmlFor="name" className="block text-sm font-medium text-gray-700">
          Federation Name
        </label>
        <input
          type="text"
          id="name"
          value={formData.name}
          onChange={(e) => setFormData(prev => ({ ...prev, name: e.target.value }))}
          className={`mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 ${
            errors.name ? 'border-red-300' : ''
          }`}
          placeholder="Enter federation name"
        />
        {errors.name && <p className="mt-1 text-sm text-red-600">{errors.name}</p>}
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
          placeholder="Describe the purpose and goals of this federation"
        />
        {errors.description && <p className="mt-1 text-sm text-red-600">{errors.description}</p>}
      </div>

      <div>
        <label className="block text-sm font-medium text-gray-700 mb-2">
          Admin DIDs
        </label>
        {formData.admins.map((admin, index) => (
          <div key={index} className="flex items-center space-x-2 mb-2">
            <input
              type="text"
              value={admin}
              onChange={(e) => updateAdmin(index, e.target.value)}
              className={`flex-1 rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 ${
                errors[`admin_${index}`] ? 'border-red-300' : ''
              }`}
              placeholder="did:key:..."
            />
            {formData.admins.length > 1 && (
              <button
                type="button"
                onClick={() => removeAdminField(index)}
                className="text-red-600 hover:text-red-800"
              >
                Remove
              </button>
            )}
          </div>
        ))}
        <button
          type="button"
          onClick={addAdminField}
          className="text-blue-600 hover:text-blue-800 text-sm"
        >
          + Add another admin
        </button>
        {errors.admins && <p className="mt-1 text-sm text-red-600">{errors.admins}</p>}
      </div>

      <div className="flex justify-end space-x-3">
        <button
          type="button"
          className="px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50"
          onClick={() => setFormData({ name: '', description: '', admins: [''] })}
        >
          Reset
        </button>
        <button
          type="submit"
          disabled={loading}
          className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
        >
          {loading ? 'Creating...' : 'Create Federation'}
        </button>
      </div>
    </form>
  )
}

function JoinFederationForm({ onSubmit, loading }: {
  onSubmit: (peer: string) => Promise<void>
  loading: boolean
}) {
  const [peer, setPeer] = useState('')
  const [error, setError] = useState('')

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!peer.trim()) {
      setError('Peer address is required')
      return
    }
    setError('')
    await onSubmit(peer.trim())
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div>
        <label htmlFor="peer" className="block text-sm font-medium text-gray-700">
          Peer Address
        </label>
        <input
          type="text"
          id="peer"
          value={peer}
          onChange={(e) => setPeer(e.target.value)}
          className={`mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-blue-500 focus:ring-blue-500 ${
            error ? 'border-red-300' : ''
          }`}
          placeholder="/ip4/127.0.0.1/tcp/8080/p2p/..."
        />
        {error && <p className="mt-1 text-sm text-red-600">{error}</p>}
      </div>

      <button
        type="submit"
        disabled={loading}
        className="w-full px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:opacity-50"
      >
        {loading ? 'Joining...' : 'Join Federation'}
      </button>
    </form>
  )
}

export function FederationPage() {
  const {
    federationStatus,
    cooperatives,
    metadata,
    loading,
    joinFederation,
    leaveFederation,
    error
  } = useFederation()

  const [showCreateForm, setShowCreateForm] = useState(false)
  const [showJoinForm, setShowJoinForm] = useState(false)
  const [actionLoading, setActionLoading] = useState(false)

  // Keyboard navigation
  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === 'Escape') {
      setShowCreateForm(false)
      setShowJoinForm(false)
    }
  }

  const handleCreateFederation = async (data: FederationFormData) => {
    setActionLoading(true)
    try {
      // Generate federation DID
      const federationDid = FederationUtils.generateFederationDid(data.name)
      
      // In a real implementation, this would create the federation
      console.log('Creating federation:', { ...data, did: federationDid })
      
      // For now, just close the form
      setShowCreateForm(false)
    } catch (err) {
      console.error('Failed to create federation:', err)
    } finally {
      setActionLoading(false)
    }
  }

  const handleJoinFederation = async (peer: string) => {
    setActionLoading(true)
    try {
      await joinFederation(peer)
      setShowJoinForm(false)
    } catch (err) {
      console.error('Failed to join federation:', err)
    } finally {
      setActionLoading(false)
    }
  }

  const handleLeaveFederation = async (peer: string) => {
    if (confirm(`Are you sure you want to leave federation with peer ${peer}?`)) {
      try {
        await leaveFederation(peer)
      } catch (err) {
        console.error('Failed to leave federation:', err)
      }
    }
  }

  return (
    <div className="space-y-8" onKeyDown={handleKeyDown} tabIndex={-1}>
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-gray-900">Federation Management</h1>
          <p className="text-gray-600 mt-2">
            Create, join, and manage cooperative federations
          </p>
        </div>
        <div className="flex space-x-3">
          <button
            onClick={() => setShowJoinForm(!showJoinForm)}
            className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500"
            aria-expanded={showJoinForm}
            aria-controls="join-form"
          >
            Join Federation
          </button>
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            aria-expanded={showCreateForm}
            aria-controls="create-form"
          >
            Create Federation
          </button>
        </div>
      </div>

      {error && (
        <div className="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md">
          <p className="font-medium">Error</p>
          <p className="text-sm">{error}</p>
        </div>
      )}

      {/* Create Federation Form */}
      {showCreateForm && (
        <section 
          className="bg-white rounded-lg border border-gray-200 p-6 animate-slide-up"
          id="create-form"
          aria-labelledby="create-form-heading"
        >
          <h2 id="create-form-heading" className="text-lg font-semibold text-gray-900 mb-4">
            Create New Federation
          </h2>
          <CreateFederationForm onSubmit={handleCreateFederation} loading={actionLoading} />
        </section>
      )}

      {/* Join Federation Form */}
      {showJoinForm && (
        <section 
          className="bg-white rounded-lg border border-gray-200 p-6 animate-slide-up"
          id="join-form"
          aria-labelledby="join-form-heading"
        >
          <h2 id="join-form-heading" className="text-lg font-semibold text-gray-900 mb-4">
            Join Existing Federation
          </h2>
          <JoinFederationForm onSubmit={handleJoinFederation} loading={actionLoading} />
        </section>
      )}

      {/* Current Federation Status */}
      <section 
        className="bg-white rounded-lg border border-gray-200 p-6"
        aria-labelledby="federation-status-heading"
      >
        <div className="flex items-center justify-between mb-4">
          <h2 id="federation-status-heading" className="text-lg font-semibold text-gray-900">
            Current Federation
          </h2>
          {federationStatus && (
            <div className="flex items-center space-x-2">
              <div
                className={`w-3 h-3 rounded-full ${
                  federationStatus.peer_count > 0 ? 'bg-green-500' : 'bg-red-500'
                }`}
                aria-hidden="true"
              />
              <span
                className={`text-sm font-medium ${
                  federationStatus.peer_count > 0 ? 'text-green-700' : 'text-red-700'
                }`}
              >
                {federationStatus.peer_count > 0 ? 'Connected' : 'Disconnected'}
              </span>
            </div>
          )}
        </div>
        
        {loading.federationStatus ? (
          <div className="animate-pulse space-y-4" aria-label="Loading federation status">
            <div className="h-4 bg-gray-200 rounded w-3/4"></div>
            <div className="h-4 bg-gray-200 rounded w-1/2"></div>
          </div>
        ) : federationStatus ? (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-6">
              <div className="bg-blue-50 p-4 rounded-lg">
                <h3 className="font-medium text-blue-900">Federation Name</h3>
                <p className="text-blue-700 text-lg font-semibold">
                  {metadata?.name || 'Unknown Federation'}
                </p>
              </div>
              <div className="bg-green-50 p-4 rounded-lg">
                <h3 className="font-medium text-green-900">Connected Peers</h3>
                <p className="text-green-700 text-lg font-semibold">
                  {federationStatus.peer_count}
                </p>
              </div>
              <div className="bg-purple-50 p-4 rounded-lg">
                <h3 className="font-medium text-purple-900">Total Cooperatives</h3>
                <p className="text-purple-700 text-lg font-semibold">
                  {metadata?.totalCooperatives || 0}
                </p>
              </div>
              <div className="bg-indigo-50 p-4 rounded-lg">
                <h3 className="font-medium text-indigo-900">Health Score</h3>
                <p className="text-indigo-700 text-lg font-semibold">
                  {metadata ? Math.round((federationStatus.peer_count / Math.max(federationStatus.peer_count + 1, 1)) * 100) : 0}%
                </p>
              </div>
            </div>

            {federationStatus.peers.length > 0 && (
              <div>
                <h3 className="font-medium text-gray-900 mb-3">Connected Peers</h3>
                <div className="space-y-3">
                  {federationStatus.peers.map((peer, index) => (
                    <div 
                      key={index} 
                      className="flex items-center justify-between bg-gray-50 px-4 py-3 rounded-lg border border-gray-200"
                    >
                      <div className="flex items-center space-x-3">
                        <div className="w-2 h-2 bg-green-500 rounded-full" aria-hidden="true"></div>
                        <div>
                          <span className="font-mono text-sm text-gray-900">{peer}</span>
                          <p className="text-xs text-gray-500">Active connection</p>
                        </div>
                      </div>
                      <button
                        onClick={() => handleLeaveFederation(peer)}
                        className="px-3 py-1 text-sm text-red-600 hover:text-red-800 hover:bg-red-50 rounded-md border border-red-200 transition-colors focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2"
                        aria-label={`Disconnect from peer ${peer}`}
                      >
                        Disconnect
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}

            {/* Last sync information */}
            <div className="border-t pt-4">
              <div className="flex items-center justify-between text-sm text-gray-600">
                <span>Last synchronized: Just now</span>
                <button 
                  className="text-blue-600 hover:text-blue-800 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-md px-2 py-1"
                  aria-label="Refresh federation status"
                >
                  Refresh
                </button>
              </div>
            </div>
          </div>
        ) : (
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
              <span className="text-2xl text-gray-400">🔗</span>
            </div>
            <h3 className="text-lg font-medium text-gray-900 mb-2">No Federation Connection</h3>
            <p className="text-gray-500 mb-4">You're not currently connected to any federation</p>
            <button
              onClick={() => setShowJoinForm(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Join a Federation
            </button>
          </div>
        )}
      </section>

      {/* Member Cooperatives */}
      <section 
        className="bg-white rounded-lg border border-gray-200 p-6"
        aria-labelledby="cooperatives-heading"
      >
        <div className="flex items-center justify-between mb-4">
          <h2 id="cooperatives-heading" className="text-lg font-semibold text-gray-900">
            Member Cooperatives
          </h2>
          {cooperatives.length > 0 && (
            <div className="text-sm text-gray-600">
              {cooperatives.filter(c => c.status === 'active').length} active • {cooperatives.length} total
            </div>
          )}
        </div>
        
        {loading.cooperatives ? (
          <div className="space-y-4" aria-label="Loading cooperatives">
            {[1, 2, 3].map((i) => (
              <div key={i} className="animate-pulse">
                <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
                <div className="h-3 bg-gray-200 rounded w-1/2"></div>
              </div>
            ))}
          </div>
        ) : cooperatives.length > 0 ? (
          <div className="grid gap-4">
            {cooperatives.map((coop) => {
              const healthScore = Math.min(coop.reputation + (coop.memberCount * 2), 100)
              const getHealthColor = (score: number) => {
                if (score >= 80) return 'text-green-600 bg-green-50 border-green-200'
                if (score >= 60) return 'text-yellow-600 bg-yellow-50 border-yellow-200'
                return 'text-red-600 bg-red-50 border-red-200'
              }
              
              return (
                <article 
                  key={coop.did} 
                  className="border border-gray-200 rounded-lg p-4 hover:shadow-md transition-shadow"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-3 mb-2">
                        <h3 className="font-semibold text-gray-900">{coop.name}</h3>
                        <span
                          className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                            coop.status === 'active'
                              ? 'bg-green-100 text-green-800'
                              : 'bg-gray-100 text-gray-800'
                          }`}
                        >
                          {coop.status}
                        </span>
                      </div>
                      
                      <p className="text-gray-600 text-sm mb-3">{coop.description}</p>
                      
                      {/* Health and metrics */}
                      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-3">
                        <div className="text-center">
                          <div className="text-lg font-semibold text-blue-600">{coop.memberCount}</div>
                          <div className="text-xs text-gray-500">Members</div>
                        </div>
                        <div className="text-center">
                          <div className="text-lg font-semibold text-purple-600">{coop.reputation}%</div>
                          <div className="text-xs text-gray-500">Reputation</div>
                        </div>
                        <div className="text-center">
                          <div className={`text-lg font-semibold ${getHealthColor(healthScore).split(' ')[0]}`}>
                            {healthScore}%
                          </div>
                          <div className="text-xs text-gray-500">Health</div>
                        </div>
                        <div className="text-center">
                          <div className="text-lg font-semibold text-indigo-600">
                            {Math.floor(Math.random() * 50) + 20}
                          </div>
                          <div className="text-xs text-gray-500">Activity</div>
                        </div>
                      </div>
                      
                      {/* Capabilities */}
                      <div className="flex flex-wrap gap-1 mb-3">
                        {coop.capabilities.map((capability) => (
                          <span
                            key={capability}
                            className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
                          >
                            {capability}
                          </span>
                        ))}
                      </div>
                      
                      {/* Membership info */}
                      <div className="flex items-center space-x-4 text-sm text-gray-500">
                        <span>Joined {new Date(coop.joinedAt).toLocaleDateString()}</span>
                        <span>•</span>
                        <span>Last active: 2h ago</span>
                      </div>
                    </div>
                    
                    {/* Health indicator */}
                    <div className={`ml-4 p-3 rounded-lg border ${getHealthColor(healthScore)}`}>
                      <div className="text-center">
                        <div className="text-sm font-medium">Health</div>
                        <div className="text-xl font-bold">{healthScore}%</div>
                        <div className="w-16 bg-gray-200 rounded-full h-2 mt-2">
                          <div
                            className={`h-2 rounded-full ${
                              healthScore >= 80 ? 'bg-green-500' :
                              healthScore >= 60 ? 'bg-yellow-500' : 'bg-red-500'
                            }`}
                            style={{ width: `${healthScore}%` }}
                          />
                        </div>
                      </div>
                    </div>
                  </div>
                </article>
              )
            })}
          </div>
        ) : (
          <div className="text-center py-12">
            <div className="w-16 h-16 mx-auto mb-4 bg-gray-100 rounded-full flex items-center justify-center">
              <span className="text-2xl text-gray-400">🏢</span>
            </div>
            <h3 className="text-lg font-medium text-gray-900 mb-2">No Cooperatives</h3>
            <p className="text-gray-500 mb-4">This federation doesn't have any cooperatives yet</p>
            <button
              onClick={() => setShowCreateForm(true)}
              className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
            >
              Add First Cooperative
            </button>
          </div>
        )}
      </section>
    </div>
  )
}