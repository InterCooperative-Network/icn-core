import React from 'react'
import ReactDOM from 'react-dom/client'
import { App } from './App'
import './index.css'

// Error Boundary for production error handling
class ErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; error?: Error }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error) {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Explorer Application Error:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
          <div className="max-w-md w-full bg-white dark:bg-gray-800 shadow-lg rounded-lg p-6">
            <div className="flex items-center mb-4">
              <div className="w-8 h-8 bg-red-500 rounded-full flex items-center justify-center mr-3">
                <svg className="w-4 h-4 text-white" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
                </svg>
              </div>
              <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
                Application Error
              </h1>
            </div>
            
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              The ICN Explorer encountered an unexpected error. Please try refreshing the page.
            </p>
            
            {this.state.error && (
              <details className="mb-4">
                <summary className="text-sm text-gray-500 cursor-pointer hover:text-gray-700">
                  Error details
                </summary>
                <pre className="mt-2 text-xs bg-gray-100 dark:bg-gray-700 p-2 rounded overflow-auto">
                  {this.state.error.message}
                </pre>
              </details>
            )}
            
            <button
              onClick={() => window.location.reload()}
              className="w-full bg-blue-600 text-white py-2 px-4 rounded hover:bg-blue-700 transition-colors"
            >
              Refresh Application
            </button>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}

// Check for production environment optimizations
const isDevelopment = import.meta.env.DEV

// Performance monitoring in development
if (isDevelopment) {
  // Enable React DevTools profiler
  if (typeof window !== 'undefined') {
    // Add performance observers for debugging
    if ('PerformanceObserver' in window) {
      const observer = new PerformanceObserver((list) => {
        const entries = list.getEntries()
        entries.forEach((entry) => {
          if (entry.entryType === 'measure') {
            console.debug(`Performance: ${entry.name} took ${entry.duration}ms`)
          }
        })
      })
      observer.observe({ entryTypes: ['measure'] })
    }
  }
}

// Create root and render application
const container = document.getElementById('root')

if (!container) {
  throw new Error('Root container not found')
}

const root = ReactDOM.createRoot(container)

root.render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>
)

// Hot Module Replacement for development
if (isDevelopment && import.meta.hot) {
  import.meta.hot.accept()
} 