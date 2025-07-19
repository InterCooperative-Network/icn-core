import React from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { ICNProvider } from '@icn/ts-sdk'
import { FederationProvider } from './contexts/FederationContext'
import { GovernanceProvider } from './contexts/GovernanceContext'
import { Dashboard } from './components/Dashboard'
import { Navigation } from './components/Navigation'
import { FederationPage } from './pages/FederationPage'
import { GovernancePage } from './pages/GovernancePage'
import './index.css'

function App() {
  const icnOptions = {
    nodeEndpoint: import.meta.env.VITE_ICN_NODE_ENDPOINT || 'http://localhost:8080',
    network: (import.meta.env.VITE_ICN_NETWORK || 'devnet') as 'mainnet' | 'testnet' | 'devnet',
  }

  return (
    <ICNProvider options={icnOptions}>
      <FederationProvider>
        <GovernanceProvider>
          <Router>
            <div className="min-h-screen bg-gray-50">
              <Navigation />
              <main className="container mx-auto px-4 py-8">
                <Routes>
                  <Route path="/" element={<Dashboard />} />
                  <Route path="/dashboard" element={<Dashboard />} />
                  <Route path="/federation" element={<FederationPage />} />
                  <Route path="/governance" element={<GovernancePage />} />
                  <Route path="/cooperatives" element={<div>Cooperatives Management (Coming Soon)</div>} />
                  <Route path="/jobs" element={<div>Mesh Jobs Management (Coming Soon)</div>} />
                  <Route path="/settings" element={<div>Settings (Coming Soon)</div>} />
                </Routes>
              </main>
            </div>
          </Router>
        </GovernanceProvider>
      </FederationProvider>
    </ICNProvider>
  )
}

export default App 