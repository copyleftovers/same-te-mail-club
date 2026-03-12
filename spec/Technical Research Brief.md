Technical research brief for the mail club app. Compiled 2026-03-11.

Captures what we investigated, what we found, what we recommend and why, and what remains open. The research is the expensive part — recommendations can be overridden without losing the underlying evidence.

---

## 1. SMS Provider for OTP Delivery in Ukraine

### Question

Which SMS provider should deliver OTP codes to Ukrainian mobile numbers (Kyivstar, Vodafone, lifecell)? Volume: 20-100 SMS/month. Budget: minimal (Добробіт covers it).

### Ukraine regulatory context

- **Sender ID (alpha-name) registration is required.** Unregistered alpha names get replaced with a generic sender. Registration takes up to 3 weeks with operators.
- **Short codes not supported** for A2P messaging in Ukraine.
- **Two-way SMS not supported** via standard A2P channels.
- **Cyrillic doubles encoding cost** — UCS-2 limits segments to 70 chars instead of 160. An OTP message in Ukrainian ("Ваш код: 123456") still fits in 1 segment, so no practical cost difference.
- **Mobile Number Portability** since May 2019 — operator prefix doesn't reliably indicate carrier. SMS providers handle routing.

### Options evaluated

| Provider | Price/SMS (USD) | Monthly (100 SMS) | Managed OTP? | Notes |
|---|---|---|---|---|
| **TurboSMS** | ~$0.025 | ~$2.50 | No | Direct UA carrier connections. They handle alpha-name registration. Pay per delivery only. REST API. Operating since 2007. |
| **AlphaSMS** | ~$0.025 | ~$2.50 | No | Comparable to TurboSMS. Less documentation. |
| **SMS Club** | ~$0.025 | ~$2.50 | No | OAuth 2.0 API. One of the first SMS services in Ukraine. Less transparent pricing. |
| **BSG** (Kyiv-based) | ~$0.011 | ~$1.10 | Yes (OTP API) | Also offers Telegram OTP channel (not relevant — SMS only). Lowest price found. Worth evaluating. |
| **Plivo** | $0.157 | $15.66 | No | International. Transparent pricing. No managed OTP. |
| **Twilio** | $0.175 | $17.47 | Yes (Verify) | Best docs and DX. Documented delivery delays to lifecell. Verify API handles OTP logic for you. |
| **Vonage** | ~$0.17 | ~$17.00 | Yes (Verify) | Comparable to Twilio. Pricing not transparent for Ukraine. |
| **Infobip** | $0.203 | $20.29 + €150/mo min | Yes | **Disqualified.** 150 EUR minimum commitment per active sender. |
| **MessageBird (Bird)** | ~$0.17 | ~$17.00 | No | **Deprioritized.** Pivoted to CRM. Opaque pricing — must contact sales. |

### Recommendation: TurboSMS

7x cheaper than Twilio. Direct carrier connections to all three Ukrainian operators means better deliverability for domestic traffic. They handle the alpha-name registration bureaucracy. Simple REST API: `POST https://api.turbosms.ua/message/send.json` with Bearer token. Pay per delivered message only — no monthly fees, no minimums.

No Rust SDK exists for any Ukrainian provider. Integration is ~30 lines of `reqwest` + `serde_json`.

BSG deserves a second look if cost optimization matters later — they're even cheaper and have a managed OTP API. But TurboSMS has 19 years of track record and broader documentation.

### Risks

- TurboSMS documentation is primarily in Ukrainian (English version exists but is thinner).
- Alpha-name registration takes up to 3 weeks — must be started well before launch.
- No managed OTP product — we build the OTP logic ourselves (generation, storage, expiry, rate limiting).
- SMS language: Ukrainian. This is the right choice for a Kyiv community, but means UCS-2 encoding.

### Explicitly rejected

- **Telegram for auth or notifications** — not now, not later. Per product spec: "No Telegram integration, no Telegram bots, no Telegram mini-apps."
- **Twilio** — 7x cost premium buys developer experience we don't need at this scale. The Verify API is genuinely useful but we can build OTP logic in ~50 lines of Rust.
- **Infobip** — €150/month minimum is disqualifying for <100 SMS/month.

---

## 2. Rust Stack and Framework Choices

### Question

What's the right Rust web stack for a mobile web app with SSR, phone-only auth, and Postgres?

### Ecosystem assessment

The Rust ecosystem for SMS-based phone auth is **immature compared to Node.js/Python/Go**. No turnkey "SMS auth" crate exists. No major SMS provider (Twilio, Vonage, MessageBird) ships an official Rust SDK. All Rust clients are community-maintained with low star counts.

The approach is to compose from well-maintained building blocks:

| Concern | Crate | Maturity |
|---|---|---|
| Web framework | axum 0.8 | Very mature. Best Tower middleware ecosystem. |
| Full-stack framework | leptos 0.8.x (current: 0.8.17) | Mature. SSR + hydration. Server functions. Active community. |
| DB access | sqlx 0.8 | Mature. Compile-time checked queries. Postgres-native. |
| HTTP client | reqwest | De facto standard. Async. |
| Phone validation | phonenumber (191 stars) | Port of Google's libphonenumber. Actively maintained. |
| OTP generation | rand with OsRng | Standard library. No special crate needed for random 6-digit codes. |
| JWT | jsonwebtoken | Well-maintained. |
| CSRF hashing | blake2 | Cryptographic hash. Used in Double Submit Cookie pattern. |
| Session token encoding | base64 + uuid | UUID v4 for token generation, base64 for cookie-safe encoding. |

### Key decision: Leptos server functions vs REST API

**Options:**
- **Server functions only** — Leptos server functions handle everything. Tightest Rust integration, least boilerplate. But: harder to add non-Leptos clients later.
- **REST API + Leptos frontend** — Separate Axum REST API. More work, but reusable by other clients.
- **Hybrid** — Server functions for participant flows, REST for admin/webhooks.

**Recommendation: Server functions only.** The product spec doesn't anticipate other clients. If that changes, server functions can be extracted into a separate API layer later without losing the implementations.

### Key decision: SSR vs CSR

**SSR + hydration recommended.** Server renders first paint (fast on Ukrainian mobile networks), then WASM hydrates for interactivity. Auth state available at render time via cookies — no flash of unauthenticated content. More complex Leptos setup but better UX.

CSR-only means blank page until WASM loads (~1-3s on mobile). Simpler but worse initial experience.

### Key decision: Database

**PostgreSQL, separate database on shared instance.** `CREATE DATABASE samete;` on a Postgres instance shared with other projects. Own connection pool, own `pg_dump` schedule. sqlx 0.8 with features: `runtime-tokio`, `postgres`, `macros`, `uuid`, `chrono`. Migrations via built-in `sqlx migrate`.

SQLite was considered (simpler, zero-config) but rejected — the user wants a shared Postgres instance for DX and unified backups across projects.

### Key decision: Starter template

**`leptos-rs/start-axum`** is the official Leptos 0.8 Axum SSR template. Single crate, `cdylib+rlib`, features `ssr`/`hydrate` mutually exclusive. A workspace variant (`start-axum-workspace`) exists for larger apps with cleaner server/frontend separation — worth considering if the codebase grows.

Nightly Rust is optional — only needed for the `nightly` feature flag (proc-macro ergonomics). Stable works.

---

## 3. Authentication Architecture

### Question

How to implement SMS OTP auth for a mobile web app with 15-50 users, long-lived sessions, and no other auth factors?

### 3a. OTP Flow

**Code length: 6 digits.** 4 digits = 10,000 combinations, too brute-forceable. 6 digits = 1,000,000. Industry standard (banks, Google, Twilio Verify). UX cost of two extra digits on a phone keyboard is negligible.

**Expiry: 5 minutes.** Short enough to limit interception window. Long enough for slow Ukrainian carrier delivery. Don't go below 2 minutes (carrier delays), don't go above 10 minutes.

**Storage:** SHA-256 hash of the OTP in Postgres. Never store plaintext. Only one valid code per phone number at a time — new request invalidates any previous unexpired code.

**Rate limiting per phone number:**
- 1 OTP request per 60 seconds (prevents SMS pumping)
- 5 OTP requests per hour
- 3 failed verification attempts per code (then invalidate and force new code)
- 10 failed verifications per hour (then lock that number for 1 hour)
- Global circuit breaker: ~50 SMS/hour max (anomaly detection at this scale)

**Allowlist-only:** Reject OTP requests for unregistered or deactivated phone numbers. Since the organizer pre-registers all users, this eliminates SMS pumping fraud entirely.

**Phone number format:** E.164 `+380XXXXXXXXX`. Regex: `^\+380\d{9}$`. Normalize on input: strip whitespace/hyphens/parentheses, replace leading `0` with `+380`, prepend `+` if starts with `380`. Store normalized, never raw.

### 3b. Session Management

**Delivery model:** Mobile web (no PWA, no native wrapper). Tokens live in browser cookies — no iOS Keychain or Android Keystore.

**Recommendation: Two-token pattern.**

- **Access token (JWT):** 1-hour lifetime. HMAC-SHA256 signed. Contains `user_id`, `issued_at`, `expires_at`. Validated locally, no DB lookup per request.
- **Refresh token (opaque):** 256-bit random. 90-day lifetime. Stored hashed (SHA-256) in Postgres alongside `user_id`, `created_at`, `expires_at`. Rotated on each use (issue new, invalidate old). If a token is reused after rotation, it signals theft — invalidate all sessions for that user.
- **Both delivered as HttpOnly + Secure + SameSite=Strict cookies.** Immune to XSS token theft. Browser sends them automatically.

**Why not pure JWT with long expiry?** A 90-day JWT cannot be revoked. If a device is lost, there's no way to kill the session.

**Why not pure opaque tokens (no JWT)?** Also valid at this scale — every API call would hit the DB, but with 15-50 users that's trivially cheap. If simplicity is preferred over architecture, a single opaque session token with 90-day expiry in DB is the simplest correct solution. Just needs revocability.

**Session lifetime: 90 days.** Active users effectively never re-authenticate via SMS — each successful token refresh resets the 90-day clock. SMS OTP only fires on: first login, session expiry (90 days of inactivity), explicit logout, cookie loss (browser clearing data, switching browsers).

**Expected steady-state SMS volume:** ~17 SMS/month for 50 users (~4 re-auths/year each). Cost: ~$0.42/month.

**Alternative considered: 30-day sessions.** More SMS (~600/year vs ~200), safer against long-lived stolen cookies. Rejected because the threat model is low (small private community, organizer-registered users).

### 3c. CSRF Protection

**Double Submit Cookie pattern** (from OWASP, validated against `Indrazar/auth-sessions-example` reference implementation):

1. On form load: server generates random value, sets it as `__Host-csrf` HttpOnly cookie, returns `Blake2s256(cookie + server_secret)` as hidden form field.
2. On submit: server reads the cookie, recomputes the hash, compares to submitted field.
3. `__Host-` cookie prefix enforces Secure + Path=/ at browser level.
4. Server secret is a u128 generated once at startup, held in app state.

SameSite=Strict on the session cookie adds defense-in-depth but is not sufficient alone for state-mutating POST requests.

### 3d. Security Considerations

**SIM swap:** Real but low-prevalence threat in Ukraine. Ukrainian carriers require in-person ID for SIM replacement (enforcement varies, wartime may relax processes). At this scale and with 90-day sessions, the attack window is tiny — an attacker would need to time a SIM swap with the exact moment a user re-authenticates. Low priority.

**OTP brute force:** 6-digit code + 3-attempt limit = 3/1,000,000 = 0.0003% guess probability per code. With the per-number hourly lockout, sustained brute force is impossible.

**Session token theft via DB compromise:** Session tokens stored hashed — if DB is compromised, raw tokens are not exposed. Learned from `auth-sessions-example` anti-pattern of storing tokens unhashed.

---

## 4. Leptos-Specific Patterns

### Question

How do auth sessions, DB access, and protected routes work in a Leptos SSR app? What can we learn from existing implementations?

### Reference repos analyzed

**`Indrazar/auth-sessions-example`** (29 stars, Leptos 0.7-beta7, SQLite)
- Hand-rolled session system — no Tower session middleware. Manual cookie read/write via `use_context::<Parts>()` and `expect_context::<ResponseOptions>()`.
- CSRF: Double Submit Cookie with Blake2s256 + server secret. `__Host-csrf` cookie.
- Session cookie: `SESSIONID=...; Secure; SameSite=Lax; HttpOnly; Path=/`
- Token generation: UUID v4 → base64url encoding (~22 chars).
- Contexts injected via `leptos_routes_with_context` closure in `main.rs`. Pool and server vars provided to every request.
- Protected routes: component-level `<Show when=is_not_logged_in>` + `<Redirect/>` with `ssr=SsrMode::Async`. Per-server-function check via `validate_session()`.
- Auth state in SSR: `get_user_data()` server fn → `Resource::new()` in App → resolved during SSR → serialized into HTML → hydrated client-side. No separate API call on initial load.

**Anti-patterns found (do NOT copy):**
- Session tokens stored **unhashed** in DB.
- `SameSite=Lax` instead of `Strict` on session cookie. Lax sends cookies on top-level GET navigations from other sites.
- **No rate limiting** on login attempts. Critical gap for OTP.
- Typo in table name: `active_sesssions` (three s's) — consistent throughout, but don't replicate.
- Logout has no CSRF protection (low severity — forced logout is an annoyance, not a breach).
- `Expires` header without `Max-Age` (Max-Age takes precedence in modern browsers and is more reliable).

**What IS directly reusable (with fixes):**
- `cookies.rs`: session cookie creation/destruction, `validate_session()` flow.
- Session DB functions: `associate_session`, `drop_session`, `validate_token`.
- Context injection boilerplate in `main.rs`.
- CSRF component (`CSRFField`) + `generate_csrf`/`validate_csrf`.
- `AppState` / `ServerVars` pattern.
- The entire architecture of hand-rolled sessions — simpler and more controllable than tower-sessions.

**`leptos-rs/start-axum`** (official template, Leptos 0.8.x)
- Single crate, `cdylib+rlib`. Features `ssr`/`hydrate` mutually exclusive at compile time.
- `main.rs`: `leptos_routes_with_context` for injecting pool/config. `generate_route_list(App)` walks the component tree to discover routes.
- Server functions auto-registered — no manual route setup. `leptos_routes()` calls `server_fn::axum::server_fn_paths()` internally.
- Auto-provided context in server fns: `use_context::<Parts>()` for request, `expect_context::<ResponseOptions>()` for response mutation.
- `leptos_axum::extract()` for typed Axum extractors inside server fns.
- `shell()` function wraps `<App/>` in `<!DOCTYPE html>` with `<HydrationScripts>` and `<AutoReload>`.
- `hydrate_body(App)` in `lib.rs` — walks server-rendered DOM and attaches reactive graph. No re-render.
- No Dockerfile. Manual deploy: binary + `target/site/` + env vars (`LEPTOS_SITE_ROOT`, `LEPTOS_SITE_ADDR`).
- Dev: `cargo leptos watch`. Prod: `cargo leptos build --release`. WASM profile: `opt-level='z'`, LTO, `panic="abort"`.

### Recommendation: integration pattern

```
main.rs:
  PgPool::connect() → sqlx::migrate!() →
  leptos_routes_with_context(provide_context(pool)) →
  .layer(CompressionLayer) →
  axum::serve()

Any server function:
  use_context::<PgPool>()     → DB access
  use_context::<Parts>()      → read cookies from request
  expect_context::<ResponseOptions>() → set cookies on response
  leptos_axum::extract()      → typed Axum extractors

Auth guard on routes:
  Component-level: <Show when=is_not_logged_in><Redirect/></Show>
  Server fn level: validate_session() at top of each protected fn
```

### Dockerfile (to write)

No official Dockerfile exists. Canonical pattern:

```
Stage 1 (builder):
  rustlang/rust:nightly
  cargo install cargo-leptos
  rustup target add wasm32-unknown-unknown
  cargo leptos build --release

Stage 2 (runtime):
  debian:bookworm-slim
  COPY binary + target/site/
  ENV LEPTOS_SITE_ROOT=site
  EXPOSE 3000
```

---

## 5. Infrastructure

### Question

How to host, deploy, and operate a Leptos SSR app with Postgres on a budget?

### Hosting: VPS with Coolify

Self-hosted PaaS on a VPS. Coolify handles Docker deployments, reverse proxy (Traefik), Let's Encrypt TLS auto-renewal. Postgres runs on the same VPS as a separate Coolify service.

### CI/CD: Dockerfile + Coolify auto-deploy

Multi-stage Dockerfile: `cargo-leptos` build stage compiles both server binary and WASM client. Minimal Debian runtime stage. Coolify watches the repo and auto-builds on push. No external CI needed.

Alternative considered: GitHub Actions builds the image, pushes to GHCR, Coolify pulls. Faster deploys (no build on VPS) but more CI configuration. Deferred unless VPS build times become painful.

### Database isolation

Separate database (`CREATE DATABASE samete;`) on the shared Postgres instance. Own connection pool, own `pg_dump` schedule. Not a separate schema — separate database gives cleaner backup/restore and prevents migration conflicts across projects.

### Admin interface

Same Leptos app, admin routes behind a role check. `is_admin` boolean in users table, manually set in DB on first deploy. Same SMS auth as participants.

Alternative considered: CLI tool for admin actions. Rejected because the organizer needs visual flows (assignment graph display, confirmed count before proceeding, override UI).

### SMS delivery failure handling

Silent fail + log. Show "SMS not sent, try again" to user. Organizer monitors logs/balance manually. At 20-100 SMS/month with a 500+ UAH top-up (~500 SMS), balance exhaustion is unlikely.

Alternative considered: alert organizer on failure (email, Telegram bot). Adds a dependency. Deferred unless failures become a pattern.

---

## 6. Schema Implications from Product Spec

These observations come from reading the product spec and affect the data model, not the architecture:

- **Name is legal/government ID name** — required for Nova Poshta parcel pickup. Single `name` field, not split first/last.
- **Phone number in assignment data** — sender sees recipient's name + phone + Nova Poshta branch. Phone needed because Nova Poshta sends the recipient a pickup SMS. Phone sharing is part of the trust contract of participation.
- **Receive-confirm free-text note** — nullable text field. Visible to organizer only. "Anything the organizer should know?" — surfaces edge cases without complicating the primary flow.
- **Account deactivation** — `is_active` boolean (or `deactivated_at` timestamp) on users table. Deactivated accounts cannot sign in, cannot enroll, are excluded from season-open SMS. Re-activation = new account created by organizer.
- **Content guidelines** — displayed during enrollment. Static text, not per-season. Storage TBD (hardcoded in app vs DB/config).
- **Minimum cohort size N ≥ 3** — algorithm must handle small cycles. At N < ~6, sender anonymity degrades (process of elimination). Organizer decides viability.
- **Assignment history retained across seasons** — past pairings feed the social-awareness algorithm. Need a persistent pairing table that survives season boundaries.
- **Data retention** — retain until participant requests deletion. Organizer handles manually in season 1.

---

## 7. Related Projects

### Community/ritual intersection

No existing project combines algorithmic matching + physical mail + seasonal cohorts + in-person meetups. The concept is novel. Closest analogues:

- **arcanis/secretsanta** (373 stars) — Secret Santa platform with Hamiltonian matching + exclusion constraints. Best overlap on matching logic.
- **Slowly** (9M users, closed-source) — pen pal matching with simulated postal delay. Proves stranger-matching for authentic connection works at scale.
- **Mail Art community projects** (MIT, Creativity Explored) — the cultural precedent. Entirely manual, no tech infrastructure.

### Matching algorithm intersection

At N=11-15, the matching problem is trivially solvable despite NP-completeness of Hamiltonian cycle in general.

- **JolonB/Secret-Santa** (Python) — backtracking DFS for constrained Hamiltonian cycles with exclusion pairs. This is essentially our algorithm. Brute force with constraint pruning finishes instantly at N≤15.
- **petgraph** (3,789 stars) — the Rust graph library. Foundation for implementing the cycle generator.
- **NetworkX coalition formation** (PR #6364) — for splitting pools >15 into optimal subgroups before cycle generation. Reference the algorithm, implement in Rust.

### Leptos/Rust stack intersection

- **auth-sessions-example** (29 stars) — analyzed in detail above. Session/CSRF patterns.
- **realworld-leptos** (73 stars) — full production app with SSR + JWT + sqlx + Playwright E2E.
- **auth_leptos** (4 stars) — Docker + Postgres + Argon2 + TOTP 2FA. Best reference for multi-service Docker setup.

---

## 8. Open Questions

- Domain name
- Exact Coolify deployment configuration
- Single crate vs workspace (`start-axum` vs `start-axum-workspace`)
- Stable vs nightly Rust toolchain
- Monitoring / alerting setup (if any beyond manual log checks)
- Backup schedule for Postgres
- Content guidelines: hardcoded in app vs stored in DB/config
- Whether season 1 runs fully manual (spreadsheet + personal SMS) or with the app — decision pending per spec
- BSG as alternative SMS provider — cheaper than TurboSMS, has managed OTP API, but less track record
- Pure opaque session token (simpler) vs JWT+refresh (more architectural) — both viable at this scale
