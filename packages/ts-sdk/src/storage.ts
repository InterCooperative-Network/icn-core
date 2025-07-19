import { StorageAdapter } from './types'

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

// High-level storage wrapper
export class ICNStorage {
  private adapter: StorageAdapter

  constructor(adapter: StorageAdapter) {
    this.adapter = adapter
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

  // Private key storage (encrypted in production)
  async getPrivateKey(): Promise<string | null> {
    return this.adapter.getItem('private-key')
  }

  async setPrivateKey(key: string): Promise<void> {
    // TODO: Encrypt private key before storage
    await this.adapter.setItem('private-key', key)
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
  }
} 