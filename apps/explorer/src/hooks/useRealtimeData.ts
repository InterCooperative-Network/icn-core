import { useState, useEffect, useCallback } from 'react'
import type { WebSocketMessage } from './useWebSocket'

export interface DAGData {
  blocks: Array<{
    cid: string
    data: any
    links: string[]
    timestamp: number
    author: string
    size: number
  }>
  totalSize: number
  blockCount: number
  recentActivity: Array<{
    type: 'put' | 'get'
    cid: string
    timestamp: number
  }>
}

export interface JobData {
  active: Array<{
    id: string
    submitter: string
    executor?: string
    status: 'pending' | 'assigned' | 'executing' | 'completed' | 'failed'
    specification: any
    cost: number
    timestamp: number
    duration?: number
  }>
  pending: Array<any>
  completed: Array<any>
  failed: Array<any>
  metrics: {
    totalJobs: number
    successRate: number
    avgDuration: number
    throughput: number
  }
}

export interface NetworkData {
  peers: Array<{
    id: string
    multiaddr: string
    protocols: string[]
    latency?: number
    connected: boolean
    lastSeen: number
  }>
  connections: Array<{
    local: string
    remote: string
    direction: 'inbound' | 'outbound'
    protocol: string
    timestamp: number
  }>
  metrics: {
    peerCount: number
    messagesSent: number
    messagesReceived: number
    bytesTransferred: number
  }
}

export interface EconomicData {
  manaSupply: {
    total: number
    circulation: number
    regenerationRate: number
  }
  accounts: Array<{
    did: string
    balance: number
    capacity: number
    lastActivity: number
  }>
  transactions: Array<{
    from: string
    to?: string
    amount: number
    type: 'spend' | 'credit' | 'regenerate'
    timestamp: number
  }>
  metrics: {
    velocity: number
    utilizationRate: number
    averageBalance: number
  }
}

export interface UseRealtimeDataReturn {
  dagData: DAGData | null
  jobData: JobData | null
  networkData: NetworkData | null
  economicData: EconomicData | null
  refreshData: () => Promise<void>
  lastUpdate: number | null
  isLoading: boolean
  error: Error | null
}

export function useRealtimeData(
  client: any,
  lastMessage: WebSocketMessage | null
): UseRealtimeDataReturn {
  const [dagData, setDAGData] = useState<DAGData | null>(null)
  const [jobData, setJobData] = useState<JobData | null>(null)
  const [networkData, setNetworkData] = useState<NetworkData | null>(null)
  const [economicData, setEconomicData] = useState<EconomicData | null>(null)
  const [lastUpdate, setLastUpdate] = useState<number | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<Error | null>(null)

  // Fetch initial data
  const fetchDAGData = useCallback(async (): Promise<DAGData | null> => {
    if (!client) return null
    
    try {
      const [blocksResponse, statsResponse] = await Promise.all([
        client.getDAGBlocks({ limit: 100 }),
        client.getDAGStats()
      ])
      
      return {
        blocks: blocksResponse.blocks || [],
        totalSize: statsResponse.totalSize || 0,
        blockCount: statsResponse.blockCount || 0,
        recentActivity: statsResponse.recentActivity || []
      }
    } catch (err) {
      console.error('Failed to fetch DAG data:', err)
      throw err
    }
  }, [client])

  const fetchJobData = useCallback(async (): Promise<JobData | null> => {
    if (!client) return null
    
    try {
      const [activeResponse, metricsResponse] = await Promise.all([
        client.getJobs({ status: 'all', limit: 100 }),
        client.getJobMetrics()
      ])
      
      const jobs = activeResponse.jobs || []
      
      return {
        active: jobs.filter((j: any) => ['pending', 'assigned', 'executing'].includes(j.status)),
        pending: jobs.filter((j: any) => j.status === 'pending'),
        completed: jobs.filter((j: any) => j.status === 'completed'),
        failed: jobs.filter((j: any) => j.status === 'failed'),
        metrics: {
          totalJobs: jobs.length,
          successRate: metricsResponse.successRate || 0,
          avgDuration: metricsResponse.avgDuration || 0,
          throughput: metricsResponse.throughput || 0
        }
      }
    } catch (err) {
      console.error('Failed to fetch job data:', err)
      throw err
    }
  }, [client])

  const fetchNetworkData = useCallback(async (): Promise<NetworkData | null> => {
    if (!client) return null
    
    try {
      const [peersResponse, metricsResponse] = await Promise.all([
        client.getNetworkPeers(),
        client.getNetworkMetrics()
      ])
      
      return {
        peers: peersResponse.peers || [],
        connections: peersResponse.connections || [],
        metrics: {
          peerCount: metricsResponse.peerCount || 0,
          messagesSent: metricsResponse.messagesSent || 0,
          messagesReceived: metricsResponse.messagesReceived || 0,
          bytesTransferred: metricsResponse.bytesTransferred || 0
        }
      }
    } catch (err) {
      console.error('Failed to fetch network data:', err)
      throw err
    }
  }, [client])

  const fetchEconomicData = useCallback(async (): Promise<EconomicData | null> => {
    if (!client) return null
    
    try {
      const [accountsResponse, transactionsResponse, metricsResponse] = await Promise.all([
        client.getManaAccounts({ limit: 100 }),
        client.getManaTransactions({ limit: 100 }),
        client.getEconomicMetrics()
      ])
      
      return {
        manaSupply: {
          total: metricsResponse.totalMana || 0,
          circulation: metricsResponse.circulatingMana || 0,
          regenerationRate: metricsResponse.regenerationRate || 0
        },
        accounts: accountsResponse.accounts || [],
        transactions: transactionsResponse.transactions || [],
        metrics: {
          velocity: metricsResponse.velocity || 0,
          utilizationRate: metricsResponse.utilizationRate || 0,
          averageBalance: metricsResponse.averageBalance || 0
        }
      }
    } catch (err) {
      console.error('Failed to fetch economic data:', err)
      throw err
    }
  }, [client])

  const refreshData = useCallback(async () => {
    if (!client || isLoading) return
    
    setIsLoading(true)
    setError(null)
    
    try {
      const [dag, jobs, network, economics] = await Promise.all([
        fetchDAGData(),
        fetchJobData(),
        fetchNetworkData(),
        fetchEconomicData()
      ])
      
      setDAGData(dag)
      setJobData(jobs)
      setNetworkData(network)
      setEconomicData(economics)
      setLastUpdate(Date.now())
    } catch (err) {
      setError(err as Error)
    } finally {
      setIsLoading(false)
    }
  }, [client, isLoading, fetchDAGData, fetchJobData, fetchNetworkData, fetchEconomicData])

  // Handle real-time WebSocket updates
  useEffect(() => {
    if (!lastMessage) return

    try {
      const { type, data } = lastMessage

      switch (type) {
        case 'dag_block_added':
          setDAGData(prev => prev ? {
            ...prev,
            blocks: [data.block, ...prev.blocks.slice(0, 99)],
            blockCount: prev.blockCount + 1,
            totalSize: prev.totalSize + (data.block.size || 0),
            recentActivity: [{
              type: 'put',
              cid: data.block.cid,
              timestamp: Date.now()
            }, ...prev.recentActivity.slice(0, 19)]
          } : null)
          break

        case 'job_status_changed':
          setJobData(prev => {
            if (!prev) return null
            
            const updatedJobs = prev.active.map(job => 
              job.id === data.jobId ? { ...job, ...data.update } : job
            )
            
            return {
              ...prev,
              active: updatedJobs,
              pending: updatedJobs.filter(j => j.status === 'pending'),
              completed: data.update.status === 'completed' 
                ? [...prev.completed, updatedJobs.find(j => j.id === data.jobId)].filter(Boolean)
                : prev.completed,
              failed: data.update.status === 'failed'
                ? [...prev.failed, updatedJobs.find(j => j.id === data.jobId)].filter(Boolean)
                : prev.failed
            }
          })
          break

        case 'peer_connected':
          setNetworkData(prev => prev ? {
            ...prev,
            peers: [data.peer, ...prev.peers.filter(p => p.id !== data.peer.id)],
            metrics: {
              ...prev.metrics,
              peerCount: prev.metrics.peerCount + 1
            }
          } : null)
          break

        case 'peer_disconnected':
          setNetworkData(prev => prev ? {
            ...prev,
            peers: prev.peers.filter(p => p.id !== data.peerId),
            metrics: {
              ...prev.metrics,
              peerCount: Math.max(0, prev.metrics.peerCount - 1)
            }
          } : null)
          break

        case 'mana_transaction':
          setEconomicData(prev => prev ? {
            ...prev,
            transactions: [data.transaction, ...prev.transactions.slice(0, 99)]
          } : null)
          break
      }
      
      setLastUpdate(Date.now())
    } catch (err) {
      console.error('Failed to process WebSocket message:', err)
    }
  }, [lastMessage])

  // Initial data fetch
  useEffect(() => {
    if (client && !dagData && !isLoading) {
      refreshData()
    }
  }, [client, dagData, isLoading, refreshData])

  return {
    dagData,
    jobData,
    networkData,
    economicData,
    refreshData,
    lastUpdate,
    isLoading,
    error
  }
} 