<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { cache } from '$lib/cache';
  import type { TicketDetail, Comment, ActivityEntry, TicketLink, RepoLink, Sprint, UserSummary } from '$lib/types';
  import PriorityBadge from '$lib/components/PriorityBadge.svelte';
  import TicketTypeIcon from '$lib/components/TicketTypeIcon.svelte';
  import UserPicker from '$lib/components/UserPicker.svelte';
  import TagPicker from '$lib/components/TagPicker.svelte';
  import ChildTickets from '$lib/components/ChildTickets.svelte';

  const slug = $derived($page.params.slug as string);
  let ticket: TicketDetail | null = $state(null);
  let comments: Comment[] = $state([]);
  let activity: ActivityEntry[] = $state([]);
  let sprints: Sprint[] = $state([]);
  let users: UserSummary[] = $state([]);
  let loading = $state(true);
  let error = $state('');

  // Edit state
  let editing = $state(false);
  let editTitle = $state('');
  let editDescription = $state('');
  let editPriority = $state('');
  let editStatus = $state('');
  let saving = $state(false);

  // Comment state
  let commentBody = $state('');
  let postingComment = $state(false);

  // Link state
  let showLinkForm = $state(false);
  let linkTargetSlug = $state('');
  let linkType = $state('relates_to');
  let addingLink = $state(false);
  let ticketLinks: TicketLink[] = $state([]);

  // Repo link state
  let showRepoForm = $state(false);
  let repoUrl = $state('');
  let branchName = $state('');
  let prUrl = $state('');
  let repoLinks: RepoLink[] = $state([]);
  let addingRepo = $state(false);

  onMount(async () => {
    await loadAll();
  });

  async function loadAll() {
    loading = true;
    error = '';
    try {
      // Stale-while-revalidate for ticket
      const cacheKey = `ticket:${slug}`;
      const cached = cache.get<TicketDetail>(cacheKey);
      if (cached) {
        ticket = cached;
        loading = false;
        setupEdit();
      }

      const [freshTicket, freshComments, freshActivity] = await Promise.all([
        api.tickets.get(slug),
        api.comments.list(slug),
        api.activity.list(slug)
      ]);
      ticket = freshTicket;
      comments = freshComments;
      activity = freshActivity;
      cache.set(cacheKey, freshTicket);
      setupEdit();

      const projectKey = slug.split('-')[0];
      const usersKey = 'users';
      const sprintsKey = `sprints:${projectKey}`;

      const cachedUsers = cache.get<UserSummary[]>(usersKey);
      if (cachedUsers) users = cachedUsers;
      const cachedSprints = cache.get<Sprint[]>(sprintsKey);
      if (cachedSprints) sprints = cachedSprints;

      if (cache.isStale(usersKey) || cache.isStale(sprintsKey)) {
        const [freshUsers, freshSprints] = await Promise.all([
          api.users.list(),
          api.sprints.list(projectKey)
        ]);
        users = freshUsers;
        sprints = freshSprints;
        cache.set(usersKey, freshUsers);
        cache.set(sprintsKey, freshSprints);
      }
    } catch (e: any) {
      error = e.message;
    } finally {
      loading = false;
    }
  }

  function setupEdit() {
    if (!ticket) return;
    editTitle = ticket.title;
    editDescription = ticket.description || '';
    editPriority = ticket.priority;
    editStatus = ticket.status;
  }

  async function saveTicket() {
    if (!ticket) return;
    saving = true;
    try {
      const updated = await api.tickets.update(slug, {
        title: editTitle,
        description: editDescription || undefined,
        priority: editPriority as import('$lib/types').TicketPriority,
        status: editStatus as import('$lib/types').TicketStatus
      });
      ticket = { ...ticket, ...updated };
      cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, ...updated }));
      editing = false;
    } catch (e: any) {
      alert(e.message);
    } finally {
      saving = false;
    }
  }

  async function assign(userId: string | null) {
    if (!ticket) return;
    const oldAssigneeId = ticket.assignee_id;
    const oldAssignee = ticket.assignee;
    // Optimistic
    const newStub = userId ? (users.find(u => u.id === userId) ?? null) : null;
    ticket = { ...ticket, assignee_id: userId, assignee: newStub };
    cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, assignee_id: userId, assignee: newStub }));
    try {
      await api.tickets.assign(slug, userId);
    } catch (e: any) {
      // Rollback
      ticket = { ...ticket, assignee_id: oldAssigneeId, assignee: oldAssignee };
      cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, assignee_id: oldAssigneeId, assignee: oldAssignee }));
      alert(e.message);
    }
  }

  async function removeTag(tagId: string) {
    if (!ticket) return;
    const oldTags = ticket.tags;
    ticket = { ...ticket, tags: ticket.tags.filter(t => t.id !== tagId) };
    cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, tags: t.tags.filter(tg => tg.id !== tagId) }));
    try {
      await api.tags.removeFromTicket(slug, tagId);
    } catch (e: any) {
      ticket = { ...ticket, tags: oldTags };
      cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, tags: oldTags }));
      alert(e.message);
    }
  }

  async function addTag(tag: import('$lib/types').Tag) {
    if (!ticket) return;
    if (ticket.tags.some(t => t.id === tag.id)) return;
    const oldTags = ticket.tags;
    ticket = { ...ticket, tags: [...ticket.tags, tag] };
    cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, tags: [...t.tags, tag] }));
    try {
      await api.tags.addToTicket(slug, tag.id);
    } catch (e: any) {
      ticket = { ...ticket, tags: oldTags };
      cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, tags: oldTags }));
      alert(e.message);
    }
  }

  async function postComment() {
    if (!commentBody.trim()) return;
    postingComment = true;
    try {
      const c = await api.comments.create(slug, { body: commentBody });
      comments = [...comments, c];
      commentBody = '';
    } catch (e: any) {
      alert(e.message);
    } finally {
      postingComment = false;
    }
  }

  async function deleteComment(id: string) {
    if (!confirm('Delete this comment?')) return;
    await api.comments.delete(id);
    comments = comments.filter(c => c.id !== id);
  }

  async function addLink() {
    if (!linkTargetSlug.trim()) return;
    addingLink = true;
    try {
      const link = await api.links.create(slug, { target_slug: linkTargetSlug, link_type: linkType });
      ticketLinks = [...ticketLinks, link];
      showLinkForm = false;
      linkTargetSlug = '';
    } catch (e: any) {
      alert(e.message);
    } finally {
      addingLink = false;
    }
  }

  async function addRepoLink() {
    addingRepo = true;
    try {
      const rl = await api.links.createRepo(slug, {
        repo_url: repoUrl,
        branch_name: branchName || undefined,
        pr_url: prUrl || undefined
      });
      repoLinks = [...repoLinks, rl];
      showRepoForm = false;
      repoUrl = branchName = prUrl = '';
    } catch (e: any) {
      alert(e.message);
    } finally {
      addingRepo = false;
    }
  }

  const statusOptions = ['backlog', 'todo', 'in_progress', 'in_review', 'done'];
  const priorityOptions = ['low', 'medium', 'high', 'critical'];
  const statusLabel: Record<string, string> = {
    backlog: 'Backlog', todo: 'To Do', in_progress: 'In Progress',
    in_review: 'In Review', done: 'Done'
  };

  function actionLabel(action: string): string {
    switch (action) {
      case 'status_changed': return 'changed status';
      case 'assignee_changed': return 'changed assignee';
      case 'comment_added': return 'commented';
      default: return action;
    }
  }

  function initials(name: string) {
    return name.split(' ').map(w => w[0]).join('').toUpperCase().slice(0, 2);
  }
</script>

{#if loading && !ticket}
  <p class="text-gray-500 py-8">Loading…</p>
{:else if error}
  <p class="text-red-600">{error}</p>
{:else if ticket}
  {@const projectKey = ticket.slug.split('-')[0]}
  <div class="flex gap-6">
    <!-- Main content -->
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2 mb-4 text-sm text-gray-500">
        <a href="/" class="hover:text-brand-600">Projects</a>
        <span>/</span>
        <a href="/projects/{projectKey}/board" class="hover:text-brand-600">{projectKey}</a>
        <span>/</span>
        <span class="font-mono">{ticket.slug}</span>
      </div>

      {#if editing}
        <div class="card p-4 mb-4">
          <input class="input text-xl font-bold mb-3 border-0 border-b" bind:value={editTitle} />
          <textarea class="input w-full mb-3 font-mono text-sm" rows={8} bind:value={editDescription}
                    placeholder="Description (markdown supported)"></textarea>
          <div class="flex gap-2">
            <button class="btn-primary btn-sm" onclick={saveTicket} disabled={saving}>
              {saving ? 'Saving…' : 'Save'}
            </button>
            <button class="btn-secondary btn-sm" onclick={() => editing = false}>Cancel</button>
          </div>
        </div>
      {:else}
        <div class="mb-4">
          <div class="flex items-start justify-between gap-3">
            <h1 class="text-2xl font-bold text-gray-900">{ticket.title}</h1>
            <button class="btn-secondary btn-sm shrink-0" onclick={() => editing = true}>Edit</button>
          </div>
          {#if ticket.description}
            <div class="prose prose-sm mt-4 text-gray-700 max-w-none">
              <pre class="whitespace-pre-wrap font-sans text-sm leading-relaxed">{ticket.description}</pre>
            </div>
          {:else}
            <p class="text-gray-400 text-sm mt-3 italic">No description</p>
          {/if}
        </div>
      {/if}

      <!-- Child Tickets -->
      <ChildTickets
        children={ticket.children}
        parentType={ticket.ticket_type}
        parentId={ticket.id}
        {projectKey}
      />

      <!-- Comments -->
      <section class="mt-6">
        <h2 class="text-sm font-semibold text-gray-700 mb-3">Comments</h2>
        {#each comments as c}
          <div class="flex gap-3 mb-4">
            <div class="w-8 h-8 rounded-full bg-brand-100 text-brand-700 text-xs font-semibold flex items-center justify-center shrink-0">
              {c.author_display_name[0].toUpperCase()}
            </div>
            <div class="flex-1">
              <div class="flex items-center gap-2 text-xs text-gray-500 mb-1">
                <span class="font-medium text-gray-900">{c.author_display_name}</span>
                <span>{new Date(c.created_at).toLocaleDateString()}</span>
                <button class="ml-auto text-red-400 hover:text-red-600" onclick={() => deleteComment(c.id)}>Delete</button>
              </div>
              <div class="bg-gray-50 rounded-lg p-3 text-sm text-gray-800 whitespace-pre-wrap">{c.body}</div>
            </div>
          </div>
        {/each}

        <div class="mt-3">
          <textarea class="input w-full text-sm" rows={3} placeholder="Add a comment…"
                    bind:value={commentBody}></textarea>
          <button class="btn-primary btn-sm mt-2" onclick={postComment} disabled={postingComment || !commentBody.trim()}>
            {postingComment ? 'Posting…' : 'Comment'}
          </button>
        </div>
      </section>

      <!-- Ticket Links -->
      <section class="mt-6">
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-sm font-semibold text-gray-700">Links</h2>
          <button class="text-xs text-brand-600 hover:underline" onclick={() => showLinkForm = !showLinkForm}>
            + Add link
          </button>
        </div>
        {#if showLinkForm}
          <div class="bg-gray-50 rounded-lg p-3 mb-3 flex gap-2">
            <select class="input text-sm py-1" bind:value={linkType}>
              {#each ['relates_to','blocks','is_blocked_by','duplicates','is_duplicated_by'] as lt}
                <option value={lt}>{lt.replace(/_/g,' ')}</option>
              {/each}
            </select>
            <input class="input flex-1 text-sm py-1 font-mono" placeholder="PROJ-42" bind:value={linkTargetSlug} />
            <button class="btn-primary btn-sm" onclick={addLink} disabled={addingLink}>Add</button>
          </div>
        {/if}
        {#each ticketLinks as link}
          <div class="flex items-center gap-2 text-sm py-1">
            <span class="text-xs text-gray-400 w-28 shrink-0">{link.link_type.replace(/_/g,' ')}</span>
            <a href="/tickets/{link.target_slug}" class="font-mono text-brand-600 hover:underline">{link.target_slug}</a>
            <span class="text-gray-600 truncate">{link.target_title}</span>
          </div>
        {/each}
      </section>

      <!-- Repo Links -->
      <section class="mt-6">
        <div class="flex items-center justify-between mb-2">
          <h2 class="text-sm font-semibold text-gray-700">Repository</h2>
          <button class="text-xs text-brand-600 hover:underline" onclick={() => showRepoForm = !showRepoForm}>
            + Add repo link
          </button>
        </div>
        {#if showRepoForm}
          <div class="bg-gray-50 rounded-lg p-3 mb-3 flex flex-col gap-2">
            <input class="input text-sm py-1" placeholder="Repo URL" bind:value={repoUrl} />
            <input class="input text-sm py-1 font-mono" placeholder="Branch (e.g. feature/PROJ-42)" bind:value={branchName} />
            <input class="input text-sm py-1" placeholder="PR URL (optional)" bind:value={prUrl} />
            <button class="btn-primary btn-sm self-end" onclick={addRepoLink} disabled={addingRepo || !repoUrl}>Add</button>
          </div>
        {/if}
        {#each repoLinks as rl}
          <div class="text-sm py-1">
            <a href={rl.repo_url} target="_blank" rel="noopener" class="text-brand-600 hover:underline truncate">{rl.repo_url}</a>
            {#if rl.branch_name}
              <span class="ml-2 font-mono text-xs text-gray-500">{rl.branch_name}</span>
            {/if}
          </div>
        {/each}
      </section>
    </div>

    <!-- Sidebar -->
    <div class="w-64 shrink-0">
      <div class="card p-4 sticky top-4">
        <div class="mb-4">
          <div class="flex items-center gap-2 mb-2">
            <TicketTypeIcon type={ticket.ticket_type} />
            <span class="text-xs text-gray-500 font-mono">{ticket.slug}</span>
          </div>
          <PriorityBadge priority={ticket.priority} />
        </div>

        <div class="space-y-3">
          <div>
            <label class="label text-xs">Status</label>
            <select class="input text-sm py-1" value={ticket.status} onchange={async (e) => {
              const sel = e.currentTarget as HTMLSelectElement;
              const updated = await api.tickets.patchStatus(slug, sel.value);
              if (ticket) ticket = { ...ticket, status: updated.status };
              cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, status: updated.status }));
            }}>
              {#each statusOptions as s}
                <option value={s}>{statusLabel[s]}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="label text-xs">Priority</label>
            <select class="input text-sm py-1" value={ticket.priority} onchange={async (e) => {
              const sel = e.currentTarget as HTMLSelectElement;
              const updated = await api.tickets.update(slug, { priority: sel.value as import('$lib/types').TicketPriority });
              if (ticket) ticket = { ...ticket, priority: updated.priority };
              cache.patch<TicketDetail>(`ticket:${slug}`, t => ({ ...t, priority: updated.priority }));
            }}>
              {#each priorityOptions as p}
                <option value={p}>{p}</option>
              {/each}
            </select>
          </div>

          <div>
            <label class="label text-xs">Type</label>
            <span class="text-sm text-gray-700">{ticket.ticket_type}</span>
          </div>

          <!-- Assignee -->
          <div>
            <label class="label text-xs">Assignee</label>
            <UserPicker {users} selectedId={ticket.assignee_id} onSelect={assign} />
          </div>

          <!-- Reporter -->
          <div>
            <label class="label text-xs">Reporter</label>
            <div class="flex items-center gap-2">
              <div class="w-6 h-6 rounded-full bg-brand-100 text-brand-700 text-xs font-semibold flex items-center justify-center shrink-0">
                {initials(ticket.reporter.display_name)}
              </div>
              <span class="text-sm text-gray-700">{ticket.reporter.display_name}</span>
            </div>
          </div>

          <!-- Parent -->
          {#if ticket.parent}
            <div>
              <label class="label text-xs">Parent</label>
              <a href="/tickets/{ticket.parent.slug}" class="text-sm text-brand-600 hover:underline font-mono">
                {ticket.parent.slug}
              </a>
              <span class="text-xs text-gray-500 ml-1 truncate">{ticket.parent.title}</span>
            </div>
          {/if}

          {#if ticket.due_date}
            <div>
              <label class="label text-xs">Due</label>
              <span class="text-sm text-gray-700">{ticket.due_date}</span>
            </div>
          {/if}

          {#if ticket.story_points}
            <div>
              <label class="label text-xs">Story Points</label>
              <span class="text-sm text-gray-700">{ticket.story_points}</span>
            </div>
          {/if}

          <!-- Tags -->
          <div>
            <label class="label text-xs">Tags</label>
            <div class="flex flex-wrap gap-1 mt-1">
              {#each ticket.tags as tag}
                <span
                  class="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full text-white"
                  style="background:{tag.color}"
                >
                  {tag.name}
                  <button
                    type="button"
                    class="hover:opacity-75 leading-none"
                    onclick={() => removeTag(tag.id)}
                    aria-label="Remove tag {tag.name}"
                  >✕</button>
                </span>
              {/each}
            </div>
            <div class="mt-1.5">
              <TagPicker
                {projectKey}
                currentTagIds={ticket.tags.map(t => t.id)}
                onAdd={addTag}
              />
            </div>
          </div>
        </div>

        <!-- Activity -->
        {#if activity.length > 0}
          <div class="mt-6 border-t pt-4">
            <h3 class="text-xs font-semibold text-gray-500 uppercase mb-3">Activity</h3>
            <div class="space-y-2">
              {#each activity.slice(0, 10) as a}
                <div class="text-xs text-gray-500">
                  <span class="font-medium text-gray-700">{a.actor_display_name}</span>
                  {actionLabel(a.action)}
                  {#if a.old_value && a.new_value}
                    <span class="text-gray-400">: {a.old_value} → {a.new_value}</span>
                  {/if}
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
