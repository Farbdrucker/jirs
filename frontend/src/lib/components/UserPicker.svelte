<script lang="ts">
  import type { UserSummary } from '$lib/types';

  let {
    users,
    selectedId,
    onSelect
  }: {
    users: UserSummary[];
    selectedId: string | null;
    onSelect: (id: string | null) => void;
  } = $props();

  let open = $state(false);
  let search = $state('');
  let container: HTMLDivElement;

  const selected = $derived(users.find(u => u.id === selectedId) ?? null);

  const filtered = $derived(
    search.trim()
      ? users.filter(u =>
          u.display_name.toLowerCase().includes(search.toLowerCase()) ||
          u.username.toLowerCase().includes(search.toLowerCase())
        )
      : users
  );

  function initials(name: string) {
    return name.split(' ').map(w => w[0]).join('').toUpperCase().slice(0, 2);
  }

  function pick(id: string | null) {
    onSelect(id);
    open = false;
    search = '';
  }

  function onClickOutside(e: MouseEvent) {
    if (container && !container.contains(e.target as Node)) {
      open = false;
    }
  }
</script>

<svelte:window onclick={onClickOutside} />

<div class="relative" bind:this={container}>
  <button
    type="button"
    class="flex items-center gap-2 w-full text-left px-2 py-1.5 rounded border border-gray-200 hover:border-brand-400 bg-white text-sm"
    onclick={() => { open = !open; search = ''; }}
  >
    {#if selected}
      <div class="w-6 h-6 rounded-full bg-brand-100 text-brand-700 text-xs font-semibold flex items-center justify-center shrink-0">
        {initials(selected.display_name)}
      </div>
      <span class="truncate">{selected.display_name}</span>
    {:else}
      <div class="w-6 h-6 rounded-full bg-gray-100 text-gray-400 text-xs flex items-center justify-center shrink-0">—</div>
      <span class="text-gray-400">Unassigned</span>
    {/if}
    <span class="ml-auto text-gray-400">▾</span>
  </button>

  {#if open}
    <div class="absolute z-20 top-full mt-1 left-0 w-full bg-white border border-gray-200 rounded-lg shadow-lg overflow-hidden">
      <div class="p-2 border-b border-gray-100">
        <input
          class="w-full text-sm px-2 py-1 border border-gray-200 rounded focus:outline-none focus:border-brand-400"
          placeholder="Search…"
          bind:value={search}
          autofocus
        />
      </div>
      <div class="max-h-48 overflow-y-auto">
        <button
          type="button"
          class="flex items-center gap-2 w-full px-3 py-2 text-sm hover:bg-gray-50 text-left"
          onclick={() => pick(null)}
        >
          <div class="w-6 h-6 rounded-full bg-gray-100 text-gray-400 text-xs flex items-center justify-center shrink-0">—</div>
          <span class="text-gray-500">Unassigned</span>
        </button>
        {#each filtered as user}
          <button
            type="button"
            class="flex items-center gap-2 w-full px-3 py-2 text-sm hover:bg-gray-50 text-left {user.id === selectedId ? 'bg-brand-50' : ''}"
            onclick={() => pick(user.id)}
          >
            <div class="w-6 h-6 rounded-full bg-brand-100 text-brand-700 text-xs font-semibold flex items-center justify-center shrink-0">
              {initials(user.display_name)}
            </div>
            <span>{user.display_name}</span>
            <span class="text-gray-400 text-xs ml-1">@{user.username}</span>
          </button>
        {/each}
        {#if filtered.length === 0}
          <p class="px-3 py-2 text-sm text-gray-400">No users found</p>
        {/if}
      </div>
    </div>
  {/if}
</div>
