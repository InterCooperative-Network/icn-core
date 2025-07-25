import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'
import { resources, defaultNS } from '@icn/i18n'

// Initialize i18next for the explorer app
i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    lng: 'en', // Default language
    fallbackLng: 'en',
    defaultNS,
    ns: ['common', 'navigation', 'dashboard', 'accessibility', 'explorer'],
    
    resources,
    
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    
    // Language detection configuration
    detection: {
      order: ['localStorage', 'navigator', 'htmlTag'],
      caches: ['localStorage'],
      lookupLocalStorage: 'icn-explorer-language',
    },
    
    // React-specific options
    react: {
      useSuspense: false, // Disable suspense to avoid loading issues
    },
    
    // Debug in development
    debug: process.env.NODE_ENV === 'development',
  })

export default i18n