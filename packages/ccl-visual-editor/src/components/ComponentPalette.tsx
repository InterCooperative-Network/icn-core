import React, { useState, useCallback } from 'react'
import type { ComponentPaletteProps, PaletteComponent } from '../types'
import { GOVERNANCE_COMPONENTS, COMPONENT_CATEGORIES, getComponentsByCategory, searchComponents } from '../components-library'

export function ComponentPalette({
  components = GOVERNANCE_COMPONENTS,
  onComponentSelect,
  searchTerm = '',
  selectedCategory = '',
  className = ''
}: ComponentPaletteProps) {
  const [draggedComponent, setDraggedComponent] = useState<PaletteComponent | null>(null)

  // Filter components based on search and category
  const filteredComponents = React.useMemo(() => {
    let filtered = components

    if (searchTerm) {
      filtered = searchComponents(searchTerm)
    }

    if (selectedCategory) {
      filtered = filtered.filter(comp => comp.category === selectedCategory)
    }

    return filtered
  }, [components, searchTerm, selectedCategory])

  const handleDragStart = useCallback((e: React.DragEvent, component: PaletteComponent) => {
    setDraggedComponent(component)
    e.dataTransfer.setData('application/json', JSON.stringify(component))
    e.dataTransfer.effectAllowed = 'copy'
  }, [])

  const handleDragEnd = useCallback(() => {
    setDraggedComponent(null)
  }, [])

  const handleComponentClick = useCallback((component: PaletteComponent) => {
    onComponentSelect?.(component)
  }, [onComponentSelect])

  return (
    <div className={`component-palette bg-gray-50 border-r border-gray-200 flex flex-col ${className}`}>
      <div className="p-4 border-b border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 m-0">Components</h3>
      </div>

      {/* Category tabs */}
      <div className="flex flex-wrap gap-2 p-3 border-b border-gray-200">
        <button className="flex items-center gap-1 px-3 py-1.5 text-sm border border-gray-300 rounded-md bg-white text-gray-600 hover:bg-gray-50 hover:border-gray-400 transition-all">
          All
        </button>
        {COMPONENT_CATEGORIES.map(category => (
          <button
            key={category.id}
            className="flex items-center gap-1 px-3 py-1.5 text-sm border border-gray-300 rounded-md bg-white text-gray-600 hover:bg-gray-50 hover:border-gray-400 transition-all"
            style={{ borderColor: category.color }}
          >
            <span className="text-base">{category.icon}</span>
            {category.name}
          </button>
        ))}
      </div>

      {/* Search bar */}
      <div className="p-3 border-b border-gray-200">
        <input
          type="text"
          placeholder="Search components..."
          className="w-full px-3 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        />
      </div>

      {/* Component list */}
      <div className="flex-1 overflow-y-auto p-2">
        {filteredComponents.map(component => (
          <div
            key={component.id}
            className={`flex items-center gap-3 p-3 mb-2 bg-white border border-gray-200 rounded-lg cursor-grab transition-all hover:border-blue-500 hover:shadow-md ${
              draggedComponent?.id === component.id ? 'opacity-50 transform rotate-1' : ''
            }`}
            draggable
            onDragStart={(e) => handleDragStart(e, component)}
            onDragEnd={handleDragEnd}
            onClick={() => handleComponentClick(component)}
          >
            <div className="text-2xl w-8 h-8 flex items-center justify-center">
              {component.icon}
            </div>
            <div className="flex-1">
              <div className="font-semibold text-gray-900 mb-1">{component.name}</div>
              <div className="text-xs text-gray-600 leading-tight">{component.description}</div>
            </div>
            <div className="text-base" style={{ color: COMPONENT_CATEGORIES.find(c => c.id === component.category)?.color }}>
              {COMPONENT_CATEGORIES.find(c => c.id === component.category)?.icon}
            </div>
          </div>
        ))}
      </div>

      {filteredComponents.length === 0 && (
        <div className="p-8 text-center text-gray-500">
          <p className="mb-2">No components found.</p>
          {searchTerm && (
            <p>Try adjusting your search term.</p>
          )}
        </div>
      )}
    </div>
  )
}