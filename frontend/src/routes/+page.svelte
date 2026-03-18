<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { auth } from '$lib/auth';
  import { cache } from '$lib/cache';
  import { goto } from '$app/navigation';
  import type { Project } from '$lib/types';

  let projects: Project[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let showCreate = $state(false);
  let creating = $state(false);

  let form = $state({ key: '', name: '', description: '' });
  let formError = $state('');

  onMount(async () => {
    if (!$auth.access_token) { goto('/login'); return; }
    const cacheKey = 'projects';
    const cached = cache.get<Project[]>(cacheKey);
    if (cached) { projects = cached; loading = false; }
    if (cache.isStale(cacheKey)) {
      try {
        const fresh = await api.projects.list();
        projects = fresh;
        cache.set(cacheKey, fresh);
      } catch (e: any) {
        if (!cached) error = e.message;
      } finally {
        loading = false;
      }
    } else {
      loading = false;
    }
  });

  async function createProject() {
    formError = '';
    creating = true;
    try {
      const p = await api.projects.create({
        key: form.key.toUpperCase(),
        name: form.name,
        description: form.description || undefined
      });
      projects = [p, ...projects];
      cache.set('projects', projects);
      showCreate = false;
      form = { key: '', name: '', description: '' };
    } catch (e: any) {
      formError = e.message;
    } finally {
      creating = false;
    }
  }
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-2xl font-bold text-gray-900">Projects</h1>
    <button class="btn-primary" onclick={() => showCreate = true}>New Project</button>
  </div>

  {#if loading}
    <p class="text-gray-500">Loading…</p>
  {:else if error}
    <p class="text-red-600">{error}</p>
  {:else if projects.length === 0}
    <div class="text-center py-16 text-gray-500">
      <p class="text-lg mb-2">No projects yet</p>
      <button class="btn-primary" onclick={() => showCreate = true}>Create your first project</button>
    </div>
  {:else}
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each projects as p}
        <div class="card p-5 hover:shadow-md transition-shadow">
          <div class="flex items-start justify-between">
            <div>
              <span class="inline-block px-2 py-0.5 rounded text-xs font-mono font-semibold bg-brand-50 text-brand-700 mb-2">
                {p.key}
              </span>
              <h3 class="font-semibold text-gray-900">{p.name}</h3>
              {#if p.description}
                <p class="text-sm text-gray-500 mt-1 line-clamp-2">{p.description}</p>
              {/if}
            </div>
          </div>
          <div class="mt-4 flex gap-2 text-xs text-gray-500">
            <a href="/projects/{p.key}/board" class="hover:text-brand-600">Board</a>
            <span>·</span>
            <a href="/projects/{p.key}/backlog" class="hover:text-brand-600">Backlog</a>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showCreate}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg p-6 w-full max-w-md shadow-xl">
      <h2 class="text-lg font-semibold mb-4">Create Project</h2>
      {#if formError}
        <p class="text-red-600 text-sm mb-3">{formError}</p>
      {/if}
      <form onsubmit={(e) => { e.preventDefault(); createProject(); }}>
        <div class="mb-4">
          <label class="label" for="key">Key (e.g. PROJ)</label>
          <input id="key" class="input font-mono uppercase" bind:value={form.key}
                 placeholder="PROJ" maxlength={10} required />
          <p class="text-xs text-gray-500 mt-1">2-10 uppercase letters, used in ticket slugs</p>
        </div>
        <div class="mb-4">
          <label class="label" for="name">Name</label>
          <input id="name" class="input" bind:value={form.name} placeholder="My Project" required />
        </div>
        <div class="mb-6">
          <label class="label" for="desc">Description (optional)</label>
          <textarea id="desc" class="input" rows={3} bind:value={form.description}></textarea>
        </div>
        <div class="flex gap-3 justify-end">
          <button type="button" class="btn-secondary" onclick={() => showCreate = false}>Cancel</button>
          <button type="submit" class="btn-primary" disabled={creating}>
            {creating ? 'Creating…' : 'Create'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
