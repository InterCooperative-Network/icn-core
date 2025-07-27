import React, { useState } from 'react'
import { useTranslation, LanguageSwitcher, useI18N } from '@icn/i18n'

export function SettingsPage() {
  const { t } = useTranslation('common')
  const { currentLanguage, supportedLanguages } = useI18N()
  const [accessibilitySettings, setAccessibilitySettings] = useState({
    reducedMotion: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
    highContrast: window.matchMedia('(prefers-contrast: high)').matches,
    fontSize: 'medium',
    screenReader: false
  })

  const handleAccessibilityChange = (setting: string, value: any) => {
    setAccessibilitySettings(prev => ({
      ...prev,
      [setting]: value
    }))

    // Apply changes immediately
    const root = document.documentElement
    switch (setting) {
      case 'reducedMotion':
        root.style.setProperty('--animation-duration', value ? '0s' : '0.3s')
        break
      case 'highContrast':
        root.classList.toggle('high-contrast', value)
        break
      case 'fontSize':
        root.style.setProperty('--base-font-size', {
          small: '14px',
          medium: '16px',
          large: '18px',
          'extra-large': '20px'
        }[value] || '16px')
        break
    }
  }

  return (
    <div className="space-y-8" id="main-content">
      {/* Header */}
      <header>
        <h1 className="text-3xl font-bold text-gray-900">{t('settings')}</h1>
        <p className="text-gray-600 mt-2">
          Manage your preferences and application settings
        </p>
      </header>

      {/* Accessibility Settings */}
      <section 
        className="bg-white rounded-lg border border-gray-200 p-6"
        aria-labelledby="accessibility-heading"
      >
        <h2 id="accessibility-heading" className="text-xl font-semibold text-gray-900 mb-4">
          Accessibility Preferences
        </h2>
        
        <div className="space-y-6">
          <div>
            <h3 className="text-lg font-medium text-gray-900 mb-3">Motion & Animation</h3>
            <div className="space-y-3">
              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={accessibilitySettings.reducedMotion}
                  onChange={(e) => handleAccessibilityChange('reducedMotion', e.target.checked)}
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span className="ml-3 text-sm text-gray-700">
                  Reduce motion and animations
                </span>
              </label>
              <p className="text-xs text-gray-500 ml-6">
                Minimizes animations for users who prefer reduced motion
              </p>
            </div>
          </div>

          <div className="border-t border-gray-200 pt-6">
            <h3 className="text-lg font-medium text-gray-900 mb-3">Visual Preferences</h3>
            <div className="space-y-4">
              <div>
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={accessibilitySettings.highContrast}
                    onChange={(e) => handleAccessibilityChange('highContrast', e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="ml-3 text-sm text-gray-700">
                    High contrast mode
                  </span>
                </label>
                <p className="text-xs text-gray-500 ml-6">
                  Increases contrast for better visibility
                </p>
              </div>

              <div>
                <label htmlFor="font-size" className="block text-sm font-medium text-gray-700 mb-2">
                  Font Size
                </label>
                <select
                  id="font-size"
                  value={accessibilitySettings.fontSize}
                  onChange={(e) => handleAccessibilityChange('fontSize', e.target.value)}
                  className="form-input w-full sm:w-auto"
                >
                  <option value="small">Small</option>
                  <option value="medium">Medium (Default)</option>
                  <option value="large">Large</option>
                  <option value="extra-large">Extra Large</option>
                </select>
              </div>
            </div>
          </div>

          <div className="border-t border-gray-200 pt-6">
            <h3 className="text-lg font-medium text-gray-900 mb-3">Screen Reader Support</h3>
            <div className="space-y-3">
              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={accessibilitySettings.screenReader}
                  onChange={(e) => handleAccessibilityChange('screenReader', e.target.checked)}
                  className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
                <span className="ml-3 text-sm text-gray-700">
                  Enhanced screen reader support
                </span>
              </label>
              <p className="text-xs text-gray-500 ml-6">
                Provides additional context and descriptions for screen readers
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* Language Settings */}
      <section 
        className="bg-white rounded-lg border border-gray-200 p-6"
        aria-labelledby="language-heading"
      >
        <h2 id="language-heading" className="text-xl font-semibold text-gray-900 mb-4">
          Language & Localization
        </h2>
        
        <div className="space-y-6">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Interface Language
            </label>
            <p className="text-sm text-gray-600 mb-4">
              Choose your preferred language for the interface. Changes take effect immediately.
            </p>
            <div className="flex flex-col sm:flex-row gap-4">
              <LanguageSwitcher 
                variant="dropdown" 
                className="w-full sm:w-auto" 
                showNativeNames={true}
              />
              <LanguageSwitcher 
                variant="buttons" 
                className="w-full sm:w-auto"
                showNativeNames={true}
              />
            </div>
          </div>

          <div className="border-t border-gray-200 pt-6">
            <h3 className="text-lg font-medium text-gray-900 mb-3">Current Language Information</h3>
            <dl className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <dt className="text-sm font-medium text-gray-500">Current Language</dt>
                <dd className="text-sm text-gray-900">
                  {supportedLanguages.find(lang => lang.code === currentLanguage)?.name}
                </dd>
              </div>
              <div>
                <dt className="text-sm font-medium text-gray-500">Native Name</dt>
                <dd className="text-sm text-gray-900">
                  {supportedLanguages.find(lang => lang.code === currentLanguage)?.nativeName}
                </dd>
              </div>
              <div>
                <dt className="text-sm font-medium text-gray-500">Language Code</dt>
                <dd className="text-sm text-gray-900 font-mono">{currentLanguage}</dd>
              </div>
              <div>
                <dt className="text-sm font-medium text-gray-500">Total Supported</dt>
                <dd className="text-sm text-gray-900">{supportedLanguages.length} languages</dd>
              </div>
            </dl>
          </div>

          <div className="border-t border-gray-200 pt-6">
            <h3 className="text-lg font-medium text-gray-900 mb-3">Supported Languages</h3>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
              {supportedLanguages.map(language => (
                <div 
                  key={language.code}
                  className={`p-3 rounded-lg border ${
                    currentLanguage === language.code
                      ? 'border-blue-300 bg-blue-50'
                      : 'border-gray-200 bg-gray-50'
                  }`}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium text-gray-900">{language.name}</p>
                      <p className="text-xs text-gray-600">{language.nativeName}</p>
                    </div>
                    {currentLanguage === language.code && (
                      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                        Active
                      </span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      {/* Accessibility Settings */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Accessibility</h2>
        
        <div className="space-y-6">
          <div>
            <h3 className="text-lg font-medium text-gray-900 mb-3">Features Available</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div className="bg-green-50 p-4 rounded-lg">
                <h4 className="font-medium text-green-900 text-sm">Screen Reader Support</h4>
                <p className="text-green-700 text-xs mt-1">
                  Compatible with NVDA, JAWS, VoiceOver, and TalkBack
                </p>
              </div>
              <div className="bg-blue-50 p-4 rounded-lg">
                <h4 className="font-medium text-blue-900 text-sm">Keyboard Navigation</h4>
                <p className="text-blue-700 text-xs mt-1">
                  Full keyboard support with Tab, Enter, and Escape keys
                </p>
              </div>
              <div className="bg-purple-50 p-4 rounded-lg">
                <h4 className="font-medium text-purple-900 text-sm">Focus Management</h4>
                <p className="text-purple-700 text-xs mt-1">
                  Clear focus indicators and logical tab order
                </p>
              </div>
              <div className="bg-orange-50 p-4 rounded-lg">
                <h4 className="font-medium text-orange-900 text-sm">ARIA Labels</h4>
                <p className="text-orange-700 text-xs mt-1">
                  Comprehensive ARIA attributes for better accessibility
                </p>
              </div>
            </div>
          </div>

          <div className="border-t border-gray-200 pt-6">
            <h3 className="text-lg font-medium text-gray-900 mb-3">Keyboard Shortcuts</h3>
            <div className="overflow-x-auto">
              <table className="min-w-full divide-y divide-gray-200">
                <thead className="bg-gray-50">
                  <tr>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Action
                    </th>
                    <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                      Shortcut
                    </th>
                  </tr>
                </thead>
                <tbody className="bg-white divide-y divide-gray-200">
                  <tr>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">Navigate to main content</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-600">Tab (from top of page)</td>
                  </tr>
                  <tr>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">Activate button or link</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-600">Enter or Space</td>
                  </tr>
                  <tr>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">Close modal or menu</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-600">Escape</td>
                  </tr>
                  <tr>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">Navigate between elements</td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-600">Tab / Shift+Tab</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>

      {/* Developer Information */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Developer Information</h2>
        
        <div className="space-y-4">
          <div>
            <h3 className="text-lg font-medium text-gray-900 mb-2">i18n Implementation</h3>
            <p className="text-sm text-gray-600 mb-3">
              The internationalization system uses react-i18next with namespace-based organization:
            </p>
            <ul className="text-sm text-gray-600 space-y-1">
              <li>• <span className="font-mono">common</span> - Common UI elements and actions</li>
              <li>• <span className="font-mono">navigation</span> - Menu items and navigation</li>
              <li>• <span className="font-mono">dashboard</span> - Dashboard-specific content</li>
              <li>• <span className="font-mono">accessibility</span> - Screen reader labels</li>
            </ul>
          </div>

          <div className="border-t border-gray-200 pt-4">
            <h3 className="text-lg font-medium text-gray-900 mb-2">Adding New Languages</h3>
            <p className="text-sm text-gray-600">
              To add a new language, create translation files in the <span className="font-mono">packages/i18n/locales/</span> directory 
              and update the configuration. See the documentation for detailed instructions.
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}