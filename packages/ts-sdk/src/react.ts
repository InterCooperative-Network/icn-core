// Note: This file should only be imported in React environments
// React/React Native specific hooks and utilities

import { ICNClient } from './client'
import { ICNClientOptions } from './types'

// Context and Provider (placeholder - would require React import)
let ICNContext: any = null
let useContext: any = null
let createContext: any = null
let useState: any = null
let useEffect: any = null

// Dynamically import React if available
try {
  // This will work in React Native and web environments
  const React = require('react')
  useContext = React.useContext
  createContext = React.createContext
  useState = React.useState
  useEffect = React.useEffect
} catch (error) {
  // React not available - hooks will throw appropriate errors
}

// Create context if React is available
if (createContext) {
  ICNContext = createContext<{
    client: ICNClient | null
    connected: boolean
    connecting: boolean
    error: string | null
  }>({
    client: null,
    connected: false,
    connecting: false,
    error: null,
  })
}

// Provider component (factory function to avoid direct React dependency)
export function ICNProvider(props: {
  children: any
  options: ICNClientOptions
}) {
  if (!useState || !useEffect || !ICNContext) {
    throw new Error('ICNProvider requires React. Import this only in React environments.')
  }

  const [client, setClient] = useState<ICNClient | null>(null)
  const [connected, setConnected] = useState(false)
  const [connecting, setConnecting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const icnClient = new ICNClient(props.options)
    setClient(icnClient)

    // Auto-connect
    setConnecting(true)
    icnClient.connect()
      .then(() => {
        setConnected(true)
        setError(null)
      })
      .catch((err) => {
        setError(err.message)
        setConnected(false)
      })
      .finally(() => {
        setConnecting(false)
      })

    return () => {
      icnClient.disconnect()
    }
  }, [props.options])

  const value = {
    client,
    connected,
    connecting,
    error,
  }

  return ICNContext.Provider({ value, children: props.children })
}

// Hook to use ICN client
export function useICNClient() {
  if (!useContext || !ICNContext) {
    throw new Error('useICNClient requires React. Import this only in React environments.')
  }

  const context = useContext(ICNContext)
  if (!context) {
    throw new Error('useICNClient must be used within an ICNProvider')
  }

  return context
}

// Hook for connection state
export function useICNConnection() {
  const { connected, connecting, error } = useICNClient()
  return { connected, connecting, error }
}

// Hook for job operations
export function useICNJobs() {
  const { client } = useICNClient()
  
  if (!useState) {
    throw new Error('useICNJobs requires React')
  }

  const [jobs, setJobs] = useState<any[]>([])
  const [loading, setLoading] = useState(false)

  const submitJob = async (jobSpec: any, options?: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      const jobId = await client.submitJob(jobSpec, options)
      // Refresh jobs list
      const updatedJobs = await client.listJobs()
      setJobs(updatedJobs)
      return jobId
    } finally {
      setLoading(false)
    }
  }

  const refreshJobs = async () => {
    if (!client) return
    
    setLoading(true)
    try {
      const jobList = await client.listJobs()
      setJobs(jobList)
    } finally {
      setLoading(false)
    }
  }

  return {
    jobs,
    loading,
    submitJob,
    refreshJobs,
  }
}

// Hook for governance operations
export function useICNGovernance() {
  const { client } = useICNClient()
  
  if (!useState) {
    throw new Error('useICNGovernance requires React')
  }

  const [proposals, setProposals] = useState<any[]>([])
  const [loading, setLoading] = useState(false)

  const submitProposal = async (proposal: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      const proposalId = await client.governance.submitProposal(proposal)
      // Refresh proposals list
      const updatedProposals = await client.governance.listProposals()
      setProposals(updatedProposals)
      return proposalId
    } finally {
      setLoading(false)
    }
  }

  const castVote = async (proposalId: string, voteOption: string) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      await client.governance.castVote({
        voter_did: client.getConnectionState().did || '',
        proposal_id: proposalId,
        vote_option: voteOption,
      })
      // Refresh proposals list
      const updatedProposals = await client.governance.listProposals()
      setProposals(updatedProposals)
    } finally {
      setLoading(false)
    }
  }

  const refreshProposals = async () => {
    if (!client) return
    
    setLoading(true)
    try {
      const proposalList = await client.governance.listProposals()
      setProposals(proposalList)
    } finally {
      setLoading(false)
    }
  }

  return {
    proposals,
    loading,
    submitProposal,
    castVote,
    refreshProposals,
  }
}

// Hook for trust operations
export function useICNTrust() {
  const { client } = useICNClient()
  
  if (!useState) {
    throw new Error('useICNTrust requires React')
  }

  const [trustScore, setTrustScore] = useState<any>(null)
  const [trustRelationships, setTrustRelationships] = useState<any[]>([])
  const [loading, setLoading] = useState(false)

  const getTrustScore = async (did: string) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      const score = await client.trust.getTrustScore(did)
      setTrustScore(score)
      return score
    } finally {
      setLoading(false)
    }
  }

  const updateTrustRelationship = async (request: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      await client.trust.updateTrustRelationship(request)
      // Refresh trust relationships if we have an entity
      if (trustScore?.did) {
        const relationships = await client.trust.getEntityTrustRelationships(trustScore.did)
        setTrustRelationships(relationships)
      }
    } finally {
      setLoading(false)
    }
  }

  const findTrustPaths = async (request: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      return await client.trust.findTrustPaths(request)
    } finally {
      setLoading(false)
    }
  }

  return {
    trustScore,
    trustRelationships,
    loading,
    getTrustScore,
    updateTrustRelationship,
    findTrustPaths,
  }
}

// Hook for credential operations
export function useICNCredentials() {
  const { client } = useICNClient()
  
  if (!useState) {
    throw new Error('useICNCredentials requires React')
  }

  const [credentials, setCredentials] = useState<any[]>([])
  const [loading, setLoading] = useState(false)

  const issueCredential = async (request: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      const response = await client.credentials.issueCredential(request)
      // Refresh credentials list
      const updatedCredentials = await client.credentials.listCredentials({})
      setCredentials(updatedCredentials.credentials)
      return response
    } finally {
      setLoading(false)
    }
  }

  const presentCredential = async (request: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      return await client.credentials.presentCredential(request)
    } finally {
      setLoading(false)
    }
  }

  const verifyCredential = async (request: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      return await client.credentials.verifyCredential(request)
    } finally {
      setLoading(false)
    }
  }

  const refreshCredentials = async (filter?: any) => {
    if (!client) return
    
    setLoading(true)
    try {
      const response = await client.credentials.listCredentials(filter || {})
      setCredentials(response.credentials)
    } finally {
      setLoading(false)
    }
  }

  return {
    credentials,
    loading,
    issueCredential,
    presentCredential,
    verifyCredential,
    refreshCredentials,
  }
}

// Hook for token operations
export function useICNTokens() {
  const { client } = useICNClient()
  
  if (!useState) {
    throw new Error('useICNTokens requires React')
  }

  const [balances, setBalances] = useState<any[]>([])
  const [tokenClasses, setTokenClasses] = useState<any[]>([])
  const [loading, setLoading] = useState(false)

  const getBalances = async (did: string) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      const tokenBalances = await client.tokens.listBalances(did)
      setBalances(tokenBalances)
      return tokenBalances
    } finally {
      setLoading(false)
    }
  }

  const transferTokens = async (request: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      await client.tokens.transferTokens(request)
      // Refresh balances for the sender
      const updatedBalances = await client.tokens.listBalances(request.from_did)
      setBalances(updatedBalances)
    } finally {
      setLoading(false)
    }
  }

  const createTokenClass = async (request: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      const tokenClass = await client.tokens.createTokenClass(request)
      // Add to local token classes list
      setTokenClasses(prev => [...prev, tokenClass])
      return tokenClass
    } finally {
      setLoading(false)
    }
  }

  return {
    balances,
    tokenClasses,
    loading,
    getBalances,
    transferTokens,
    createTokenClass,
  }
}

// Hook for mutual aid operations
export function useICNMutualAid() {
  const { client } = useICNClient()
  
  if (!useState) {
    throw new Error('useICNMutualAid requires React')
  }

  const [resources, setResources] = useState<any[]>([])
  const [loading, setLoading] = useState(false)

  const registerResource = async (resource: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      await client.mutualAid.registerResource(resource)
      // Refresh resources list
      const updatedResources = await client.mutualAid.listResources()
      setResources(updatedResources)
    } finally {
      setLoading(false)
    }
  }

  const updateResource = async (id: string, resource: any) => {
    if (!client) throw new Error('ICN client not available')
    
    setLoading(true)
    try {
      await client.mutualAid.updateResource(id, resource)
      // Refresh resources list
      const updatedResources = await client.mutualAid.listResources()
      setResources(updatedResources)
    } finally {
      setLoading(false)
    }
  }

  const refreshResources = async () => {
    if (!client) return
    
    setLoading(true)
    try {
      const resourceList = await client.mutualAid.listResources()
      setResources(resourceList)
    } finally {
      setLoading(false)
    }
  }

  return {
    resources,
    loading,
    registerResource,
    updateResource,
    refreshResources,
  }
} 