import React from 'react'
import { YStack, XStack, StackProps } from '@tamagui/stacks'

export interface LayoutProps extends StackProps {
  children?: React.ReactNode
}

export const VStack: React.FC<LayoutProps> = ({ children, ...props }) => {
  return <YStack {...props}>{children}</YStack>
}

export const HStack: React.FC<LayoutProps> = ({ children, ...props }) => {
  return <XStack {...props}>{children}</XStack>
}

export const Container: React.FC<LayoutProps> = ({ children, ...props }) => {
  return (
    <YStack
      maxWidth="$20"
      marginHorizontal="auto"
      paddingHorizontal="$4"
      {...props}
    >
      {children}
    </YStack>
  )
} 