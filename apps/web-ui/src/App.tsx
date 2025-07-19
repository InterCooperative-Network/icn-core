import React from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { ICNProvider } from '@icn/ts-sdk'
import { Dashboard } from './components/Dashboard'
import { Navigation } from './components/Navigation'
import './index.css'

function App() {
  const icnOptions = {
    nodeEndpoint: import.meta.env.VITE_ICN_NODE_ENDPOINT || 'http://localhost:8080',
    network: (import.meta.env.VITE_ICN_NETWORK || 'devnet') as 'mainnet' | 'testnet' | 'devnet',
  }

  return (
    <ICNProvider options={icnOptions}>
      <Router>
        <div className="min-h-screen bg-gray-50">
          <Navigation />
          <main className="container mx-auto px-4 py-8">
            <Routes>
              <Route path="/" element={<Dashboard />} />
              <Route path="/dashboard" element={<Dashboard />} />
              <Route path="/jobs" element={<div>Jobs Management</div>} />
              <Route path="/governance" element={<div>Governance</div>} />
              <Route path="/members" element={<div>Member Management</div>} />
              <Route path="/settings" element={<div>Settings</div>} />
            </Routes>
          </main>
        </div>
      </Router>
    </ICNProvider>
  )
}

export default App 