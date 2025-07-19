import { ICNTokens } from './types'

export const lightTokens: ICNTokens = {
  color: {
    primary: '#0066CC',
    secondary: '#6B7280',
    success: '#10B981',
    warning: '#F59E0B',
    danger: '#EF4444',
    background: '#FFFFFF',
    surface: '#F9FAFB',
    text: '#111827',
    textSecondary: '#6B7280',
    border: '#E5E7EB',
  },
  space: {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
  },
  size: {
    xs: 24,
    sm: 32,
    md: 40,
    lg: 48,
    xl: 56,
  },
  radius: {
    sm: 4,
    md: 8,
    lg: 12,
  },
  fontSize: {
    xs: 12,
    sm: 14,
    md: 16,
    lg: 18,
    xl: 24,
    xxl: 32,
  },
  fontWeight: {
    normal: 400,
    medium: 500,
    semibold: 600,
    bold: 700,
  },
}

export const darkTokens: ICNTokens = {
  ...lightTokens,
  color: {
    primary: '#3B82F6',
    secondary: '#9CA3AF',
    success: '#34D399',
    warning: '#FBBF24',
    danger: '#F87171',
    background: '#111827',
    surface: '#1F2937',
    text: '#F9FAFB',
    textSecondary: '#9CA3AF',
    border: '#374151',
  },
} 