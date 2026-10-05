# d-ops — production/deploy readiness (HEAD 1a3c8a5, static read-only)

Binding (manifestos read): stop-yapping — I must not pad, I will emit tables + fragments. first-principles — I must not trust docs as evidence, I will cite file:line. kiss — I must not invent mitigations, I will name minimal fixes. dry — I must not restate facts across sections, I will cross-reference by finding ID.

## Run/Deploy

Declared (README.md "Deployment", Dockerfile): Coolify auto-deploy on push; cargo-chef multi-stage; distroless cc-debian12 nonroot; CMD `/app/samete`; `LEPTOS_SITE_ROOT=site`, `LEPTOS_SITE_ADDR=0.0.0.0:3000`, EXPOSE 3000. Postgres is NOT in deploy artifacts (compose is dev-DB only, docker-compose.yml:1-14) — Coolify DB provisioning undocumented.

| Check | Evidence | Verdict |
|---|---|---|
| Binary path | `target/release/samete` copied Dockerfile:14; package `samete` (Cargo.toml:2), `output-name="samete"` (Cargo.toml:111) | OK |
| Site root | cargo-leptos emits `target/site` (Cargo.toml:112); copied to /app/site Dockerfile:15; `LEPTOS_SITE_ROOT=site` rel. to WORKDIR /app | OK |
| pkg dir / assets | `site-pkg-dir="pkg"`, `assets-dir="public"` → fonts/logos under site/ (public/ tracked, git ls-files) | OK |
| Cargo.toml at runtime | `get_configuration(None)` (main.rs:109) reads Cargo.toml from cwd; copied Dockerfile:16, WORKDIR /app:17 | OK |
| Bind addr | env `LEPTOS_SITE_ADDR` overrides Cargo.toml `127.0.0.1:3000` (Cargo.toml:117) | OK |
| Migrations at runtime | `sqlx::migrate!()` embedded at compile time (db.rs:28), run at boot (main.rs:101); no migrations/ copy needed | OK |
| sqlx offline in image | no `SQLX_OFFLINE`/DATABASE_URL in Dockerfile; `.sqlx/` tracked (82 files), not ignored, not in .dockerignore. sqlx macros fall back to cache when DATABASE_URL unset | OK (unverified by build) |
| TLS in distroless | reqwest 0.13 → rustls; Cargo.lock has no openssl-sys/native-tls | OK |
| `LEPTOS_ENV` | unset in Dockerfile; Cargo.toml `env="DEV"` (Cargo.toml:122) ⇒ runtime Env::DEV. No `Env::DEV` consumers in leptos 0.8.17/leptos_axum src (grep). `AutoReload` gated on `LEPTOS_WATCH` env (leptos hydration/mod.rs:19), unset in image | MINOR (set `LEPTOS_ENV=PROD`) |
| Pre-compression | README/just `build` pre-compresses; Dockerfile runs plain `cargo leptos build --release` (Dockerfile:11), no brotli step. CompressionLayer compresses on the fly (main.rs:135) with `compression-br` | MINOR (CLAUDE.md notes SSR stalled under 14MB dev WASM; release 471KB so low risk) |
| cargo-chef layer | `cargo chef cook --release` (Dockerfile:9) builds with default features (`default=[]`, Cargo.toml:47) — not the `ssr` bin build cargo-leptos does; dep cache likely miss | MINOR (perf only) |
| `cargo install cargo-chef cargo-leptos` on `rust:1.91-slim` | slim has no pkg-config/libssl-dev; cargo-leptos TLS dep unverifiable statically; also downloads tailwind/wasm-opt/wasm-bindgen at build time (network) | MAJOR-unverifiable: first Coolify build is the only proof; no docker build run (forbidden) |
| Unpinned tooling | cargo-chef/cargo-leptos unpinned (Dockerfile:2) vs project on cargo-leptos 0.3.2 (codebase_state) | MINOR |
| Healthcheck / migrations race / backups | none declared; pool `acquire_timeout` 5s only (db.rs:15) | MINOR |
| Reverse proxy | Secure cookies require HTTPS at Traefik (Coolify provides per README) | OK (claimed) |
| Rust toolchain | README "1.85+"; image 1.91; edition 2024 | OK |

CI (`.github/workflows/ci.yml`) never builds the Dockerfile → deploy artifact is untested anywhere before Coolify. MAJOR.

## Env vars table

| Var | Read at | Default | .env.example | README |
|---|---|---|---|---|
| DATABASE_URL | config.rs:40 | none; boot panics (main.rs:95) | yes | yes |
| TURBOSMS_TOKEN | config.rs:45,48 | required unless dry-run; empty rejected | NO | yes |
| TURBOSMS_SENDER | config.rs:56,59 | same | NO | yes |
| SAMETE_SMS_DRY_RUN | config.rs:42 | false (only literal `true` enables) | yes (=true) | NO |
| SAMETE_TEST_MODE | auth.rs:77, auth.rs:121 | false | yes (=true) | NO |
| SAMETE_LOG_POOL | main.rs:139 | off | no | no |
| LEPTOS_SITE_ROOT / LEPTOS_SITE_ADDR | leptos_config (get_configuration, main.rs:109) | Cargo.toml values | no | no (set in Dockerfile) |
| RUST_LOG (EnvFilter) | main.rs:88 | `samete=info,tower_http=info` | no | no |
| CSRF_SECRET | **nowhere** (grep `env::var`: only the 9 above) | — | no | README claims it exists: "generated at startup if absent" — FALSE. config.rs:6 comment says SameSite=Strict is the mitigation |

Findings: README documents a nonexistent var (CSRF_SECRET) and omits SAMETE_TEST_MODE / SAMETE_SMS_DRY_RUN — the two most dangerous flags. `.env.example` ships both ON and omits TURBOSMS_*; `.dockerignore:4` excludes `.env*` so it won't enter the image (OK).

## Security table

Cookie flags: `set_cookie_header` login.rs:481-482 → `HttpOnly; Secure; SameSite=Strict; Max-Age; Path=/` for session (90d, login.rs:180,458) and pending_phone (300s, login.rs:214). OK.
Session: 32 random bytes base64 (auth.rs:~158), stored as SHA-256 hash, 90d expiry, deleted on expiry-read and logout, `current_user` rechecks status=Active (auth.rs:~262). Expired rows never swept except on access (MINOR).
OTP: 6-digit `rand::rng()` (auth.rs:81), SHA-256 unsalted (10^6 space → trivially reversible if DB leaks; 10-min TTL mitigates; MINOR), 10-min expiry (auth.rs:100), 3 attempts/code (auth.rs:~190, login.rs:~252), constant-time compare (auth.rs:44), limit 1/60s + 5/h per phone (auth.rs:134-170). No IP dimension (deferred item, still open).

| Server fn | Guard | file:line | Verdict |
|---|---|---|---|
| generate_assignments_action | require_admin | admin/assignments.rs:246 | OK |
| swap_assignment | require_admin | admin/assignments.rs:328 | OK |
| get_assignment_preview | require_admin | admin/assignments.rs:423 | OK |
| create_season | require_admin | admin/season.rs:30 | OK |
| launch_season | require_admin | admin/season.rs:110 | OK |
| advance_season | require_admin | admin/season.rs:152 | OK |
| cancel_season | require_admin | admin/season.rs:191 | OK |
| get_admin_state | require_admin | admin/state.rs:129 | OK |
| list_participants | require_admin | admin/participants.rs:38 | OK |
| deactivate_participant | require_admin | admin/participants.rs:79 | OK |
| generate_invite_code | require_admin | admin/invite_codes.rs:90 | OK |
| list_invite_codes | require_admin | admin/invite_codes.rs:144 | OK |
| revoke_invite_code | require_admin | admin/invite_codes.rs:199 | OK |
| list_distributor_options | require_admin | admin/invite_codes.rs:269 | OK |
| send_season_open_sms | require_admin | admin/sms.rs:163 | OK |
| send_assignment_sms | require_admin | admin/sms.rs:246 | OK |
| send_confirm_nudge_sms | require_admin | admin/sms.rs:335 | OK |
| send_receipt_nudge_sms | require_admin | admin/sms.rs:433 | OK |
| get_current_user | require_auth (None on fail) | app.rs:30 | OK |
| complete_onboarding | require_auth | pages/onboarding.rs:23 | OK |
| get_home_state | require_auth | pages/home.rs:285 | OK |
| enroll_in_season | require_auth | pages/home.rs:381 | OK |
| confirm_ready | require_auth | pages/home.rs:504 | OK |
| confirm_receipt | require_auth | pages/home.rs:563 | OK |
| request_otp | public (by design) | pages/login.rs:39 | per-phone limit only; SMS-pumping + enumeration oracle (F3, F4) |
| verify_otp_code | public (by design) | pages/login.rs:126 | OK (3 attempts/code, 5 codes/h) |
| validate_invite_code | gated only by forgeable `pending_phone` cookie | pages/login.rs:313-326 | **F1** |
| register_with_code | gated only by forgeable `pending_phone` cookie | pages/login.rs:382-395 | **F1** |
| check_pending_registration | public, read-only | pages/login.rs:507 | OK |
| logout | cookie-scoped | pages/login.rs:517 | OK |

All 31 `#[server]` sites accounted (grep count 31; table = 18 admin + 6 auth'd + 7 public/cookie). Authorization is uniform: `require_admin` = `require_auth` + `role == Admin` (auth.rs:~418).

Test-mode / dry-run leakage:
- `SAMETE_TEST_MODE=true` ⇒ every OTP is `000000` (auth.rs:77) AND rate limit skipped (auth.rs:121). In prod this = anyone logs in as any existing phone incl. admin. No startup guard, no refuse-to-boot, no log banner. Only protection = env discipline; `.env.example` sets it true; README doesn't mention it. (F2)
- `SAMETE_SMS_DRY_RUN=true` ⇒ TurboSMS creds not required and no SMS sent; also logs full phone+message **including OTP code** at info (sms.rs:41-45). Prod misconfig = silent no-SMS (and OTP in logs). Boot logs nothing about mode. (F2)
- Test seed `seed/test_admin.sql` fixed admin +380670000001 not run by app; only `just db-seed`.

## Data

| Item | Evidence | Verdict |
|---|---|---|
| Migration order | 5 files, timestamp-prefixed: 20260314{1,2,3}, 20260624, 20260712 | OK |
| Destructive | grep `drop|truncate|delete` in migrations: only `ON DELETE CASCADE/RESTRICT/SET NULL` FKs; `ALTER TABLE … ADD COLUMN` (nullable) ×2; indexes ×3 (non-CONCURRENT, tiny tables) | OK |
| Applied automatically | boot-time `migrate!().run` (main.rs:101-103) — failure panics = fail-fast | OK |
| Cascade risk | `users` delete cascades to sessions/enrollments/assignments (create_tables.sql:12,20,49,50); app only deactivates (status) | OK |
| First admin in prod | **No path.** `INSERT INTO users` appears only in register_with_code (role defaults participant, create_tables.sql users.role DEFAULT 'participant') and seed/test_admin.sql (hard-coded test phone). No CLI, env var, or doc. Only route: manual SQL `UPDATE users SET role='admin'` after self-registering — but registration needs an invite code which needs an admin (generate_invite_code requires admin). Circular ⇒ prod needs manual `INSERT` of an admin row. Undocumented in README/justfile (grep bootstrap/first admin: no hits outside login.rs comment). (F5) | BLOCKER (undocumented operational prerequisite) |
| Backups | none declared | MINOR |

## Deferred re-verify (deferred_items.md vs source)

| Item | Status now | Launch impact |
|---|---|---|
| Leptos SSR reactive-disposal panic / tower_http 500s | Cannot verify statically (runtime). No fix commit in log; `grep unwrap` n/a. Treat open | MAJOR-unverifiable: intermittent 500s possible |
| SubagentStart oath hook | Out of repo; not verifiable here | none for launch |
| IP-based OTP rate-limit | OPEN: `grep ConnectInfo|x-forwarded|remote_addr src` = 0 hits; limits phone-keyed only (auth.rs:134-170) | MAJOR: SMS-pumping cost vector (F3) |
| Geometric visual assertions | OPEN: `scrollWidth|innerWidth|getBoundingClientRect` = 0 in all 3 specs | none for ops |
| `cfg(any(ssr,test))` WHY comment | OPEN (now 12 sites, item says 10; phone.rs:14 has none) | none; stale count |
| Orphan DB `samete_ssr_debug2` | Not checkable without DB access | none (dev DB) |
| cohort-seed bare ON CONFLICT | OPEN: 5 clauses (cohort-seed.sql:32,48,63,74,90) | none |
| create-form unreachable after cancel | OPEN: get_admin_state `ORDER BY created_at DESC LIMIT 1`, no phase filter (admin/state.rs:~145-150) | MINOR: after a cancelled season admin cannot create a new one via UI — **verify product intent; may block season 2** |
| `implementation_plan.md` keep/archive | **STALE — file does not exist** (`ls orchestration_log/reference/` = codebase_state, conventions, deferred_items). Delete this entry | none |

Not in deferred list but verified open: README CSRF_SECRET, forgeable pending_phone, no admin bootstrap.

## Findings (severity-ranked)

| ID | Sev | Finding | Evidence | Fix (minimal) |
|---|---|---|---|---|
| F1 | BLOCKER | Phone-ownership check bypass: `pending_phone` cookie holds the raw phone, unsigned (login.rs:214; read by prefix-strip login.rs:~287). HttpOnly does not stop a client crafting `Cookie: pending_phone=+380…` via curl. `validate_invite_code` (login.rs:325) and `register_with_code` (login.rs:391) trust it ⇒ OTP step skipped, any phone number registerable with just an invite code; also invite-code guessing is unthrottled (200-word list, ~40k combos, invite_codes.rs:268) so codes can be enumerated | login.rs:214,287,325,391 | Sign cookie (HMAC with a startup secret) or store verified-phone server-side keyed by random token; rate-limit validate_invite_code |
| F2 | BLOCKER | Production-fatal flags have no guard/docs: TEST_MODE ⇒ OTP 000000 for everyone incl. admin (auth.rs:77,121); `.env.example` enables it; README silent; no boot-time warning/refusal | auth.rs:77,121; .env.example:2-3; README env table | Document; refuse boot or log loud ERROR when TEST_MODE set and not E2E; add deploy checklist |
| F5 | BLOCKER | No documented/implemented first-admin bootstrap; invite flow circular | see Data | One-line SQL in README, or `ADMIN_PHONE` env bootstrap |
| F3 | MAJOR | Unauthenticated `request_otp` sends real SMS to any Ukrainian number; per-phone only limits ⇒ SMS-pumping cost attack | login.rs:39-100, auth.rs:134-170 | IP/global throttle at Traefik or middleware |
| F4 | MAJOR | `request_otp` returns `NewAccount` vs `AccountExists` ⇒ phone-registration oracle despite doc comment claiming no leakage (rate-limited path masks as AccountExists, success path does not) | login.rs:63-70 | Accept (small private club) or unify outcome |
| F6 | MAJOR | Dockerfile never built in CI; cargo-leptos install on slim image (no pkg-config/libssl) unverified; first Coolify deploy is first test | Dockerfile:2; ci.yml (no docker step) | Add CI `docker build`, or install `pkg-config libssl-dev` defensively |
| F7 | MAJOR | README env table wrong: documents nonexistent CSRF_SECRET, omits SAMETE_* | README Env table vs grep | Fix table |
| F8 | MAJOR | Dry-run logs OTP codes + phones at info (sms.rs:41); if DRY_RUN left on in prod, no SMS is sent and codes sit in logs | sms.rs:41-45 | covered by F2 guard |
| F9 | MINOR | `LEPTOS_ENV` unset ⇒ Env::DEV in prod (no current consumer) | Cargo.toml:122 | `ENV LEPTOS_ENV=PROD` |
| F10 | MINOR | Sessions/OTP unswept except piggyback; OTP SHA-256 unsalted; no HEALTHCHECK/backups/security headers; chef cook no-op for ssr; unpinned tool versions; no pre-compression in image | auth.rs:100; Dockerfile | defer |
| F11 | MINOR | create-season UI unreachable once any season row exists (incl. cancelled/complete) — confirm intent for season 2 | admin/state.rs | product call |
| F12 | MINOR | deferred_items.md has stale `implementation_plan.md` entry | ls reference/ | delete entry |
