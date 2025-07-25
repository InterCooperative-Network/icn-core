# ICN Internationalization Language Addition Guide

## Overview

This guide provides step-by-step instructions for adding new languages to the ICN Core internationalization system.

## Current Language Support

| Language | Code | Status | Namespaces | Coverage |
|----------|------|--------|------------|----------|
| English  | `en` | ✅ Complete | 6 | 100% |
| Spanish  | `es` | ✅ Complete | 6 | 100% |
| French   | `fr` | ✅ Complete | 6 | 100% |

## Adding a New Language

### Step 1: Create Language Directory

Create a new directory for your language in the locales folder:

```bash
mkdir packages/i18n/locales/{language-code}
```

Example for German:
```bash
mkdir packages/i18n/locales/de
```

### Step 2: Create Translation Files

Create all required namespace files in your language directory:

```bash
# Required namespace files
touch packages/i18n/locales/de/common.json
touch packages/i18n/locales/de/navigation.json  
touch packages/i18n/locales/de/dashboard.json
touch packages/i18n/locales/de/accessibility.json
touch packages/i18n/locales/de/explorer.json
touch packages/i18n/locales/de/wallet.json
```

### Step 3: Translate Content

#### common.json
Contains general UI elements, buttons, and status messages.

```json
{
  "loading": "Laden",
  "error": "Fehler", 
  "success": "Erfolg",
  "warning": "Warnung",
  "info": "Information",
  "cancel": "Abbrechen",
  "confirm": "Bestätigen",
  "save": "Speichern",
  "delete": "Löschen",
  "edit": "Bearbeiten",
  "create": "Erstellen",
  "update": "Aktualisieren",
  "submit": "Absenden",
  "close": "Schließen",
  "open": "Öffnen",
  "search": "Suchen",
  "filter": "Filtern",
  "sort": "Sortieren",
  "refresh": "Aktualisieren",
  "reload": "Neu laden",
  "retry": "Wiederholen",
  "back": "Zurück",
  "next": "Weiter",
  "previous": "Vorherige",
  "continue": "Fortfahren",
  "finish": "Fertig",
  "yes": "Ja",
  "no": "Nein",
  "ok": "OK",
  "status": {
    "online": "Online",
    "offline": "Offline", 
    "connected": "Verbunden",
    "disconnected": "Getrennt",
    "active": "Aktiv",
    "inactive": "Inaktiv"
  }
}
```

#### navigation.json
Contains menu items and navigation elements.

```json
{
  "brand": "ICN Föderation",
  "menu": {
    "demo": "Demo",
    "dashboard": "Dashboard", 
    "federation": "Föderation",
    "governance": "Governance",
    "cooperatives": "Genossenschaften",
    "meshJobs": "Mesh-Aufträge",
    "settings": "Einstellungen"
  },
  "status": {
    "connected": "Verbunden",
    "disconnected": "Getrennt",
    "network": "Netzwerk",
    "did": "DID"
  },
  "accessibility": {
    "skipToContent": "Zum Hauptinhalt springen",
    "mainNavigation": "Hauptnavigation", 
    "currentPage": "Aktuelle Seite"
  }
}
```

#### dashboard.json
Contains dashboard-specific content and metrics.

```json
{
  "title": "Föderations-Dashboard",
  "subtitle": "Überwachen und verwalten Sie Ihre Genossenschaftsföderation",
  "metrics": {
    "totalCooperatives": "Genossenschaften gesamt",
    "totalMembers": "Mitglieder gesamt",
    "activeProposals": "Aktive Vorschläge",
    "networkPeers": "Netzwerk-Peers"
  },
  "health": {
    "title": "Föderationsgesundheit",
    "overall": "Gesamtgesundheit",
    "network": "Netzwerkkonnektivität",
    "governance": "Governance-Aktivität"
  }
}
```

#### accessibility.json
Contains accessibility labels and screen reader text.

```json
{
  "languageSwitcher": {
    "label": "Sprache wechseln",
    "switchTo": "Wechseln zu {{language}}"
  },
  "buttons": {
    "loading": "Laden, bitte warten",
    "submit": "Formular absenden",
    "cancel": "Aktion abbrechen"
  },
  "forms": {
    "required": "Dieses Feld ist erforderlich",
    "invalid": "Dieses Feld ist ungültig"
  },
  "status": {
    "success": "Erfolg",
    "error": "Fehler",
    "warning": "Warnung", 
    "info": "Information"
  }
}
```

#### explorer.json
Contains DAG Explorer interface elements.

```json
{
  "title": "DAG Explorer",
  "status": {
    "connected": "Verbunden",
    "disconnected": "Getrennt",
    "realtime": "Echtzeit",
    "paused": "Pausiert"
  },
  "controls": {
    "pause": "Updates pausieren",
    "resume": "Updates fortsetzen", 
    "search": "Blöcke suchen",
    "searchPlaceholder": "Nach CID oder Autor suchen..."
  },
  "dag": {
    "noBlocks": "Keine DAG-Blöcke verfügbar",
    "blockCount": "{{count}} Blöcke",
    "selectedBlock": "Ausgewählter Block"
  }
}
```

#### wallet.json
Contains wallet interface and security features.

```json
{
  "title": "ICN Wallet",
  "subtitle": "Sichere DID- und Schlüsselverwaltung für das InterCooperative Network",
  "actions": {
    "createWallet": "Neue Wallet erstellen",
    "importWallet": "Vorhandene Wallet importieren",
    "connectNode": "Mit Knoten verbinden"
  },
  "security": {
    "enterPassword": "Passwort eingeben",
    "confirmPassword": "Passwort bestätigen",
    "createPassword": "Sicheres Passwort erstellen"
  },
  "wallet": {
    "balance": "Guthaben",
    "address": "Adresse",
    "did": "DID",
    "keys": "Schlüssel"
  }
}
```

### Step 4: Update Configuration

Update `packages/i18n/src/config.ts` to include the new language:

```typescript
// Add import for new language
import deCommon from '../locales/de/common.json'
import deNavigation from '../locales/de/navigation.json'
import deDashboard from '../locales/de/dashboard.json'
import deAccessibility from '../locales/de/accessibility.json'
import deExplorer from '../locales/de/explorer.json'
import deWallet from '../locales/de/wallet.json'

// Add to supportedLanguages array
export const supportedLanguages = [
  { code: 'en', name: 'English', nativeName: 'English' },
  { code: 'es', name: 'Spanish', nativeName: 'Español' },
  { code: 'fr', name: 'French', nativeName: 'Français' },
  { code: 'de', name: 'German', nativeName: 'Deutsch' }, // Add this line
] as const

// Add to resources object
export const resources = {
  en: { /* existing english resources */ },
  es: { /* existing spanish resources */ },
  fr: { /* existing french resources */ },
  de: { // Add this section
    common: deCommon,
    navigation: deNavigation,
    dashboard: deDashboard,
    accessibility: deAccessibility,
    explorer: deExplorer,
    wallet: deWallet,
  },
} as const
```

### Step 5: Verify Translation

Run the verification script to ensure all translations are correct:

```bash
cd /path/to/icn-core
node scripts/verify-i18n.js
```

The script will check:
- All language files exist
- JSON syntax is valid
- Translation keys are consistent across languages
- Package dependencies are correct

### Step 6: Test Integration

Test the new language in each application:

#### Web UI
```bash
cd apps/web-ui
npm run dev
# Navigate to settings and select the new language
```

#### Explorer
```bash
cd apps/explorer  
npm run dev
# Use the language switcher to test translation
```

#### Wallet
```bash
cd apps/wallet-ui
npm run dev
# Verify all wallet-specific terms are translated
```

## Language-Specific Considerations

### Right-to-Left (RTL) Languages

For RTL languages like Arabic or Hebrew, additional CSS considerations are needed:

```typescript
// In config.ts, add RTL direction
{ code: 'ar', name: 'Arabic', nativeName: 'العربية', direction: 'rtl' }
```

Add RTL CSS support:
```css
[dir="rtl"] {
  text-align: right;
  direction: rtl;
}

[dir="rtl"] .flex {
  flex-direction: row-reverse;
}
```

### Character Encoding

Ensure proper Unicode support for languages with special characters:

```typescript
// In translation files, use proper Unicode
{
  "welcome": "Добро пожаловать", // Cyrillic
  "chinese": "欢迎", // Chinese characters
  "emoji": "🌍 Global" // Emoji support
}
```

### Pluralization

Some languages have complex pluralization rules. Use i18next pluralization:

```json
{
  "itemCount": "{{count}} item",
  "itemCount_plural": "{{count}} items",
  "itemCount_zero": "No items"
}
```

For languages with more complex plural forms:
```json
{
  "itemCount_0": "No items",
  "itemCount_1": "One item", 
  "itemCount_2": "Two items",
  "itemCount_few": "A few items",
  "itemCount_many": "Many items",
  "itemCount_other": "{{count}} items"
}
```

### Date and Number Formatting

Use locale-specific formatting:

```typescript
import { format } from 'date-fns'
import { de } from 'date-fns/locale'

// German date formatting
const formattedDate = format(new Date(), 'dd.MM.yyyy', { locale: de })

// Number formatting
const germanNumber = new Intl.NumberFormat('de-DE').format(1234.56) // "1.234,56"
```

## Translation Best Practices

### 1. Context Matters
Provide context for translators:
```json
{
  "button": {
    "_comment": "This button submits the form",
    "submit": "Submit"
  }
}
```

### 2. Keep Keys Descriptive
Use meaningful key names:
```json
{
  "loginForm": {
    "emailLabel": "Email Address",
    "passwordLabel": "Password",
    "submitButton": "Sign In"
  }
}
```

### 3. Handle Variable Content
Use interpolation for dynamic content:
```json
{
  "welcome": "Welcome, {{username}}!",
  "itemsSelected": "{{count}} items selected"
}
```

### 4. Accessibility Considerations
Provide clear, descriptive accessibility text:
```json
{
  "accessibility": {
    "closeDialog": "Close this dialog window",
    "sortByName": "Sort the table by name column",
    "loadingData": "Loading data, please wait"
  }
}
```

## Quality Assurance

### Translation Review Checklist
- [ ] All keys translated accurately
- [ ] Cultural appropriateness verified
- [ ] Technical terms consistent
- [ ] UI layout accommodates text length
- [ ] Accessibility labels make sense
- [ ] Pluralization rules applied correctly
- [ ] Date/time formats appropriate
- [ ] Number formats match locale

### Testing Checklist
- [ ] UI layout doesn't break with longer text
- [ ] All text fits in allocated space
- [ ] Language switching works smoothly
- [ ] Search functionality works with local characters
- [ ] Form validation messages are clear
- [ ] Error messages are helpful
- [ ] Screen reader compatibility verified

## Maintenance

### Regular Updates
- Monitor for new translation keys
- Update translations when features are added
- Review cultural appropriateness annually
- Test with native speakers when possible

### Community Contributions
- Accept community translation contributions
- Provide translation guidelines
- Establish review process for community translations
- Recognize translator contributions

## Resources

### Translation Tools
- [Crowdin](https://crowdin.com/) - Translation management platform
- [Weblate](https://weblate.org/) - Open source translation platform
- [i18next](https://www.i18next.com/) - Internationalization framework

### Language Resources
- [Unicode CLDR](http://cldr.unicode.org/) - Locale data standards
- [Mozilla L10n](https://mozilla-l10n.github.io/localizer-documentation/) - Localization guidelines
- [Google Style Guides](https://developers.google.com/style) - Writing and style guidelines

---

This guide provides a comprehensive framework for adding new languages to the ICN Core internationalization system. Following these steps ensures consistent, high-quality translations across all applications.