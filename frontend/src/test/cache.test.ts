import { beforeEach, describe, expect, it, vi } from 'vitest';

// Re-import cache fresh in each test by resetting module cache
// We use a dynamic import pattern to get a fresh module per test group.
// Since the module uses a module-level _store, we use vi.resetModules() where needed.

const STORAGE_KEY = 'jirs_cache';

describe('cache', () => {
  let cache: typeof import('../lib/cache').cache;

  beforeEach(async () => {
    localStorage.clear();
    vi.resetModules();
    cache = (await import('../lib/cache')).cache;
  });

  it('get returns null for unknown key', () => {
    expect(cache.get('missing')).toBeNull();
  });

  it('set/get round-trip', () => {
    cache.set('myKey', { value: 42 });
    expect(cache.get('myKey')).toEqual({ value: 42 });
  });

  it('set persists to localStorage', () => {
    cache.set('myKey', 'hello');
    const raw = localStorage.getItem(STORAGE_KEY);
    expect(raw).not.toBeNull();
    const parsed = JSON.parse(raw!);
    expect(parsed['myKey']).toBeDefined();
    expect(parsed['myKey'].data).toBe('hello');
  });

  it('isStale true for absent key', () => {
    expect(cache.isStale('missing')).toBe(true);
  });

  it('isStale false immediately after set', () => {
    cache.set('fresh', 'data');
    expect(cache.isStale('fresh')).toBe(false);
  });

  it('isStale true past TTL', () => {
    const now = Date.now();
    vi.setSystemTime(now);
    cache.set('item', 'data');

    // Advance 6 minutes past the default 5-minute TTL
    vi.setSystemTime(now + 6 * 60 * 1000);
    expect(cache.isStale('item')).toBe(true);
    vi.useRealTimers();
  });

  it('isStale respects custom ttlMs', () => {
    const now = Date.now();
    vi.setSystemTime(now);
    cache.set('item', 'data');

    // Advance 500ms, TTL is 1000ms — should not be stale
    vi.setSystemTime(now + 500);
    expect(cache.isStale('item', 1000)).toBe(false);
    vi.useRealTimers();
  });

  it('invalidate removes key', () => {
    cache.set('toRemove', 'value');
    cache.invalidate('toRemove');
    expect(cache.get('toRemove')).toBeNull();
  });

  it('invalidate removes from localStorage', () => {
    cache.set('toRemove', 'value');
    cache.invalidate('toRemove');
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const parsed = JSON.parse(raw);
      expect(parsed['toRemove']).toBeUndefined();
    }
  });

  it('patch applies transform', () => {
    cache.set('counter', 5);
    cache.patch<number>('counter', (n) => n + 1);
    expect(cache.get<number>('counter')).toBe(6);
  });

  it('patch is no-op for absent key', () => {
    expect(() => cache.patch('missing', (v) => v)).not.toThrow();
    expect(cache.get('missing')).toBeNull();
  });

  it('clear wipes all keys', () => {
    cache.set('a', 1);
    cache.set('b', 2);
    cache.clear();
    expect(cache.get('a')).toBeNull();
    expect(cache.get('b')).toBeNull();
  });

  it('clear removes localStorage entry', () => {
    cache.set('a', 1);
    cache.clear();
    expect(localStorage.getItem(STORAGE_KEY)).toBeNull();
  });

  it('corrupt localStorage gracefully ignored', async () => {
    localStorage.setItem(STORAGE_KEY, '{broken');
    vi.resetModules();
    const freshCache = (await import('../lib/cache')).cache;
    expect(() => freshCache.get('anything')).not.toThrow();
    expect(freshCache.get('anything')).toBeNull();
  });

  it('quota exceeded clears store', () => {
    const originalSetItem = Storage.prototype.setItem;
    Storage.prototype.setItem = () => {
      throw new DOMException('QuotaExceededError');
    };
    try {
      expect(() => cache.set('key', 'value')).not.toThrow();
    } finally {
      Storage.prototype.setItem = originalSetItem;
    }
  });
});
