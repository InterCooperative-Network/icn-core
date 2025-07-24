# ICN Internationalization (@icn/i18n)

Internationalization (i18n) and accessibility support for ICN applications.

## Features

- 🌍 **Multi-language support** - Currently supports English and Spanish
- ♿ **Accessibility utilities** - Screen reader support, ARIA attributes, keyboard navigation
- 🎨 **Language switcher component** - Dropdown and button variants
- 🔄 **RTL support** - Ready for right-to-left languages
- 💾 **Language persistence** - Remembers user's language preference
- 🎯 **TypeScript support** - Full type safety for translations

## Supported Languages

- English (en) - Default
- Spanish (es) - Español

## Installation

```bash
# Install peer dependencies if not already installed
pnpm add i18next react-i18next

# The package is available as @icn/i18n in the monorepo
```

## Usage

### Basic Setup

Wrap your app with the I18N provider:

```tsx
import { I18NProvider } from '@icn/i18n'
import './path/to/i18n/config' // Initialize i18n

function App() {
  return (
    <I18NProvider>
      <YourApp />
    </I18NProvider>
  )
}
```

### Using Translations

```tsx
import { useTranslation } from '@icn/i18n'

function MyComponent() {
  const { t } = useTranslation('navigation')
  
  return (
    <h1>{t('menu.dashboard')}</h1>
  )
}
```

### Language Switcher

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

## Translation Namespaces

- `common` - Common UI elements (buttons, status, etc.)
- `navigation` - Navigation menu items and status
- `dashboard` - Dashboard specific content
- `accessibility` - Accessibility labels and announcements

## Adding New Languages

1. Create language files in `locales/{language-code}/`
2. Add the language to `supportedLanguages` in `config.ts`
3. Import and add resources in `config.ts`

Example for French (fr):

```typescript
// locales/fr/common.json
{
  "loading": "Chargement",
  "error": "Erreur",
  // ... other translations
}

// Update config.ts
export const supportedLanguages = [
  { code: 'en', name: 'English', nativeName: 'English' },
  { code: 'es', name: 'Spanish', nativeName: 'Español' },
  { code: 'fr', name: 'French', nativeName: 'Français' },
] as const
```

## Accessibility Features

### Screen Reader Support
- Live announcements for status changes
- Proper ARIA labels and descriptions
- Form validation announcements

### Keyboard Navigation
- Tab trapping for modals
- Enter/Space key handling
- Escape key support

### ARIA Attributes
- Auto-generated field associations
- Loading states
- Dialog/modal support
- Table sorting indicators

## Best Practices

1. **Always provide default values** for translations:
   ```tsx
   {t('key', 'Default text')}
   ```

2. **Use semantic namespaces** to organize translations
3. **Include context** in translation keys for better maintenance
4. **Test with screen readers** to ensure accessibility
5. **Use the accessibility utilities** for consistent behavior

## Development

```bash
# Build the package
pnpm build

# Watch for changes
pnpm dev

# Run type checking
pnpm type-check

# Lint code
pnpm lint
```