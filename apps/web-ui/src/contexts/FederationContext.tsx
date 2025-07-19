import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react'
import { useICNClient } from '@icn/ts-sdk'
import type { 
  FederationStatus, 
  CooperativeInfo, 
  FederationMetadata,
  NodeInfo,
  NodeStatus 
} from '@icn/ts-sdk'

interface FederationContextType {
  // Federation state
  federationStatus: FederationStatus | null
  cooperatives: CooperativeInfo[]
  metadata: FederationMetadata | null
  
  // Node information
  nodeInfo: NodeInfo | null
  nodeStatus: NodeStatus | null
  
  // Loading states
  loading: {
    federationStatus: boolean
    cooperatives: boolean
    metadata: boolean
    nodeInfo: boolean
  }
  
  // Actions
  joinFederation: (peer: string) => Promise<void>
  leaveFederation: (peer: string) => Promise<void>
  refreshFederationData: () => Promise<void>
  
  // Error state
  error: string | null
}

const FederationContext = createContext<FederationContextType | undefined>(undefined)

interface FederationProviderProps {
  children: ReactNode
}

export function FederationProvider({ children }: FederationProviderProps) {
  const icnClient = useICNClient()
  
  // State
  const [federationStatus, setFederationStatus] = useState<FederationStatus | null>(null)
  const [cooperatives, setCooperatives] = useState<CooperativeInfo[]>([])
  const [metadata, setMetadata] = useState<FederationMetadata | null>(null)
  const [nodeInfo, setNodeInfo] = useState<NodeInfo | null>(null)
  const [nodeStatus, setNodeStatus] = useState<NodeStatus | null>(null)
  const [error, setError] = useState<string | null>(null)
  
  const [loading, setLoading] = useState({
    federationStatus: false,
    cooperatives: false,
    metadata: false,
    nodeInfo: false,
  })

  // Load federation status
  const loadFederationStatus = async () => {
    try {
      setLoading(prev => ({ ...prev, federationStatus: true }))
      setError(null)
      const status = await icnClient.federation.getStatus()
      setFederationStatus(status)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load federation status')
    } finally {
      setLoading(prev => ({ ...prev, federationStatus: false }))
    }
  }

  // Load node information
  const loadNodeInfo = async () => {
    try {
      setLoading(prev => ({ ...prev, nodeInfo: true }))
      const [info, status] = await Promise.all([
        icnClient.system.getInfo(),
        icnClient.system.getStatus()
      ])
      setNodeInfo(info)
      setNodeStatus(status)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load node information')
    } finally {
      setLoading(prev => ({ ...prev, nodeInfo: false }))
    }
  }

  // Load cooperatives (mock data for now - would come from API)
  const loadCooperatives = async () => {
    try {
      setLoading(prev => ({ ...prev, cooperatives: true }))
      // Mock data - in real implementation, this would come from federation API
      const mockCooperatives: CooperativeInfo[] = [
        {
          did: 'did:coop:solar-energy-collective',
          name: 'Solar Energy Collective',
          description: 'Community-owned solar power generation',
          status: 'active',
          memberCount: 45,
          reputation: 95,
          capabilities: ['energy-trading', 'grid-balancing'],
          joinedAt: '2024-01-15T10:00:00Z'
        },
        {
          did: 'did:coop:local-food-hub',
          name: 'Local Food Hub',
          description: 'Connecting local farmers with consumers',
          status: 'active',
          memberCount: 23,
          reputation: 88,
          capabilities: ['food-distribution', 'supply-chain'],
          joinedAt: '2024-02-20T14:30:00Z'
        }
      ]
      setCooperatives(mockCooperatives)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load cooperatives')
    } finally {
      setLoading(prev => ({ ...prev, cooperatives: false }))
    }
  }

  // Generate metadata from other data
  const generateMetadata = () => {
    if (!federationStatus || !cooperatives.length) return

    const activeProposals = 3 // Mock - would come from governance API
    const totalProposals = 12 // Mock
    const activeJobs = 7 // Mock - would come from mesh API
    const totalJobs = 34 // Mock

    const newMetadata: FederationMetadata = {
      name: 'Regional Cooperative Federation',
      description: 'A federation of local cooperatives working together',
      created: '2024-01-01T00:00:00Z',
      admins: ['did:key:admin1', 'did:key:admin2'],
      totalMembers: cooperatives.reduce((sum, coop) => sum + coop.memberCount, 0),
      totalCooperatives: cooperatives.length,
      governance: {
        activeProposals,
        totalProposals,
      },
      mesh: {
        activeJobs,
        totalJobs,
      },
      dag: {
        blockCount: 1547,
        syncStatus: 'synced' as const,
      }
    }

    setMetadata(newMetadata)
  }

  // Actions
  const joinFederation = async (peer: string) => {
    try {
      await icnClient.federation.joinFederation({ peer })
      await refreshFederationData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to join federation')
      throw err
    }
  }

  const leaveFederation = async (peer: string) => {
    try {
      await icnClient.federation.leaveFederation({ peer })
      await refreshFederationData()
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to leave federation')
      throw err
    }
  }

  const refreshFederationData = async () => {
    await Promise.all([
      loadFederationStatus(),
      loadCooperatives(),
      loadNodeInfo()
    ])
  }

  // Effects
  useEffect(() => {
    if (icnClient.getConnectionState().connected) {
      refreshFederationData()
    }
  }, [icnClient.getConnectionState().connected])

  useEffect(() => {
    generateMetadata()
  }, [federationStatus, cooperatives])

  const contextValue: FederationContextType = {
    federationStatus,
    cooperatives,
    metadata,
    nodeInfo,
    nodeStatus,
    loading,
    joinFederation,
    leaveFederation,
    refreshFederationData,
    error,
  }

  return (
    <FederationContext.Provider value={contextValue}>
      {children}
    </FederationContext.Provider>
  )
}

export function useFederation() {
  const context = useContext(FederationContext)
  if (context === undefined) {
    throw new Error('useFederation must be used within a FederationProvider')
  }
  return context
}