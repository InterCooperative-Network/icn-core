import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import { resources, defaultNS } from '@icn/i18n'

// Initialize i18next for the wallet app
i18n
  .use(initReactI18next)
  .init({
    lng: 'en', // Default language
    fallbackLng: 'en',
    defaultNS,
    ns: ['common', 'navigation', 'dashboard', 'accessibility', 'explorer', 'wallet'],
    
    resources,
    
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    
    // React-specific options
    react: {
      useSuspense: false, // Disable suspense to avoid loading issues
    },
    
    // Debug in development
    debug: process.env.NODE_ENV === 'development',
  })

export default i18n