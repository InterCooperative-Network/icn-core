# ICN Internationalization and Accessibility Implementation

This document describes the internationalization (i18n) and accessibility features implemented for the ICN Core project.

## Overview

The ICN Core now supports multiple languages and follows accessibility best practices to ensure the platform is inclusive and accessible to users worldwide.

## Features Implemented

### 🌍 Internationalization (i18n)

- **Multi-language support**: English (default) and Spanish
- **Namespace organization**: Translations organized by functionality (common, navigation, dashboard, accessibility)
- **Language persistence**: User's language preference is remembered across sessions
- **Automatic language detection**: Detects browser language with fallback to English
- **TypeScript support**: Full type safety for translation keys
- **RTL support**: Ready for right-to-left languages (Arabic, Hebrew, etc.)

### ♿ Accessibility Features

- **Screen reader support**: Proper ARIA labels, live regions, and announcements
- **Keyboard navigation**: Tab trapping, Enter/Space key handling, Escape key support
- **Focus management**: Visible focus indicators and logical tab order
- **Semantic HTML**: Proper use of headings, landmarks, and roles
- **Loading states**: Accessible loading indicators with proper ARIA attributes
- **Error handling**: Screen reader announcements for errors and status changes

## Supported Languages

| Language | Code | Status | Coverage |
|----------|------|--------|----------|
| English  | `en` | ✅ Complete | 100% |
| Spanish  | `es` | ✅ Complete | 100% |

## Implementation Details

### Package Structure

```
packages/i18n/
├── src/
│   ├── components/
│   │   └── LanguageSwitcher.tsx    # Language switching component
│   ├── utils/
│   │   └── accessibility.ts        # Accessibility utilities
│   ├── config.ts                   # i18n configuration
│   ├── provider.tsx                # React context provider
│   └── index.ts                    # Main exports
├── locales/
│   ├── en/                         # English translations
│   │   ├── common.json
│   │   ├── navigation.json
│   │   ├── dashboard.json
│   │   └── accessibility.json
│   └── es/                         # Spanish translations
│       ├── common.json
│       ├── navigation.json
│       ├── dashboard.json
│       └── accessibility.json
└── package.json
```

### Translation Namespaces

1. **`common`**: Common UI elements (buttons, status messages, actions)
2. **`navigation`**: Navigation menu items and brand name
3. **`dashboard`**: Dashboard-specific content and metrics
4. **`accessibility`**: Accessibility labels and screen reader text

### Components Updated

#### Navigation Component
- Translated menu items and status indicators
- Added skip-to-content link for screen readers
- Implemented mobile menu with proper ARIA attributes
- Added language switcher in header
- Focus management for keyboard navigation

#### Dashboard Component
- Translated all user-facing text
- Added ARIA live regions for dynamic content
- Proper loading state announcements
- Error messages with screen reader support
- Semantic heading structure

#### UI Kit Button Component
- Enhanced with accessibility attributes
- Loading state support with proper ARIA labels
- Keyboard navigation support
- Screen reader announcements

## Usage Examples

### Basic Translation

```tsx
import { useTranslation } from '@icn/i18n'

function MyComponent() {
  const { t } = useTranslation('navigation')
  
  return <h1>{t('menu.dashboard')}</h1>
}
```

### Language Switching

```tsx
import { LanguageSwitcher } from '@icn/i18n'

function Header() {
  return (
    <div>
      {/* Dropdown variant */}
      <LanguageSwitcher variant="dropdown" />
      
      {/* Button variant */}
      <LanguageSwitcher variant="buttons" />
    </div>
  )
}
```

### Accessibility Utilities

```tsx
import { 
  announceToScreenReader, 
  createFieldProps, 
  handleEnterOrSpace 
} from '@icn/i18n'

function AccessibleForm() {
  const fieldProps = createFieldProps('email', 'Email', error, 'Enter your email')
  
  const handleSubmit = () => {
    announceToScreenReader('Form submitted successfully')
  }
  
  return (
    <form>
      <label id="email-label">Email</label>
      <input 
        {...fieldProps}
        onKeyDown={handleEnterOrSpace(handleSubmit)}
      />
    </form>
  )
}
```

## Adding New Languages

To add support for a new language:

1. **Create language files** in `packages/i18n/locales/{language-code}/`
2. **Translate all namespaces** (common.json, navigation.json, dashboard.json, accessibility.json)
3. **Update configuration** in `packages/i18n/src/config.ts`:
   - Add to `supportedLanguages` array
   - Import and add to `resources` object
4. **Test thoroughly** with the new language

Example for French (fr):

```typescript
// Add to supportedLanguages
{ code: 'fr', name: 'French', nativeName: 'Français' }

// Import translations
import frCommon from '../locales/fr/common.json'
// ... other imports

// Add to resources
fr: {
  common: frCommon,
  navigation: frNavigation,
  dashboard: frDashboard,
  accessibility: frAccessibility,
}
```

## Accessibility Guidelines Followed

### WCAG 2.1 AA Compliance

- **Perceivable**: Text alternatives, captions, and proper color contrast
- **Operable**: Keyboard accessible, no seizure-inducing content
- **Understandable**: Readable text, predictable functionality
- **Robust**: Compatible with assistive technologies

### Screen Reader Support

- Proper heading hierarchy (h1 → h2 → h3)
- ARIA landmarks (navigation, main, banner)
- Live regions for dynamic content updates
- Descriptive link text and button labels

### Keyboard Navigation

- Logical tab order
- Visible focus indicators
- Skip links for main content
- Modal focus trapping

## Testing

### Manual Testing

1. **Language switching**: Verify all text changes when switching languages
2. **Screen reader**: Test with NVDA, JAWS, or VoiceOver
3. **Keyboard navigation**: Navigate entire app using only keyboard
4. **Mobile accessibility**: Test on mobile devices with TalkBack/VoiceOver

### Automated Testing

Consider adding these tests:

```typescript
// Test translation coverage
test('all translation keys have values', () => {
  // Verify no missing translations
})

// Test accessibility
test('navigation is keyboard accessible', () => {
  // Test tab navigation
})

test('screen reader announcements work', () => {
  // Test ARIA live regions
})
```

## Future Enhancements

### Additional Languages
- **Arabic** (ar) - RTL support
- **Chinese** (zh) - Traditional and Simplified
- **French** (fr)
- **German** (de)
- **Portuguese** (pt)
- **Hindi** (hi)

### Advanced Features
- **Pluralization** support for complex languages
- **Date/time localization** with proper formatting
- **Number formatting** based on locale
- **Currency formatting** for different regions
- **Contextual translations** based on user role

### Accessibility Enhancements
- **High contrast mode** support
- **Reduced motion** preferences
- **Font size scaling** support
- **Color blind friendly** design
- **Voice control** compatibility

## Best Practices

### For Developers

1. **Always provide default values** for translations
2. **Use semantic HTML** elements
3. **Test with keyboard only**
4. **Verify screen reader compatibility**
5. **Keep translation keys descriptive**

### For Translators

1. **Maintain context** when translating
2. **Keep UI text concise** but clear
3. **Test translations** in the actual UI
4. **Consider cultural differences**
5. **Verify accessibility labels** make sense

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [React i18next Documentation](https://react.i18next.com/)
- [Inclusive Design Principles](https://inclusivedesignprinciples.org/)

## Contributing

To contribute to i18n and accessibility:

1. Follow the established patterns for translations
2. Test all changes with screen readers
3. Verify keyboard navigation works
4. Update documentation for new features
5. Consider cultural and linguistic differences

---

This implementation provides a solid foundation for making ICN Core accessible to users worldwide, regardless of their language or accessibility needs.