import React, { useState, useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom'
import { useICNClient } from '@icn/ts-sdk'
import { I18NProvider, LanguageSwitcher } from '@icn/i18n'
import { Sidebar } from './components/layout/Sidebar'
import { Header } from './components/layout/Header'
import { StatusBar } from './components/layout/StatusBar'
import { DAGExplorer } from './components/dag/DAGExplorer'
import { JobBrowser } from './components/jobs/JobBrowser'
import { NetworkMonitor } from './components/network/NetworkMonitor'
import { EconomicDashboard } from './components/economics/EconomicDashboard'
import { SearchPage } from './components/search/SearchPage'
import { SettingsPage } from './components/settings/SettingsPage'
import { useWebSocket } from './hooks/useWebSocket'
import { useRealtimeData } from './hooks/useRealtimeData'
import { ThemeProvider } from './contexts/ThemeContext'
import { NotificationProvider } from './contexts/NotificationContext'
import './i18n' // Initialize i18n
import './App.css'

export function App() {
  const [isConnected, setIsConnected] = useState(false)
  const [nodeEndpoint, setNodeEndpoint] = useState(
    import.meta.env.VITE_ICN_NODE_ENDPOINT || 'http://localhost:7845'
  )

  // Initialize ICN client
  const { client, isInitialized, error: clientError } = useICNClient({
    endpoint: nodeEndpoint
  })

  // WebSocket connection for real-time updates
  const { 
    isConnected: wsConnected, 
    lastMessage, 
    sendMessage,
    connect,
    disconnect 
  } = useWebSocket(`${nodeEndpoint.replace('http', 'ws')}/ws`)

  // Real-time data streams
  const {
    dagData,
    jobData,
    networkData,
    economicData,
    refreshData
  } = useRealtimeData(client, lastMessage)

  useEffect(() => {
    if (isInitialized && !wsConnected) {
      connect()
    }
  }, [isInitialized, wsConnected, connect])

  useEffect(() => {
    setIsConnected(isInitialized && wsConnected)
  }, [isInitialized, wsConnected])

  // Handle connection errors
  useEffect(() => {
    if (clientError) {
      console.error('ICN Client Error:', clientError)
    }
  }, [clientError])

  return (
    <I18NProvider>
      <ThemeProvider>
        <NotificationProvider>
          <Router>
            <div className="flex h-screen bg-gray-50 dark:bg-gray-900">
              <Sidebar />
              
              <div className="flex flex-col flex-1 overflow-hidden">
                <Header 
                  isConnected={isConnected}
                  nodeEndpoint={nodeEndpoint}
                  onEndpointChange={setNodeEndpoint}
                  onRefresh={refreshData}
                />
                
                <main className="flex-1 overflow-x-hidden overflow-y-auto bg-gray-50 dark:bg-gray-900">
                  <div className="container mx-auto px-6 py-8">
                    <Routes>
                      <Route path="/" element={<Navigate to="/dag" replace />} />
                      
                      <Route 
                        path="/dag" 
                        element={
                          <DAGExplorer 
                            data={dagData}
                            isConnected={isConnected}
                            isRealtime={wsConnected}
                          />
                        } 
                      />
                      
                      <Route 
                        path="/jobs" 
                        element={
                          <JobBrowser 
                            data={jobData}
                            isConnected={isConnected}
                            isRealtime={wsConnected}
                          />
                        } 
                      />
                      
                      <Route 
                        path="/network" 
                        element={
                          <NetworkMonitor 
                            data={networkData}
                            isConnected={isConnected}
                            isRealtime={wsConnected}
                          />
                        } 
                      />
                      
                      <Route 
                        path="/economics" 
                        element={
                          <EconomicDashboard 
                            data={economicData}
                            isConnected={isConnected}
                            isRealtime={wsConnected}
                          />
                        } 
                      />
                      
                      <Route 
                        path="/search" 
                        element={
                          <SearchPage 
                            client={client}
                            isConnected={isConnected}
                          />
                        } 
                      />
                      
                      <Route 
                        path="/settings" 
                        element={
                          <SettingsPage 
                            nodeEndpoint={nodeEndpoint}
                            onEndpointChange={setNodeEndpoint}
                          />
                        } 
                      />
                    </Routes>
                  </div>
                </main>
                
                <StatusBar 
                  isConnected={isConnected}
                  wsConnected={wsConnected}
                  lastUpdate={lastMessage?.timestamp}
                  dagBlocks={dagData?.blocks?.length || 0}
                  activeJobs={jobData?.active?.length || 0}
                  networkPeers={networkData?.peers?.length || 0}
                />
              </div>
            </div>
          </Router>
        </NotificationProvider>
      </ThemeProvider>
    </I18NProvider>
  )
} 