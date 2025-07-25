import React from 'react'
import { Input as TamaguiInput, InputProps as TamaguiInputProps } from '@tamagui/input'
import { Text } from '@tamagui/text'
import { styled } from '@tamagui/core'
import { createFieldProps, generateId, AriaProps } from '@icn/i18n'

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

export interface ICNInputProps extends TamaguiInputProps, AriaProps {
  size?: 'sm' | 'md' | 'lg'
  error?: boolean
  label?: string
  helperText?: string
  errorText?: string
  id?: string
}

export const Input: React.FC<ICNInputProps> = ({ 
  label,
  helperText,
  errorText,
  error,
  id: providedId,
  'aria-label': ariaLabel,
  ...props 
}) => {
  const id = providedId || generateId('input')
  const fieldProps = createFieldProps(id, label, error ? errorText : undefined, helperText)
  
  return (
    <>
      {label && (
        <Text
          id={`${id}-label`}
          as="label"
          htmlFor={id}
          color={error ? '$red10' : '$gray11'}
          fontSize="$3"
          marginBottom="$1"
        >
          {label}
        </Text>
      )}
      <StyledInput 
        {...props}
        {...fieldProps}
        error={error}
        aria-label={ariaLabel || label}
      />
      {helperText && !error && (
        <Text
          id={`${id}-description`}
          color="$gray10"
          fontSize="$2"
          marginTop="$1"
        >
          {helperText}
        </Text>
      )}
      {error && errorText && (
        <Text
          id={`${id}-error`}
          color="$red10"
          fontSize="$2"
          marginTop="$1"
          role="alert"
        >
          {errorText}
        </Text>
      )}
    </>
  )
}

export type { ICNInputProps as InputProps } 