# Codebase State

Last updated: 2026-04-29

## Module Inventory

| Module | Path | Purpose | Tests | Status |
|--------|------|---------|-------|--------|
| App shell | `src/app.rs` | Routes, guards, current_user Resource | E2E | Stable |
| Auth | `src/auth.rs` | Session, OTP, require_auth | E2E | Stable |
| Login | `src/pages/login.rs` | OTP flow, verify, logout; self-registration with invite code for new phones | E2E | Stable |
| Onboarding | `src/pages/onboarding.rs` | Nova Poshta address | E2E | Stable |
| Home | `src/pages/home.rs` | Participant dashboard, season states | E2E | Stable |
| Admin: Dashboard | `src/admin/dashboard.rs` | Overview stats | E2E | Stable |
| Admin: Season | `src/admin/season.rs` | Create, launch, advance, cancel | E2E | Stable |
| Admin: Participants | `src/admin/participants.rs` | Deactivate (register form replaced by invite codes) | E2E | Stable |
| Admin: Invite Codes | `src/admin/invite_codes.rs` | Generate, list, revoke invite codes | E2E | Stable |
| Invite Codes | `src/invite_codes.rs` | Word list, code generation | Unit | Stable |
| Admin: Assignments | `src/admin/assignments.rs` | Generate, swap, release | E2E | Stable |
| Admin: SMS | `src/admin/sms.rs` | Trigger SMS notifications | E2E | Stable |
| DB | `src/db.rs` | Pool creation, migrations | — | Stable (defaults) |
| Config | `src/config.rs` | Env-based config | — | Stable |
| Types | `src/types.rs` | Domain types, enums | — | Stable |
| Phone | `src/phone.rs` | E.164 normalization | Unit | Stable |

## E2E Test Suite

- **Total:** 75 tests across 3 serial blocks
- **Pass rate:** 75/75 — CI-verified stable (3 consecutive green runs as of 2026-04-29)
- **Runtime:** 18.2s (release, CI)
- **Structure:** Main lifecycle chain (68 tests, includes invite code stories 1.1, 1.5, 1.6) + Account Management (5) + Session Management (2)
- **Fixture:** `cached-context.ts` caches WASM/JS/CSS/fonts across tests
- **Pre-compression:** `precompress-and-test.sh` runs before every E2E

## WASM

- Dev: 14MB raw, 1.9MB brotli (pre-compressed)
- Release: 1.87MB raw, 471KB brotli
- Profile: `opt-level = 'z'`, `lto = true`, `codegen-units = 1`
- wasm-opt: automatic via cargo-leptos 0.3.2 (`-Oz`)
- Optimization floor reached. Remaining gains: `build-std` (nightly) or `--split` (code splitting).

## Infrastructure

- Postgres 16 in Docker (`docker-compose.yml`)
- cargo-leptos 0.3.2
- Leptos 0.8.17, Axum 0.8.8
- macOS linker: classic (`-Wl,-ld_classic`) due to Apple ld assertion bug with thin LTO
- GitHub Actions CI: `.github/workflows/ci.yml` — `check` job + `e2e` job with Postgres service

## Known Limitations

1. Leptos SSR has no per-Resource timeout (router-level middleware was incompatible with Suspense streaming)
2. `leptos_config` pulls `regex` into WASM dependency tree (LTO eliminates it, but compilation is slower)
3. Docker Postgres adds latency vs native
4. `CompressionLayer` re-compresses SSR HTML on the fly (small, fast — not a bottleneck)
