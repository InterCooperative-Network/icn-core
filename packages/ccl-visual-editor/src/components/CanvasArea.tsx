import React from 'react'
import { VStack, Text } from '@tamagui/core'
import { CanvasNode, Connection, Platform } from '../types'

interface CanvasAreaProps {
  nodes: CanvasNode[]
  connections: Connection[]
  selectedNodes: string[]
  onNodeUpdate: (nodeId: string, updates: Partial<CanvasNode>) => void
  onNodeSelect: (nodeIds: string[]) => void
  touchOptimized?: boolean
  platform?: Platform
  flex?: number
}

export const CanvasArea: React.FC<CanvasAreaProps> = ({
  nodes,
  connections,
  selectedNodes,
  onNodeUpdate,
  onNodeSelect,
  touchOptimized = false,
  platform = 'web',
  ...props
}) => {
  return (
    <VStack 
      flex={1} 
      backgroundColor="$gray0" 
      alignItems="center" 
      justifyContent="center"
      {...props}
    >
      <Text fontSize="$4" color="$gray10">
        Canvas Area ({nodes.length} nodes)
      </Text>
      <Text fontSize="$3" color="$gray8">
        Drag and drop components here
      </Text>
    </VStack>
  )
} 