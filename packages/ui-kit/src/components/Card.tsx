import React from 'react'
import { Card as TamaguiCard, CardProps } from '@tamagui/card'

export interface ICNCardProps extends CardProps {
  children?: React.ReactNode
}

export const Card: React.FC<ICNCardProps> = ({ children, ...props }) => {
  return (
    <TamaguiCard
      elevate
      size="$4"
      bordered
      borderRadius="$4"
      backgroundColor="$background"
      {...props}
    >
      {children}
    </TamaguiCard>
  )
}

export type { ICNCardProps as CardProps } 