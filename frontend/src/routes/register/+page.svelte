<script lang="ts">
  import { api } from '$lib/api';
  import { auth } from '$lib/auth';
  import { goto } from '$app/navigation';

  let email = $state('');
  let username = $state('');
  let display_name = $state('');
  let password = $state('');
  let error = $state('');
  let loading = $state(false);

  async function register() {
    error = '';
    loading = true;
    try {
      const res = await api.auth.register({ email, username, display_name, password });
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
      <p class="text-gray-500 mt-1">Create your account</p>
    </div>
    <div class="card p-6">
      {#if error}
        <div class="bg-red-50 text-red-700 text-sm px-3 py-2 rounded mb-4">{error}</div>
      {/if}
      <form onsubmit={(e) => { e.preventDefault(); register(); }}>
        <div class="mb-4">
          <label class="label" for="email">Email</label>
          <input id="email" type="email" class="input" bind:value={email} required />
        </div>
        <div class="mb-4">
          <label class="label" for="username">Username</label>
          <input id="username" class="input" bind:value={username} required />
        </div>
        <div class="mb-4">
          <label class="label" for="display_name">Display name</label>
          <input id="display_name" class="input" bind:value={display_name} required />
        </div>
        <div class="mb-6">
          <label class="label" for="password">Password</label>
          <input id="password" type="password" class="input" bind:value={password} required minlength={6} />
        </div>
        <button type="submit" class="btn-primary w-full justify-center" disabled={loading}>
          {loading ? 'Creating account…' : 'Create account'}
        </button>
      </form>
      <p class="text-center text-sm text-gray-500 mt-4">
        Already have an account? <a href="/login" class="text-brand-600 hover:underline">Sign in</a>
      </p>
    </div>
  </div>
</div>
