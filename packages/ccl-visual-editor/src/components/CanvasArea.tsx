import React, { useState, useCallback, useRef, useEffect } from 'react'
import type { CanvasAreaProps, CanvasNode, Connection, Position, PaletteComponent } from '../types'

interface DragState {
  isDragging: boolean
  dragOffset: Position
  draggedNodeId: string | null
}

export function CanvasArea({
  nodes = [],
  connections = [],
  onNodeCreate,
  onNodeUpdate,
  onNodeDelete,
  onNodeSelect,
  onConnectionCreate,
  onConnectionDelete,
  readOnly = false,
  className = ''
}: CanvasAreaProps) {
  const canvasRef = useRef<HTMLDivElement>(null)
  const [dragState, setDragState] = useState<DragState>({
    isDragging: false,
    dragOffset: { x: 0, y: 0 },
    draggedNodeId: null
  })
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)
  const [connectionPreview, setConnectionPreview] = useState<{
    sourceNodeId: string
    sourcePortId: string
    mousePosition: Position
  } | null>(null)

  // Handle component drop from palette
  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    if (readOnly) return

    try {
      const componentData = e.dataTransfer.getData('application/json')
      const component: PaletteComponent = JSON.parse(componentData)
      
      const rect = canvasRef.current?.getBoundingClientRect()
      if (rect) {
        const position: Position = {
          x: e.clientX - rect.left,
          y: e.clientY - rect.top
        }
        onNodeCreate?.(component, position)
      }
    } catch (error) {
      console.error('Failed to handle drop:', error)
    }
  }, [readOnly, onNodeCreate])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'copy'
  }, [])

  // Handle node dragging
  const handleNodeMouseDown = useCallback((e: React.MouseEvent, nodeId: string) => {
    if (readOnly) return
    
    e.preventDefault()
    e.stopPropagation()

    const node = nodes.find(n => n.id === nodeId)
    if (!node) return

    const rect = e.currentTarget.getBoundingClientRect()
    const offset: Position = {
      x: e.clientX - rect.left,
      y: e.clientY - rect.top
    }

    setDragState({
      isDragging: true,
      dragOffset: offset,
      draggedNodeId: nodeId
    })

    setSelectedNodeId(nodeId)
    onNodeSelect?.(nodeId)
  }, [readOnly, nodes, onNodeSelect])

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!dragState.isDragging || !dragState.draggedNodeId) return

    const rect = canvasRef.current?.getBoundingClientRect()
    if (!rect) return

    const newPosition: Position = {
      x: e.clientX - rect.left - dragState.dragOffset.x,
      y: e.clientY - rect.top - dragState.dragOffset.y
    }

    onNodeUpdate?.(dragState.draggedNodeId, { position: newPosition })
  }, [dragState, onNodeUpdate])

  const handleMouseUp = useCallback(() => {
    if (dragState.isDragging) {
      setDragState({
        isDragging: false,
        dragOffset: { x: 0, y: 0 },
        draggedNodeId: null
      })
    }
  }, [dragState.isDragging])

  // Handle connection creation
  const handlePortMouseDown = useCallback((e: React.MouseEvent, nodeId: string, portId: string) => {
    if (readOnly) return
    
    e.preventDefault()
    e.stopPropagation()

    const node = nodes.find(n => n.id === nodeId)
    const port = node?.ports.find(p => p.id === portId)
    
    if (port?.type === 'output') {
      setConnectionPreview({
        sourceNodeId: nodeId,
        sourcePortId: portId,
        mousePosition: { x: e.clientX, y: e.clientY }
      })
    }
  }, [readOnly, nodes])

  const handlePortMouseUp = useCallback((e: React.MouseEvent, nodeId: string, portId: string) => {
    if (!connectionPreview || readOnly) return

    e.preventDefault()
    e.stopPropagation()

    const sourceNode = nodes.find(n => n.id === connectionPreview.sourceNodeId)
    const targetNode = nodes.find(n => n.id === nodeId)
    const sourcePort = sourceNode?.ports.find(p => p.id === connectionPreview.sourcePortId)
    const targetPort = targetNode?.ports.find(p => p.id === portId)

    if (sourcePort?.type === 'output' && targetPort?.type === 'input' && 
        connectionPreview.sourceNodeId !== nodeId) {
      
      const newConnection = {
        sourceNodeId: connectionPreview.sourceNodeId,
        targetNodeId: nodeId,
        sourcePortId: connectionPreview.sourcePortId,
        targetPortId: portId
      }

      onConnectionCreate?.(newConnection)
    }

    setConnectionPreview(null)
  }, [connectionPreview, readOnly, nodes, onConnectionCreate])

  // Handle keyboard shortcuts
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (readOnly) return

    if (e.key === 'Delete' && selectedNodeId) {
      onNodeDelete?.(selectedNodeId)
      setSelectedNodeId(null)
      onNodeSelect?.(null)
    }
  }, [readOnly, selectedNodeId, onNodeDelete, onNodeSelect])

  // Set up event listeners
  useEffect(() => {
    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
    document.addEventListener('keydown', handleKeyDown)

    return () => {
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [handleMouseMove, handleMouseUp, handleKeyDown])

  // Handle canvas click (deselect nodes)
  const handleCanvasClick = useCallback((e: React.MouseEvent) => {
    if (e.target === canvasRef.current) {
      setSelectedNodeId(null)
      onNodeSelect?.(null)
    }
  }, [onNodeSelect])

  return (
    <div
      ref={canvasRef}
      className={`canvas-area relative w-full h-full bg-gray-100 overflow-hidden ${className}`}
      onDrop={handleDrop}
      onDragOver={handleDragOver}
      onClick={handleCanvasClick}
    >
      {/* Grid background */}
      <div 
        className="absolute inset-0 opacity-20"
        style={{
          backgroundImage: `
            radial-gradient(circle, #9ca3af 1px, transparent 1px)
          `,
          backgroundSize: '20px 20px'
        }}
      />

      {/* Render connections */}
      <svg className="absolute inset-0 w-full h-full pointer-events-none">
        {connections.map(connection => {
          const sourceNode = nodes.find(n => n.id === connection.sourceNodeId)
          const targetNode = nodes.find(n => n.id === connection.targetNodeId)
          
          if (!sourceNode || !targetNode) return null

          const sourcePort = sourceNode.ports.find(p => p.id === connection.sourcePortId)
          const targetPort = targetNode.ports.find(p => p.id === connection.targetPortId)
          
          if (!sourcePort || !targetPort) return null

          // Calculate port positions
          const sourceX = sourceNode.position.x + sourceNode.size.width
          const sourceY = sourceNode.position.y + 30 // Approximate port position
          const targetX = targetNode.position.x
          const targetY = targetNode.position.y + 30

          return (
            <ConnectionLine
              key={connection.id}
              sourceX={sourceX}
              sourceY={sourceY}
              targetX={targetX}
              targetY={targetY}
              selected={false}
              onClick={() => onConnectionDelete?.(connection.id)}
            />
          )
        })}

        {/* Connection preview */}
        {connectionPreview && (() => {
          const sourceNode = nodes.find(n => n.id === connectionPreview.sourceNodeId)
          if (!sourceNode) return null
          
          // Calculate actual source port position
          const sourceX = sourceNode.position.x + sourceNode.size.width
          const sourceY = sourceNode.position.y + 30 // Approximate port position
          
          return (
            <ConnectionLine
              sourceX={sourceX}
              sourceY={sourceY}
              targetX={connectionPreview.mousePosition.x}
              targetY={connectionPreview.mousePosition.y}
              selected={false}
              preview={true}
            />
          )
        })()}
      </svg>

      {/* Render nodes */}
      {nodes.map(node => (
        <CanvasNodeComponent
          key={node.id}
          node={node}
          selected={selectedNodeId === node.id}
          onMouseDown={(e) => handleNodeMouseDown(e, node.id)}
          onPortMouseDown={(portId) => (e) => handlePortMouseDown(e, node.id, portId)}
          onPortMouseUp={(portId) => (e) => handlePortMouseUp(e, node.id, portId)}
          readOnly={readOnly}
        />
      ))}

      {/* Empty state */}
      {nodes.length === 0 && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center text-gray-500">
            <div className="text-4xl mb-4">🎨</div>
            <h3 className="text-lg font-medium mb-2">Start Building Your Contract</h3>
            <p className="text-sm">Drag components from the palette to begin</p>
          </div>
        </div>
      )}
    </div>
  )
}

// Component for individual canvas nodes
interface CanvasNodeProps {
  node: CanvasNode
  selected: boolean
  onMouseDown: (e: React.MouseEvent) => void
  onPortMouseDown: (portId: string) => (e: React.MouseEvent) => void
  onPortMouseUp: (portId: string) => (e: React.MouseEvent) => void
  readOnly: boolean
}

function CanvasNodeComponent({ 
  node, 
  selected, 
  onMouseDown, 
  onPortMouseDown, 
  onPortMouseUp, 
  readOnly 
}: CanvasNodeProps) {
  return (
    <div
      className={`absolute bg-white border-2 rounded-lg shadow-lg cursor-move transition-all ${
        selected ? 'border-blue-500 shadow-xl' : 'border-gray-300'
      } ${readOnly ? 'cursor-default' : 'cursor-move'}`}
      style={{
        left: node.position.x,
        top: node.position.y,
        width: node.size.width,
        height: node.size.height,
        minWidth: 200,
        minHeight: 100
      }}
      onMouseDown={onMouseDown}
    >
      {/* Node header */}
      <div className="p-3 border-b border-gray-200 bg-gray-50 rounded-t-lg">
        <div className="flex items-center gap-2">
          <span className="text-lg">{node.component.icon}</span>
          <span className="font-medium text-gray-900">{node.component.name}</span>
        </div>
        <p className="text-xs text-gray-600 mt-1">{node.component.description}</p>
      </div>

      {/* Input ports */}
      <div className="absolute -left-2 top-1/2 transform -translate-y-1/2">
        {node.ports.filter(p => p.type === 'input').map(port => (
          <div
            key={port.id}
            className="w-4 h-4 bg-blue-500 border-2 border-white rounded-full cursor-pointer hover:bg-blue-600 mb-2"
            onMouseUp={onPortMouseUp(port.id)}
            title={port.label}
          />
        ))}
      </div>

      {/* Output ports */}
      <div className="absolute -right-2 top-1/2 transform -translate-y-1/2">
        {node.ports.filter(p => p.type === 'output').map(port => (
          <div
            key={port.id}
            className="w-4 h-4 bg-green-500 border-2 border-white rounded-full cursor-pointer hover:bg-green-600 mb-2"
            onMouseDown={onPortMouseDown(port.id)}
            title={port.label}
          />
        ))}
      </div>

      {/* Node content */}
      <div className="p-3">
        <div className="space-y-1">
          {Object.entries(node.config).slice(0, 3).map(([key, value]) => (
            <div key={key} className="text-xs">
              <span className="text-gray-500">{key}:</span>{' '}
              <span className="text-gray-900">{String(value)}</span>
            </div>
          ))}
        </div>
      </div>
    </div>
  )
}

// Component for connection lines
interface ConnectionLineProps {
  sourceX: number
  sourceY: number
  targetX: number
  targetY: number
  selected: boolean
  preview?: boolean
  onClick?: () => void
}

function ConnectionLine({ 
  sourceX, 
  sourceY, 
  targetX, 
  targetY, 
  selected, 
  preview = false, 
  onClick 
}: ConnectionLineProps) {
  const midX = (sourceX + targetX) / 2
  const pathData = `M ${sourceX} ${sourceY} C ${midX} ${sourceY}, ${midX} ${targetY}, ${targetX} ${targetY}`

  return (
    <path
      d={pathData}
      stroke={preview ? '#9ca3af' : selected ? '#3b82f6' : '#6b7280'}
      strokeWidth={preview ? 1 : selected ? 3 : 2}
      strokeDasharray={preview ? '5,5' : 'none'}
      fill="none"
      className={`${onClick ? 'cursor-pointer pointer-events-auto' : ''} transition-all`}
      onClick={onClick}
      markerEnd="url(#arrowhead)"
    />
  )
}