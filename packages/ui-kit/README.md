# @icn/ui-kit

Shared UI components for ICN cross-platform applications built with Tamagui.

## Overview

This package provides a consistent set of UI components that work across all ICN applications:
- React Native (iOS/Android)
- Web browsers
- Desktop applications (via Tauri)

## Technology Stack

- **Tamagui**: Cross-platform styling and components
- **TypeScript**: Type safety and developer experience
- **React**: Component framework
- **tsup**: Fast TypeScript bundler

## Components

### Basic Components
- `Button` - Interactive button with variants (primary, secondary, ghost, danger)
- `Input` - Text input with validation states
- `Card` - Container component for content grouping

### Layout Components
- `VStack` - Vertical stack layout
- `HStack` - Horizontal stack layout  
- `Container` - Content container with max width

### Typography Components
- `Heading` - Primary heading text
- `Subheading` - Secondary heading text
- `Body` - Regular body text
- `Caption` - Small caption text

## Installation

```bash
# Install the package
pnpm add @icn/ui-kit

# Install peer dependencies
pnpm add react react-native @tamagui/core @tamagui/config
```

## Usage

### Setup (Required)

First, configure Tamagui in your app root:

```tsx
import { TamaguiProvider, icnConfig } from '@icn/ui-kit'

export default function App() {
  return (
    <TamaguiProvider config={icnConfig}>
      {/* Your app content */}
    </TamaguiProvider>
  )
}
```

### Using Components

```tsx
import { Button, Input, Card, VStack, Heading } from '@icn/ui-kit'

function ExampleScreen() {
  return (
    <Card>
      <VStack space="$4" padding="$4">
        <Heading>Welcome to ICN</Heading>
        <Input 
          placeholder="Enter your DID"
          label="Decentralized Identifier"
        />
        <Button variant="primary" onPress={handleSubmit}>
          Connect to Network
        </Button>
      </VStack>
    </Card>
  )
}
```

### Theming

The UI kit includes light and dark themes:

```tsx
import { Theme } from '@icn/ui-kit'

function ThemedComponent() {
  return (
    <Theme name="dark_icn">
      {/* Components will use dark theme */}
    </Theme>
  )
}
```

## Platform Support

| Component | Web | iOS | Android | Desktop |
|-----------|-----|-----|---------|---------|
| Button    | ✅  | ✅  | ✅      | ✅      |
| Input     | ✅  | ✅  | ✅      | ✅      |
| Card      | ✅  | ✅  | ✅      | ✅      |
| Layout    | ✅  | ✅  | ✅      | ✅      |
| Typography| ✅  | ✅  | ✅      | ✅      |

## Development

```bash
# Install dependencies
pnpm install

# Start development build
pnpm dev

# Build for production
pnpm build

# Type checking
pnpm type-check

# Linting
pnpm lint

# Format code
pnpm format
```

## Contributing

1. Follow the existing component patterns
2. Ensure all components work across platforms
3. Add TypeScript types for all props
4. Test on web, iOS, and Android
5. Update documentation

## Design Tokens

The design system includes:

- **Colors**: Primary, secondary, success, warning, danger
- **Spacing**: xs, sm, md, lg, xl (4px increments)
- **Typography**: Font sizes and weights
- **Border Radius**: Consistent corner radius values

## Examples

See the example applications in the monorepo:
- `apps/wallet-ui` - Wallet application example
- `apps/agoranet` - Governance interface example

## License

Apache-2.0 