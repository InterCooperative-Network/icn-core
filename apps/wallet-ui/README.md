# ICN Wallet UI

> **⚠️ Development Status**: This is experimental wallet software with incomplete security implementations. DO NOT use for real keys or valuable assets. For development and testing only.

Cross-platform DID and key management interface for the InterCooperative Network.

## Overview

The ICN Wallet is a **prototype** application for managing decentralized identities (DIDs), private keys, and interacting with the ICN. It provides a user-friendly interface for wallet creation, DID registration, and network interactions.

**Security Warning**: This wallet is not production-ready. Private key storage, cryptographic implementations, and security measures are incomplete or stubbed.

## Technology Stack

- **React Native**: Cross-platform mobile development
- **Expo**: Development and build toolchain
- **Tamagui**: Cross-platform UI components
- **Expo Router**: File-based navigation
- **Tauri**: Desktop application wrapper
- **TypeScript**: Type safety

## Platform Support

| Platform | Status | Build Command | Notes |
|----------|--------|---------------|-------|
| **Web** | ✅ | `pnpm web` | PWA-enabled browser app |
| **iOS** | ✅ | `pnpm ios` | Native iOS app via Expo |
| **Android** | ✅ | `pnpm android` | Native Android app via Expo |
| **Desktop** | ✅ | `pnpm tauri:dev` | Desktop app via Tauri |

## Features

### Core Wallet Functions
- ✅ DID creation and management
- ✅ Private key secure storage
- ✅ Wallet import/export
- ✅ Multiple wallet support
- ✅ Backup and recovery

### ICN Integration
- ✅ Node connection management
- ✅ Mana balance tracking
- ✅ Job submission interface
- ✅ Governance participation
- ✅ Network status monitoring

### Security Features
- ✅ Encrypted key storage (Expo SecureStore)
- ✅ Biometric authentication
- ✅ PIN/password protection
- ✅ Local storage encryption
- ✅ Secure communication

### Recent Updates
- **CRDT Real-Time Sync** to keep wallet data consistent across devices
- **Advanced Governance Features** including multi-sig voting and proposal tracking
- **Updated API Integrations** with ICN Node v0.7 for identity and transaction APIs
- **Accessibility Additions** like improved contrast and screen reader support

## Installation

```bash
# Install dependencies
pnpm install

# Install iOS pods (macOS only)
cd ios && pod install && cd ..
```

## Development

### Web Development
```bash
# Start web development server
pnpm dev
pnpm web

# Access at http://localhost:8081
```

### Mobile Development
```bash
# Start Expo development server
pnpm dev

# Run on iOS simulator
pnpm ios

# Run on Android emulator
pnpm android

# Run on physical device
pnpm preview  # Creates tunnel for device testing
```

### Desktop Development
```bash
# Start Tauri development (requires Rust)
pnpm tauri:dev

# Build desktop app
pnpm tauri:build
```

## Building for Production

### Web (PWA)
```bash
# Build web app for production
pnpm build:web

# Output: dist/
```

### Mobile (App Stores)
```bash
# Configure EAS Build
expo install @expo/cli

# Build for iOS App Store
pnpm build:ios

# Build for Google Play Store
pnpm build:android

# Build for both platforms
pnpm mobile:build
```

### Desktop
```bash
# Build desktop applications
pnpm tauri:build

# Output: .tauri/target/release/bundle/
```

## Configuration

### Environment Variables
Create `.env.local` file:

```bash
# ICN Node Configuration
EXPO_PUBLIC_ICN_NODE_ENDPOINT=http://localhost:8080
EXPO_PUBLIC_ICN_NETWORK=devnet

# Security Configuration
EXPO_PUBLIC_WALLET_ENCRYPTION_KEY=your-encryption-key

# Feature Flags
EXPO_PUBLIC_ENABLE_BIOMETRICS=true
EXPO_PUBLIC_ENABLE_DESKTOP_MODE=true
```

### Network Configuration
Update `src/config/networks.ts`:

```typescript
export const networks = {
  mainnet: {
    name: 'ICN Mainnet',
    endpoint: 'https://mainnet.icn.coop',
    chainId: 'icn-1'
  },
  testnet: {
    name: 'ICN Testnet', 
    endpoint: 'https://testnet.icn.coop',
    chainId: 'icn-testnet'
  },
  devnet: {
    name: 'ICN Devnet',
    endpoint: 'http://localhost:8080',
    chainId: 'icn-dev'
  }
}
```

## Project Structure

```
apps/wallet-ui/
├── src/
│   ├── app/                 # Expo Router pages
│   │   ├── (tabs)/         # Tab navigation
│   │   ├── auth/           # Authentication screens
│   │   ├── wallet/         # Wallet management
│   │   └── settings/       # App settings
│   ├── components/         # Reusable components
│   ├── hooks/             # Custom React hooks
│   ├── utils/             # Utility functions
│   └── config/            # App configuration
├── assets/                # Static assets
├── .tauri/               # Tauri desktop config
├── app.json              # Expo configuration
└── tauri.conf.json       # Tauri configuration
```

## Key Components

### Wallet Management
```typescript
import { useWallet } from '@/hooks/useWallet'

function WalletScreen() {
  const { wallets, createWallet, importWallet } = useWallet()
  
  // Wallet creation and management logic
}
```

### ICN Integration
```typescript
import { useICNClient, useICNConnection } from '@icn/ts-sdk'

function ICNStatus() {
  const { connected, error } = useICNConnection()
  const { client } = useICNClient()
  
  // ICN network interaction logic
}
```

### Secure Storage
```typescript
import * as SecureStore from 'expo-secure-store'

// Store encrypted private keys
await SecureStore.setItemAsync('wallet_key', encryptedKey)
const key = await SecureStore.getItemAsync('wallet_key')
```

## Testing

```bash
# Run unit tests
pnpm test

# Run tests with coverage
pnpm test --coverage

# Run E2E tests (requires device/simulator)
pnpm test:e2e
```

## Security Considerations

### Key Management
- Private keys never leave the device
- Keys encrypted with device-specific entropy
- Biometric authentication where available
- Secure enclave utilization on iOS

### Network Security
- Certificate pinning for API calls
- Request signing with DID
- Encrypted communication channels
- Proper session management

### Code Security
- No sensitive data in logs
- Obfuscated release builds
- Runtime application self-protection
- Regular security audits

## Deployment

### Web Deployment
```bash
# Build and deploy to hosting service
pnpm build:web
# Deploy dist/ to your hosting provider
```

### App Store Deployment
```bash
# iOS App Store
pnpm build:ios
# Upload to App Store Connect

# Google Play Store  
pnpm build:android
# Upload to Google Play Console
```

### Desktop Distribution
```bash
# Build desktop apps
pnpm tauri:build

# Distribute via:
# - Direct download
# - App stores (Microsoft Store, Mac App Store)
# - Package managers (brew, chocolatey)
```

## Contributing

1. Follow React Native and Expo best practices
2. Use TypeScript for all new code
3. Follow the established component patterns
4. Test on all target platforms
5. Update documentation for new features

## Troubleshooting

### Common Issues

**Metro bundler issues:**
```bash
pnpm start --reset-cache
```

**iOS build issues:**
```bash
cd ios && pod install && cd ..
pnpm ios
```

**Android build issues:**
```bash
pnpm android --clear
```

**Tauri build issues:**
```bash
# Ensure Rust is installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
pnpm tauri:dev
```

## License

Apache-2.0 