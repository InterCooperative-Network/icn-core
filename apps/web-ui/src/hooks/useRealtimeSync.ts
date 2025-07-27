import { useEffect, useRef } from 'react'

interface SyncData {
  timestamp: number
  data: any
  source: string
}

/**
 * A simple CRDT-like real-time synchronization system using browser storage events
 * This enables synchronization across multiple browser tabs/windows
 */
export function useRealtimeSync<T>(
  key: string,
  initialData: T,
  onDataChange: (data: T) => void
) {
  const lastUpdateRef = useRef<number>(0)
  const sourceIdRef = useRef<string>(`tab-${Date.now()}-${Math.random()}`)

  // Broadcast data change to other tabs
  const broadcastChange = (data: T) => {
    const syncData: SyncData = {
      timestamp: Date.now(),
      data,
      source: sourceIdRef.current
    }
    
    try {
      localStorage.setItem(`sync-${key}`, JSON.stringify(syncData))
      lastUpdateRef.current = syncData.timestamp
    } catch (error) {
      console.warn('Failed to broadcast sync data:', error)
    }
  }

  // Listen for changes from other tabs
  useEffect(() => {
    const handleStorageChange = (event: StorageEvent) => {
      if (event.key === `sync-${key}` && event.newValue) {
        try {
          const syncData: SyncData = JSON.parse(event.newValue)
          
          // Only process if this change is from another source and is newer
          if (
            syncData.source !== sourceIdRef.current &&
            syncData.timestamp > lastUpdateRef.current
          ) {
            lastUpdateRef.current = syncData.timestamp
            onDataChange(syncData.data)
          }
        } catch (error) {
          console.warn('Failed to parse sync data:', error)
        }
      }
    }

    window.addEventListener('storage', handleStorageChange)
    return () => window.removeEventListener('storage', handleStorageChange)
  }, [key, onDataChange])

  // Periodic sync check (fallback for missed events)
  useEffect(() => {
    const interval = setInterval(() => {
      try {
        const stored = localStorage.getItem(`sync-${key}`)
        if (stored) {
          const syncData: SyncData = JSON.parse(stored)
          if (
            syncData.source !== sourceIdRef.current &&
            syncData.timestamp > lastUpdateRef.current
          ) {
            lastUpdateRef.current = syncData.timestamp
            onDataChange(syncData.data)
          }
        }
      } catch (error) {
        // Ignore parse errors
      }
    }, 2000) // Check every 2 seconds

    return () => clearInterval(interval)
  }, [key, onDataChange])

  return {
    broadcastChange,
    sourceId: sourceIdRef.current
  }
}

/**
 * Hook for real-time collaborative form editing
 */
export function useCollaborativeForm<T extends Record<string, any>>(
  formId: string,
  initialValues: T,
  onRemoteChange?: (values: T, field?: string) => void
) {
  const { broadcastChange } = useRealtimeSync(
    `form-${formId}`,
    initialValues,
    (data: { values: T; field?: string; timestamp: number }) => {
      onRemoteChange?.(data.values, data.field)
    }
  )

  const broadcastFieldChange = (field: string, value: any, allValues: T) => {
    broadcastChange({
      values: allValues,
      field,
      timestamp: Date.now()
    })
  }

  return {
    broadcastFieldChange
  }
}

/**
 * Hook for real-time dashboard updates
 */
export function useDashboardSync(
  onDataUpdate: (data: any) => void
) {
  return useRealtimeSync('dashboard-data', {}, onDataUpdate)
}