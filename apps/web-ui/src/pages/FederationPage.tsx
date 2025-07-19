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
    <div className="space-y-8">
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
            className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-green-600 hover:bg-green-700"
          >
            Join Federation
          </button>
          <button
            onClick={() => setShowCreateForm(!showCreateForm)}
            className="px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700"
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
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Create New Federation</h2>
          <CreateFederationForm onSubmit={handleCreateFederation} loading={actionLoading} />
        </div>
      )}

      {/* Join Federation Form */}
      {showJoinForm && (
        <div className="bg-white rounded-lg border border-gray-200 p-6">
          <h2 className="text-lg font-semibold text-gray-900 mb-4">Join Existing Federation</h2>
          <JoinFederationForm onSubmit={handleJoinFederation} loading={actionLoading} />
        </div>
      )}

      {/* Current Federation Status */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Current Federation</h2>
        {loading.federationStatus ? (
          <div className="animate-pulse space-y-4">
            <div className="h-4 bg-gray-200 rounded w-3/4"></div>
            <div className="h-4 bg-gray-200 rounded w-1/2"></div>
          </div>
        ) : federationStatus ? (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
              <div>
                <h3 className="font-medium text-gray-900">Federation Name</h3>
                <p className="text-gray-600">{metadata?.name || 'Unknown Federation'}</p>
              </div>
              <div>
                <h3 className="font-medium text-gray-900">Connected Peers</h3>
                <p className="text-gray-600">{federationStatus.peer_count}</p>
              </div>
              <div>
                <h3 className="font-medium text-gray-900">Total Cooperatives</h3>
                <p className="text-gray-600">{metadata?.totalCooperatives || 0}</p>
              </div>
            </div>

            {federationStatus.peers.length > 0 && (
              <div>
                <h3 className="font-medium text-gray-900 mb-2">Connected Peers</h3>
                <div className="space-y-2">
                  {federationStatus.peers.map((peer, index) => (
                    <div key={index} className="flex items-center justify-between bg-gray-50 px-3 py-2 rounded">
                      <span className="font-mono text-sm">{peer}</span>
                      <button
                        onClick={() => handleLeaveFederation(peer)}
                        className="text-red-600 hover:text-red-800 text-sm"
                      >
                        Leave
                      </button>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        ) : (
          <p className="text-gray-500">Not connected to any federation</p>
        )}
      </div>

      {/* Member Cooperatives */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Member Cooperatives</h2>
        {loading.cooperatives ? (
          <div className="space-y-4">
            {[1, 2, 3].map((i) => (
              <div key={i} className="animate-pulse">
                <div className="h-4 bg-gray-200 rounded w-3/4 mb-2"></div>
                <div className="h-3 bg-gray-200 rounded w-1/2"></div>
              </div>
            ))}
          </div>
        ) : cooperatives.length > 0 ? (
          <div className="grid gap-6">
            {cooperatives.map((coop) => (
              <div key={coop.did} className="border border-gray-200 rounded-lg p-4">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <h3 className="font-semibold text-gray-900">{coop.name}</h3>
                    <p className="text-gray-600 text-sm mt-1">{coop.description}</p>
                    <div className="flex items-center space-x-4 mt-2 text-sm text-gray-500">
                      <span>{coop.memberCount} members</span>
                      <span>{coop.reputation}% reputation</span>
                      <span>Joined {new Date(coop.joinedAt).toLocaleDateString()}</span>
                    </div>
                    <div className="flex flex-wrap gap-1 mt-2">
                      {coop.capabilities.map((capability) => (
                        <span
                          key={capability}
                          className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800"
                        >
                          {capability}
                        </span>
                      ))}
                    </div>
                  </div>
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
              </div>
            ))}
          </div>
        ) : (
          <p className="text-gray-500 text-center py-8">No cooperatives in federation</p>
        )}
      </div>
    </div>
  )
}