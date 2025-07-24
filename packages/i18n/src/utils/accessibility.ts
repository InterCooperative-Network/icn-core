/**
 * Accessibility utilities for ICN applications
 * Provides helpers for screen readers, keyboard navigation, and ARIA attributes
 */

// Generate unique IDs for accessibility attributes
let idCounter = 0
export function generateId(prefix = 'icn'): string {
  return `${prefix}-${++idCounter}-${Date.now()}`
}

// Screen reader utilities
export const announceToScreenReader = (message: string, priority: 'polite' | 'assertive' = 'polite') => {
  const announcement = document.createElement('div')
  announcement.setAttribute('aria-live', priority)
  announcement.setAttribute('aria-atomic', 'true')
  announcement.className = 'sr-only'
  announcement.textContent = message
  
  document.body.appendChild(announcement)
  
  // Remove after announcement
  setTimeout(() => {
    document.body.removeChild(announcement)
  }, 1000)
}

// Focus management
export const trapFocus = (element: HTMLElement) => {
  const focusableElements = element.querySelectorAll(
    'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
  ) as NodeListOf<HTMLElement>
  
  const firstElement = focusableElements[0]
  const lastElement = focusableElements[focusableElements.length - 1]
  
  const handleTabKey = (e: KeyboardEvent) => {
    if (e.key !== 'Tab') return
    
    if (e.shiftKey) {
      if (document.activeElement === firstElement) {
        lastElement.focus()
        e.preventDefault()
      }
    } else {
      if (document.activeElement === lastElement) {
        firstElement.focus()
        e.preventDefault()
      }
    }
  }
  
  element.addEventListener('keydown', handleTabKey)
  firstElement?.focus()
  
  return () => {
    element.removeEventListener('keydown', handleTabKey)
  }
}

// Keyboard navigation helpers
export const handleEnterOrSpace = (callback: () => void) => (e: React.KeyboardEvent) => {
  if (e.key === 'Enter' || e.key === ' ') {
    e.preventDefault()
    callback()
  }
}

export const handleEscapeKey = (callback: () => void) => (e: React.KeyboardEvent) => {
  if (e.key === 'Escape') {
    e.preventDefault()
    callback()
  }
}

// ARIA label helpers
export interface AriaProps {
  'aria-label'?: string
  'aria-labelledby'?: string
  'aria-describedby'?: string
  'aria-expanded'?: boolean
  'aria-pressed'?: boolean
  'aria-checked'?: boolean
  'aria-selected'?: boolean
  'aria-current'?: boolean | 'page' | 'step' | 'location' | 'date' | 'time'
  'aria-hidden'?: boolean
  'aria-live'?: 'off' | 'polite' | 'assertive'
  'aria-atomic'?: boolean
  role?: string
  tabIndex?: number
}

export const getAriaProps = (props: AriaProps): AriaProps => {
  const ariaProps: AriaProps = {}
  
  Object.keys(props).forEach((key) => {
    if (key.startsWith('aria-') || key === 'role' || key === 'tabIndex') {
      ariaProps[key as keyof AriaProps] = props[key as keyof AriaProps]
    }
  })
  
  return ariaProps
}

// Status announcements
export const announceStatus = (
  status: 'success' | 'error' | 'warning' | 'info',
  message: string,
  t?: (key: string, defaultValue?: string) => string
) => {
  const priority = status === 'error' ? 'assertive' : 'polite'
  const statusText = t ? t(`status.${status}`, status) : status
  announceToScreenReader(`${statusText}: ${message}`, priority)
}

// Loading state helpers
export const createLoadingProps = (
  isLoading: boolean,
  loadingText?: string,
  t?: (key: string, defaultValue?: string) => string
): AriaProps => {
  if (!isLoading) return {}
  
  const defaultLoadingText = t ? t('common.loading', 'Loading') : 'Loading'
  
  return {
    'aria-busy': true,
    'aria-label': loadingText || defaultLoadingText,
  }
}

// Form accessibility helpers
export const createFieldProps = (
  id: string,
  label?: string,
  error?: string,
  description?: string
) => {
  const fieldProps: {
    id: string
    'aria-labelledby'?: string
    'aria-describedby'?: string
    'aria-invalid'?: boolean
  } = { id }
  
  if (label) {
    fieldProps['aria-labelledby'] = `${id}-label`
  }
  
  const describedByIds: string[] = []
  if (description) {
    describedByIds.push(`${id}-description`)
  }
  if (error) {
    describedByIds.push(`${id}-error`)
    fieldProps['aria-invalid'] = true
  }
  
  if (describedByIds.length > 0) {
    fieldProps['aria-describedby'] = describedByIds.join(' ')
  }
  
  return fieldProps
}

// Modal/Dialog accessibility
export const createDialogProps = (
  isOpen: boolean,
  titleId?: string,
  descriptionId?: string
): AriaProps => {
  if (!isOpen) return { 'aria-hidden': true }
  
  return {
    role: 'dialog',
    'aria-modal': true,
    'aria-labelledby': titleId,
    'aria-describedby': descriptionId,
    tabIndex: -1,
  }
}

// Table accessibility helpers
export const createTableHeaderProps = (sortable?: boolean, sortDirection?: 'asc' | 'desc') => {
  const props: AriaProps = {
    role: 'columnheader',
    tabIndex: sortable ? 0 : undefined,
  }
  
  if (sortable && sortDirection) {
    props['aria-sort'] = sortDirection === 'asc' ? 'ascending' : 'descending'
  }
  
  return props
}