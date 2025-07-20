# Security Documentation for ICN TypeScript SDK

## Overview

The ICN TypeScript SDK implements robust, cross-platform cryptographic security for protecting sensitive data like private keys. This document outlines the security features, implementation details, and best practices.

## Security Features

### 1. Cross-Platform Cryptography

The SDK uses the **Web Crypto API** for all cryptographic operations, ensuring compatibility across:

- ✅ **Browsers** (Chrome, Firefox, Safari, Edge)
- ✅ **React Native** (with polyfills)
- ✅ **Node.js** (v16+ with webcrypto support)
- ✅ **Web Workers** and **Service Workers**

### 2. Secure Private Key Storage

Private keys are **always encrypted** before storage using:

- **AES-256-GCM** encryption (authenticated encryption)
- **PBKDF2** key derivation with 100,000 iterations
- **Random salt** generation for each encryption
- **Secure IV/nonce** generation per encryption operation

```typescript
import { ICNClient, createSecureStorage } from '@icn/ts-sdk'

// Create client with secure storage
const client = new ICNClient({
  nodeEndpoint: 'https://node.icn.coop',
  encryptionConfig: {
    enableEncryption: true,
    passphrase: 'user-provided-secure-passphrase' // Optional
  }
})

// Or create secure storage directly
const secureStorage = createSecureStorage('@myapp:', {
  enableEncryption: true,
  passphrase: 'custom-passphrase'
})
```

### 3. Configurable Security Levels

#### High Security (Default)
```typescript
const client = new ICNClient({
  nodeEndpoint: 'https://node.icn.coop',
  encryptionConfig: {
    enableEncryption: true,
    passphrase: await promptUserForPassphrase()
  }
})
```

#### Medium Security (Session-based)
```typescript
const client = new ICNClient({
  nodeEndpoint: 'https://node.icn.coop',
  encryptionConfig: {
    enableEncryption: true
    // Uses session-specific entropy if no passphrase provided
  }
})
```

#### Development Only (No Encryption)
```typescript
const client = new ICNClient({
  nodeEndpoint: 'http://localhost:8080',
  encryptionConfig: {
    enableEncryption: false // Only for development!
  }
})
```

## Security Implementation Details

### Encryption Algorithm

- **Algorithm**: AES-256-GCM
- **Key Derivation**: PBKDF2 with SHA-256
- **Iterations**: 100,000 (configurable)
- **Salt Size**: 128 bits (16 bytes)
- **IV/Nonce Size**: 96 bits (12 bytes)

### Key Generation Process

1. **Salt Generation**: Cryptographically secure random 128-bit salt
2. **Key Derivation**: PBKDF2-SHA256 with 100k iterations
3. **Encryption**: AES-256-GCM with random 96-bit IV
4. **Storage Format**: `base64(salt):base64(iv+ciphertext+tag)`

### Entropy Sources

When no passphrase is provided, the SDK generates session-specific entropy from:

- Current timestamp
- Secure random values
- Browser/environment fingerprint
- Current URL (in browser environments)

**Note**: This provides reasonable protection for development but should not be used in production without a user-provided passphrase.

## Security Best Practices

### For Production Applications

1. **Always use user-provided passphrases**:
```typescript
const passphrase = await securePasswordPrompt('Enter encryption passphrase:')
const client = new ICNClient({
  nodeEndpoint: 'https://mainnet.icn.coop',
  encryptionConfig: {
    enableEncryption: true,
    passphrase
  }
})
```

2. **Clear sensitive data from memory**:
```typescript
// Clear encryption cache when user logs out
client.getStorage().clearEncryptionCache()
```

3. **Use secure passphrase generation**:
```typescript
// Generate strong passphrase
const passphrase = generateSecurePassphrase(128) // 128-bit entropy
```

### For Development

1. **Use localhost endpoints with disabled encryption**:
```typescript
const client = new ICNClient({
  nodeEndpoint: 'http://localhost:8080',
  encryptionConfig: { enableEncryption: false }
})
```

2. **Enable encryption for testnet testing**:
```typescript
const client = new ICNClient({
  nodeEndpoint: 'https://testnet.icn.coop',
  encryptionConfig: { enableEncryption: true }
})
```

## React Native Considerations

React Native requires additional setup for crypto support:

### Installation
```bash
npm install react-native-get-random-values
# For Expo:
npx expo install expo-crypto
```

### Setup
```typescript
// At the top of your App.js/index.js
import 'react-native-get-random-values'

// For Expo
import { polyfillWebCrypto } from 'expo-crypto'
polyfillWebCrypto()
```

## Threat Model and Security Guarantees

### What This Protects Against

✅ **Storage Sniffing**: Encrypted private keys prevent plaintext exposure
✅ **Device Compromise**: Keys remain encrypted at rest
✅ **Memory Dumps**: Sensitive data cleared from memory
✅ **Cross-Platform Attacks**: Consistent security across environments

### What This Does NOT Protect Against

❌ **Malicious Code Injection**: XSS or malicious packages can steal keys
❌ **Compromised Devices**: Malware with root access
❌ **Social Engineering**: Users revealing passphrases
❌ **Browser Extension Attacks**: Malicious extensions with storage access

### Recommendations

1. **Use Hardware Security Modules (HSMs)** for high-value applications
2. **Implement multi-factor authentication** for critical operations
3. **Regular security audits** of your application code
4. **User education** about passphrase security

## Migration from Insecure Version

If upgrading from a version that used Node.js crypto with hardcoded secrets:

```typescript
// Old (insecure)
const client = new ICNClient({
  nodeEndpoint: 'https://node.icn.coop'
  // Used ICN_TS_SDK_SECRET env var or 'icn-default-secret'
})

// New (secure)
const client = new ICNClient({
  nodeEndpoint: 'https://node.icn.coop',
  encryptionConfig: {
    enableEncryption: true,
    passphrase: 'user-chosen-secure-passphrase'
  }
})
```

**Note**: Old encrypted keys cannot be decrypted with the new system. Users will need to regenerate their keys.

## Reporting Security Issues

If you discover a security vulnerability, please:

1. **Do NOT** open a public issue
2. Email security@intercooperative.network
3. Include detailed reproduction steps
4. Allow time for assessment and patching

## Security Changelog

### v0.2.0 (Current)
- ✅ Replaced Node.js crypto with Web Crypto API
- ✅ Eliminated hardcoded encryption secrets
- ✅ Added configurable encryption options
- ✅ Implemented secure key derivation (PBKDF2)
- ✅ Added cross-platform compatibility

### v0.1.0 (Previous)
- ❌ Used Node.js-specific crypto APIs
- ❌ Hardcoded default encryption secret
- ❌ Browser/React Native incompatible
- ❌ Vulnerable to key exposure 