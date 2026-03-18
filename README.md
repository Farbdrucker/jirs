# jirs

A self-hosted, Jira-like ticket system. Rust backend, Svelte 5 frontend, PostgreSQL.

## Tech Stack

| Layer | Technology |
|---|---|
| Backend | Rust · Axum 0.8 · sqlx (compile-time checked queries) |
| Frontend | SvelteKit · Svelte 5 runes · TypeScript · Tailwind CSS |
| Database | PostgreSQL 16 |
| Auth | JWT — 15-min access tokens + 7-day refresh tokens |
| Deployment | Docker / Docker Compose (single container) |

## Features

- **Projects** — create projects with a short key (e.g. `PROJ`), invite members
- **Tickets** — types: epic, story, task, subtask, bug · priorities · due dates · story points
- **Ticket hierarchy** — child tickets scoped by parent type (epic → story/task/bug, etc.)
- **Kanban & Scrum boards** — drag-and-drop across status columns; scrum mode shows the active sprint
- **Backlog & sprints** — create, start, and complete sprints
- **Tags** — colour-coded tags per project, add/remove from ticket detail
- **Assignment** — searchable user picker on every ticket
- **Comments** — threaded comments per ticket
- **Ticket links** — relates\_to, blocks, is\_blocked\_by, duplicates (bidirectional)
- **Repo links** — attach repo URL, branch, and PR URL to a ticket
- **Activity log** — audit trail of status/assignee changes
- **User profile & settings** — update display name, avatar URL, change password
- **Local-first UI** — stale-while-revalidate cache (localStorage) for instant page loads; optimistic mutations with rollback

## Installation

### Docker (recommended)

```sh
docker compose up --build
```

Opens on `http://localhost:8080`. Migrations run automatically on startup.

### Local development

**Prerequisites:** Rust (stable), Node 18+, Docker (for Postgres)

```sh
# 1. Start the database
docker compose up -d db

# 2. Run the backend
cd backend
DATABASE_URL=postgres://jirs:jirs@localhost:5432/jirs cargo run

# 3. Run the frontend (separate terminal)
cd frontend
npm install
npm run dev        # http://localhost:5173 — proxies /api to :8080
```

### Environment variables

The backend reads these from the environment (or a `.env` file):

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — | Postgres connection string |
| `JWT_SECRET` | — | Secret for access tokens |
| `JWT_REFRESH_SECRET` | — | Secret for refresh tokens |
| `PORT` | `8080` | HTTP listen port |

The `docker-compose.yml` sets all defaults for local use.
