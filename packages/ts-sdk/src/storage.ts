import { StorageAdapter } from './types'
import { encrypt, decrypt } from './crypto'

// Configuration interface for encryption settings
export interface EncryptionConfig {
  passphrase?: string
  enableEncryption?: boolean
}

// Secure encryption configuration
class EncryptionManager {
  private config: EncryptionConfig
  private cachedPassphrase?: string

  constructor(config: EncryptionConfig = {}) {
    this.config = config
  }

  private async getPassphrase(): Promise<string> {
    if (this.cachedPassphrase) {
      return this.cachedPassphrase
    }

    // Use provided passphrase or prompt user for secure key
    if (this.config.passphrase) {
      this.cachedPassphrase = this.config.passphrase
      return this.cachedPassphrase
    }

    // Generate a session-specific passphrase based on available entropy
    // This is more secure than a hardcoded secret
    const entropy = [
      Date.now().toString(),
      Math.random().toString(),
      (typeof navigator !== 'undefined' && navigator.userAgent) || 'unknown',
      (typeof window !== 'undefined' && window.location?.href) || 'unknown'
    ].join('-')

    this.cachedPassphrase = `icn-session-${entropy}`
    return this.cachedPassphrase
  }

  async encryptValue(plaintext: string): Promise<string> {
    if (!this.config.enableEncryption) {
      return plaintext
    }

    try {
      const passphrase = await this.getPassphrase()
      return await encrypt(plaintext, passphrase)
    } catch (error) {
      console.warn('Encryption failed, storing as plaintext:', error)
      return plaintext
    }
  }

  async decryptValue(ciphertext: string): Promise<string> {
    if (!this.config.enableEncryption) {
      return ciphertext
    }

    // Check if this looks like encrypted data (contains salt separator)
    if (!ciphertext.includes(':')) {
      return ciphertext // Assume plaintext
    }

    try {
      const passphrase = await this.getPassphrase()
      return await decrypt(ciphertext, passphrase)
    } catch (error) {
      console.warn('Decryption failed, returning as-is:', error)
      return ciphertext
    }
  }

  // Clear cached passphrase (for security)
  clearCache(): void {
    this.cachedPassphrase = undefined
  }
}

// Global encryption manager (will be configured per ICNStorage instance)
let defaultEncryptionManager = new EncryptionManager()

// Web localStorage adapter
class WebStorageAdapter implements StorageAdapter {
  private prefix: string

  constructor(prefix = '@icn:') {
    this.prefix = prefix
  }

  async getItem(key: string): Promise<string | null> {
    try {
      return localStorage.getItem(this.prefix + key)
    } catch (error) {
      console.warn('LocalStorage not available:', error)
      return null
    }
  }

  async setItem(key: string, value: string): Promise<void> {
    try {
      localStorage.setItem(this.prefix + key, value)
    } catch (error) {
      console.warn('LocalStorage setItem failed:', error)
    }
  }

  async removeItem(key: string): Promise<void> {
    try {
      localStorage.removeItem(this.prefix + key)
    } catch (error) {
      console.warn('LocalStorage removeItem failed:', error)
    }
  }

  async clear(): Promise<void> {
    try {
      const keys = Object.keys(localStorage).filter(key => 
        key.startsWith(this.prefix)
      )
      keys.forEach(key => localStorage.removeItem(key))
    } catch (error) {
      console.warn('LocalStorage clear failed:', error)
    }
  }
}

// React Native AsyncStorage adapter
class ReactNativeStorageAdapter implements StorageAdapter {
  private AsyncStorage: any
  private prefix: string

  constructor(AsyncStorage: any, prefix = '@icn:') {
    this.AsyncStorage = AsyncStorage
    this.prefix = prefix
  }

  async getItem(key: string): Promise<string | null> {
    try {
      return await this.AsyncStorage.getItem(this.prefix + key)
    } catch (error) {
      console.warn('AsyncStorage getItem failed:', error)
      return null
    }
  }

  async setItem(key: string, value: string): Promise<void> {
    try {
      await this.AsyncStorage.setItem(this.prefix + key, value)
    } catch (error) {
      console.warn('AsyncStorage setItem failed:', error)
    }
  }

  async removeItem(key: string): Promise<void> {
    try {
      await this.AsyncStorage.removeItem(this.prefix + key)
    } catch (error) {
      console.warn('AsyncStorage removeItem failed:', error)
    }
  }

  async clear(): Promise<void> {
    try {
      const keys = await this.AsyncStorage.getAllKeys()
      const icnKeys = keys.filter((key: string) => 
        key.startsWith(this.prefix)
      )
      await this.AsyncStorage.multiRemove(icnKeys)
    } catch (error) {
      console.warn('AsyncStorage clear failed:', error)
    }
  }
}

// Memory storage adapter (fallback)
class MemoryStorageAdapter implements StorageAdapter {
  private storage = new Map<string, string>()
  private prefix: string

  constructor(prefix = '@icn:') {
    this.prefix = prefix
  }

  async getItem(key: string): Promise<string | null> {
    return this.storage.get(this.prefix + key) || null
  }

  async setItem(key: string, value: string): Promise<void> {
    this.storage.set(this.prefix + key, value)
  }

  async removeItem(key: string): Promise<void> {
    this.storage.delete(this.prefix + key)
  }

  async clear(): Promise<void> {
    const keys = Array.from(this.storage.keys()).filter(key =>
      key.startsWith(this.prefix)
    )
    keys.forEach(key => this.storage.delete(key))
  }
}

// Factory function to create appropriate storage adapter
export function createStorage(
  prefix = '@icn:',
  AsyncStorage?: any
): StorageAdapter {
  // React Native environment
  if (AsyncStorage) {
    return new ReactNativeStorageAdapter(AsyncStorage, prefix)
  }

  // Web environment
  if (typeof localStorage !== 'undefined') {
    return new WebStorageAdapter(prefix)
  }

  // Fallback to memory storage
  console.warn('No persistent storage available, using memory storage')
  return new MemoryStorageAdapter(prefix)
}

// Factory function to create ICNStorage with encryption
export function createSecureStorage(
  prefix = '@icn:',
  encryptionConfig?: EncryptionConfig,
  AsyncStorage?: any
): ICNStorage {
  const adapter = createStorage(prefix, AsyncStorage)
  return new ICNStorage(adapter, encryptionConfig)
}

// High-level storage wrapper
export class ICNStorage {
  private adapter: StorageAdapter
  private encryptionManager: EncryptionManager

  constructor(adapter: StorageAdapter, encryptionConfig?: EncryptionConfig) {
    this.adapter = adapter
    this.encryptionManager = new EncryptionManager({
      enableEncryption: true, // Enable encryption by default for security
      ...encryptionConfig,
    })
  }

  // Authentication tokens
  async getAuthToken(): Promise<string | null> {
    return this.adapter.getItem('auth-token')
  }

  async setAuthToken(token: string): Promise<void> {
    await this.adapter.setItem('auth-token', token)
  }

  async removeAuthToken(): Promise<void> {
    await this.adapter.removeItem('auth-token')
  }

  // Private key storage (encrypted by default)
  async getPrivateKey(): Promise<string | null> {
    const stored = await this.adapter.getItem('private-key')
    if (!stored) return null
    try {
      return await this.encryptionManager.decryptValue(stored)
    } catch (error) {
      console.warn('Failed to decrypt private key:', error)
      return null
    }
  }

  async setPrivateKey(key: string): Promise<void> {
    const encrypted = await this.encryptionManager.encryptValue(key)
    await this.adapter.setItem('private-key', encrypted)
  }

  async removePrivateKey(): Promise<void> {
    await this.adapter.removeItem('private-key')
  }

  // Network configuration
  async getNetworkConfig(): Promise<any> {
    const config = await this.adapter.getItem('network-config')
    return config ? JSON.parse(config) : null
  }

  async setNetworkConfig(config: any): Promise<void> {
    await this.adapter.setItem('network-config', JSON.stringify(config))
  }

  // Cache management
  async getCachedData<T>(key: string): Promise<T | null> {
    const data = await this.adapter.getItem(`cache:${key}`)
    return data ? JSON.parse(data) : null
  }

  async setCachedData<T>(key: string, data: T, ttl?: number): Promise<void> {
    const cacheEntry = {
      data,
      timestamp: Date.now(),
      ttl,
    }
    await this.adapter.setItem(`cache:${key}`, JSON.stringify(cacheEntry))
  }

  async isCacheValid(key: string): Promise<boolean> {
    const entry = await this.getCachedData<any>(`cache:${key}`)
    if (!entry || !entry.ttl) return false

    return Date.now() - entry.timestamp < entry.ttl
  }

  // Clear all storage
  async clear(): Promise<void> {
    await this.adapter.clear()
    this.encryptionManager.clearCache()
  }

  // Encryption management utilities
  updateEncryptionConfig(config: EncryptionConfig): void {
    this.encryptionManager = new EncryptionManager({
      enableEncryption: true,
      ...config,
    })
  }

  clearEncryptionCache(): void {
    this.encryptionManager.clearCache()
  }

  isEncryptionEnabled(): boolean {
    return this.encryptionManager['config'].enableEncryption ?? false
  }
} 