<script lang="ts">
  import { api } from '$lib/api';
  import { cache } from '$lib/cache';
  import type { TicketSummary, TicketType } from '$lib/types';
  import TicketTypeIcon from './TicketTypeIcon.svelte';

  let {
    children: initialChildren,
    parentType,
    parentId,
    projectKey
  }: {
    children: TicketSummary[];
    parentType: TicketType;
    parentId: string;
    projectKey: string;
  } = $props();

  let children = $state(initialChildren);

  const ALLOWED_CHILDREN: Record<TicketType, TicketType[]> = {
    epic: ['story', 'task', 'bug'],
    story: ['task', 'subtask', 'bug'],
    task: ['subtask', 'bug'],
    subtask: ['bug'],
    bug: []
  };

  const allowedTypes = $derived(ALLOWED_CHILDREN[parentType] ?? []);

  let showForm = $state(false);
  let newTitle = $state('');
  let newType: TicketType = $state(allowedTypes[0] ?? 'task');
  let adding = $state(false);

  const statusLabel: Record<string, string> = {
    backlog: 'Backlog', todo: 'To Do', in_progress: 'In Progress',
    in_review: 'In Review', done: 'Done'
  };

  const statusColor: Record<string, string> = {
    backlog: 'bg-gray-100 text-gray-600',
    todo: 'bg-blue-100 text-blue-700',
    in_progress: 'bg-yellow-100 text-yellow-700',
    in_review: 'bg-purple-100 text-purple-700',
    done: 'bg-green-100 text-green-700'
  };

  async function addChild() {
    if (!newTitle.trim()) return;
    adding = true;
    try {
      const ticket = await api.tickets.create(projectKey, {
        title: newTitle.trim(),
        ticket_type: newType,
        parent_id: parentId
      });
      // Optimistically push to children list
      const summary: TicketSummary = {
        id: ticket.id,
        slug: ticket.slug,
        title: ticket.title,
        ticket_type: ticket.ticket_type,
        status: ticket.status
      };
      children = [...children, summary];
      cache.invalidate(`board:${projectKey}`);
      cache.invalidate(`tickets:${projectKey}`);
      newTitle = '';
      newType = allowedTypes[0] ?? 'task';
      showForm = false;
    } catch (e: any) {
      alert(e.message);
    } finally {
      adding = false;
    }
  }
</script>

{#if allowedTypes.length > 0}
  <section class="mt-6">
    <div class="flex items-center justify-between mb-2">
      <h2 class="text-sm font-semibold text-gray-700">Child Tickets ({children.length})</h2>
      <button
        type="button"
        class="text-xs text-brand-600 hover:underline"
        onclick={() => showForm = !showForm}
      >
        + Add child
      </button>
    </div>

    {#if showForm}
      <div class="bg-gray-50 rounded-lg p-3 mb-3 flex gap-2">
        <select class="input text-sm py-1 w-28 shrink-0" bind:value={newType}>
          {#each allowedTypes as t}
            <option value={t}>{t}</option>
          {/each}
        </select>
        <input
          class="input flex-1 text-sm py-1"
          placeholder="Child ticket title"
          bind:value={newTitle}
          onkeydown={(e) => e.key === 'Enter' && addChild()}
        />
        <button class="btn-primary btn-sm" onclick={addChild} disabled={adding || !newTitle.trim()}>
          {adding ? '…' : 'Add'}
        </button>
        <button class="btn-secondary btn-sm" onclick={() => showForm = false}>Cancel</button>
      </div>
    {/if}

    {#if children.length > 0}
      <div class="card divide-y divide-gray-100">
        {#each children as child}
          <div class="flex items-center gap-3 px-3 py-2 hover:bg-gray-50">
            <TicketTypeIcon type={child.ticket_type} />
            <a href="/tickets/{child.slug}" class="text-xs font-mono text-brand-600 hover:underline w-20 shrink-0">{child.slug}</a>
            <a href="/tickets/{child.slug}" class="flex-1 text-sm text-gray-900 hover:text-brand-600 truncate">{child.title}</a>
            <span class="text-xs px-1.5 py-0.5 rounded {statusColor[child.status]}">
              {statusLabel[child.status] ?? child.status}
            </span>
          </div>
        {/each}
      </div>
    {:else}
      <p class="text-sm text-gray-400 italic">No child tickets yet.</p>
    {/if}
  </section>
{/if}
