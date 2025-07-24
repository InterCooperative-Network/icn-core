import React from 'react'
import { useTranslation } from 'react-i18next'
import { useI18N } from './provider'

interface LanguageSwitcherProps {
  variant?: 'dropdown' | 'buttons'
  className?: string
  showNativeNames?: boolean
  'aria-label'?: string
}

export function LanguageSwitcher({
  variant = 'dropdown',
  className = '',
  showNativeNames = true,
  'aria-label': ariaLabel,
  ...props
}: LanguageSwitcherProps) {
  const { t } = useTranslation('accessibility')
  const { currentLanguage, setLanguage, supportedLanguages } = useI18N()

  const defaultAriaLabel = t('languageSwitcher.label', 'Switch language')

  if (variant === 'buttons') {
    return (
      <div
        className={`flex space-x-2 ${className}`}
        role="group"
        aria-label={ariaLabel || defaultAriaLabel}
        {...props}
      >
        {supportedLanguages.map((language) => (
          <button
            key={language.code}
            onClick={() => setLanguage(language.code)}
            className={`px-3 py-1 rounded-md text-sm font-medium transition-colors ${
              currentLanguage === language.code
                ? 'bg-blue-600 text-white'
                : 'bg-gray-200 text-gray-700 hover:bg-gray-300'
            }`}
            aria-pressed={currentLanguage === language.code}
            aria-label={t('languageSwitcher.switchTo', {
              language: showNativeNames ? language.nativeName : language.name,
              defaultValue: `Switch to ${showNativeNames ? language.nativeName : language.name}`,
            })}
          >
            {showNativeNames ? language.nativeName : language.name}
          </button>
        ))}
      </div>
    )
  }

  return (
    <div className={className}>
      <label
        htmlFor="language-select"
        className="sr-only"
      >
        {ariaLabel || defaultAriaLabel}
      </label>
      <select
        id="language-select"
        value={currentLanguage}
        onChange={(e) => setLanguage(e.target.value as any)}
        className="border border-gray-300 rounded-md px-3 py-2 text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
        aria-label={ariaLabel || defaultAriaLabel}
        {...props}
      >
        {supportedLanguages.map((language) => (
          <option key={language.code} value={language.code}>
            {showNativeNames ? language.nativeName : language.name}
          </option>
        ))}
      </select>
    </div>
  )
}