# Phase 2 E2E Troubleshooting — Agent Brief

## Objective

Make ALL Epic 1 E2E tests pass: `end2end/tests/mail_club.spec.ts` → `"Epic 1: Join the Community"` block (stories 1.1, 1.2, 1.3).

## FIRST ACTION — Do This Before Anything Else

```bash
git stash pop
```

This restores 4 files with proven fixes from prior debugging sessions. These are NOT speculative — they are tested corrections. Build on them, do not revert them.

## Entry State

Phase 2 code is complete and compiles. The stash you just popped contains:
- `src/pages/login.rs` — hydration gate pattern + Memo-based reactivity
- `src/app.rs` — route tuple syntax fix
- `src/pages/onboarding.rs` — single-field form matching POM selector
- `src/admin/participants.rs` — hydration gate on register button

## Known Issues (all in the stash)

### 1. Hydration Race Condition
Leptos 0.8 forms with `on:submit` + `prevent_default()` submit via HTML default before WASM hydrates. Playwright's `.click()` fires immediately — if WASM hasn't loaded, the JS handler isn't attached, so the browser does a full-form POST (which fails silently).

**Fix pattern** (already applied in stash):
```rust
let (hydrated, set_hydrated) = signal(false);
Effect::new(move |_| { set_hydrated.set(true); });
// In view:
<button type="submit" disabled=move || !hydrated.get()>
```
Playwright's `.click()` waits for `disabled` to become `false`, naturally gating on hydration.

### 2. Route Tuple Syntax
`StaticSegment("admin/participants")` does NOT work in Leptos 0.8. Must use tuple:
```rust
<Route path=(StaticSegment("admin"), StaticSegment("participants")) view=.../>
```
Already fixed in stash.

### 3. Onboarding Form — Single Field
POM expects ONE label matching `/nova poshta|branch|відділення/i`. Implementation must use a single `branch` text field, not separate city/number fields. Already fixed in stash.

### 4. `uuid` `js` Feature — SSR Panic (NOT in stash, NOT yet fixed)
`Cargo.toml` has `uuid = { version = "1", features = ["v4", "serde", "js"] }`. The `js` feature enables `getrandom/js` which uses `js-sys` — this panics on native (SSR) targets with "cannot access imported statics on non-wasm targets."

**Fix**: Change `Cargo.toml`:
```toml
# In [dependencies]
uuid = { version = "1", features = ["v4", "serde"] }
```
Then in the `[features]` section, add `"uuid/js"` to the `hydrate` feature array so it's only active for WASM builds.

After this change, run `cargo sqlx prepare --workspace -- --features ssr` to regenerate `.sqlx/`.

## How to Run E2E Tests

The `cargo leptos end-to-end` command starts the server AND runs Playwright. Do NOT use `cargo run --features ssr` — it doesn't set `LEPTOS_OUTPUT_NAME`, causing empty WASM file paths.

```bash
# Kill any lingering servers
lsof -ti :3000 | xargs kill -9 2>/dev/null
lsof -ti :3001 | xargs kill -9 2>/dev/null

# Reset DB
DATABASE_URL="postgres://samete:samete@localhost:5432/samete" just db-reset

# Run E2E (builds server + WASM, starts server, runs playwright)
DATABASE_URL="postgres://samete:samete@localhost:5432/samete" \
TURBOSMS_TOKEN=test \
TURBOSMS_SENDER=test \
SAMETE_TEST_MODE=true \
SAMETE_SMS_DRY_RUN=true \
cargo leptos end-to-end
```

If `just db-reset` fails because of active connections:
```bash
psql "postgres://samete:samete@localhost:5432/postgres" -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'samete' AND pid <> pg_backend_pid();"
```

## Selector Contract

The POM (`end2end/tests/fixtures/mail_club_page.ts`) defines the exact selectors. Key ones for Epic 1:
- Login phone: `getByLabel(/phone/i)` → needs `<label>` with "phone" in text
- Login send button: `getByRole("button", { name: /send|submit|code/i })` → button text must match
- OTP code: `getByLabel(/code/i)` → needs `<label>` with "code" in text
- OTP verify button: `getByRole("button", { name: /verify|submit|sign/i })`
- Register button: `getByTestId("register-button")`
- Onboarding field: `getByLabel(/nova poshta|branch|відділення/i)`
- Onboarding save: `getByRole("button", { name: /save|submit|continue/i })`

## E2E Test Environment

- `SAMETE_TEST_MODE=true` → OTP is always "000000"
- `SAMETE_SMS_DRY_RUN=true` → SMS logged, not sent
- Admin user: seeded by `just db-seed` (check `justfile` for seed command)
- If there's no seed command, you need to create the admin user directly in the DB or via a migration

## Verification

Phase 2 is done when:
1. `cargo clippy --features ssr -- -D warnings` exits clean
2. `cargo test --features ssr` — all pass
3. `cargo leptos end-to-end` — Epic 1 tests pass (stories 1.1, 1.2, 1.3)

## Development Protocol

- **Correct By Construction**: Types first. Compiler-driven. Invalid states unrepresentable.
- **Simple Made Easy**: No entanglement. Clean parts over tangled fewer.
- Pedantic clippy is DENY level. Every finding fixed.
- Use LSP tool for diagnostics — they are BLOCKING.
- After any sqlx query changes: `cargo sqlx prepare --workspace -- --features ssr`

## Commit Style

One-line conventional commits. No `Co-Authored-By`. No AI attribution. Example:
```
fix(auth): resolve hydration race and uuid SSR panic
```
