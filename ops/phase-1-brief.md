# Phase 1: Foundation — Agent Brief

## Read First

1. `spec/Implementation Plan.md` — from "## Phase 1: Foundation" through "## Phase 2"
2. `spec/Data Model.md` — **authoritative schema**. If the plan's migration SQL conflicts with Data Model.md, Data Model.md wins.

## Entry State

Bare Leptos 0.8 scaffold:
- `src/main.rs` — Axum server, basic Leptos route setup
- `src/lib.rs` — WASM hydrate entry point, declares app module
- `src/app.rs` — shell(), App component, placeholder HomePage

All crates declared in Cargo.toml. No migrations, no domain types, no modules beyond `app`.
Docker Postgres running (db=samete, user=samete, pass=samete, port=5432).

## Correction: Add `launched_at` to seasons table

The E2E tests require a two-step season flow: create (not yet visible to participants) then launch (enrollment opens). Add this column to the seasons migration:

```sql
launched_at TIMESTAMPTZ  -- NULL = created but not launched. Non-null = enrollment is open to participants.
```

This follows the Data Model's own philosophy: "nullable timestamps as one-way latches." Participant-facing queries for active seasons must check `launched_at IS NOT NULL`.

## Traps

- `types.rs` compiles on BOTH SSR and WASM. Use `#[cfg_attr(feature = "ssr", derive(sqlx::Type))]` — never bare `derive(sqlx::Type)`.
- `Phase` Display impl must output exact lowercase Postgres enum values: `"enrollment"`, `"preparation"`, `"assignment"`, `"delivery"`, `"complete"`, `"cancelled"`.
- `phonenumber` crate: check the actual API. Parse with default region "UA". The method signature may be `phonenumber::parse(None, raw)` or `phonenumber::parse(Some(Country::UA), raw)` — let the compiler guide you.
- `rand` crate: the plan uses `rand::random::<u128>` for CSRF secret. Check the actual rand version in Cargo.toml for the correct API (rand 0.8 vs 0.9 differ).
- The plan says `phone::normalize` has `todo!()` — you MUST implement it, not leave the todo. The plan's "Gate 10" allows it, but a later phase depends on it.

## Exit

Run every command in "Phase 1 Verification Gates" from the plan. All must exit 0.
