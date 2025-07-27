import React, { useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { useICNClient } from '@icn/ts-sdk'
import { useTranslation, LanguageSwitcher } from '@icn/i18n'

const navigation = [
  { name: 'menu.demo', href: '/', icon: '🎯' },
  { name: 'menu.dashboard', href: '/dashboard', icon: '🏠' },
  { name: 'menu.federation', href: '/federation', icon: '🤝' },
  { name: 'menu.governance', href: '/governance', icon: '🗳️' },
  { name: 'menu.contractEditor', href: '/contracts/editor', icon: '🎨' },
  { name: 'menu.cooperatives', href: '/cooperatives', icon: '🏢' },
  { name: 'menu.meshJobs', href: '/jobs', icon: '⚡' },
  { name: 'menu.settings', href: '/settings', icon: '⚙️' },
]

export function Navigation() {
  const location = useLocation()
  const icnClient = useICNClient()
  const connectionState = icnClient.getConnectionState()
  const { t } = useTranslation('navigation')
  const [isMobileMenuOpen, setIsMobileMenuOpen] = useState(false)

  const toggleMobileMenu = () => {
    setIsMobileMenuOpen(!isMobileMenuOpen)
  }

  return (
    <nav className="bg-white shadow-lg border-b border-gray-200" role="navigation" aria-label={t('accessibility.mainNavigation')}>
      {/* Skip to content link for accessibility */}
      <a
        href="#main-content"
        className="sr-only focus:not-sr-only focus:absolute focus:top-0 focus:left-0 bg-blue-600 text-white px-4 py-2 z-50"
      >
        {t('accessibility.skipToContent')}
      </a>
      
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex">
            {/* Logo */}
            <div className="flex-shrink-0 flex items-center">
              <Link 
                to="/" 
                className="text-xl font-bold text-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-md px-2 py-1"
                aria-label={t('brand')}
              >
                {t('brand')}
              </Link>
            </div>

            {/* Main Navigation */}
            <div className="hidden sm:ml-6 sm:flex sm:space-x-8">
              {navigation.map((item) => {
                const isActive = location.pathname === item.href
                return (
                  <Link
                    key={item.name}
                    to={item.href}
                    className={`inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 rounded-t-md ${
                      isActive
                        ? 'border-blue-500 text-gray-900'
                        : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
                    }`}
                    aria-current={isActive ? 'page' : undefined}
                    aria-label={isActive ? t('accessibility.currentPage') : undefined}
                  >
                    <span className="mr-2" aria-hidden="true">{item.icon}</span>
                    {t(item.name)}
                  </Link>
                )
              })}
            </div>
          </div>

          {/* Right side items */}
          <div className="flex items-center space-x-4">
            {/* Language Switcher */}
            <LanguageSwitcher variant="dropdown" className="hidden sm:block" />
            
            {/* Connection Status */}
            <div className="flex items-center space-x-2">
              <div
                className={`w-2 h-2 rounded-full ${
                  connectionState.connected ? 'bg-green-500' : 'bg-red-500'
                }`}
                aria-hidden="true"
              />
              <span className="text-sm text-gray-600">
                {connectionState.connected ? t('status.connected') : t('status.disconnected')}
              </span>
            </div>

            {connectionState.connected && (
              <div className="text-sm text-gray-600">
                <span className="font-medium">{t('status.network')}:</span> {connectionState.network}
              </div>
            )}

            {connectionState.did && (
              <div className="text-sm text-gray-600">
                <span className="font-medium">{t('status.did')}:</span>{' '}
                <span className="font-mono text-xs">
                  {connectionState.did.length > 20
                    ? `${connectionState.did.slice(0, 10)}...${connectionState.did.slice(-6)}`
                    : connectionState.did}
                </span>
              </div>
            )}
            
            {/* Mobile menu button */}
            <button
              type="button"
              className="sm:hidden inline-flex items-center justify-center p-2 rounded-md text-gray-500 hover:text-gray-700 hover:bg-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2"
              aria-expanded={isMobileMenuOpen}
              aria-controls="mobile-menu"
              aria-label={isMobileMenuOpen ? t('mobileMenu.close') : t('mobileMenu.open')}
              onClick={toggleMobileMenu}
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                {isMobileMenuOpen ? (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                ) : (
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                )}
              </svg>
            </button>
          </div>
        </div>
      </div>

      {/* Mobile Navigation */}
      <div 
        className={`sm:hidden ${isMobileMenuOpen ? 'block' : 'hidden'}`}
        id="mobile-menu"
      >
        <div className="pt-2 pb-3 space-y-1 bg-gray-50 border-t border-gray-200">
          {navigation.map((item) => {
            const isActive = location.pathname === item.href
            return (
              <Link
                key={item.name}
                to={item.href}
                className={`block pl-3 pr-4 py-2 border-l-4 text-base font-medium focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 ${
                  isActive
                    ? 'bg-blue-50 border-blue-500 text-blue-700'
                    : 'border-transparent text-gray-600 hover:bg-gray-50 hover:border-gray-300 hover:text-gray-800'
                }`}
                aria-current={isActive ? 'page' : undefined}
                onClick={() => setIsMobileMenuOpen(false)}
              >
                <span className="mr-3" aria-hidden="true">{item.icon}</span>
                {t(item.name)}
              </Link>
            )
          })}
          
          {/* Language switcher for mobile */}
          <div className="pl-3 pr-4 py-2">
            <LanguageSwitcher variant="buttons" />
          </div>
        </div>
      </div>
    </nav>
  )
}