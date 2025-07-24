import React, { createContext, useContext, ReactNode, useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { supportedLanguages, SupportedLanguage } from './config'

interface I18NContextType {
  currentLanguage: SupportedLanguage
  setLanguage: (language: SupportedLanguage) => void
  supportedLanguages: typeof supportedLanguages
  isLanguageSupported: (language: string) => boolean
  direction: 'ltr' | 'rtl'
}

const I18NContext = createContext<I18NContextType | undefined>(undefined)

interface I18NProviderProps {
  children: ReactNode
  defaultLanguage?: SupportedLanguage
}

export function I18NProvider({ children, defaultLanguage = 'en' }: I18NProviderProps) {
  const { i18n } = useTranslation()
  const [currentLanguage, setCurrentLanguage] = useState<SupportedLanguage>(
    (i18n.language as SupportedLanguage) || defaultLanguage
  )

  const setLanguage = (language: SupportedLanguage) => {
    i18n.changeLanguage(language)
    setCurrentLanguage(language)
    
    // Store in localStorage for persistence
    localStorage.setItem('icn-language', language)
    
    // Update document attributes for accessibility
    document.documentElement.lang = language
    document.documentElement.dir = getLanguageDirection(language)
  }

  const isLanguageSupported = (language: string): boolean => {
    return supportedLanguages.some(lang => lang.code === language)
  }

  const getLanguageDirection = (language: string): 'ltr' | 'rtl' => {
    // Add RTL languages here when supported
    const rtlLanguages = ['ar', 'he', 'fa', 'ur']
    return rtlLanguages.includes(language) ? 'rtl' : 'ltr'
  }

  const direction = getLanguageDirection(currentLanguage)

  // Update language on i18n change
  useEffect(() => {
    const handleLanguageChanged = (language: string) => {
      if (isLanguageSupported(language)) {
        setCurrentLanguage(language as SupportedLanguage)
        document.documentElement.lang = language
        document.documentElement.dir = getLanguageDirection(language)
      }
    }

    i18n.on('languageChanged', handleLanguageChanged)
    return () => i18n.off('languageChanged', handleLanguageChanged)
  }, [i18n])

  // Set initial language attributes
  useEffect(() => {
    document.documentElement.lang = currentLanguage
    document.documentElement.dir = direction
  }, [currentLanguage, direction])

  const value: I18NContextType = {
    currentLanguage,
    setLanguage,
    supportedLanguages,
    isLanguageSupported,
    direction,
  }

  return <I18NContext.Provider value={value}>{children}</I18NContext.Provider>
}

export function useI18N() {
  const context = useContext(I18NContext)
  if (!context) {
    throw new Error('useI18N must be used within an I18NProvider')
  }
  return context
}

// Convenience hook that combines translation and i18n context
export function useTranslationWithContext(namespace?: string) {
  const i18nContext = useI18N()
  const translation = useTranslation(namespace)
  
  return {
    ...translation,
    ...i18nContext,
  }
}