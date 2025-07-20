/**
 * Cross-platform crypto utilities using Web Crypto API
 * Compatible with browsers, React Native, and Node.js
 */

// Platform detection and crypto API access
function getCrypto(): Crypto {
  // Browser environment
  if (typeof globalThis !== 'undefined' && globalThis.crypto && globalThis.crypto.subtle) {
    return globalThis.crypto
  }
  
  // Node.js environment (v16+)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const hasNodeGlobal = typeof (globalThis as any).global !== 'undefined'
  if (hasNodeGlobal) {
    try {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const nodeRequire = (globalThis as any).require || eval('require')
      const { webcrypto } = nodeRequire('crypto')
      if (webcrypto && webcrypto.subtle) {
        return webcrypto as Crypto
      }
    } catch {
      // Fall back to node crypto if webcrypto not available
    }
  }
  
  // React Native with polyfill
  if (typeof self !== 'undefined' && self.crypto && self.crypto.subtle) {
    return self.crypto
  }
  
  throw new Error(
    'Web Crypto API not available. ' +
    'For React Native, consider using a crypto polyfill like react-native-get-random-values and expo-crypto.'
  )
}

// Get secure random bytes
export function getRandomBytes(length: number): Uint8Array {
  const crypto = getCrypto()
  const array = new Uint8Array(length)
  crypto.getRandomValues(array)
  return array
}

// Convert string to Uint8Array
function stringToBytes(str: string): Uint8Array {
  return new TextEncoder().encode(str)
}

// Convert Uint8Array to string
function bytesToString(bytes: Uint8Array): string {
  return new TextDecoder().decode(bytes)
}

// Convert Uint8Array to base64
function bytesToBase64(bytes: Uint8Array): string {
  // For browsers
  if (typeof btoa !== 'undefined') {
    return btoa(String.fromCharCode(...bytes))
  }
  
  // For Node.js
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const BufferClass = (globalThis as any).Buffer
  if (BufferClass) {
    return BufferClass.from(bytes).toString('base64')
  }
  
  // Manual implementation fallback
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
  let result = ''
  for (let i = 0; i < bytes.length; i += 3) {
    const a = bytes[i]
    const b = bytes[i + 1] || 0
    const c = bytes[i + 2] || 0
    
    const bitmap = (a << 16) | (b << 8) | c
    
    result += chars.charAt((bitmap >> 18) & 63)
    result += chars.charAt((bitmap >> 12) & 63)
    result += i + 1 < bytes.length ? chars.charAt((bitmap >> 6) & 63) : '='
    result += i + 2 < bytes.length ? chars.charAt(bitmap & 63) : '='
  }
  return result
}

// Convert base64 to Uint8Array
function base64ToBytes(base64: string): Uint8Array {
  // For browsers
  if (typeof atob !== 'undefined') {
    const binaryString = atob(base64)
    const bytes = new Uint8Array(binaryString.length)
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.charCodeAt(i)
    }
    return bytes
  }
  
  // For Node.js
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const BufferClass = (globalThis as any).Buffer
  if (BufferClass) {
    return new Uint8Array(BufferClass.from(base64, 'base64'))
  }
  
  // Manual implementation fallback
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
  base64 = base64.replace(/[^A-Za-z0-9+/]/g, '')
  
  const bufferLength = Math.floor(base64.length * 0.75)
  const bytes = new Uint8Array(bufferLength)
  
  let p = 0
  for (let i = 0; i < base64.length; i += 4) {
    const encoded1 = chars.indexOf(base64[i])
    const encoded2 = chars.indexOf(base64[i + 1])
    const encoded3 = chars.indexOf(base64[i + 2])
    const encoded4 = chars.indexOf(base64[i + 3])
    
    const bitmap = (encoded1 << 18) | (encoded2 << 12) | (encoded3 << 6) | encoded4
    
    bytes[p++] = (bitmap >> 16) & 255
    if (encoded3 !== 64) bytes[p++] = (bitmap >> 8) & 255
    if (encoded4 !== 64) bytes[p++] = bitmap & 255
  }
  
  return bytes.slice(0, p)
}

// Derive encryption key from passphrase using PBKDF2
export async function deriveKey(
  passphrase: string,
  salt: Uint8Array,
  iterations: number = 100000
): Promise<CryptoKey> {
  const crypto = getCrypto()
  
  // Import the passphrase as a key
  const baseKey = await crypto.subtle.importKey(
    'raw',
    stringToBytes(passphrase),
    'PBKDF2',
    false,
    ['deriveBits', 'deriveKey']
  )
  
  // Derive the actual encryption key
  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt: salt,
      iterations: iterations,
      hash: 'SHA-256',
    },
    baseKey,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt']
  )
}

// Generate a secure salt
export function generateSalt(): Uint8Array {
  return getRandomBytes(16)
}

// SHA-256 hash function
export async function sha256(data: string | Uint8Array): Promise<Uint8Array> {
  const crypto = getCrypto()
  const bytes = typeof data === 'string' ? stringToBytes(data) : data
  const hashBuffer = await crypto.subtle.digest('SHA-256', bytes)
  return new Uint8Array(hashBuffer)
}

// AES-GCM encryption
export async function encryptAES(
  plaintext: string,
  key: CryptoKey
): Promise<string> {
  const crypto = getCrypto()
  
  // Generate random IV
  const iv = getRandomBytes(12)
  
  // Encrypt the data
  const encryptedBuffer = await crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv: iv,
    },
    key,
    stringToBytes(plaintext)
  )
  
  const encrypted = new Uint8Array(encryptedBuffer)
  
  // Combine IV and encrypted data
  const combined = new Uint8Array(iv.length + encrypted.length)
  combined.set(iv)
  combined.set(encrypted, iv.length)
  
  return bytesToBase64(combined)
}

// AES-GCM decryption
export async function decryptAES(
  encryptedData: string,
  key: CryptoKey
): Promise<string> {
  const crypto = getCrypto()
  
  const combined = base64ToBytes(encryptedData)
  
  // Extract IV and encrypted data
  const iv = combined.slice(0, 12)
  const encrypted = combined.slice(12)
  
  // Decrypt the data
  const decryptedBuffer = await crypto.subtle.decrypt(
    {
      name: 'AES-GCM',
      iv: iv,
    },
    key,
    encrypted
  )
  
  return bytesToString(new Uint8Array(decryptedBuffer))
}

// High-level encryption with passphrase
export async function encrypt(plaintext: string, passphrase: string): Promise<string> {
  const salt = generateSalt()
  const key = await deriveKey(passphrase, salt)
  const encrypted = await encryptAES(plaintext, key)
  
  // Combine salt and encrypted data
  const saltBase64 = bytesToBase64(salt)
  return `${saltBase64}:${encrypted}`
}

// High-level decryption with passphrase
export async function decrypt(encryptedData: string, passphrase: string): Promise<string> {
  const [saltBase64, encrypted] = encryptedData.split(':')
  if (!saltBase64 || !encrypted) {
    throw new Error('Invalid encrypted data format')
  }
  
  const salt = base64ToBytes(saltBase64)
  const key = await deriveKey(passphrase, salt)
  return decryptAES(encrypted, key)
} 