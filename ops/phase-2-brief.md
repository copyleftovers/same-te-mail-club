# Phase 2: Auth — Agent Brief

## Read First

1. `spec/Implementation Plan.md` — from "## Phase 2: Auth" through "## Phase 3"
2. `spec/Data Model.md` — sessions, otp_codes, users, delivery_addresses tables
3. `spec/User Stories.md` — Stories 1.1, 1.2, 1.3 (acceptance criteria for test derivation)

## Entry State

Phase 1 complete. These files exist:
- `src/types.rs` — Phase, UserRole, UserStatus, ReceiptStatus enums (compiles on SSR + WASM)
- `src/error.rs` — AppError enum (SSR-only)
- `src/config.rs` — Config struct (SSR-only)
- `src/phone.rs` — phone normalization, implemented (SSR-only)
- `src/db.rs` — pool creation + migration runner (SSR-only)
- `src/main.rs` — full server setup with PgPool and Config in Leptos context
- `src/lib.rs` — module declarations
- `migrations/` — all tables created, DB migrated
- `.sqlx/` — offline query data committed

Database is migrated and running. `cargo build --features ssr` succeeds. `cargo clippy --features ssr -- -D warnings` is clean.

## Key Design Decisions Already Made

- OTP codes are INSERTED, never upserted. Old rows enable rate limit counting via `COUNT(*)`.
- `request_otp` must NOT reveal whether a phone number is registered. Always return `Ok(())`.
- `SAMETE_TEST_MODE=true` env var → OTP is always `"000000"`. Check env var in `create_otp`.
- `SAMETE_SMS_DRY_RUN=true` env var → log SMS content via `tracing::info!`, don't call TurboSMS API.
- Onboarding writes to `delivery_addresses` table. Only the `onboarded = true` flag goes to `users`.
- Session cookie: `session={token}; HttpOnly; SameSite=Strict; Max-Age=7776000; Path=/`

## Traps

- Leptos 0.8 server functions: verify the exact `#[server]` macro syntax and how to access request parts (for cookies). Check if `use leptos_axum::extract` or `leptos::server_fn::extract` is the current API.
- Cookie setting: use `leptos_axum::ResponseOptions` to append `Set-Cookie` header. Don't try to return cookies in the body.
- `ServerFnError::new()` may not exist — the plan notes this. Check the actual type; `ServerFnError::ServerError(String)` is a common fallback.
- SHA-256 hashing: use the `sha2` crate (`use sha2::{Sha256, Digest}`). Check it's in Cargo.toml.
- Admin guard in `current_user`: check BOTH `status = Active` AND the role. But don't put role checks in page modules — admin checks live in `src/admin/` modules.
- Rate limiting has two tiers: max 1 OTP per 60 seconds, max 5 per hour. Both are DB COUNT queries, no in-memory state.

## E2E Tests

Target: `end2end/tests/mail_club.spec.ts` — the `"Epic 1: Join the Community"` describe block.
All tests in that block (stories 1.1, 1.2, 1.3) should pass after this phase.

## Exit

Run every command in "Phase 2 Verification Gates" from the plan.
CRITICAL: `cargo sqlx prepare --workspace` after all queries are in place.
