import React, { useState, useEffect, useCallback } from 'react'
import { 
  VStack, 
  HStack, 
  YStack,
  XStack,
  Card,
  Tabs,
  Button,
  Text,
  Theme,
  useMedia
} from '@tamagui/core'

import { VisualEditorProps, EditorState, CanvasNode, Connection, Position } from '../types'
import { GOVERNANCE_COMPONENTS } from '../palette/governanceComponents'
import { ComponentPalette } from './ComponentPalette'
import { CanvasArea } from './CanvasArea'
import { PropertyInspector } from './PropertyInspector'
import { CodePreview } from './CodePreview'

// Default editor configuration
const DEFAULT_CONFIG = {
  platform: 'web' as const,
  inputMethods: ['mouse', 'keyboard', 'touch'] as const,
  accessibility: {
    screenReader: false,
    highContrast: false,
    largeText: false,
    keyboardOnly: false,
  },
  features: {
    voiceCommands: false,
    gestures: false,
    collaboration: false,
    autoSave: true,
  },
  layout: {
    showPalette: true,
    showInspector: true,
    showCode: true,
    panelSizes: {
      palette: 280,
      inspector: 320,
      code: 250,
    }
  }
}

const DEFAULT_STATE: EditorState = {
  nodes: [],
  connections: [],
  selectedNodes: [],
  dragState: null,
  canvasOffset: { x: 0, y: 0 },
  canvasZoom: 1.0,
  isDirty: false,
}

export const VisualEditor: React.FC<VisualEditorProps> = ({
  config = DEFAULT_CONFIG,
  initialState = {},
  onNodeCreate,
  onNodeUpdate,
  onNodeDelete,
  onNodeSelect,
  onConnectionCreate,
  onConnectionDelete,
  onCodeGenerated,
  onContractDeploy,
  onSave,
  onLoad,
  ...props
}) => {
  // Merge configurations
  const editorConfig = { ...DEFAULT_CONFIG, ...config }
  
  // Editor state
  const [editorState, setEditorState] = useState<EditorState>({
    ...DEFAULT_STATE,
    ...initialState,
  })
  
  const [selectedNode, setSelectedNode] = useState<CanvasNode | null>(null)
  const [generatedCode, setGeneratedCode] = useState<string>('')
  
  // Platform detection
  const media = useMedia()
  const isMobile = editorConfig.platform === 'mobile' || media.sm
  const isTouch = editorConfig.inputMethods.includes('touch')
  
  // Handle component creation from palette
  const handleComponentSelect = useCallback((component: typeof GOVERNANCE_COMPONENTS[0], position?: Position) => {
    const newNode: CanvasNode = {
      id: `node_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      type: 'component',
      position: position || { x: 100, y: 100 },
      size: { width: 200, height: 120 },
      component,
      config: { ...component.defaultConfig },
      connections: [],
      selected: false,
      zIndex: editorState.nodes.length,
    }
    
    setEditorState(prev => ({
      ...prev,
      nodes: [...prev.nodes, newNode],
      selectedNodes: [newNode.id],
      isDirty: true,
    }))
    
    setSelectedNode(newNode)
    onNodeCreate?.(component, newNode.position)
  }, [editorState.nodes.length, onNodeCreate])
  
  // Handle node updates
  const handleNodeUpdate = useCallback((nodeId: string, updates: Partial<CanvasNode>) => {
    setEditorState(prev => ({
      ...prev,
      nodes: prev.nodes.map(node => 
        node.id === nodeId ? { ...node, ...updates } : node
      ),
      isDirty: true,
    }))
    
    // Update selected node if it's the one being updated
    if (selectedNode?.id === nodeId) {
      setSelectedNode(prev => prev ? { ...prev, ...updates } : null)
    }
    
    onNodeUpdate?.(nodeId, updates)
  }, [selectedNode?.id, onNodeUpdate])
  
  // Handle node selection
  const handleNodeSelect = useCallback((nodeIds: string[]) => {
    setEditorState(prev => ({
      ...prev,
      selectedNodes: nodeIds,
      nodes: prev.nodes.map(node => ({
        ...node,
        selected: nodeIds.includes(node.id)
      }))
    }))
    
    // Set selected node for property inspector
    if (nodeIds.length === 1) {
      const node = editorState.nodes.find(n => n.id === nodeIds[0])
      setSelectedNode(node || null)
    } else {
      setSelectedNode(null)
    }
    
    onNodeSelect?.(nodeIds)
  }, [editorState.nodes, onNodeSelect])
  
  // Handle property changes
  const handlePropertyChange = useCallback((nodeId: string, property: string, value: any) => {
    const updates = {
      config: {
        ...selectedNode?.config,
        [property]: value
      }
    }
    
    handleNodeUpdate(nodeId, updates)
  }, [selectedNode?.config, handleNodeUpdate])
  
  // Generate CCL code
  const handleCodeGeneration = useCallback(() => {
    // This would integrate with the existing CCL utilities
    // For now, generate a simple template
    const cclCode = generateCCLFromNodes(editorState.nodes, editorState.connections)
    setGeneratedCode(cclCode)
    onCodeGenerated?.(cclCode)
  }, [editorState.nodes, editorState.connections, onCodeGenerated])
  
  // Auto-generate code when nodes change
  useEffect(() => {
    if (editorState.nodes.length > 0) {
      handleCodeGeneration()
    }
  }, [editorState.nodes, editorState.connections, handleCodeGeneration])
  
  // Mobile layout with tabs
  if (isMobile) {
    return (
      <Theme name="light">
        <YStack flex={1} backgroundColor="$background" {...props}>
          <Tabs
            defaultValue="canvas"
            orientation="horizontal"
            flexDirection="column"
            flex={1}
          >
            <Tabs.List
              separator={<Text color="$gray8">|</Text>}
              backgroundColor="$gray2"
              padding="$2"
            >
              <Tabs.Tab value="components" padding="$3">
                <Text>Components</Text>
              </Tabs.Tab>
              <Tabs.Tab value="canvas" padding="$3">
                <Text>Canvas</Text>
              </Tabs.Tab>
              {selectedNode && (
                <Tabs.Tab value="properties" padding="$3">
                  <Text>Properties</Text>
                </Tabs.Tab>
              )}
              <Tabs.Tab value="code" padding="$3">
                <Text>Code</Text>
              </Tabs.Tab>
            </Tabs.List>
            
            <Tabs.Content value="components" flex={1}>
              <ComponentPalette
                components={GOVERNANCE_COMPONENTS}
                onComponentSelect={handleComponentSelect}
                touchOptimized={isTouch}
                searchable
              />
            </Tabs.Content>
            
            <Tabs.Content value="canvas" flex={1}>
              <CanvasArea
                nodes={editorState.nodes}
                connections={editorState.connections}
                selectedNodes={editorState.selectedNodes}
                onNodeUpdate={handleNodeUpdate}
                onNodeSelect={handleNodeSelect}
                touchOptimized={isTouch}
                platform={editorConfig.platform}
              />
            </Tabs.Content>
            
            {selectedNode && (
              <Tabs.Content value="properties" flex={1}>
                <PropertyInspector
                  selectedNode={selectedNode}
                  onPropertyChange={handlePropertyChange}
                  touchOptimized={isTouch}
                />
              </Tabs.Content>
            )}
            
            <Tabs.Content value="code" flex={1}>
              <CodePreview
                code={generatedCode}
                onCompile={onContractDeploy}
                editorState={editorState}
                errors={[]}
                warnings={[]}
              />
            </Tabs.Content>
          </Tabs>
        </YStack>
      </Theme>
    )
  }
  
  // Desktop layout with side panels
  return (
    <Theme name="light">
      <YStack flex={1} backgroundColor="$background" {...props}>
        {/* Header toolbar */}
        <XStack 
          backgroundColor="$gray2" 
          padding="$3" 
          borderBottomWidth={1} 
          borderColor="$gray6"
          alignItems="center"
          justifyContent="space-between"
        >
          <XStack space="$3" alignItems="center">
            <Text fontSize="$6" fontWeight="600">CCL Visual Editor</Text>
            {editorState.isDirty && (
              <Text fontSize="$3" color="$orange10">● Unsaved changes</Text>
            )}
          </XStack>
          
          <XStack space="$2">
            <Button 
              size="$3" 
              theme="blue"
              onPress={handleCodeGeneration}
            >
              Generate Code
            </Button>
            {onContractDeploy && (
              <Button 
                size="$3" 
                theme="green"
                onPress={() => onContractDeploy(generatedCode, { nodeCount: editorState.nodes.length })}
                disabled={!generatedCode}
              >
                Deploy Contract
              </Button>
            )}
          </XStack>
        </XStack>
        
        {/* Main editor area */}
        <HStack flex={1}>
          {/* Component palette */}
          {editorConfig.layout.showPalette && (
            <Card 
              width={editorConfig.layout.panelSizes.palette}
              backgroundColor="$gray1"
              borderRightWidth={1}
              borderColor="$gray6"
            >
              <ComponentPalette
                components={GOVERNANCE_COMPONENTS}
                onComponentSelect={handleComponentSelect}
                touchOptimized={isTouch}
                searchable
              />
            </Card>
          )}
          
          {/* Main canvas area */}
          <VStack flex={1}>
            <CanvasArea
              flex={1}
              nodes={editorState.nodes}
              connections={editorState.connections}
              selectedNodes={editorState.selectedNodes}
              onNodeUpdate={handleNodeUpdate}
              onNodeSelect={handleNodeSelect}
              touchOptimized={isTouch}
              platform={editorConfig.platform}
            />
            
            {/* Code preview at bottom */}
            {editorConfig.layout.showCode && (
              <Card 
                height={editorConfig.layout.panelSizes.code}
                borderTopWidth={1}
                borderColor="$gray6"
              >
                <CodePreview
                  code={generatedCode}
                  onCompile={onContractDeploy}
                  editorState={editorState}
                  errors={[]}
                  warnings={[]}
                />
              </Card>
            )}
          </VStack>
          
          {/* Property inspector */}
          {editorConfig.layout.showInspector && (
            <Card 
              width={editorConfig.layout.panelSizes.inspector}
              backgroundColor="$gray1"
              borderLeftWidth={1}
              borderColor="$gray6"
            >
              <PropertyInspector
                selectedNode={selectedNode}
                onPropertyChange={handlePropertyChange}
                touchOptimized={isTouch}
              />
            </Card>
          )}
        </HStack>
      </YStack>
    </Theme>
  )
}

// Helper function to generate CCL code from visual nodes
function generateCCLFromNodes(nodes: CanvasNode[], connections: Connection[]): string {
  if (nodes.length === 0) {
    return '// Add components to generate CCL code\n\nfn run() -> Integer {\n    return 0;\n}'
  }
  
  const lines: string[] = []
  
  // Generate contract header
  lines.push('// Generated CCL Contract')
  lines.push('// Created with ICN Visual Editor')
  lines.push('')
  
  // Generate functions for each component
  nodes.forEach((node, index) => {
    const component = node.component
    const config = node.config
    
    switch (component.id) {
      case 'voting_mechanism':
        lines.push(`fn create_voting_${index}() -> Proposal {`)
        lines.push(`    let quorum = ${config.quorum};`)
        lines.push(`    let threshold = ${config.threshold};`)
        lines.push(`    let duration = ${config.votingDuration};`)
        lines.push('')
        lines.push(`    // Create voting mechanism with specified parameters`)
        lines.push(`    let proposal = create_proposal(quorum, threshold, duration);`)
        lines.push(`    return proposal;`)
        lines.push(`}`)
        lines.push('')
        break
        
      case 'member_role':
        lines.push(`fn assign_role_${index}(member: Did) -> Bool {`)
        lines.push(`    let role_name = "${config.roleName}";`)
        lines.push(`    let permissions = "${config.permissions}";`)
        lines.push('')
        lines.push(`    // Assign role to member`)
        lines.push(`    assign_member_role(member, role_name, permissions);`)
        lines.push(`    return true;`)
        lines.push(`}`)
        lines.push('')
        break
        
      case 'budget_request':
        lines.push(`fn create_budget_request_${index}() -> Bool {`)
        lines.push(`    let amount = ${config.amount};`)
        lines.push(`    let category = "${config.category}";`)
        lines.push(`    let approval_tier = "${config.approvalTier}";`)
        lines.push('')
        lines.push(`    // Create budget request`)
        lines.push(`    let request = create_budget_request(amount, category, approval_tier);`)
        lines.push(`    return true;`)
        lines.push(`}`)
        lines.push('')
        break
        
      default:
        lines.push(`// Component: ${component.name}`)
        lines.push(`// Configuration: ${JSON.stringify(config, null, 2)}`)
        lines.push('')
    }
  })
  
  // Generate main run function
  lines.push('fn run() -> Integer {')
  lines.push('    // Main contract execution')
  
  nodes.forEach((node, index) => {
    if (node.component.id === 'voting_mechanism') {
      lines.push(`    let proposal_${index} = create_voting_${index}();`)
    } else if (node.component.id === 'budget_request') {
      lines.push(`    create_budget_request_${index}();`)
    }
  })
  
  lines.push('')
  lines.push('    return 0;')
  lines.push('}')
  
  return lines.join('\n')
} 