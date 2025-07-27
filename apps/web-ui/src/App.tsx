import React, { useEffect } from 'react'
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import { ICNProvider } from '@icn/ts-sdk'
import { I18NProvider, useTranslation } from '@icn/i18n'
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

function AppContent() {
  const { i18n } = useTranslation()
  
  // Handle RTL languages
  useEffect(() => {
    const isRTL = ['ar', 'fa', 'he', 'ur'].includes(i18n.language.split('-')[0])
    document.documentElement.dir = isRTL ? 'rtl' : 'ltr'
    document.documentElement.lang = i18n.language
  }, [i18n.language])

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
  )
}

function App() {
  return (
    <I18NProvider>
      <AppContent />
    </I18NProvider>
  )
}

export default App 