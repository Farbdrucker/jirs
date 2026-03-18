<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { cache } from '$lib/cache';
  import type { Board, Ticket, Sprint } from '$lib/types';
  import PriorityBadge from '$lib/components/PriorityBadge.svelte';
  import TicketTypeIcon from '$lib/components/TicketTypeIcon.svelte';

  const key = $derived($page.params.key as string);
  let board: Board | null = $state(null);
  let sprints: Sprint[] = $state([]);
  let activeSprint: Sprint | null = $state(null);
  let scrumMode = $state(false);
  let loading = $state(true);
  let error = $state('');
  let showCreate = $state(false);
  let newTicket = $state<{ title: string; ticket_type: string; priority: string; status: string }>({ title: '', ticket_type: 'task', priority: 'medium', status: 'backlog' });
  let creating = $state(false);
  let createError = $state('');

  // Dragging state
  let dragTicket: Ticket | null = null;

  async function loadBoard() {
    const boardKey = `board:${key}`;
    const sprintsKey = `sprints:${key}`;

    const cachedBoard = cache.get<Board>(boardKey);
    const cachedSprints = cache.get<Sprint[]>(sprintsKey);
    if (cachedBoard) { board = cachedBoard; loading = false; }
    if (cachedSprints) { sprints = cachedSprints; activeSprint = sprints.find(s => s.status === 'active') || null; }

    if (cache.isStale(boardKey) || cache.isStale(sprintsKey)) {
      error = '';
      try {
        const [freshBoard, freshSprints] = await Promise.all([
          scrumMode ? api.board.scrum(key) : api.board.kanban(key),
          api.sprints.list(key)
        ]);
        board = freshBoard;
        sprints = freshSprints;
        activeSprint = sprints.find(s => s.status === 'active') || null;
        cache.set(boardKey, freshBoard);
        cache.set(sprintsKey, freshSprints);
      } catch (e: any) {
        if (!cachedBoard) error = e.message;
      } finally {
        loading = false;
      }
    } else {
      loading = false;
    }
  }

  onMount(() => loadBoard());

  function onDragStart(ticket: Ticket) {
    dragTicket = ticket;
  }

  async function onDrop(status: string) {
    if (!dragTicket || !board) return;
    const old = dragTicket;
    if (old.status === status) return;

    // Optimistic update
    board = {
      columns: board.columns.map(col => ({
        ...col,
        tickets: col.status === status
          ? [{ ...old, status: status as import('$lib/types').TicketStatus }, ...col.tickets.filter(t => t.id !== old.id)]
          : col.tickets.filter(t => t.id !== old.id)
      }))
    };
    cache.set(`board:${key}`, board);
    dragTicket = null;

    try {
      await api.board.move(old.slug, status);
    } catch {
      // Rollback
      cache.invalidate(`board:${key}`);
      loadBoard();
    }
  }

  async function createTicket() {
    createError = '';
    creating = true;
    try {
      const t = await api.tickets.create(key, {
        title: newTicket.title,
        ticket_type: newTicket.ticket_type as import('$lib/types').TicketType,
        priority: newTicket.priority as import('$lib/types').TicketPriority,
        status: newTicket.status as import('$lib/types').TicketStatus
      });
      if (board) {
        board = {
          columns: board.columns.map(col =>
            col.status === t.status
              ? { ...col, tickets: [...col.tickets, t] }
              : col
          )
        };
        cache.set(`board:${key}`, board);
      }
      showCreate = false;
      newTicket = { title: '', ticket_type: 'task', priority: 'medium', status: 'backlog' };
    } catch (e: any) {
      createError = e.message;
    } finally {
      creating = false;
    }
  }

  const statusLabel: Record<string, string> = {
    backlog: 'Backlog',
    todo: 'To Do',
    in_progress: 'In Progress',
    in_review: 'In Review',
    done: 'Done'
  };

  const statusColor: Record<string, string> = {
    backlog: 'bg-gray-100',
    todo: 'bg-blue-50',
    in_progress: 'bg-yellow-50',
    in_review: 'bg-purple-50',
    done: 'bg-green-50'
  };
</script>

<div>
  <div class="flex items-center justify-between mb-4">
    <div class="flex items-center gap-3">
      <h1 class="text-xl font-bold">{key} Board</h1>
      <div class="flex rounded-md border border-gray-200 text-sm overflow-hidden">
        <button
          class="px-3 py-1.5 {!scrumMode ? 'bg-brand-600 text-white' : 'bg-white text-gray-600'}"
          onclick={() => { scrumMode = false; cache.invalidate(`board:${key}`); loadBoard(); }}
        >Kanban</button>
        <button
          class="px-3 py-1.5 {scrumMode ? 'bg-brand-600 text-white' : 'bg-white text-gray-600'}"
          onclick={() => { scrumMode = true; cache.invalidate(`board:${key}`); loadBoard(); }}
        >Scrum</button>
      </div>
    </div>
    <div class="flex gap-2">
      <a href="/projects/{key}/backlog" class="btn-secondary btn-sm">Backlog</a>
      <button class="btn-primary btn-sm" onclick={() => showCreate = true}>+ Ticket</button>
    </div>
  </div>

  {#if scrumMode && activeSprint}
    <div class="bg-blue-50 border border-blue-100 rounded-lg px-4 py-2 mb-4 text-sm flex items-center justify-between">
      <div>
        <span class="font-medium text-blue-900">{activeSprint.name}</span>
        {#if activeSprint.goal}
          <span class="text-blue-700 ml-2">— {activeSprint.goal}</span>
        {/if}
      </div>
      {#if activeSprint.end_date}
        <span class="text-blue-600">Ends {activeSprint.end_date}</span>
      {/if}
    </div>
  {:else if scrumMode && !activeSprint}
    <div class="bg-yellow-50 border border-yellow-100 rounded-lg px-4 py-3 mb-4 text-sm text-yellow-800">
      No active sprint. <a href="/projects/{key}/backlog" class="underline">Start a sprint from the backlog.</a>
    </div>
  {/if}

  {#if loading && !board}
    <p class="text-gray-500 py-8 text-center">Loading board…</p>
  {:else if error}
    <p class="text-red-600">{error}</p>
  {:else if board}
    <div class="flex gap-3 overflow-x-auto pb-4" style="min-height: 60vh;">
      {#each board.columns as column}
        <div
          class="flex-shrink-0 w-72 rounded-lg {statusColor[column.status]} border border-gray-200"
          ondragover={(e) => e.preventDefault()}
          ondrop={() => onDrop(column.status)}
          role="region"
          aria-label={statusLabel[column.status]}
        >
          <div class="px-3 py-2 border-b border-gray-200">
            <div class="flex items-center justify-between">
              <h3 class="text-xs font-semibold text-gray-600 uppercase tracking-wide">
                {statusLabel[column.status]}
              </h3>
              <span class="text-xs text-gray-400">{column.tickets.length}</span>
            </div>
          </div>
          <div class="p-2 flex flex-col gap-2">
            {#each column.tickets as ticket}
              <div
                class="bg-white rounded-md border border-gray-200 p-3 shadow-sm cursor-grab hover:shadow-md transition-shadow"
                draggable={true}
                ondragstart={() => onDragStart(ticket)}
                role="button"
                tabindex="0"
              >
                <div class="flex items-center gap-1.5 mb-1.5">
                  <TicketTypeIcon type={ticket.ticket_type} />
                  <PriorityBadge priority={ticket.priority} />
                  <span class="text-xs text-gray-400 font-mono ml-auto">{ticket.slug}</span>
                </div>
                <a href="/tickets/{ticket.slug}" class="text-sm font-medium text-gray-900 hover:text-brand-600 line-clamp-2">
                  {ticket.title}
                </a>
                {#if ticket.story_points}
                  <div class="mt-1.5 text-xs text-gray-400">{ticket.story_points} pts</div>
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

{#if showCreate}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg p-6 w-full max-w-md shadow-xl">
      <h2 class="text-lg font-semibold mb-4">Create Ticket</h2>
      {#if createError}<p class="text-red-600 text-sm mb-3">{createError}</p>{/if}
      <form onsubmit={(e) => { e.preventDefault(); createTicket(); }}>
        <div class="mb-4">
          <label class="label">Title</label>
          <input class="input" bind:value={newTicket.title} required />
        </div>
        <div class="grid grid-cols-2 gap-4 mb-4">
          <div>
            <label class="label">Type</label>
            <select class="input" bind:value={newTicket.ticket_type}>
              {#each ['epic','story','task','subtask','bug'] as t}
                <option value={t}>{t}</option>
              {/each}
            </select>
          </div>
          <div>
            <label class="label">Priority</label>
            <select class="input" bind:value={newTicket.priority}>
              {#each ['low','medium','high','critical'] as p}
                <option value={p}>{p}</option>
              {/each}
            </select>
          </div>
        </div>
        <div class="mb-6">
          <label class="label">Status</label>
          <select class="input" bind:value={newTicket.status}>
            {#each ['backlog','todo','in_progress','in_review','done'] as s}
              <option value={s}>{statusLabel[s]}</option>
            {/each}
          </select>
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
