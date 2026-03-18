const STORAGE_KEY = 'jirs_cache';
const DEFAULT_TTL_MS = 5 * 60 * 1000; // 5 minutes

interface CacheEntry {
  data: unknown;
  fetchedAt: number;
}

// Module-level cache map, hydrated from localStorage
let _store: Record<string, CacheEntry> = (() => {
  if (typeof localStorage === 'undefined') return {};
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : {};
  } catch {
    return {};
  }
})();

function persist() {
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(_store));
  } catch {
    // quota exceeded — clear old entries
    _store = {};
  }
}

export const cache = {
  get<T>(key: string): T | null {
    const entry = _store[key];
    return entry ? (entry.data as T) : null;
  },

  set<T>(key: string, value: T): void {
    _store[key] = { data: value, fetchedAt: Date.now() };
    persist();
  },

  patch<T>(key: string, fn: (current: T) => T): void {
    const entry = _store[key];
    if (entry) {
      _store[key] = { data: fn(entry.data as T), fetchedAt: entry.fetchedAt };
      persist();
    }
  },

  isStale(key: string, ttlMs = DEFAULT_TTL_MS): boolean {
    const entry = _store[key];
    if (!entry) return true;
    return Date.now() - entry.fetchedAt > ttlMs;
  },

  invalidate(key: string): void {
    delete _store[key];
    persist();
  },

  clear(): void {
    _store = {};
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem(STORAGE_KEY);
    }
  }
};
