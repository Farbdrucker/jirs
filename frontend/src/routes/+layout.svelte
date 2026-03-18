<script lang="ts">
  import '../app.css';
  import { auth } from '$lib/auth';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  let { children } = $props();

  const publicRoutes = ['/login', '/register'];

  onMount(() => {
    const isPublic = publicRoutes.some(r => $page.url.pathname.startsWith(r));
    if (!$auth.access_token && !isPublic) {
      goto('/login');
    }
  });
</script>

<div class="min-h-screen bg-gray-50">
  {#if $auth.user && !publicRoutes.some(r => $page.url.pathname.startsWith(r))}
    <nav class="bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between">
      <div class="flex items-center gap-4">
        <a href="/" class="text-xl font-bold text-brand-600">jirs</a>
        <a href="/" class="text-sm text-gray-600 hover:text-gray-900">Projects</a>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-sm text-gray-600">{$auth.user.display_name}</span>
        <a href="/settings" class="text-sm text-gray-500 hover:text-gray-900">Settings</a>
        <button
          onclick={() => { auth.logout(); goto('/login'); }}
          class="text-sm text-gray-500 hover:text-gray-900"
        >
          Sign out
        </button>
      </div>
    </nav>
  {/if}
  <main class="container mx-auto px-4 py-6 max-w-7xl">
    {@render children()}
  </main>
</div>
