import React from 'react'
import { Button as TamaguiButton, ButtonProps as TamaguiButtonProps } from '@tamagui/button'
import { styled } from '@tamagui/core'

// Extended button with ICN-specific styling
const StyledButton = styled(TamaguiButton, {
  name: 'ICNButton',
  variants: {
    variant: {
      primary: {
        backgroundColor: '$blue10',
        color: '$white1',
        hoverStyle: { backgroundColor: '$blue11' },
        pressStyle: { backgroundColor: '$blue12' },
      },
      secondary: {
        backgroundColor: '$gray5',
        color: '$gray12',
        hoverStyle: { backgroundColor: '$gray6' },
        pressStyle: { backgroundColor: '$gray7' },
      },
      ghost: {
        backgroundColor: 'transparent',
        color: '$blue10',
        hoverStyle: { backgroundColor: '$blue3' },
        pressStyle: { backgroundColor: '$blue4' },
      },
      danger: {
        backgroundColor: '$red10',
        color: '$white1',
        hoverStyle: { backgroundColor: '$red11' },
        pressStyle: { backgroundColor: '$red12' },
      },
    },
    size: {
      sm: {
        height: '$3',
        paddingHorizontal: '$3',
        fontSize: '$3',
      },
      md: {
        height: '$4',
        paddingHorizontal: '$4',
        fontSize: '$4',
      },
      lg: {
        height: '$5',
        paddingHorizontal: '$5',
        fontSize: '$5',
      },
    },
  } as const,
  defaultVariants: {
    variant: 'primary',
    size: 'md',
  },
})

export interface ICNButtonProps extends TamaguiButtonProps {
  variant?: 'primary' | 'secondary' | 'ghost' | 'danger'
  size?: 'sm' | 'md' | 'lg'
  loading?: boolean
}

export const Button: React.FC<ICNButtonProps> = ({ 
  children, 
  loading = false, 
  disabled,
  ...props 
}) => {
  return (
    <StyledButton 
      {...props}
      disabled={disabled || loading}
      opacity={loading ? 0.7 : 1}
    >
      {loading ? 'Loading...' : children}
    </StyledButton>
  )
}

export type { ICNButtonProps as ButtonProps } 