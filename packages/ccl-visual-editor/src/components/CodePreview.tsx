import React from 'react'
import { VStack, Text, Button, ScrollView } from '@tamagui/core'
import { EditorState } from '../types'

interface CodePreviewProps {
  code: string
  onCompile?: (code: string, metadata: any) => void
  editorState: EditorState
  errors: string[]
  warnings: string[]
  height?: number
}

export const CodePreview: React.FC<CodePreviewProps> = ({
  code,
  onCompile,
  editorState,
  errors,
  warnings,
  height,
}) => {
  return (
    <VStack flex={1} padding="$4" space="$3" height={height}>
      <Text fontSize="$5" fontWeight="600">Generated CCL Code</Text>
      
      <ScrollView flex={1} backgroundColor="$gray1" padding="$3" borderRadius="$4">
        <Text fontFamily="$mono" fontSize="$2" color="$gray12">
          {code || '// No code generated yet'}
        </Text>
      </ScrollView>
      
      {onCompile && (
        <Button 
          theme="green" 
          onPress={() => onCompile(code, { nodeCount: editorState.nodes.length })}
          disabled={!code}
        >
          Compile & Deploy
        </Button>
      )}
    </VStack>
  )
} 