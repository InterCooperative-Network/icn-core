import React, { useState, useEffect } from 'react'
import type { PropertyInspectorProps, CanvasNode, CCLParameter } from '../types'
import { CCLUtils } from '@icn/ts-sdk'

export function PropertyInspector({
  selectedNode,
  onPropertyChange,
  readOnly = false,
  className = ''
}: PropertyInspectorProps) {
  const [localConfig, setLocalConfig] = useState<Record<string, any>>({})
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({})

  // Update local config when selected node changes
  useEffect(() => {
    if (selectedNode) {
      setLocalConfig({ ...selectedNode.config })
      validateConfiguration(selectedNode, selectedNode.config)
    } else {
      setLocalConfig({})
      setValidationErrors({})
    }
  }, [selectedNode])

  const validateConfiguration = (node: CanvasNode, config: Record<string, any>) => {
    if (!node.component.parameters) {
      setValidationErrors({})
      return
    }

    const validation = CCLUtils.validateTemplateParameters(node.component, config)
    const errors: Record<string, string> = {}

    if (!validation.valid) {
      validation.errors.forEach(error => {
        // Extract parameter name from error message
        const paramMatch = error.match(/^(\w+)/)
        if (paramMatch) {
          errors[paramMatch[1]] = error
        }
      })
    }

    setValidationErrors(errors)
  }

  const handleParameterChange = (parameterName: string, value: any) => {
    if (readOnly || !selectedNode) return

    const newConfig = { ...localConfig, [parameterName]: value }
    setLocalConfig(newConfig)
    validateConfiguration(selectedNode, newConfig)
    
    onPropertyChange?.(selectedNode.id, parameterName, value)
  }

  if (!selectedNode) {
    return (
      <div className={`property-inspector bg-gray-50 border-l border-gray-200 flex flex-col ${className}`}>
        <div className="p-4 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 m-0">Properties</h3>
        </div>
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center text-gray-500">
            <div className="text-4xl mb-4">⚙️</div>
            <p className="text-sm">Select a component to view its properties</p>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className={`property-inspector bg-gray-50 border-l border-gray-200 flex flex-col ${className}`}>
      {/* Header */}
      <div className="p-4 border-b border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 m-0">Properties</h3>
      </div>

      {/* Component info */}
      <div className="p-4 border-b border-gray-200 bg-white">
        <div className="flex items-center gap-3 mb-3">
          <span className="text-2xl">{selectedNode.component.icon}</span>
          <div>
            <h4 className="font-semibold text-gray-900">{selectedNode.component.name}</h4>
            <p className="text-sm text-gray-600">{selectedNode.component.description}</p>
          </div>
        </div>
        
        <div className="flex items-center gap-2">
          <span className="px-2 py-1 text-xs font-medium bg-gray-100 text-gray-700 rounded-full">
            {selectedNode.component.category}
          </span>
          <span className="text-xs text-gray-500">ID: {selectedNode.id.slice(0, 8)}</span>
        </div>
      </div>

      {/* Parameters */}
      <div className="flex-1 overflow-y-auto">
        {selectedNode.component.parameters && selectedNode.component.parameters.length > 0 ? (
          <div className="p-4 space-y-4">
            <h5 className="font-medium text-gray-900 mb-3">Configuration</h5>
            {selectedNode.component.parameters.map(parameter => (
              <ParameterInput
                key={parameter.name}
                parameter={parameter}
                value={localConfig[parameter.name]}
                error={validationErrors[parameter.name]}
                onChange={(value) => handleParameterChange(parameter.name, value)}
                readOnly={readOnly}
              />
            ))}
          </div>
        ) : (
          <div className="p-4 text-center text-gray-500">
            <p className="text-sm">This component has no configurable parameters.</p>
          </div>
        )}

        {/* Position info */}
        <div className="p-4 border-t border-gray-200 bg-gray-50">
          <h5 className="font-medium text-gray-900 mb-2">Position</h5>
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div>
              <label className="text-gray-600">X:</label>
              <span className="ml-2 text-gray-900">{Math.round(selectedNode.position.x)}</span>
            </div>
            <div>
              <label className="text-gray-600">Y:</label>
              <span className="ml-2 text-gray-900">{Math.round(selectedNode.position.y)}</span>
            </div>
          </div>
        </div>

        {/* Ports info */}
        {selectedNode.ports.length > 0 && (
          <div className="p-4 border-t border-gray-200">
            <h5 className="font-medium text-gray-900 mb-2">Ports</h5>
            <div className="space-y-2">
              {selectedNode.ports.map(port => (
                <div key={port.id} className="flex items-center gap-2 text-sm">
                  <div className={`w-3 h-3 rounded-full ${
                    port.type === 'input' ? 'bg-blue-500' : 'bg-green-500'
                  }`} />
                  <span className="text-gray-900">{port.label}</span>
                  <span className="text-gray-500 text-xs">({port.dataType})</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}

// Component for individual parameter inputs
interface ParameterInputProps {
  parameter: CCLParameter
  value: any
  error?: string
  onChange: (value: any) => void
  readOnly: boolean
}

function ParameterInput({ parameter, value, error, onChange, readOnly }: ParameterInputProps) {
  const handleChange = (newValue: any) => {
    if (readOnly) return
    onChange(newValue)
  }

  const renderInput = () => {
    switch (parameter.type) {
      case 'boolean':
        return (
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={value ?? parameter.default ?? false}
              onChange={(e) => handleChange(e.target.checked)}
              disabled={readOnly}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="text-sm text-gray-700">
              {value ? 'Enabled' : 'Disabled'}
            </span>
          </label>
        )

      case 'number':
        return (
          <input
            type="number"
            value={value ?? parameter.default ?? ''}
            onChange={(e) => handleChange(Number(e.target.value))}
            min={parameter.validation?.min}
            max={parameter.validation?.max}
            disabled={readOnly}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
          />
        )

      case 'string':
        if (parameter.validation?.options) {
          // Select dropdown for options
          return (
            <select
              value={value ?? parameter.default ?? ''}
              onChange={(e) => handleChange(e.target.value)}
              disabled={readOnly}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
            >
              <option value="">Select an option...</option>
              {parameter.validation.options.map(option => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          )
        } else {
          // Text input
          return (
            <input
              type="text"
              value={value ?? parameter.default ?? ''}
              onChange={(e) => handleChange(e.target.value)}
              disabled={readOnly}
              className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
            />
          )
        }

      case 'did':
        return (
          <input
            type="text"
            value={value ?? parameter.default ?? ''}
            onChange={(e) => handleChange(e.target.value)}
            placeholder="did:method:identifier"
            disabled={readOnly}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
          />
        )

      case 'duration':
        return (
          <div className="flex gap-2">
            <input
              type="number"
              value={value ?? parameter.default ?? ''}
              onChange={(e) => handleChange(Number(e.target.value))}
              min={parameter.validation?.min}
              max={parameter.validation?.max}
              disabled={readOnly}
              className="flex-1 px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
            />
            <span className="px-3 py-2 text-sm text-gray-600 bg-gray-100 rounded-md">
              days
            </span>
          </div>
        )

      default:
        return (
          <input
            type="text"
            value={value ?? parameter.default ?? ''}
            onChange={(e) => handleChange(e.target.value)}
            disabled={readOnly}
            className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:bg-gray-100"
          />
        )
    }
  }

  return (
    <div className="space-y-2">
      <label className="block">
        <div className="flex items-center gap-2 mb-1">
          <span className="text-sm font-medium text-gray-900">
            {parameter.name}
          </span>
          {parameter.required && (
            <span className="text-red-500 text-xs">*</span>
          )}
        </div>
        
        {parameter.description && (
          <p className="text-xs text-gray-600 mb-2">{parameter.description}</p>
        )}
        
        {renderInput()}
      </label>

      {error && (
        <p className="text-xs text-red-600 mt-1">{error}</p>
      )}

      {parameter.validation && !error && (
        <div className="text-xs text-gray-500">
          {parameter.validation.min !== undefined && parameter.validation.max !== undefined && (
            <span>Range: {parameter.validation.min} - {parameter.validation.max}</span>
          )}
          {parameter.validation.pattern && (
            <span>Pattern: {parameter.validation.pattern}</span>
          )}
        </div>
      )}
    </div>
  )
}