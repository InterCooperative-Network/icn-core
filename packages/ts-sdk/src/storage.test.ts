import { describe, it, expect } from 'vitest'
import { ICNStorage } from './storage'
import type { StorageAdapter } from './types'

class TestAdapter implements StorageAdapter {
  private store = new Map<string, string>()
  async getItem(key: string) {
    return this.store.get(key) ?? null
  }
  async setItem(key: string, value: string) {
    this.store.set(key, value)
  }
  async removeItem(key: string) {
    this.store.delete(key)
  }
  async clear() {
    this.store.clear()
  }
}

describe('ICNStorage private key encryption', () => {
  it('does not store plaintext private key', async () => {
    const adapter = new TestAdapter()
    const storage = new ICNStorage(adapter, { enableEncryption: true })
    await storage.setPrivateKey('my-secret')
    const raw = await adapter.getItem('private-key')
    expect(raw).not.toBe('my-secret')
    expect(raw?.includes('my-secret')).toBe(false)
    const decrypted = await storage.getPrivateKey()
    expect(decrypted).toBe('my-secret')
  })

  it('can disable encryption when needed', async () => {
    const adapter = new TestAdapter()
    const storage = new ICNStorage(adapter, { enableEncryption: false })
    await storage.setPrivateKey('my-secret')
    const raw = await adapter.getItem('private-key')
    expect(raw).toBe('my-secret') // Should be plaintext when encryption disabled
    const retrieved = await storage.getPrivateKey()
    expect(retrieved).toBe('my-secret')
  })

  it('can use custom encryption passphrase', async () => {
    const adapter = new TestAdapter()
    const storage = new ICNStorage(adapter, { 
      enableEncryption: true,
      passphrase: 'custom-secure-passphrase-123'
    })
    await storage.setPrivateKey('my-secret-key')
    const raw = await adapter.getItem('private-key')
    expect(raw).not.toBe('my-secret-key')
    expect(raw?.includes(':')).toBe(true) // Should contain salt separator
    const decrypted = await storage.getPrivateKey()
    expect(decrypted).toBe('my-secret-key')
  })
})
