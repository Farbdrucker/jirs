<script lang="ts">
  import { api } from '$lib/api';
  import { auth } from '$lib/auth';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  let email = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  onMount(() => {
    import('$lib/auth').then(({ auth: a }) => {
      // already logged in
    });
  });

  async function login() {
    error = '';
    loading = true;
    try {
      const res = await api.auth.login({ email, password });
      auth.login(res.user, res.access_token, res.refresh_token);
      goto('/');
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  }
</script>

<div class="min-h-screen flex items-center justify-center -mt-6">
  <div class="w-full max-w-sm">
    <div class="text-center mb-8">
      <h1 class="text-3xl font-bold text-brand-600">jirs</h1>
      <p class="text-gray-500 mt-1">Sign in to your workspace</p>
    </div>
    <div class="card p-6">
      {#if error}
        <div class="bg-red-50 text-red-700 text-sm px-3 py-2 rounded mb-4">{error}</div>
      {/if}
      <form onsubmit={(e) => { e.preventDefault(); login(); }}>
        <div class="mb-4">
          <label class="label" for="email">Email</label>
          <input id="email" type="email" class="input" bind:value={email} required autocomplete="email" />
        </div>
        <div class="mb-6">
          <label class="label" for="password">Password</label>
          <input id="password" type="password" class="input" bind:value={password} required autocomplete="current-password" />
        </div>
        <button type="submit" class="btn-primary w-full justify-center" disabled={loading}>
          {loading ? 'Signing in…' : 'Sign in'}
        </button>
      </form>
      <p class="text-center text-sm text-gray-500 mt-4">
        No account? <a href="/register" class="text-brand-600 hover:underline">Register</a>
      </p>
    </div>
  </div>
</div>
