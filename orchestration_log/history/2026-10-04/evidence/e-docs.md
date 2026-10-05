# e-docs — self-declaration audit vs source (HEAD 1a3c8a5, static only, no builds)

Binding: stop-yapping (I must answer only; I will emit tables/fragments). first-principles (must verify from source; will cite file:line/counts). kiss (must keep report minimal; one table). dry (must not restate; findings reference table rows).

| # | Claim | Source doc:line | Verdict | Evidence |
|---|-------|-----------------|---------|----------|
| 1 | `just test`: 55 tests bare / 62 with `--features ssr` | CLAUDE.md Key Commands | STALE | `#[test]` count per file: assignment 22, admin/page 13, types 9, home 9, phone 8, auth 7, invite_codes 7 = 75. `auth` is `#[cfg(feature="ssr")] pub mod` (lib.rs) so bare = 68, ssr = 75. codebase_state "62→68" (FU-12) is also stale |
| 2 | `just test` = `cargo test` (CI too) | justfile `test`, ci.yml:52-53 | TRUE (but see F2) | CI runs bare `cargo test`, no ssr test step |
| 3 | `just clippy` = "Run clippy (SSR)" | CLAUDE.md; README "on all targets" | STALE | justfile `clippy`: ssr AND wasm32 hydrate (2 invocations). README wording is closer |
| 4 | `just prepare` = `cargo sqlx prepare --workspace -- --features ssr` | CLAUDE.md | TRUE | justfile `prepare` |
| 5 | Recipes dev/e2e/e2e-release/e2e-dev/e2e-single/e2e-rerun/build/serve/check/db-*/capture-isolated exist | CLAUDE.md, README | TRUE | justfile (all present). README table omits `capture-isolated`, `fmt`, `fmt-check` (MINOR) |
| 6 | E2E "119 tests (75 + 43 + 1)" | codebase_state.md E2E Suite | STALE | `grep -cE "\btest\("`: mail_club 75, visual-audit 42, cohort 1 = 118. Skips: 1 `test.skip` in visual-audit (home no-season, :467) + cohort file-level skip |
| 7 | "117/119 passed, 2 by-design skips" | codebase_state.md | UNVERIFIABLE-STATICALLY | needs run; total already off by 1 |
| 8 | Main chain + "Account Management" + "Session Management" serial blocks | CLAUDE.md, e2e README | TRUE | mail_club.spec.ts:72, 849, 900 |
| 9 | `navigationTimeout` 15s | CLAUDE.md (redundant nav), e2e README, mail_club_page.ts:136 comment | STALE | playwright.config.ts:38 `navigationTimeout: 30_000`; actionTimeout 10_000, expect 15_000 |
| 10 | Reporters list+html (README), json results.json (CLAUDE.md) | e2e README, CLAUDE.md | TRUE | playwright.config.ts:25-27 |
| 11 | No `webServer` block | e2e README | TRUE | config:9 comment; no block in grep |
| 12 | `waitForTimeout` banned everywhere | e2e README, conventions | STALE (documented exception) | capture-constants.ts:40 `paintSettle` uses it; sanctioned in comment :35. README Banned section has no exception |
| 13 | `waitForLoadState` zero calls | codebase_state.md | TRUE (code) | only comment refs in mail_club_page.ts:293. `waitForURL("/")` used at :101,:127 (no waitUntil) — OK |
| 14 | login asserts `not.toHaveURL(/\/login/)`; goHome skips goto on "/" | CLAUDE.md pitfalls | TRUE | mail_club_page.ts:80,93; :137-140 |
| 15 | capture-constants exports MOBILE/DESKTOP_VIEWPORT, LAYOUT_REFLOW_MS, ADMIN_PHONE, futureDeadline, paintSettle | CLAUDE.md | TRUE | capture-constants.ts:15-52 |
| 16 | POM method names in README tables | e2e README | TRUE | all listed methods present in mail_club_page.ts |
| 17 | `cached-context.ts`, `precompress-and-test.sh` exist | CLAUDE.md | TRUE | files present |
| 18 | Bridge `tests/seed.spec.ts`, `.playwright/project-config.md` | CLAUDE.md QA | UNVERIFIABLE-STATICALLY | gitignored (.gitignore:17); absent in clean tree |
| 19 | Module inventory (app/auth/login/onboarding/home/admin/*, assignment, error, hooks, i18n, sms, date_format, pages, db, config, types, phone, invite_codes, components) | codebase_state.md | TRUE | src/ listing; admin/{assignments,db_helpers,invite_codes,page,participants,season,sms,state}.rs all present. Omits components/* (stepper/toast/skeleton) from table (MINOR) |
| 20 | README project-structure tree | README | STALE (minor) | omits `invite_codes.rs`; admin/components internals unlisted |
| 21 | `Phase` enum, "All 6 phases complete" | dev-protocol.md | TRUE | types.rs:13 six variants Enrollment…Cancelled; all story-phase modules present |
| 22 | blake2 0.10 "OTP/session hashing, CSRF"; rand 0.9 | README Dependencies | FALSE (blake2) | Cargo.toml has no blake2 and no csrf code (`grep -rn "blake2\|csrf" Cargo.toml src` → 0). Hashing is sha2 + `subtle` (+ base64) — README omits subtle/base64/leptos-use/leptos_i18n |
| 23 | Versions: leptos 0.8 (0.8.17), axum 0.8.8, sqlx 0.8, reqwest 0.13, phonenumber 0.3, sha2 0.10, time 0.3, thiserror 2, uuid 1 | README, codebase_state | TRUE | Cargo.lock: leptos 0.8.17, axum 0.8.8, sqlx 0.8.6, reqwest 0.13.4, sha2 0.10.9, thiserror 2.0.18, uuid 1.22.0. Leptos 0.8.17 matches |
| 24 | uuid features "v4, serde, js" | README | STALE (minor) | Cargo.toml: uuid features v4,serde; `js` only via hydrate feature |
| 25 | Edition 2024, `unsafe_code=forbid`, clippy all+pedantic deny, release strip/thin LTO/panic abort, wasm-release opt-level z/lto true/cgu 1 | README, dev-protocol | TRUE | Cargo.toml lints + profiles (clippy lints use priority -1; dev-protocol snippet omits priority — cosmetic) |
| 26 | `must_use_candidate` allow crate-level | README | TRUE | lib.rs:2 (`allow(clippy::must_use_candidate)`) |
| 27 | WASM 471KB brotli / 1.87MB / 14MB dev; opt 'z' | CLAUDE.md, codebase_state | UNVERIFIABLE-STATICALLY | needs build. Profile settings TRUE (#25) |
| 28 | cargo-leptos 0.3.2, Tailwind v4.2.1 | codebase_state, frontend-protocol | UNVERIFIABLE-STATICALLY | ci.yml:104 `cargo binstall cargo-leptos` unpinned; no version in repo |
| 29 | Style layout: tailwind.css 4-line orchestrator; tokens.css; components.css; main.css empty | frontend-protocol | TRUE | tailwind.css 4 lines; main.css 1 line; tokens.css 171; components.css 1525 |
| 30 | tailwind.css "1626 lines"/"split when exceeded 1600" | codebase_state/frontend-protocol (history) | N/A (historical) | now 1525 in components.css |
| 31 | Palette: 6 raw tokens exact oklch | design-system.md | TRUE | tokens.css:7-12 identical |
| 32 | Semantic aliases + dark reassignments (surface, raised 0.22, text, muted 0.65, border 0.58, error 0.68, success 0.72, focus, panel-dark 0.27, step-idle 0.30) | design-system.md | TRUE | tokens.css:43-73, 147-169. Light `--color-panel-dark`/`--color-step-idle` fallbacks present (:72-73) |
| 33 | Badge tokens success .50/.16/160, amber .82/.16/85, info .70/.09/240, error .55/.22/25 | design-system.md | TRUE | tokens.css:23-30 |
| 34 | Focus light `oklch(0.58 0.15 250)` | design-system.md | TRUE | tokens.css:52 |
| 35 | Fonts: 3 woff2 in public/fonts; stacks; logo files | design-system.md | TRUE | public/ listing; tokens.css:32-33 |
| 36 | Density tokens participant 3/5/8, admin 1.5/3/5 | design-system.md | TRUE | tokens.css:75-77, 123-125 |
| 37 | Text tokens: page-title clamp(1.8rem,5vw,2.8rem), section 1.3rem, body 1.05rem, overline 0.8rem, secondary 0.875rem | design-system.md | TRUE | tokens.css:80-88; admin overrides section 1.1rem, page-title 1.75rem, admin max 64rem (:126-132) |
| 38 | Radii sm/md/lg/pill | design-system.md | TRUE | tokens.css:35-38 |
| 39 | Grain: opacity .04, z-index 1, no mix-blend-mode, hidden under reduced-motion | design-system, frontend-protocol | TRUE | components.css:91,93; `mix-blend` only in explanatory comment :94 |
| 40 | Banned: no `@apply`, no hex in style/, no `!important`, no testid in CSS | frontend-protocol, conventions | TRUE (1 documented exception) | grep: 0 @apply, 0 hex, 0 data-testid; 2 `!important` at components.css:112-113 inside reduced-motion block with exception comment (contradicts "banned", documented) |
| 41 | z-index documented layers (grain 1, dropdown 40, modal 50, confetti 60) | frontend-protocol | STALE/UNVERIFIED | only one `z-index: 1` declaration in style/; layer map comment at components.css:82. Dropdown/modal/confetti layers unused in CSS — aspirational text |
| 42 | No `format!()`-built class names | frontend-protocol | TRUE | grep `class=.*format!` → 0 |
| 43 | Every `type="submit"` has hydration gate | component-eval C3 | UNVERIFIABLE-STATICALLY (sampled TRUE) | 23 submit sites; sampled admin/page.rs:493 `disabled=move \|\| pending.get() \|\| !hydrated.get()`. Did not check all 23 |
| 44 | "All submit buttons have `aria-busy`"; bare `aria-invalid` (no `attr:` on native elements) | codebase_state 2026-06-22, conventions 2026-07-04 | CONFLICT (see F1) | 13 `attr:aria-busy=` on NATIVE `<button>` in admin/page.rs (e.g. :497, :675…). conventions.md says `attr:` on a native element emits literal attr named `attr:…`. `attr:aria-invalid` has 0 hits (fixed) |
| 45 | CI: fmt, clippy ssr+hydrate, cargo audit, cargo test, e2e release | codebase_state | TRUE | ci.yml:37-53 (+ audit step not mentioned in docs) |
| 46 | CI `SQLX_OFFLINE=true` + `.sqlx/` committed | dev-protocol, conventions | TRUE | ci.yml:11; `.sqlx` dir present |
| 47 | Migrations / seed | README | TRUE | 5 migrations incl. sms_idempotency, add_missing_indexes; seed/test_admin.sql |
| 48 | README setup: pre-commit installed | README | TRUE | .pre-commit-config.yaml (fmt, cargo-check, clippy, whitespace, eof, yaml, large-files) |
| 49 | Docs-referenced paths exist: spec/technical/{Architecture,Data Model,User Stories}.md, spec/product/*, archive/e2e-research.md, archive/spec/E2E Test Blueprint.md, guidance/ui-review-prompt.md, orchestration_log/reference/{conventions,codebase_state,deferred_items}.md | CLAUDE.md et al | TRUE | all present |
| 50 | deferred_items: `reference/implementation_plan.md` (22KB, April) needs keep/archive/delete | deferred_items.md:Decisions pending | STALE | file absent; reference/ holds only 3 files. `archive/spec/implementation-plan.md` + `Implementation Plan.md` exist — likely moved |
| 53 | dev-protocol "unit tests: OTP generation/hashing/verification/expiry/rate-limiting, session creation/validation" | dev-protocol.md "What to Test Where" | FALSE (aspirational) | auth.rs tests (7) cover only sha256_hex + constant_time_hash_eq. OTP expiry/rate limit/session are DB-bound, covered only by E2E |
| 54 | Leptos MCP tool names `mcp__plugin_leptos-mcp_leptos__*`; `~/leptos-mcp-server` | CLAUDE.md, leptos-idioms | UNVERIFIABLE-STATICALLY | environment-specific |
| 55 | `.manifestos.yaml` hooks inject oaths | conventions | UNVERIFIABLE-STATICALLY | plugin-side |

## Findings (severity-ranked)

**F1 — MAJOR (unverifiable, conflicting)** `attr:aria-busy` on native `<button>` ×13 (src/admin/page.rs:497,675,705,753,865,906,955,992,1102,1647…). conventions.md documents that `attr:`-prefixed attrs on native elements leak as literal `attr:…` attribute names. If true, the "all submit buttons have aria-busy" claim (codebase_state 2026-06-22) is FALSE and a11y busy state is dead. Needs rendered-HTML check (`grep 'aria-busy' ` on SSR output).

**F2 — MAJOR** CI and `just test` run bare `cargo test` (68 tests); the 7 auth.rs tests (ssr-gated module) never run in CI or `just check`. Docs say 55/62; reality 68/75. Fix docs and add `cargo test --features ssr` to CI/justfile.

**F3 — MAJOR (doc FALSE)** README Dependencies lists blake2 0.10 + "CSRF"; neither exists in Cargo.toml/src. Hashing is sha2 + subtle. Also omits subtle, base64, leptos-use, leptos_i18n.

**F4 — MINOR/MAJOR (doc FALSE, aspirational)** dev-protocol §What to Test Where promises unit tests for OTP expiry, rate limiting, session lifecycle; none exist (auth.rs tests = hash helpers only).

**F5 — STALE** E2E count: docs 119 (75+43+1), source 118 (75+42+1). Pass-rate line "117/119" unverifiable and inconsistent.

**F6 — STALE** navigationTimeout documented 15s (CLAUDE.md, e2e README, POM comment); config is 30_000.

**F7 — STALE** CLAUDE.md `just clippy` "(SSR)": runs SSR + wasm32 hydrate. README table fine; README omits `capture-isolated`, `fmt`, `fmt-check`.

**F8 — STALE** e2e README / conventions say `waitForTimeout` banned; `paintSettle` in capture-constants.ts:40 is a sanctioned exception not reflected in README. `!important` banned in frontend-protocol but 2 documented uses at components.css:112-113.

**F9 — STALE** frontend-protocol z-index layers (dropdown 40, modal 50, confetti 60) have no CSS declarations; only grain z-index 1 exists.

**F10 — STALE** deferred_items "Decisions pending" references reference/implementation_plan.md which no longer exists; delete the item. README tree omits invite_codes.rs; uuid `js` feature claim imprecise.

**F11 — UNVERIFIABLE** WASM sizes, cargo-leptos 0.3.2, Tailwind 4.2.1, E2E pass rates, submit-gate completeness (23 sites, 1 sampled), deferred #[cfg] site count.

Design-system token claims (palette, semantic, dark, badge, density, text, radii, grain, fonts): all TRUE vs tokens.css. Module inventory, Phase enum, "6 phases complete", CI shape, dependency versions: TRUE.
