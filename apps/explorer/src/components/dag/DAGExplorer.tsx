import React, { useRef, useEffect, useState, useCallback } from 'react'
import * as d3 from 'd3'
import { ChevronDownIcon, ChevronRightIcon, PlayIcon, PauseIcon, MagnifyingGlassIcon } from '@heroicons/react/24/outline'
import type { DAGData } from '../../hooks/useRealtimeData'

interface DAGExplorerProps {
  data: DAGData | null
  isConnected: boolean
  isRealtime: boolean
}

interface DAGNode {
  id: string
  cid: string
  data: any
  links: string[]
  timestamp: number
  author: string
  size: number
  x?: number
  y?: number
  fx?: number | null
  fy?: number | null
}

interface DAGLink {
  source: string | DAGNode
  target: string | DAGNode
  value: number
}

export function DAGExplorer({ data, isConnected, isRealtime }: DAGExplorerProps) {
  const svgRef = useRef<SVGSVGElement>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  
  const [selectedNode, setSelectedNode] = useState<DAGNode | null>(null)
  const [searchTerm, setSearchTerm] = useState('')
  const [isPaused, setIsPaused] = useState(false)
  const [showDetails, setShowDetails] = useState(true)
  const [zoomLevel, setZoomLevel] = useState(1)
  
  // Convert DAG data to D3 format
  const processDAGData = useCallback((dagData: DAGData): { nodes: DAGNode[], links: DAGLink[] } => {
    if (!dagData?.blocks) return { nodes: [], links: [] }
    
    const nodes: DAGNode[] = dagData.blocks.map(block => ({
      id: block.cid,
      cid: block.cid,
      data: block.data,
      links: block.links,
      timestamp: block.timestamp,
      author: block.author,
      size: block.size
    }))
    
    const links: DAGLink[] = []
    
    // Create links based on block references
    dagData.blocks.forEach(block => {
      block.links.forEach(linkCid => {
        if (nodes.find(n => n.cid === linkCid)) {
          links.push({
            source: block.cid,
            target: linkCid,
            value: 1
          })
        }
      })
    })
    
    return { nodes, links }
  }, [])

  // Initialize and update D3 visualization
  useEffect(() => {
    if (!svgRef.current || !containerRef.current || !data || isPaused) return
    
    const svg = d3.select(svgRef.current)
    const container = containerRef.current
    const { width, height } = container.getBoundingClientRect()
    
    // Clear previous content
    svg.selectAll('*').remove()
    
    const { nodes, links } = processDAGData(data)
    
    if (nodes.length === 0) {
      // Show empty state
      svg.append('text')
        .attr('x', width / 2)
        .attr('y', height / 2)
        .attr('text-anchor', 'middle')
        .attr('class', 'fill-gray-400 text-lg')
        .text('No DAG blocks available')
      return
    }
    
    // Set up zoom behavior
    const zoom = d3.zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.1, 4])
      .on('zoom', (event) => {
        g.attr('transform', event.transform)
        setZoomLevel(event.transform.k)
      })
    
    svg.call(zoom)
    
    // Main group for all content
    const g = svg.append('g')
    
    // Create force simulation
    const simulation = d3.forceSimulation<DAGNode>(nodes)
      .force('link', d3.forceLink<DAGNode, DAGLink>(links)
        .id(d => d.id)
        .distance(100)
        .strength(0.3)
      )
      .force('charge', d3.forceManyBody().strength(-200))
      .force('center', d3.forceCenter(width / 2, height / 2))
      .force('collision', d3.forceCollide().radius(20))
    
    // Create links
    const link = g.append('g')
      .attr('class', 'links')
      .selectAll('line')
      .data(links)
      .enter().append('line')
      .attr('class', 'stroke-gray-300 dark:stroke-gray-600')
      .attr('stroke-width', 2)
      .attr('opacity', 0.6)
    
    // Create nodes
    const node = g.append('g')
      .attr('class', 'nodes')
      .selectAll('circle')
      .data(nodes)
      .enter().append('circle')
      .attr('r', d => Math.max(8, Math.min(20, Math.sqrt(d.size / 1000))))
      .attr('class', d => {
        const age = Date.now() - d.timestamp
        if (age < 60000) return 'fill-green-500' // New blocks (< 1min)
        if (age < 3600000) return 'fill-blue-500' // Recent blocks (< 1hr)
        return 'fill-gray-400' // Older blocks
      })
      .attr('stroke', '#fff')
      .attr('stroke-width', 2)
      .style('cursor', 'pointer')
      .call(d3.drag<SVGCircleElement, DAGNode>()
        .on('start', (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart()
          d.fx = d.x
          d.fy = d.y
        })
        .on('drag', (event, d) => {
          d.fx = event.x
          d.fy = event.y
        })
        .on('end', (event, d) => {
          if (!event.active) simulation.alphaTarget(0)
          d.fx = null
          d.fy = null
        })
      )
      .on('click', (event, d) => {
        setSelectedNode(d)
      })
      .on('mouseover', function(event, d) {
        // Highlight connected nodes
        const connectedNodeIds = new Set()
        links.forEach(l => {
          if (l.source === d.id) connectedNodeIds.add(l.target)
          if (l.target === d.id) connectedNodeIds.add(l.source)
        })
        
        node.style('opacity', n => connectedNodeIds.has(n.id) || n.id === d.id ? 1 : 0.3)
        link.style('opacity', l => l.source === d.id || l.target === d.id ? 0.8 : 0.1)
        
        // Show tooltip
        const tooltip = d3.select('body').append('div')
          .attr('class', 'absolute bg-gray-900 text-white p-2 rounded shadow-lg pointer-events-none z-50')
          .style('left', (event.pageX + 10) + 'px')
          .style('top', (event.pageY - 10) + 'px')
        
        tooltip.html(`
          <div class="text-sm">
            <div class="font-medium">${d.cid.slice(0, 12)}...</div>
            <div class="text-gray-300">Size: ${formatBytes(d.size)}</div>
            <div class="text-gray-300">Links: ${d.links.length}</div>
            <div class="text-gray-300">Age: ${formatTime(Date.now() - d.timestamp)}</div>
          </div>
        `)
        
        d3.select(this).transition().duration(100).attr('r', d => Math.max(12, Math.min(24, Math.sqrt(d.size / 1000))))
      })
      .on('mouseout', function() {
        node.style('opacity', 1)
        link.style('opacity', 0.6)
        d3.selectAll('.tooltip').remove()
        d3.select('body').selectAll('div').filter(function() {
          return d3.select(this).classed('absolute') && d3.select(this).classed('bg-gray-900')
        }).remove()
        
        d3.select(this).transition().duration(100).attr('r', d => Math.max(8, Math.min(20, Math.sqrt(d.size / 1000))))
      })
    
    // Add labels for important nodes
    const label = g.append('g')
      .attr('class', 'labels')
      .selectAll('text')
      .data(nodes.filter(d => d.size > 10000 || Date.now() - d.timestamp < 300000)) // Large nodes or very recent
      .enter().append('text')
      .attr('class', 'fill-gray-700 dark:fill-gray-300 text-xs pointer-events-none')
      .attr('text-anchor', 'middle')
      .attr('dy', -25)
      .text(d => d.cid.slice(0, 8) + '...')
    
    // Update positions on simulation tick
    simulation.on('tick', () => {
      link
        .attr('x1', d => (d.source as DAGNode).x!)
        .attr('y1', d => (d.source as DAGNode).y!)
        .attr('x2', d => (d.target as DAGNode).x!)
        .attr('y2', d => (d.target as DAGNode).y!)
      
      node
        .attr('cx', d => d.x!)
        .attr('cy', d => d.y!)
      
      label
        .attr('x', d => d.x!)
        .attr('y', d => d.y!)
    })
    
    // Cleanup
    return () => {
      simulation.stop()
    }
    
  }, [data, isPaused, processDAGData])

  // Filter nodes based on search
  const filteredData = React.useMemo(() => {
    if (!data || !searchTerm) return data
    
    const filtered = data.blocks.filter(block => 
      block.cid.toLowerCase().includes(searchTerm.toLowerCase()) ||
      block.author.toLowerCase().includes(searchTerm.toLowerCase())
    )
    
    return {
      ...data,
      blocks: filtered
    }
  }, [data, searchTerm])

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i]
  }

  const formatTime = (ms: number): string => {
    const seconds = Math.floor(ms / 1000)
    const minutes = Math.floor(seconds / 60)
    const hours = Math.floor(minutes / 60)
    
    if (hours > 0) return `${hours}h ${minutes % 60}m ago`
    if (minutes > 0) return `${minutes}m ${seconds % 60}s ago`
    return `${seconds}s ago`
  }

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-800 rounded-lg shadow-lg">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center space-x-4">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
            DAG Explorer
          </h2>
          
          <div className="flex items-center space-x-2">
            <div className={`w-2 h-2 rounded-full ${isConnected ? 'bg-green-500' : 'bg-red-500'}`} />
            <span className="text-sm text-gray-500 dark:text-gray-400">
              {isConnected ? 'Connected' : 'Disconnected'}
            </span>
            {isRealtime && (
              <span className="text-xs bg-green-100 text-green-800 px-2 py-1 rounded-full">
                Real-time
              </span>
            )}
          </div>
        </div>
        
        <div className="flex items-center space-x-2">
          <div className="relative">
            <MagnifyingGlassIcon className="w-4 h-4 absolute left-3 top-1/2 transform -translate-y-1/2 text-gray-400" />
            <input
              type="text"
              placeholder="Search blocks..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-10 pr-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg text-sm bg-white dark:bg-gray-700"
            />
          </div>
          
          <button
            onClick={() => setIsPaused(!isPaused)}
            className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            {isPaused ? <PlayIcon className="w-5 h-5" /> : <PauseIcon className="w-5 h-5" />}
          </button>
          
          <button
            onClick={() => setShowDetails(!showDetails)}
            className="p-2 text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200"
          >
            {showDetails ? <ChevronDownIcon className="w-5 h-5" /> : <ChevronRightIcon className="w-5 h-5" />}
          </button>
        </div>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Main visualization */}
        <div className="flex-1 relative" ref={containerRef}>
          <svg
            ref={svgRef}
            className="w-full h-full"
            style={{ background: 'radial-gradient(circle at 50% 50%, #f8fafc 0%, #e2e8f0 100%)' }}
          />
          
          {/* Zoom indicator */}
          <div className="absolute bottom-4 left-4 bg-white dark:bg-gray-800 px-3 py-1 rounded shadow text-sm">
            Zoom: {Math.round(zoomLevel * 100)}%
          </div>
          
          {/* Stats overlay */}
          <div className="absolute top-4 left-4 bg-white dark:bg-gray-800 p-3 rounded shadow">
            <div className="text-sm space-y-1">
              <div>Blocks: <span className="font-medium">{data?.blockCount || 0}</span></div>
              <div>Size: <span className="font-medium">{formatBytes(data?.totalSize || 0)}</span></div>
              <div>Recent: <span className="font-medium">{data?.recentActivity?.length || 0}</span></div>
            </div>
          </div>
        </div>

        {/* Details panel */}
        {showDetails && (
          <div className="w-80 border-l border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 overflow-y-auto">
            <div className="p-4">
              <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
                {selectedNode ? 'Block Details' : 'Recent Activity'}
              </h3>
              
              {selectedNode ? (
                <div className="space-y-4">
                  <div>
                    <label className="text-sm font-medium text-gray-700 dark:text-gray-300">CID</label>
                    <div className="mt-1 p-2 bg-white dark:bg-gray-800 rounded border text-xs font-mono break-all">
                      {selectedNode.cid}
                    </div>
                  </div>
                  
                  <div>
                    <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Author</label>
                    <div className="mt-1 text-sm text-gray-600 dark:text-gray-400 font-mono">
                      {selectedNode.author}
                    </div>
                  </div>
                  
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Size</label>
                      <div className="mt-1 text-sm text-gray-600 dark:text-gray-400">
                        {formatBytes(selectedNode.size)}
                      </div>
                    </div>
                    
                    <div>
                      <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Links</label>
                      <div className="mt-1 text-sm text-gray-600 dark:text-gray-400">
                        {selectedNode.links.length}
                      </div>
                    </div>
                  </div>
                  
                  <div>
                    <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Timestamp</label>
                    <div className="mt-1 text-sm text-gray-600 dark:text-gray-400">
                      {new Date(selectedNode.timestamp).toLocaleString()}
                    </div>
                  </div>
                  
                  {selectedNode.links.length > 0 && (
                    <div>
                      <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Linked Blocks</label>
                      <div className="mt-2 space-y-1">
                        {selectedNode.links.slice(0, 5).map(link => (
                          <div key={link} className="text-xs font-mono text-blue-600 dark:text-blue-400 truncate">
                            {link}
                          </div>
                        ))}
                        {selectedNode.links.length > 5 && (
                          <div className="text-xs text-gray-500">
                            +{selectedNode.links.length - 5} more
                          </div>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              ) : (
                <div className="space-y-3">
                  {data?.recentActivity?.slice(0, 10).map((activity, index) => (
                    <div key={index} className="flex items-center space-x-3 p-2 bg-white dark:bg-gray-800 rounded">
                      <div className={`w-2 h-2 rounded-full ${activity.type === 'put' ? 'bg-green-500' : 'bg-blue-500'}`} />
                      <div className="flex-1 min-w-0">
                        <div className="text-xs font-mono text-gray-600 dark:text-gray-400 truncate">
                          {activity.cid}
                        </div>
                        <div className="text-xs text-gray-500">
                          {formatTime(Date.now() - activity.timestamp)}
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  )
} 