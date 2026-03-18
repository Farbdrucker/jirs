<script lang="ts">
  import { api } from '$lib/api';
  import { cache } from '$lib/cache';
  import type { Tag } from '$lib/types';

  let {
    projectKey,
    currentTagIds,
    onAdd
  }: {
    projectKey: string;
    currentTagIds: string[];
    onAdd: (tag: Tag) => void;
  } = $props();

  let open = $state(false);
  let search = $state('');
  let showNewForm = $state(false);
  let newName = $state('');
  let newColor = $state('#6366f1');
  let creating = $state(false);
  let allTags: Tag[] = $state([]);
  let container: HTMLDivElement;

  const filtered = $derived(
    allTags.filter(t =>
      !currentTagIds.includes(t.id) &&
      (!search.trim() || t.name.toLowerCase().includes(search.toLowerCase()))
    )
  );

  async function loadTags() {
    const cacheKey = `tags:${projectKey}`;
    const cached = cache.get<Tag[]>(cacheKey);
    if (cached) allTags = cached;
    if (cache.isStale(cacheKey)) {
      const fresh = await api.tags.list(projectKey);
      allTags = fresh;
      cache.set(cacheKey, fresh);
    }
  }

  async function openPicker() {
    open = true;
    search = '';
    showNewForm = false;
    await loadTags();
  }

  async function createTag() {
    if (!newName.trim()) return;
    creating = true;
    try {
      const tag = await api.tags.create(projectKey, { name: newName.trim(), color: newColor });
      allTags = [...allTags, tag];
      cache.set(`tags:${projectKey}`, allTags);
      onAdd(tag);
      newName = '';
      newColor = '#6366f1';
      showNewForm = false;
    } catch (e: any) {
      alert(e.message);
    } finally {
      creating = false;
    }
  }

  function onClickOutside(e: MouseEvent) {
    if (container && !container.contains(e.target as Node)) {
      open = false;
    }
  }
</script>

<svelte:window onclick={onClickOutside} />

<div class="relative inline-block" bind:this={container}>
  <button
    type="button"
    class="text-xs text-brand-600 hover:underline"
    onclick={openPicker}
  >
    + Add tag
  </button>

  {#if open}
    <div class="absolute z-20 top-full mt-1 left-0 w-52 bg-white border border-gray-200 rounded-lg shadow-lg overflow-hidden">
      <div class="p-2 border-b border-gray-100">
        <input
          class="w-full text-sm px-2 py-1 border border-gray-200 rounded focus:outline-none focus:border-brand-400"
          placeholder="Search tags…"
          bind:value={search}
          autofocus
        />
      </div>
      <div class="max-h-40 overflow-y-auto">
        {#each filtered as tag}
          <button
            type="button"
            class="flex items-center gap-2 w-full px-3 py-1.5 text-sm hover:bg-gray-50 text-left"
            onclick={() => { onAdd(tag); open = false; }}
          >
            <span class="w-3 h-3 rounded-full shrink-0" style="background:{tag.color}"></span>
            <span>{tag.name}</span>
          </button>
        {/each}
        {#if filtered.length === 0 && !showNewForm}
          <p class="px-3 py-1.5 text-xs text-gray-400">No tags to add</p>
        {/if}
      </div>
      <div class="border-t border-gray-100">
        {#if showNewForm}
          <div class="p-2 flex flex-col gap-1.5">
            <input
              class="text-sm px-2 py-1 border border-gray-200 rounded focus:outline-none focus:border-brand-400"
              placeholder="Tag name"
              bind:value={newName}
            />
            <div class="flex items-center gap-2">
              <input type="color" class="w-8 h-7 rounded cursor-pointer border border-gray-200" bind:value={newColor} />
              <button
                type="button"
                class="flex-1 btn-primary btn-sm text-xs"
                onclick={createTag}
                disabled={creating || !newName.trim()}
              >
                {creating ? 'Creating…' : 'Create'}
              </button>
              <button type="button" class="text-xs text-gray-400 hover:text-gray-600" onclick={() => showNewForm = false}>✕</button>
            </div>
          </div>
        {:else}
          <button
            type="button"
            class="w-full px-3 py-1.5 text-sm text-brand-600 hover:bg-gray-50 text-left"
            onclick={() => showNewForm = true}
          >
            ＋ New tag
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
