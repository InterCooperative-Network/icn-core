import React from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { ICNProvider } from '@icn/ts-sdk'
import { I18NProvider } from '@icn/i18n'
import { FederationProvider } from './contexts/FederationContext'
import { GovernanceProvider } from './contexts/GovernanceContext'
import { Dashboard } from './components/Dashboard'
import { Navigation } from './components/Navigation'
import { FederationPage } from './pages/FederationPage'
import { GovernancePage } from './pages/GovernancePage'
import { CooperativesPage } from './pages/CooperativesPage'
import { DemoPage } from './pages/DemoPage'
import { JobsPage } from './pages/JobsPage'
import { SettingsPage } from './pages/SettingsPage'
import './i18n'
import './index.css'

function App() {
  const icnOptions = {
    nodeEndpoint: import.meta.env.VITE_ICN_NODE_ENDPOINT || 'http://localhost:8080',
    network: (import.meta.env.VITE_ICN_NETWORK || 'devnet') as 'mainnet' | 'testnet' | 'devnet',
  }

  return (
    <I18NProvider>
      <ICNProvider options={icnOptions}>
        <FederationProvider>
          <GovernanceProvider>
            <Router>
              <div className="min-h-screen bg-gray-50">
                <Navigation />
                <main className="container mx-auto px-4 py-8">
                  <Routes>
                    <Route path="/" element={<DemoPage />} />
                    <Route path="/demo" element={<DemoPage />} />
                    <Route path="/dashboard" element={<Dashboard />} />
                    <Route path="/federation" element={<FederationPage />} />
                    <Route path="/governance" element={<GovernancePage />} />
                    <Route path="/cooperatives" element={<CooperativesPage />} />
                    <Route path="/jobs" element={<JobsPage />} />
                    <Route path="/settings" element={<SettingsPage />} />
                  </Routes>
                </main>
              </div>
            </Router>
          </GovernanceProvider>
        </FederationProvider>
      </ICNProvider>
    </I18NProvider>
  )
}

export default App 