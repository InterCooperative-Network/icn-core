import { useState, useEffect, useRef, useCallback } from 'react'

export interface WebSocketMessage {
  type: string
  data: any
  timestamp: number
}

export interface UseWebSocketReturn {
  isConnected: boolean
  lastMessage: WebSocketMessage | null
  sendMessage: (message: any) => void
  connect: () => void
  disconnect: () => void
  error: Error | null
}

export function useWebSocket(url: string): UseWebSocketReturn {
  const [isConnected, setIsConnected] = useState(false)
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null)
  const [error, setError] = useState<Error | null>(null)
  
  const ws = useRef<WebSocket | null>(null)
  const reconnectTimeout = useRef<NodeJS.Timeout | null>(null)
  const reconnectAttempts = useRef(0)
  const maxReconnectAttempts = 5
  const reconnectDelay = 1000

  const connect = useCallback(() => {
    try {
      if (ws.current?.readyState === WebSocket.OPEN) {
        return
      }

      setError(null)
      ws.current = new WebSocket(url)

      ws.current.onopen = () => {
        console.log('WebSocket connected:', url)
        setIsConnected(true)
        reconnectAttempts.current = 0
        
        // Subscribe to real-time updates
        ws.current?.send(JSON.stringify({
          type: 'subscribe',
          channels: ['dag', 'jobs', 'network', 'economics']
        }))
      }

      ws.current.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data)
          const message: WebSocketMessage = {
            type: data.type || 'unknown',
            data: data,
            timestamp: Date.now()
          }
          setLastMessage(message)
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err)
        }
      }

      ws.current.onclose = (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason)
        setIsConnected(false)
        
        // Attempt to reconnect if not manually closed
        if (event.code !== 1000 && reconnectAttempts.current < maxReconnectAttempts) {
          const delay = reconnectDelay * Math.pow(2, reconnectAttempts.current)
          reconnectTimeout.current = setTimeout(() => {
            reconnectAttempts.current++
            console.log(`Reconnecting WebSocket (attempt ${reconnectAttempts.current})...`)
            connect()
          }, delay)
        }
      }

      ws.current.onerror = (event) => {
        console.error('WebSocket error:', event)
        setError(new Error('WebSocket connection error'))
        setIsConnected(false)
      }
    } catch (err) {
      console.error('Failed to create WebSocket connection:', err)
      setError(err as Error)
    }
  }, [url])

  const disconnect = useCallback(() => {
    if (reconnectTimeout.current) {
      clearTimeout(reconnectTimeout.current)
      reconnectTimeout.current = null
    }
    
    if (ws.current) {
      ws.current.close(1000, 'Manual disconnect')
      ws.current = null
    }
    
    setIsConnected(false)
    setLastMessage(null)
    reconnectAttempts.current = 0
  }, [])

  const sendMessage = useCallback((message: any) => {
    if (ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(message))
    } else {
      console.warn('WebSocket not connected, cannot send message:', message)
    }
  }, [])

  useEffect(() => {
    return () => {
      disconnect()
    }
  }, [disconnect])

  return {
    isConnected,
    lastMessage,
    sendMessage,
    connect,
    disconnect,
    error
  }
} 