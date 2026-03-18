import { writable } from 'svelte/store';
import { cache } from './cache';
import type { User } from './types';

interface AuthState {
  user: User | null;
  access_token: string | null;
  refresh_token: string | null;
}

function createAuthStore() {
  const stored = typeof localStorage !== 'undefined' ? localStorage.getItem('auth') : null;
  const initial: AuthState = stored ? JSON.parse(stored) : { user: null, access_token: null, refresh_token: null };

  const { subscribe, set, update } = writable<AuthState>(initial);

  return {
    subscribe,
    login(user: User, access_token: string, refresh_token: string) {
      const state = { user, access_token, refresh_token };
      localStorage.setItem('auth', JSON.stringify(state));
      set(state);
    },
    logout() {
      localStorage.removeItem('auth');
      cache.clear();
      set({ user: null, access_token: null, refresh_token: null });
    },
    updateTokens(access_token: string, refresh_token: string) {
      update(s => {
        const state = { ...s, access_token, refresh_token };
        localStorage.setItem('auth', JSON.stringify(state));
        return state;
      });
    },
    updateUser(user: User) {
      update(s => {
        const state = { ...s, user };
        localStorage.setItem('auth', JSON.stringify(state));
        return state;
      });
    }
  };
}

export const auth = createAuthStore();
