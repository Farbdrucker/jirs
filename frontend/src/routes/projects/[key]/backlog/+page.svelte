<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { cache } from '$lib/cache';
  import type { Ticket, Sprint } from '$lib/types';
  import PriorityBadge from '$lib/components/PriorityBadge.svelte';
  import TicketTypeIcon from '$lib/components/TicketTypeIcon.svelte';

  const key = $derived($page.params.key as string);
  let tickets: Ticket[] = $state([]);
  let sprints: Sprint[] = $state([]);
  let loading = $state(true);
  let error = $state('');
  let showCreateSprint = $state(false);
  let newSprint = $state({ name: '', goal: '', start_date: '', end_date: '' });
  let creatingS = $state(false);
  let sprintError = $state('');

  onMount(async () => {
    const ticketsKey = `tickets:${key}`;
    const sprintsKey = `sprints:${key}`;

    const cachedTickets = cache.get<Ticket[]>(ticketsKey);
    const cachedSprints = cache.get<Sprint[]>(sprintsKey);
    if (cachedTickets) { tickets = cachedTickets; loading = false; }
    if (cachedSprints) { sprints = cachedSprints; }

    if (cache.isStale(ticketsKey) || cache.isStale(sprintsKey)) {
      try {
        const [freshTickets, freshSprints] = await Promise.all([
          api.tickets.list(key, { status: 'backlog' }),
          api.sprints.list(key)
        ]);
        tickets = freshTickets;
        sprints = freshSprints;
        cache.set(ticketsKey, freshTickets);
        cache.set(sprintsKey, freshSprints);
      } catch (e: any) {
        if (!cachedTickets) error = e.message;
      } finally {
        loading = false;
      }
    } else {
      loading = false;
    }
  });

  async function startSprint(id: string) {
    try {
      const s = await api.sprints.start(id);
      sprints = sprints.map(sp => sp.id === id ? s : sp);
      cache.set(`sprints:${key}`, sprints);
    } catch (e: any) {
      alert(e.message);
    }
  }

  async function completeSprint(id: string) {
    try {
      const s = await api.sprints.complete(id);
      sprints = sprints.map(sp => sp.id === id ? s : sp);
      cache.set(`sprints:${key}`, sprints);
    } catch (e: any) {
      alert(e.message);
    }
  }

  async function createSprint() {
    sprintError = '';
    creatingS = true;
    try {
      const s = await api.sprints.create(key, {
        name: newSprint.name,
        goal: newSprint.goal || undefined,
        start_date: newSprint.start_date || undefined,
        end_date: newSprint.end_date || undefined
      });
      sprints = [s, ...sprints];
      cache.set(`sprints:${key}`, sprints);
      showCreateSprint = false;
      newSprint = { name: '', goal: '', start_date: '', end_date: '' };
    } catch (e: any) {
      sprintError = e.message;
    } finally {
      creatingS = false;
    }
  }

  const statusColors: Record<string, string> = {
    planning: 'bg-gray-100 text-gray-700',
    active: 'bg-green-100 text-green-700',
    completed: 'bg-blue-100 text-blue-700'
  };
</script>

<div>
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-xl font-bold">{key} Backlog</h1>
    <div class="flex gap-2">
      <a href="/projects/{key}/board" class="btn-secondary btn-sm">Board</a>
      <button class="btn-primary btn-sm" onclick={() => showCreateSprint = true}>+ Sprint</button>
    </div>
  </div>

  {#if loading && tickets.length === 0}
    <p class="text-gray-500">Loading…</p>
  {:else if error}
    <p class="text-red-600">{error}</p>
  {:else}
    <!-- Sprints -->
    <section class="mb-8">
      <h2 class="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-3">Sprints</h2>
      {#if sprints.length === 0}
        <p class="text-gray-400 text-sm">No sprints yet.</p>
      {:else}
        <div class="flex flex-col gap-3">
          {#each sprints as sprint}
            <div class="card p-4">
              <div class="flex items-center justify-between">
                <div>
                  <div class="flex items-center gap-2">
                    <span class="font-medium">{sprint.name}</span>
                    <span class="text-xs px-2 py-0.5 rounded-full {statusColors[sprint.status]}">
                      {sprint.status}
                    </span>
                  </div>
                  {#if sprint.goal}
                    <p class="text-sm text-gray-500 mt-0.5">{sprint.goal}</p>
                  {/if}
                </div>
                <div class="flex gap-2">
                  {#if sprint.status === 'planning'}
                    <button class="btn-primary btn-sm" onclick={() => startSprint(sprint.id)}>Start</button>
                  {:else if sprint.status === 'active'}
                    <button class="btn-secondary btn-sm" onclick={() => completeSprint(sprint.id)}>Complete</button>
                  {/if}
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <!-- Backlog tickets -->
    <section>
      <h2 class="text-sm font-semibold text-gray-500 uppercase tracking-wide mb-3">
        Backlog ({tickets.length})
      </h2>
      {#if tickets.length === 0}
        <p class="text-gray-400 text-sm">No backlog items.</p>
      {:else}
        <div class="card divide-y divide-gray-100">
          {#each tickets as ticket}
            <div class="flex items-center gap-3 px-4 py-3 hover:bg-gray-50">
              <TicketTypeIcon type={ticket.ticket_type} />
              <PriorityBadge priority={ticket.priority} />
              <span class="text-xs text-gray-400 font-mono w-20 shrink-0">{ticket.slug}</span>
              <a href="/tickets/{ticket.slug}" class="flex-1 text-sm text-gray-900 hover:text-brand-600 truncate">
                {ticket.title}
              </a>
              {#if ticket.story_points}
                <span class="text-xs bg-gray-100 px-1.5 py-0.5 rounded">{ticket.story_points}</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</div>

{#if showCreateSprint}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg p-6 w-full max-w-md shadow-xl">
      <h2 class="text-lg font-semibold mb-4">Create Sprint</h2>
      {#if sprintError}<p class="text-red-600 text-sm mb-3">{sprintError}</p>{/if}
      <form onsubmit={(e) => { e.preventDefault(); createSprint(); }}>
        <div class="mb-4">
          <label class="label">Sprint name</label>
          <input class="input" bind:value={newSprint.name} required placeholder="Sprint 1" />
        </div>
        <div class="mb-4">
          <label class="label">Goal (optional)</label>
          <input class="input" bind:value={newSprint.goal} />
        </div>
        <div class="grid grid-cols-2 gap-4 mb-6">
          <div>
            <label class="label">Start date</label>
            <input type="date" class="input" bind:value={newSprint.start_date} />
          </div>
          <div>
            <label class="label">End date</label>
            <input type="date" class="input" bind:value={newSprint.end_date} />
          </div>
        </div>
        <div class="flex gap-3 justify-end">
          <button type="button" class="btn-secondary" onclick={() => showCreateSprint = false}>Cancel</button>
          <button type="submit" class="btn-primary" disabled={creatingS}>
            {creatingS ? 'Creating…' : 'Create sprint'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
