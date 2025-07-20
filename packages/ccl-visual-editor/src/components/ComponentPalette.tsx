import React from 'react'
import { VStack, ScrollView, Text } from '@tamagui/core'
import { PaletteComponent, Position } from '../types'

interface ComponentPaletteProps {
  components: PaletteComponent[]
  onComponentSelect: (component: PaletteComponent, position?: Position) => void
  touchOptimized?: boolean
  searchable?: boolean
}

export const ComponentPalette: React.FC<ComponentPaletteProps> = ({
  components,
  onComponentSelect,
  touchOptimized = false,
  searchable = false,
}) => {
  return (
    <VStack flex={1} padding="$4" space="$3">
      <Text fontSize="$5" fontWeight="600">Components</Text>
      <ScrollView flex={1}>
        <VStack space="$2">
          {components.map((component) => (
            <Text
              key={component.id}
              padding="$3"
              backgroundColor="$gray3"
              borderRadius="$4"
              onPress={() => onComponentSelect(component)}
              pressStyle={{ backgroundColor: '$gray5' }}
            >
              {component.icon} {component.name}
            </Text>
          ))}
        </VStack>
      </ScrollView>
    </VStack>
  )
} 