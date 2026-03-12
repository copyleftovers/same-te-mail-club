# The Mail Club

Seasonal offline-first self-expression ritual for Kyiv. Participants create physical mail without knowing who will receive it, send via Nova Poshta, then meet in person.

The app handles logistics only: sign-ups, assignments, SMS reminders, delivery confirmations. Everything that matters happens offline.

## Stack

- **Leptos 0.8** (SSR + hydration) on **Axum**
- **sqlx** with compile-time checked queries against **PostgreSQL**
- **TurboSMS** for OTP auth and notifications
- Stable Rust, edition 2024
- Deployed via **Coolify** (Docker on VPS)

## Prerequisites

- Rust stable (1.85+)
- `cargo-leptos` (`cargo binstall cargo-leptos`)
- `sqlx-cli` (`cargo binstall sqlx-cli --no-default-features --features postgres,rustls`)
- Docker (for local Postgres)
- Node.js (for Playwright E2E tests)
- `pre-commit` (`pip install pre-commit`)

## Setup

```sh
# Start Postgres
docker compose up -d

# Run migrations
export DATABASE_URL="postgres://samete:samete@localhost/samete"
just db-migrate

# Install git hooks
pre-commit install

# Start dev server (hot reload)
just dev
```

## Development Commands

| Command | What it does |
|---------|-------------|
| `just dev` | Start dev server with hot reload |
| `just test` | Run unit tests |
| `just clippy` | Run clippy on all targets |
| `just e2e` | Run Playwright E2E tests |
| `just build` | Release build |
| `just check` | Full validation (fmt + clippy + test) |
| `just db-migrate` | Run pending migrations |
| `just db-reset` | Drop, create, and migrate database |
| `just db-new <name>` | Create a new migration |
| `just prepare` | Generate sqlx offline query data |
| `bacon` | Continuous clippy (default: SSR target) |

Bacon keybindings: `s` (SSR clippy), `h` (hydrate clippy), `t` (tests).

## Project Structure

```
src/
  main.rs       — SSR entry point, server setup
  lib.rs        — WASM hydrate entry point
  app.rs        — shell(), App component, route definitions
  auth.rs       — OTP flow, session management, CSRF
  sms.rs        — TurboSMS API client
  config.rs     — Config from environment variables
  error.rs      — AppError enum
  phone.rs      — Phone number normalization
  db.rs         — PgPool setup, migrations
  types.rs      — Phase enum, newtypes, shared domain types
  pages/        — Participant-facing page components
  admin/        — Organizer-facing admin components
migrations/     — sqlx SQL migrations
end2end/        — Playwright E2E tests
style/          — CSS
public/         — Static assets
```

## Architecture

Full spec and architecture docs at `/Users/ryzhakar/notes/same te/mail club/`.

Key decisions:
- **Single opaque session token** (256-bit random, SHA-256 hashed in DB, 90-day expiry)
- **Phase enum** with transition methods — the single source of season state rules
- **Hamiltonian cycle** assignment via backtracking DFS with social-awareness scoring
- **No ORM, no session middleware, no auth library** — each concern is a small composed piece
- **clippy::pedantic = deny** — pedantic findings are errors, not warnings

## Environment Variables

| Variable | Required | Purpose |
|----------|----------|---------|
| `DATABASE_URL` | Yes | Postgres connection string |
| `TURBOSMS_TOKEN` | Yes | TurboSMS API bearer token |
| `TURBOSMS_SENDER` | Yes | Registered alpha-name |
| `CSRF_SECRET` | No | Override for CSRF secret (generated at startup if absent) |

## Deployment

Multi-stage Docker build with cargo-chef for dependency caching and distroless runtime image. Coolify watches the repo and auto-builds on push.

```sh
just build          # local release build
docker compose up   # local with Postgres
```
