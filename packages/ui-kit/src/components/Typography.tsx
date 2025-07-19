import React from 'react'
import { Text, TextProps } from '@tamagui/text'

export interface ICNTextProps extends TextProps {
  children?: React.ReactNode
}

export const Heading: React.FC<ICNTextProps> = ({ children, ...props }) => {
  return (
    <Text
      fontSize="$8"
      fontWeight="bold"
      color="$color12"
      {...props}
    >
      {children}
    </Text>
  )
}

export const Subheading: React.FC<ICNTextProps> = ({ children, ...props }) => {
  return (
    <Text
      fontSize="$6"
      fontWeight="600"
      color="$color11"
      {...props}
    >
      {children}
    </Text>
  )
}

export const Body: React.FC<ICNTextProps> = ({ children, ...props }) => {
  return (
    <Text
      fontSize="$4"
      color="$color"
      lineHeight="$2"
      {...props}
    >
      {children}
    </Text>
  )
}

export const Caption: React.FC<ICNTextProps> = ({ children, ...props }) => {
  return (
    <Text
      fontSize="$3"
      color="$color10"
      {...props}
    >
      {children}
    </Text>
  )
} 