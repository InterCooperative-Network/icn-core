import React from 'react'
import { VStack, Text } from '@tamagui/core'
import { CanvasNode } from '../types'

interface PropertyInspectorProps {
  selectedNode: CanvasNode | null
  onPropertyChange: (nodeId: string, property: string, value: any) => void
  touchOptimized?: boolean
}

export const PropertyInspector: React.FC<PropertyInspectorProps> = ({
  selectedNode,
  onPropertyChange,
  touchOptimized = false,
}) => {
  return (
    <VStack flex={1} padding="$4" space="$3">
      <Text fontSize="$5" fontWeight="600">Properties</Text>
      {selectedNode ? (
        <VStack space="$3">
          <Text fontSize="$4">{selectedNode.component.name}</Text>
          <Text fontSize="$3" color="$gray10">
            {selectedNode.component.description}
          </Text>
        </VStack>
      ) : (
        <Text fontSize="$3" color="$gray8">
          Select a component to edit properties
        </Text>
      )}
    </VStack>
  )
} 