import React from 'react'
import { useTranslation, LanguageSwitcher, useI18N } from '@icn/i18n'

export function SettingsPage() {
  const { t } = useTranslation('common')
  const { currentLanguage, supportedLanguages } = useI18N()

  return (
    <div className="space-y-8" id="main-content">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900">{t('settings')}</h1>
        <p className="text-gray-600 mt-2">
          Manage your preferences and application settings
        </p>
      </div>

      {/* Language Settings */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h2 className="text-xl font-semibold text-gray-900 mb-4">Language & Localization</h2>
        
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