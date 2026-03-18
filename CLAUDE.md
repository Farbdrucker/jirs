# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`jirs` is a Jira-like ticket system with a Rust (Axum) backend, Svelte 5 frontend, and PostgreSQL.

## Repository Layout

```
jirs/
├── backend/          # Rust Axum API server
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── auth/, projects/, tickets/, comments/
│   │   ├── boards/, sprints/, tags/, links/
│   │   ├── users/, activity/
│   │   └── config.rs, db.rs, error.rs
│   ├── migrations/   # 001-009 SQL migrations (run on startup)
│   └── .sqlx/        # sqlx offline query cache (must be kept in sync)
├── frontend/         # SvelteKit + Tailwind CSS SPA
│   └── src/routes/   # login, projects/[key]/board|backlog, tickets/[slug]
├── Dockerfile        # multi-stage build
└── docker-compose.yml
```

## Backend Commands

```sh
cd backend
cargo build          # compile (requires DATABASE_URL or SQLX_OFFLINE=true)
cargo run            # run (requires DATABASE_URL)
cargo test           # run all tests
cargo clippy         # lint
cargo fmt            # format

# When changing SQL queries, regenerate the sqlx cache:
DATABASE_URL=postgres://jirs:jirs@localhost:5432/jirs cargo sqlx prepare
```

## Frontend Commands

```sh
cd frontend
npm install          # install deps
npm run dev          # dev server (proxies /api to localhost:8080)
npm run build        # production build → frontend/build/
npm run check        # type-check
```

## Docker / Full Stack

```sh
# Start postgres only (for local dev):
docker compose up -d db

# Start the full stack:
docker compose up --build

# Apply migrations manually (dev):
DATABASE_URL=postgres://jirs:jirs@localhost:5432/jirs sqlx migrate run
```

## Dev Setup

1. `docker compose up -d db`
2. `cd backend && DATABASE_URL=postgres://jirs:jirs@localhost:5432/jirs cargo run`
3. `cd frontend && npm run dev`  (opens on port 5173, proxies API to 8080)

## Key Design Decisions

- **AppState** struct holds both `PgPool` and `Config`; passed as single Axum state
- **sqlx compile-time queries**: `.sqlx/` cache committed to repo for Docker offline builds
- **Ticket slugs**: atomically generated `PROJECT-N` via `UPDATE counter ... RETURNING`
- **JWT**: access tokens (15 min) + refresh tokens (7 day), Bearer header auth
- **Bidirectional links**: inserting a `blocks` link also inserts `is_blocked_by` inverse
- **Static SPA**: SvelteKit adapter-static builds to `backend/static/`, served by Axum's `ServeDir`
