import React from 'react'
import { Input as TamaguiInput, InputProps as TamaguiInputProps } from '@tamagui/input'
import { styled } from '@tamagui/core'

const StyledInput = styled(TamaguiInput, {
  name: 'ICNInput',
  borderRadius: '$2',
  borderWidth: 1,
  borderColor: '$gray7',
  paddingHorizontal: '$3',
  backgroundColor: '$background',
  color: '$color',
  focusStyle: {
    borderColor: '$blue8',
    borderWidth: 2,
  },
  variants: {
    size: {
      sm: {
        height: '$3',
        fontSize: '$3',
      },
      md: {
        height: '$4',
        fontSize: '$4',
      },
      lg: {
        height: '$5',
        fontSize: '$5',
      },
    },
    error: {
      true: {
        borderColor: '$red8',
        focusStyle: {
          borderColor: '$red9',
        },
      },
    },
  } as const,
  defaultVariants: {
    size: 'md',
  },
})

export interface ICNInputProps extends TamaguiInputProps {
  size?: 'sm' | 'md' | 'lg'
  error?: boolean
  label?: string
  helperText?: string
}

export const Input: React.FC<ICNInputProps> = ({ 
  label,
  helperText,
  error,
  ...props 
}) => {
  return (
    <>
      {label && (
        <TamaguiInput.Label color={error ? '$red10' : '$gray11'}>
          {label}
        </TamaguiInput.Label>
      )}
      <StyledInput {...props} error={error} />
      {helperText && (
        <TamaguiInput.HelperText color={error ? '$red10' : '$gray10'}>
          {helperText}
        </TamaguiInput.HelperText>
      )}
    </>
  )
}

export type { ICNInputProps as InputProps } 