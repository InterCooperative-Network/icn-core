import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import Backend from 'i18next-http-backend'

// Import default language resources
import enCommon from '../locales/en/common.json'
import enNavigation from '../locales/en/navigation.json'
import enDashboard from '../locales/en/dashboard.json'
import enAccessibility from '../locales/en/accessibility.json'

import esCommon from '../locales/es/common.json'
import esNavigation from '../locales/es/navigation.json'
import esDashboard from '../locales/es/dashboard.json'
import esAccessibility from '../locales/es/accessibility.json'

export const defaultNS = 'common'

export const resources = {
  en: {
    common: enCommon,
    navigation: enNavigation,
    dashboard: enDashboard,
    accessibility: enAccessibility,
  },
  es: {
    common: esCommon,
    navigation: esNavigation,
    dashboard: esDashboard,
    accessibility: esAccessibility,
  },
} as const

// Supported languages configuration
export const supportedLanguages = [
  { code: 'en', name: 'English', nativeName: 'English' },
  { code: 'es', name: 'Spanish', nativeName: 'Español' },
] as const

export type SupportedLanguage = typeof supportedLanguages[number]['code']

// Initialize i18next
i18n
  .use(Backend)
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    lng: 'en', // Default language
    fallbackLng: 'en',
    defaultNS,
    ns: ['common', 'navigation', 'dashboard', 'accessibility'],
    
    resources,
    
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    
    // Language detection configuration
    detection: {
      order: ['localStorage', 'navigator', 'htmlTag'],
      caches: ['localStorage'],
      lookupLocalStorage: 'icn-language',
    },
    
    // Backend configuration for loading translations
    backend: {
      loadPath: '/locales/{{lng}}/{{ns}}.json',
    },
    
    // React-specific options
    react: {
      useSuspense: false, // Disable suspense to avoid loading issues
    },
    
    // Debug in development
    debug: process.env.NODE_ENV === 'development',
  })

export default i18n