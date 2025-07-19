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