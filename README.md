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

## Spec

Full product spec, personas, user stories, technical research, and architecture in `spec/`.

## Configuration Decisions

Choices made during project setup, with rationale. Authoritative source: `spec/technical/Architecture.md`.

### Crate & Toolchain

| Decision | Choice | Why |
|----------|--------|-----|
| Template | `leptos-rs/start-axum` | Official Leptos 0.8 SSR template. Single crate, `cdylib+rlib` |
| Edition | 2024 | New project, no legacy. Leptos 0.8 is compatible (editions are per-crate) |
| Toolchain | Stable (1.85+) | Leptos 0.8 works on stable. No nightly risk for marginal ergonomics |
| Crate layout | Single crate | App scope fits one `Cargo.toml`. Workspace variant available if it outgrows this |
| Package name | `samete` | Matches database name and tracing filter |

### Linting & Code Quality

| Decision | Choice | Why |
|----------|--------|-----|
| `unsafe_code` | `forbid` | No reason to ever use unsafe in this app |
| `clippy::all` | `deny` | All standard lints are errors |
| `clippy::pedantic` | `deny` | Pedantic findings are errors, not warnings. No `#[allow]` without a comment |
| `must_use_candidate` | `allow` (crate-level) | Leptos components return views consumed by macros, not callers. Known false positive |
| Formatter | `cargo fmt` (defaults) | No `rustfmt.toml` — standard style, no bikeshedding |

### Dependencies

| Concern | Crate | Notes |
|---------|-------|-------|
| Full-stack framework | leptos 0.8 | SSR + hydration via feature gates |
| Web framework | axum 0.8 | Behind `ssr` feature |
| DB | sqlx 0.8 (postgres, macros, uuid, time) | Compile-time checked queries. Behind `ssr` feature |
| HTTP client | reqwest 0.13 | TurboSMS API. Behind `ssr` feature |
| Phone validation | phonenumber 0.3 | Behind `ssr` feature |
| Session tokens | rand 0.9 | `OsRng` for cryptographic randomness. Behind `ssr` feature |
| Hashing | sha2 0.10, blake2 0.10 | OTP/session hashing, CSRF. Behind `ssr` feature |
| Time | time 0.3 | Not chrono — simpler, no historical soundness issues, native sqlx support |
| Errors | thiserror 2, anyhow 1 | Typed errors + `.context()` |
| UUID | uuid 1 (v4, serde, js) | `js` feature needed for WASM UUID generation (no-op on native) |
| Tracing | tracing 0.1 | Shared between SSR and client |
| WASM | wasm-bindgen 0.2 | Behind `hydrate` feature |

Server-heavy deps (sqlx, reqwest, phonenumber, rand, blake2, sha2, anyhow, tower-http, tracing-subscriber) are all behind the `ssr` feature gate. They don't compile into the WASM bundle.

### Build Profiles

| Profile | Settings | Purpose |
|---------|----------|---------|
| `release` | `strip = true`, `lto = "thin"`, `panic = "abort"` | Server binary |
| `wasm-release` | inherits release + `opt-level = 'z'`, `lto = true`, `codegen-units = 1` | Smallest WASM bundle |

### Development Tooling

| Tool | Config file | Purpose |
|------|-------------|---------|
| just | `justfile` | Task runner: dev, test, clippy, build, db commands |
| bacon | `bacon.toml` | Continuous clippy watcher. Default job: `clippy-ssr`. Keys: `s`/`h`/`t` |
| pre-commit | `.pre-commit-config.yaml` | Git hooks: `cargo fmt`, `cargo check --features ssr`, `clippy --fix`, trailing whitespace, EOF fixer, YAML check, large file check |
| cargo-leptos | `[package.metadata.leptos]` in `Cargo.toml` | Dev server, WASM bundling, E2E orchestration |
| Playwright | `end2end/` | E2E tests. Runs via `cargo leptos end-to-end` |

### Docker & Deployment

| Decision | Choice | Why |
|----------|--------|-----|
| Build strategy | cargo-chef (3-stage) | Dependency layer caching. Code changes don't rebuild deps |
| Runtime image | `gcr.io/distroless/cc-debian12:nonroot` | Minimal attack surface, non-root |
| Deploy | Coolify auto-deploy on push | Self-hosted PaaS. Handles Traefik, TLS |
| Local dev DB | `docker-compose.yml` with Postgres 16 | `samete:samete@localhost/samete` |

### Architecture Decisions (from spec)

| Decision | Choice | Why |
|----------|--------|-----|
| Session model | Single opaque token, 90-day, DB-backed | Simplest correct. Revocable by construction — delete the row |
| Assignment algorithm | Hand-rolled backtracking DFS | No petgraph needed. N≤15 is trivially small |
| Phase enforcement | DB enum + Rust enum with transition methods | Correct by construction. Rules gathered, not scattered |
| Season concurrency | One active season at a time | "Current season" is unambiguous. Partial unique index enforces |
| SMS sending | Inline — server function awaits TurboSMS API | Simplest at 50 users. No background jobs |
| Admin | Same binary, `/admin/*` routes behind `is_admin` check | One deploy, one session system |
| Content guidelines | Hardcoded in app | Static text. No DB indirection for a single string |
| Time-triggered SMS | Organizer triggers all SMS from admin UI | No cron, no schedulers. App is purely request-response |
| SQL verification | Compile-time checked queries (`sqlx::query!()`) | Wrong column names, type mismatches become compiler errors |

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
