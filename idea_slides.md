---
marp: true
theme: default
paginate: true
style: |
  section {
    font-family: 'Inter', 'Segoe UI', sans-serif;
    background: #f8fafc;
    color: #1e293b;
  }
  section.lead {
    background: #1e293b;
    color: #f8fafc;
  }
  section.lead h1 {
    color: #f8fafc;
    font-size: 3rem;
  }
  section.lead p {
    color: #94a3b8;
  }
  h1 { color: #1e293b; border-bottom: 2px solid #6366f1; padding-bottom: 0.3em; }
  h2 { color: #6366f1; }
  code { background: #e2e8f0; padding: 0.1em 0.4em; border-radius: 4px; }
  table { width: 100%; border-collapse: collapse; }
  th { background: #6366f1; color: white; padding: 0.5em 1em; }
  td { padding: 0.5em 1em; border-bottom: 1px solid #e2e8f0; }
  blockquote { border-left: 4px solid #6366f1; padding-left: 1em; color: #475569; }
---

<!-- _class: lead -->

# jirs

### A self-hosted, Jira-like ticket system
Built with Rust · Svelte 5 · PostgreSQL

---

# What is jirs?

A **self-hosted project management tool** modelled after Jira — built for small teams who want full control over their data without a SaaS subscription.

> "The power of a ticket system, the simplicity of a single `docker compose up`."

**Core idea:** one Docker container, zero external dependencies, instant setup.

---

# Why build it?

- **Privacy** — your tickets, your server, your data
- **Cost** — no per-seat pricing
- **Customisability** — open source, change anything
- **Learning** — a real-world full-stack app in Rust + Svelte 5

---

# Tech Stack

| Layer | Technology |
|---|---|
| **Backend** | Rust · Axum 0.8 |
| **Database** | PostgreSQL 16 · sqlx |
| **Auth** | JWT (access + refresh tokens) |
| **Frontend** | SvelteKit · Svelte 5 · TypeScript |
| **Styling** | Tailwind CSS |
| **Deploy** | Docker · Docker Compose |

---

# Why Rust for the Backend?

- **Memory safety** with zero garbage collection pauses
- **sqlx compile-time query checking** — SQL errors caught at `cargo build`, not at runtime
- **Axum** is ergonomic, async-native, and production-ready
- Single statically linked binary — tiny Docker image

```rust
// sqlx: if your SQL is wrong, this won't compile
let ticket = sqlx::query_as!(Ticket,
  "SELECT id, slug, title FROM tickets WHERE slug = $1",
  slug
).fetch_one(&pool).await?;
```

---

# Why Svelte 5?

- **Runes syntax** (`$state`, `$derived`, `$props`) — reactive by default, no boilerplate
- **SvelteKit adapter-static** — builds to plain HTML/JS, served directly by Axum
- No virtual DOM — smaller bundle, faster renders

```svelte
<script lang="ts">
  let ticket: TicketDetail | null = $state(null);
  const projectKey = $derived(ticket?.slug.split('-')[0]);
</script>
```

---

# Architecture Overview

```
Browser
  │  HTTP/JSON
  ▼
Axum (port 8080)
  ├── /api/*        → protected routes (JWT middleware)
  ├── /api/auth/*   → public routes (login, register)
  └── /*            → ServeDir → SvelteKit static build
          │
          ▼
     PostgreSQL 16
     (migrations run on startup)
```

Single process. No microservices. No message queues. Simple.

---

# Data Model

```
Project  ──┬──  Ticket  ──┬──  Comment
           │              ├──  Tag  (via ticket_tags)
           ├──  Sprint     ├──  TicketLink  (blocks / relates_to …)
           ├──  Tag        ├──  RepoLink
           └──  Member     ├──  Activity
                           └──  parent_id → Ticket  (hierarchy)
```

Ticket types: **epic → story → task → subtask / bug**

---

# Authentication

- **Argon2** password hashing (memory-hard, resistant to brute force)
- **JWT access token** — 15-minute TTL, sent as `Authorization: Bearer`
- **JWT refresh token** — 7-day TTL, used to silently renew access tokens
- Frontend auto-retries failed requests after token refresh — transparent to the user

---

# Feature: Boards

Two modes on the same data:

**Kanban** — all tickets across all sprints, 5 columns

**Scrum** — only tickets in the active sprint

Both support **drag-and-drop** with optimistic UI:
1. Move the card in the UI instantly
2. PATCH the backend
3. Rollback if the request fails

---

# Feature: Ticket Hierarchy

Parent–child relationships with enforced type rules:

| Parent | Allowed children |
|---|---|
| Epic | Story, Task, Bug |
| Story | Task, Subtask, Bug |
| Task | Subtask, Bug |
| Subtask | Bug |
| Bug | — |

Child tickets are shown inline on the parent's detail view.

---

# Feature: Tags

- Per-project coloured tags
- Add / remove tags from ticket detail with a live-search picker
- Create new tags inline (name + colour picker) without leaving the ticket
- Optimistic updates — UI changes immediately, API call happens in background

---

# Feature: Local-First Cache

Every page uses a **stale-while-revalidate** pattern:

```
1. Check localStorage cache
2. If cached → render immediately (zero loading spinner)
3. Fetch fresh data in the background
4. Update UI when fresh data arrives
```

Optimistic mutations follow the same principle — update the cache first, rollback on error. 5-minute TTL per cache key.

---

# Feature: User Assignment

- Searchable user picker on every ticket sidebar
- Shows avatar initials + display name
- Optimistic: assignee updates instantly in the UI
- Assignee changes are logged in the activity trail

---

# Feature: Activity Log

Every significant action is recorded:

- Status changed (`backlog → in_progress`)
- Assignee changed
- Comment added

Displayed per-ticket in the sidebar. No additional infrastructure — just a `ticket_activity` table with `actor_id`, `action`, `old_value`, `new_value`.

---

# Feature: Ticket Links

Bidirectional relationships between tickets:

| Type | Inverse (auto-created) |
|---|---|
| `blocks` | `is_blocked_by` |
| `duplicates` | `is_duplicated_by` |
| `relates_to` | `relates_to` |

One INSERT creates both directions — queries only need to look one way.

---

# Feature: User Settings

- Update **display name** and **avatar URL**
- **Change password** with current-password verification
- Profile changes immediately reflected in the navbar (auth store updated in-memory)

All handled by two new backend endpoints: `PUT /api/users/me` and `POST /api/users/me/password`.

---

# Deployment

```sh
# Full stack — one command
docker compose up --build
```

- Multi-stage Dockerfile: `rust:alpine` builder → `alpine` runtime (~20 MB image)
- PostgreSQL migrations run automatically on startup via `sqlx::migrate!()`
- Frontend is compiled to `backend/static/` and served by Axum's `ServeDir`
- No Nginx. No reverse proxy needed for single-server setups.

---

# sqlx Offline Mode

The `.sqlx/` directory stores **cached query metadata** checked into git.

```sh
# After changing any SQL query, regenerate:
DATABASE_URL=postgres://... cargo sqlx prepare
```

This lets Docker builds compile without a live database — the query types are resolved from the cache. CI/CD and offline builds both work without a running Postgres instance.

---

# API Design

RESTful, resource-oriented:

```
GET    /api/projects/:key/tickets
POST   /api/projects/:key/tickets
GET    /api/tickets/:slug           → enriched TicketDetail
PUT    /api/tickets/:slug
PATCH  /api/tickets/:slug/status
PATCH  /api/tickets/:slug/assign
GET    /api/tickets/:slug/children
GET    /api/tickets/:slug/tags
```

All protected routes require `Authorization: Bearer <token>`. Auth middleware injects `CurrentUser` via Axum `Extension`.

---

# Ideas & Future Directions

- **Notifications** — in-app or email on mention / assignment
- **Roadmap view** — Gantt-style timeline for epics and sprints
- **File attachments** — S3-compatible object storage for ticket attachments
- **Webhooks** — emit events on ticket create/update for external integrations
- **OAuth** — GitHub / Google login
- **Search** — full-text search across ticket titles and descriptions

---

# Ideas: Real-time Collaboration

Currently: stale-while-revalidate (poll on mount)

Next step: **Server-Sent Events (SSE)** for live board updates

```
Browser ←── SSE stream ──── Axum
              (ticket moved, comment added, …)
```

No WebSocket complexity, no additional broker. Axum supports SSE natively.

---

# Ideas: Mobile-Friendly PWA

- SvelteKit already outputs a static SPA — add a `manifest.json` and service worker
- Offline-first: local cache already in place, extend it with a service worker
- Push notifications via Web Push for assignment events

---

# Summary

| What | How |
|---|---|
| Self-hosted ticket system | Single Docker container |
| Type-safe SQL | sqlx compile-time macros |
| Fast, reactive UI | Svelte 5 runes + local-first cache |
| Secure auth | Argon2 + JWT access/refresh |
| Full ticket lifecycle | Hierarchy · sprints · boards · tags · links |

---

<!-- _class: lead -->

# Get Started

```sh
git clone <repo>
docker compose up --build
# open http://localhost:8080
```

Register an account, create a project, and start shipping.
