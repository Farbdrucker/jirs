import { describe, expect, it, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

const mockUser = {
  id: '00000000-0000-0000-0000-000000000001',
  email: 'alice@example.com',
  username: 'alice',
  display_name: 'Alice',
  avatar_url: null,
  created_at: new Date().toISOString()
};

describe('auth store', () => {
  let auth: typeof import('../lib/auth').auth;
  let cache: typeof import('../lib/cache').cache;

  beforeEach(async () => {
    localStorage.clear();
    vi.resetModules();
    cache = (await import('../lib/cache')).cache;
    auth = (await import('../lib/auth')).auth;
    // Ensure logged out state
    auth.logout();
  });

  it('initial state null when localStorage empty', () => {
    const state = get(auth);
    expect(state.user).toBeNull();
    expect(state.access_token).toBeNull();
    expect(state.refresh_token).toBeNull();
  });

  it('login sets store', () => {
    auth.login(mockUser, 'access-tok', 'refresh-tok');
    const state = get(auth);
    expect(state.user).toEqual(mockUser);
    expect(state.access_token).toBe('access-tok');
    expect(state.refresh_token).toBe('refresh-tok');
  });

  it('login persists to localStorage', () => {
    auth.login(mockUser, 'access-tok', 'refresh-tok');
    const raw = localStorage.getItem('auth');
    expect(raw).not.toBeNull();
    const parsed = JSON.parse(raw!);
    expect(parsed.access_token).toBe('access-tok');
    expect(parsed.user.email).toBe('alice@example.com');
  });

  it('logout clears store', () => {
    auth.login(mockUser, 'access-tok', 'refresh-tok');
    auth.logout();
    const state = get(auth);
    expect(state.user).toBeNull();
    expect(state.access_token).toBeNull();
    expect(state.refresh_token).toBeNull();
  });

  it('logout removes localStorage', () => {
    auth.login(mockUser, 'access-tok', 'refresh-tok');
    auth.logout();
    expect(localStorage.getItem('auth')).toBeNull();
  });

  it('logout calls cache.clear', () => {
    const clearSpy = vi.spyOn(cache, 'clear');
    auth.logout();
    expect(clearSpy).toHaveBeenCalledOnce();
  });

  it('updateTokens keeps user, replaces tokens', () => {
    auth.login(mockUser, 'old-access', 'old-refresh');
    auth.updateTokens('new-access', 'new-refresh');
    const state = get(auth);
    expect(state.user).toEqual(mockUser);
    expect(state.access_token).toBe('new-access');
    expect(state.refresh_token).toBe('new-refresh');
  });

  it('updateUser keeps tokens, replaces user', () => {
    auth.login(mockUser, 'access-tok', 'refresh-tok');
    const updatedUser = { ...mockUser, display_name: 'Alice Updated' };
    auth.updateUser(updatedUser);
    const state = get(auth);
    expect(state.access_token).toBe('access-tok');
    expect(state.refresh_token).toBe('refresh-tok');
    expect(state.user?.display_name).toBe('Alice Updated');
  });

  it('initial state hydrated from localStorage', async () => {
    // Pre-populate localStorage before module import
    localStorage.clear();
    const stored = { user: mockUser, access_token: 'stored-access', refresh_token: 'stored-refresh' };
    localStorage.setItem('auth', JSON.stringify(stored));

    vi.resetModules();
    const { auth: freshAuth } = await import('../lib/auth');
    const state = get(freshAuth);
    expect(state.user?.email).toBe('alice@example.com');
    expect(state.access_token).toBe('stored-access');
    expect(state.refresh_token).toBe('stored-refresh');
  });
});
