import React from 'react'
import { Link, useLocation } from 'react-router-dom'
import { useICNClient } from '@icn/ts-sdk'

const navigation = [
  { name: 'Dashboard', href: '/', icon: '🏠' },
  { name: 'Federation', href: '/federation', icon: '🤝' },
  { name: 'Governance', href: '/governance', icon: '🗳️' },
  { name: 'Cooperatives', href: '/cooperatives', icon: '🏢' },
  { name: 'Mesh Jobs', href: '/jobs', icon: '⚡' },
  { name: 'Settings', href: '/settings', icon: '⚙️' },
]

export function Navigation() {
  const location = useLocation()
  const icnClient = useICNClient()
  const connectionState = icnClient.getConnectionState()

  return (
    <nav className="bg-white shadow-lg border-b border-gray-200">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex">
            {/* Logo */}
            <div className="flex-shrink-0 flex items-center">
              <Link to="/" className="text-xl font-bold text-blue-600">
                ICN Federation
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
                    className={`inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium ${
                      isActive
                        ? 'border-blue-500 text-gray-900'
                        : 'border-transparent text-gray-500 hover:border-gray-300 hover:text-gray-700'
                    }`}
                  >
                    <span className="mr-2">{item.icon}</span>
                    {item.name}
                  </Link>
                )
              })}
            </div>
          </div>

          {/* Connection Status */}
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <div
                className={`w-2 h-2 rounded-full ${
                  connectionState.connected ? 'bg-green-500' : 'bg-red-500'
                }`}
              />
              <span className="text-sm text-gray-600">
                {connectionState.connected ? 'Connected' : 'Disconnected'}
              </span>
            </div>

            {connectionState.connected && (
              <div className="text-sm text-gray-600">
                <span className="font-medium">Network:</span> {connectionState.network}
              </div>
            )}

            {connectionState.did && (
              <div className="text-sm text-gray-600">
                <span className="font-medium">DID:</span>{' '}
                <span className="font-mono text-xs">
                  {connectionState.did.length > 20
                    ? `${connectionState.did.slice(0, 10)}...${connectionState.did.slice(-6)}`
                    : connectionState.did}
                </span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Mobile Navigation */}
      <div className="sm:hidden">
        <div className="pt-2 pb-3 space-y-1 bg-gray-50 border-t border-gray-200">
          {navigation.map((item) => {
            const isActive = location.pathname === item.href
            return (
              <Link
                key={item.name}
                to={item.href}
                className={`block pl-3 pr-4 py-2 border-l-4 text-base font-medium ${
                  isActive
                    ? 'bg-blue-50 border-blue-500 text-blue-700'
                    : 'border-transparent text-gray-600 hover:bg-gray-50 hover:border-gray-300 hover:text-gray-800'
                }`}
              >
                <span className="mr-3">{item.icon}</span>
                {item.name}
              </Link>
            )
          })}
        </div>
      </div>
    </nav>
  )
}