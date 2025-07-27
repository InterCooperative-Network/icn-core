import React, { useState, useEffect } from 'react'
import type { CodePreviewProps, CCLGenerationResult } from '../types'
import { CCLGenerator } from '../ccl-generator'

export function CodePreview({
  contract,
  generationResult,
  onRefresh,
  onCopy,
  className = ''
}: CodePreviewProps) {
  const [result, setResult] = useState<CCLGenerationResult | null>(null)
  const [isGenerating, setIsGenerating] = useState(false)
  const [showLineNumbers, setShowLineNumbers] = useState(true)
  const [activeTab, setActiveTab] = useState<'code' | 'errors'>('code')

  // Generate CCL code when contract changes
  useEffect(() => {
    if (contract) {
      generateCode()
    }
  }, [contract])

  // Use provided result if available
  useEffect(() => {
    if (generationResult) {
      setResult(generationResult)
    }
  }, [generationResult])

  const generateCode = async () => {
    setIsGenerating(true)
    try {
      // Small delay to show loading state
      await new Promise(resolve => setTimeout(resolve, 100))
      const generatedResult = CCLGenerator.generateFromContract(contract)
      setResult(generatedResult)
    } catch (error) {
      setResult({
        code: '',
        valid: false,
        errors: [{
          message: `Generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`,
          severity: 'error'
        }],
        warnings: []
      })
    } finally {
      setIsGenerating(false)
    }
  }

  const handleCopyCode = async () => {
    if (result?.code) {
      try {
        await navigator.clipboard.writeText(result.code)
        onCopy?.()
        // Could add toast notification here
      } catch (error) {
        console.error('Failed to copy code:', error)
      }
    }
  }

  const handleRefresh = () => {
    generateCode()
    onRefresh?.()
  }

  if (!result) {
    return (
      <div className={`code-preview bg-white border-t border-gray-200 flex flex-col ${className}`}>
        <div className="p-4 flex items-center justify-center text-gray-500">
          <p>No contract to preview</p>
        </div>
      </div>
    )
  }

  const hasErrors = result.errors.length > 0
  const hasWarnings = result.warnings.length > 0
  const codeLines = result.code.split('\n')

  return (
    <div className={`code-preview bg-white border-t border-gray-200 flex flex-col ${className}`}>
      {/* Header */}
      <div className="p-3 border-b border-gray-200 bg-gray-50">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <h3 className="text-sm font-semibold text-gray-900">Generated CCL Code</h3>
            
            {/* Status indicator */}
            <div className="flex items-center gap-2">
              {isGenerating ? (
                <div className="flex items-center gap-2 text-blue-600">
                  <div className="w-4 h-4 border-2 border-blue-600 border-t-transparent rounded-full animate-spin" />
                  <span className="text-xs">Generating...</span>
                </div>
              ) : (
                <div className={`flex items-center gap-1 text-xs px-2 py-1 rounded-full ${
                  hasErrors 
                    ? 'bg-red-100 text-red-700' 
                    : hasWarnings 
                      ? 'bg-yellow-100 text-yellow-700'
                      : 'bg-green-100 text-green-700'
                }`}>
                  {hasErrors ? '❌' : hasWarnings ? '⚠️' : '✅'}
                  {hasErrors ? 'Errors' : hasWarnings ? 'Warnings' : 'Valid'}
                </div>
              )}
            </div>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowLineNumbers(!showLineNumbers)}
              className="px-2 py-1 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded"
              title="Toggle line numbers"
            >
              #
            </button>
            <button
              onClick={handleRefresh}
              disabled={isGenerating}
              className="px-2 py-1 text-xs text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded disabled:opacity-50"
              title="Refresh code"
            >
              🔄
            </button>
            <button
              onClick={handleCopyCode}
              disabled={!result.code}
              className="px-3 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
              title="Copy code"
            >
              Copy
            </button>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex gap-1 mt-3">
          <button
            onClick={() => setActiveTab('code')}
            className={`px-3 py-1 text-xs rounded ${
              activeTab === 'code'
                ? 'bg-white text-gray-900 border border-gray-200'
                : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            Code
          </button>
          {(hasErrors || hasWarnings) && (
            <button
              onClick={() => setActiveTab('errors')}
              className={`px-3 py-1 text-xs rounded ${
                activeTab === 'errors'
                  ? 'bg-white text-gray-900 border border-gray-200'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              Issues ({result.errors.length + result.warnings.length})
            </button>
          )}
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden">
        {activeTab === 'code' ? (
          <div className="h-full overflow-auto">
            {result.code ? (
              <div className="relative">
                <pre className="p-4 text-sm font-mono leading-relaxed">
                  {showLineNumbers && (
                    <div className="absolute left-0 top-0 bottom-0 w-12 bg-gray-50 border-r border-gray-200 flex flex-col">
                      {codeLines.map((_, index) => (
                        <div key={index} className="px-2 py-0 text-xs text-gray-500 text-right">
                          {index + 1}
                        </div>
                      ))}
                    </div>
                  )}
                  <code className={`block ${showLineNumbers ? 'ml-12' : ''}`}>
                    {result.code}
                  </code>
                </pre>
              </div>
            ) : (
              <div className="p-8 text-center text-gray-500">
                <p className="text-sm">No code generated yet</p>
                <p className="text-xs mt-1">Add some components to your contract</p>
              </div>
            )}
          </div>
        ) : (
          <div className="h-full overflow-auto p-4">
            <div className="space-y-3">
              {/* Errors */}
              {result.errors.map((error, index) => (
                <div key={`error-${index}`} className="p-3 bg-red-50 border border-red-200 rounded-lg">
                  <div className="flex items-start gap-2">
                    <span className="text-red-500 mt-0.5">❌</span>
                    <div className="flex-1">
                      <p className="text-sm text-red-800 font-medium">Error</p>
                      <p className="text-sm text-red-700 mt-1">{error.message}</p>
                      {error.nodeId && (
                        <p className="text-xs text-red-600 mt-1">Component: {error.nodeId}</p>
                      )}
                    </div>
                  </div>
                </div>
              ))}

              {/* Warnings */}
              {result.warnings.map((warning, index) => (
                <div key={`warning-${index}`} className="p-3 bg-yellow-50 border border-yellow-200 rounded-lg">
                  <div className="flex items-start gap-2">
                    <span className="text-yellow-500 mt-0.5">⚠️</span>
                    <div className="flex-1">
                      <p className="text-sm text-yellow-800 font-medium">Warning</p>
                      <p className="text-sm text-yellow-700 mt-1">{warning.message}</p>
                      {warning.nodeId && (
                        <p className="text-xs text-yellow-600 mt-1">Component: {warning.nodeId}</p>
                      )}
                    </div>
                  </div>
                </div>
              ))}

              {!hasErrors && !hasWarnings && (
                <div className="p-3 bg-green-50 border border-green-200 rounded-lg">
                  <div className="flex items-center gap-2">
                    <span className="text-green-500">✅</span>
                    <p className="text-sm text-green-800 font-medium">No issues found</p>
                  </div>
                  <p className="text-sm text-green-700 mt-1">Your contract code is valid and ready to deploy!</p>
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Footer with stats */}
      <div className="px-4 py-2 border-t border-gray-200 bg-gray-50">
        <div className="flex items-center justify-between text-xs text-gray-600">
          <div className="flex items-center gap-4">
            <span>Lines: {codeLines.length}</span>
            <span>Characters: {result.code.length}</span>
          </div>
          <div className="flex items-center gap-4">
            {hasErrors && <span className="text-red-600">Errors: {result.errors.length}</span>}
            {hasWarnings && <span className="text-yellow-600">Warnings: {result.warnings.length}</span>}
            <span>Last updated: {new Date().toLocaleTimeString()}</span>
          </div>
        </div>
      </div>
    </div>
  )
}