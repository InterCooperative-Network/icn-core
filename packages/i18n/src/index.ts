// Main i18n export
export { default as i18n } from './config'
export * from './config'

// Provider and hooks
export * from './provider'

// Components
export * from './components/LanguageSwitcher'

// Utilities
export * from './utils/accessibility'

// Re-export react-i18next for convenience
export { useTranslation, Trans } from 'react-i18next'