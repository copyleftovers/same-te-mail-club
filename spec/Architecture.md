# Architecture: The Mail Club

Technical architecture for the mail club app. Companion to the Product Spec (WHAT), Personas (WHO), and User Stories (VALUE). This document covers HOW.

Written 2026-03-11. Supersedes the Technical Research Brief.

---

## Constitutional Constraints

This architecture is governed by two manifestos, active for the duration of design and implementation:

**Simple Made Easy** (after Rich Hickey) — Simple means untangled, not familiar. Optimize for the artifact (the running system), not the construct (the typing experience). More parts hanging cleanly beats fewer parts tangled together. Composition requires simple parts.

**Correct By Construction** (after Tris Oaten / Rust) — Make invalid states unrepresentable. Every compiler error is a bug you didn't ship. Pay upfront at compile time, not forever in production. Trust the type system.

**Where they amplify:** Both reject deferred consequences. Both distrust guardrails-as-strategy. Both favor explicit over implicit.

**Where they tension:** Correctness encourages richer type encodings. Simplicity warns that machinery can itself become complected. Resolution: type richness is warranted when it eliminates entanglement (enum replacing boolean flags). It is not warranted when it introduces entanglement (generic trait hierarchies braiding concerns).

---

## Foundational Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Target | App ships for season 1 | Build and ship before the first season runs |
| Session model | Opaque token, 90-day, DB-backed | Simplest correct. Revocable by construction — delete the row |
| Season concurrency | One active season at a time | "Current season" is unambiguous. Schema enforces this |
| Assignment algorithm | Hand-rolled backtracking DFS | Petgraph adds a dependency without subtracting logic. N≤15 is trivially small |
| Crate layout | Single crate (start-axum template) | App scope fits comfortably. One Cargo.toml, one compilation unit |
| Toolchain | Stable Rust, edition 2024 | Leptos 0.8 works on stable. No nightly risk for marginal ergonomics |
| Phase enforcement | DB enum + Rust enum with transition methods | Correct by construction at the domain level. Rules gathered, not scattered |
| Admin separation | `/admin/*` route tree, same binary | Module-level boundary. Organizer components import shared types but live in their own module tree |
| Time-triggered SMS | Organizer triggers all SMS batches from admin UI | No cron, no schedulers, no background processing. App is purely request-response |
| SMS sending | Inline — server function awaits TurboSMS API | Simplest at 50 users. User waits 1-2s for API response |
| Organizer account | Same account + `role = Admin` enum | One login, one session. Organizer enrolls in seasons like anyone else |
| SQL verification | Compile-time checked queries (`sqlx::query!()`) | Correct by construction — wrong column names, type mismatches, bad SQL become compiler errors |
| Time library | `time` crate (not chrono) | Simpler, no historical soundness issues, native sqlx support. New project, no legacy to carry |
| Content guidelines | Hardcoded in app | Static text, changes at most once per season. No DB/config indirection for a single string |

---

## Stack

### Rust Crates

| Concern | Crate | Version | Notes |
|---------|-------|---------|-------|
| Full-stack framework | leptos | 0.8.17+ | SSR + hydration. Server functions. Stable Rust |
| Leptos Axum integration | leptos_axum | 0.8.x | Context injection, server fn handling, SSR |
| Leptos meta tags | leptos_meta | 0.8.x | `<Title>`, `<Meta>` in SSR |
| Leptos router | leptos_router | 0.8.x | `<Route>`, `<ProtectedRoute>`, `SsrMode` |
| Web framework | axum | 0.8.x | Tower middleware, routing, extractors |
| HTTP middleware | tower-http | 0.6.x | `CompressionLayer`, `TraceLayer` |
| Database | sqlx | 0.8.x | Compile-time checked queries, Postgres-native. Features: `runtime-tokio`, `postgres`, `macros`, `uuid`, `time` |
| HTTP client | reqwest | 0.13.x | TurboSMS API calls. Features: `json` |
| Async runtime | tokio | 1.x | Features: `full` |
| Error types | thiserror | 1.x | Typed error enums for domain and infra errors |
| Error propagation | anyhow | 1.x | `.context()` in application logic |
| Serialization | serde | 1.x | Features: `derive` |
| JSON | serde_json | 1.x | Request/response bodies |
| UUID | uuid | 1.x | Features: `v4`, `serde` |
| Time | time | 0.3.x | Features: `serde`, `formatting`, `macros` |
| Phone validation | phonenumber | 0.3.x | Port of Google's libphonenumber |
| Session tokens | rand | 0.9.x | `OsRng` for cryptographic randomness |
| Hashing | blake2 | 0.10.x | CSRF double-submit cookie hashing |
| Hashing (OTP/sessions) | sha2 | 0.10.x | SHA-256 for OTP codes and session token storage |
| Tracing | tracing | 0.1.x | Structured logging |
| Tracing subscriber | tracing-subscriber | 0.3.x | Features: `env-filter`, `fmt` |

No ORM. No session middleware. No auth library. Each concern is a small, composed piece.

### Infrastructure

| Concern | Choice | Notes |
|---------|--------|-------|
| Hosting | VPS with Coolify | Self-hosted PaaS. Handles Docker deploys, Traefik reverse proxy, Let's Encrypt TLS |
| Database | PostgreSQL on same VPS | Separate database: `CREATE DATABASE samete;`. Own connection pool, own backup schedule |
| SMS provider | TurboSMS | Direct Ukrainian carrier connections. REST API. ~$0.025/SMS. Alpha-name registration required (allow 3 weeks) |
| Container build | Multi-stage Docker with cargo-chef | cargo-chef for dependency layer caching. distroless runtime image |
| CI/CD | Dockerfile + Coolify auto-deploy | Coolify watches repo, auto-builds on push. No external CI needed initially |
| Task runner | just | Justfile for dev commands: `just dev`, `just test`, `just e2e`, `just build` |
| Dev server | cargo leptos watch | Hot reload for both server and WASM client |

### Runtime Image

```
Stage 1 (planner):
  rust:1.85-slim (or later stable)
  cargo install cargo-chef cargo-leptos
  rustup target add wasm32-unknown-unknown
  cargo chef prepare → recipe.json

Stage 2 (builder):
  cargo chef cook --release → cached dependencies
  cargo leptos build --release

Stage 3 (runtime):
  gcr.io/distroless/cc-debian12:nonroot
  COPY binary + target/site/
  ENV LEPTOS_SITE_ROOT=site
  EXPOSE 3000
```

Binary size reduced with `[profile.release]`: `strip = true`, `lto = "thin"`, `opt-level = "z"` for WASM, `panic = "abort"`.

---

## SMS Integration: TurboSMS

### Why TurboSMS

7x cheaper than Twilio for Ukrainian traffic. Direct carrier connections to Kyivstar, Vodafone, lifecell. 19 years of track record. They handle alpha-name registration bureaucracy. Pay per delivered message — no monthly fees, no minimums.

### API

Single endpoint: `POST https://api.turbosms.ua/message/send.json` with Bearer token.

Request body: phone number (E.164 `+380XXXXXXXXX`), message text (Ukrainian, UCS-2 encoding, fits in 1 segment for OTP-length messages), sender alpha-name.

Integration is ~30 lines of `reqwest` + `serde_json`. No SDK exists or is needed.

### Alpha-Name Registration

Required for Ukrainian A2P SMS. Unregistered alpha names get replaced with generic sender. Registration takes up to 3 weeks with operators. Must be started well before launch.

### SMS Language

Ukrainian. This is the right choice for a Kyiv community. UCS-2 encoding limits segments to 70 chars (vs 160 for ASCII), but OTP messages ("Ваш код: 123456") fit in 1 segment.

### Costs

At 50 users with 90-day sessions: ~17 SMS/month for auth (~4 re-auths/year per user) + ~5-10 notification SMS per season. Roughly 30 SMS/month = ~$0.75/month. A 500+ UAH top-up covers months.

### Explicitly Rejected

- **Telegram** for auth or notifications — per product spec: "No Telegram integration, no Telegram bots, no Telegram mini-apps — not now, not later."
- **Twilio** — 7x cost premium buys developer experience not needed at this scale.
- **BSG** — cheaper ($0.011/SMS) with managed OTP API. Worth revisiting if TurboSMS becomes problematic. Less track record.
- **Infobip** — €150/month minimum is disqualifying for <100 SMS/month.

---

## Authentication

### OTP Flow

1. User enters phone number
2. Server validates: registered, active, not rate-limited
3. Server generates 6-digit code via `rand::OsRng`
4. Server stores SHA-256 hash of code in `otp_codes` table (invalidating any previous code for this phone)
5. Server sends code via TurboSMS (inline, awaits response)
6. User enters code
7. Server hashes input, compares to stored hash
8. On match: create session, set cookie, redirect
9. On mismatch: increment attempt counter, respond with error

**Allowlist-only:** OTP requests rejected for unregistered or deactivated phone numbers. Since the organizer pre-registers all users, SMS pumping fraud is eliminated by construction.

### Rate Limiting

| Constraint | Limit | Purpose |
|------------|-------|---------|
| OTP requests per phone | 1 per 60 seconds | Prevents SMS pumping |
| OTP requests per phone per hour | 5 | Caps cost exposure |
| Failed verifications per code | 3 (then invalidate) | Prevents brute force |
| Failed verifications per phone per hour | 10 (then lock 1 hour) | Prevents sustained attack |
| Global SMS per hour | ~50 | Anomaly detection circuit breaker |

### Phone Number Format

E.164: `+380XXXXXXXXX`. Regex: `^\+380\d{9}$`.

Normalize on input: strip whitespace/hyphens/parentheses, replace leading `0` with `+380`, prepend `+` if starts with `380`. Store normalized, never raw.

Validate with `phonenumber` crate for structural correctness.

### Session Management

**Single opaque token.** 256-bit random value generated via `OsRng`. Stored as SHA-256 hash in `sessions` table. 90-day expiry. Delivered as a single `HttpOnly; Secure; SameSite=Strict; Path=/` cookie.

**Session lifecycle:**
- Created on successful OTP verification
- Validated on every request: hash the cookie value, look up in DB, check expiry
- Destroyed on logout (delete row) or deactivation (delete all rows for user)
- Expired sessions cleaned up lazily (checked on access) or by periodic DB query from admin UI

**No JWT.** At 50 users, a DB lookup per request is trivially cheap. A single opaque token is one concept, one cookie, one table. Revocable by construction — delete the row. JWT would add token signing, expiry logic, two cookie types, and a time-based validity dimension for zero practical benefit.

**Session cookie format:** `session=<base64url(random_bytes)>; HttpOnly; Secure; SameSite=Strict; Max-Age=7776000; Path=/`

### CSRF Protection

Double Submit Cookie pattern:

1. On form load: server generates random value, sets it as `__Host-csrf` HttpOnly cookie, returns `Blake2s256(cookie_value + server_secret)` as hidden form field
2. On submit: server reads the cookie, recomputes the hash, compares to submitted field
3. `__Host-` prefix enforces `Secure` + `Path=/` at browser level
4. Server secret is a `u128` generated once at startup, held in app state

`SameSite=Strict` on the session cookie adds defense-in-depth but is not sufficient alone for state-mutating POST requests.

### Security Model

**Threat model is low.** Small private community, organizer-registered users, no financial transactions, no sensitive content beyond phone numbers.

- **SIM swap:** Low-prevalence in Ukraine (in-person ID required for SIM replacement). 90-day sessions minimize the attack window — attacker must time SIM swap with the rare re-authentication event.
- **OTP brute force:** 6-digit code + 3-attempt limit = 0.0003% guess probability per code. Hourly lockout prevents sustained attempts.
- **Session theft via DB compromise:** Tokens stored hashed. DB leak does not expose raw tokens.
- **XSS token theft:** HttpOnly cookies are inaccessible to JavaScript. Not a vector.

---

## Data Model

**Authoritative schema is in `spec/Data Model.md`.** This section summarizes the entity relationships and key design decisions. If any conflict exists, Data Model.md wins.

### Entity Overview

```
users ──< enrollments >── seasons
  │                         │
  ├── delivery_addresses    │
  │   (1:1)                 ├──< assignments (sender → recipient,
  ├──< sessions             │     includes receipt_status + receipt_note)
  ├──< otp_codes            │
  │                         │
  ├──< known_group_members  │
  │         │               │
  │         │               │
  └── known_groups          │
```

### Season Phase Enum

```
enrollment → preparation → assignment → delivery → complete
                                          │
                                          └→ cancelled (from any non-terminal phase)
```

Six phases (collapsed from eight). Creating/Confirming merged into Preparation (the confirm deadline gates the end, not a phase transition). Sending/Receiving merged into Delivery (receipt timing is driven by `notified_at` timestamp on assignments, not a phase transition).

The phase is a Postgres enum column mirrored by a Rust enum. Transition logic lives as methods on the Rust enum:

- `try_advance(&self) -> Result<Phase, InvalidTransition>` — moves to the next phase in sequence
- `can_advance(&self) -> bool` — checks if advancement is valid
- `cancel(&self) -> Result<Phase, InvalidTransition>` — only valid from non-terminal phases

Invalid transitions return `Err`. No scattered if-checks in server functions — the phase enum is the single source of transition rules.

### Key Design Decisions

- **Enums over bools** for expandable axes: `user_role` (Participant, Admin), `user_status` (Active, Deactivated), `receipt_status` (NoResponse, Received, NotReceived)
- **Separate table for delivery logistics:** `delivery_addresses` (1:1 with users) keeps Nova Poshta data out of the user identity table
- **No NP data on enrollments:** enrollment means "I'm in this season." Delivery address is read live from `delivery_addresses`
- **Nullable timestamps as one-way latches:** `confirmed_ready_at` (null = not confirmed), `notified_at` (null = SMS not delivered)
- **No `released` flag:** assignment visibility gated by season phase = Delivery
- **No `receipts` table:** receipt data (`receipt_status` enum + `receipt_note`) lives on assignments
- **No `past_pairings` table:** past pairings queried from assignments joined with completed seasons
- **OTP codes retained, not upserted:** enables rate limit counting from existing rows

### Social Graph

Season 1: organizer manages known_groups and known_group_members directly in the database. No in-app UI. The assignment algorithm reads this data plus past pairings (queried from assignments of completed seasons) to score candidate cycles.

The social graph is not a graph traversal problem. It is a weight lookup: "how strongly connected are participants A and B?" Computed as the sum of group weights for groups containing both A and B, plus a bonus for each past pairing. Representable as a simple weight matrix — no graph library needed.

---

## Assignment Algorithm

### Input

- Set of confirmed participants (N ≥ 3)
- Social weight matrix (from known_groups + past pairings queried from assignments of completed seasons)
- Target cohort size: 11-15

### Cohort Splitting

When N > 15, split into cohorts. Find the partition of N into groups of 11-15 that minimizes the maximum group size deviation from the mean. For N ≤ 15, single cohort.

Example: N=25 → 13+12 (not 15+10). N=30 → 15+15. N=31 → 16 is too large → 11+10+10 is too small → 16+15 with the 16 split? No — 11+10+10=31. Actually: find all valid partitions where each part is in [11,15] (or [3,15] if organizer approves small cohorts). Pick the most balanced.

For season 1 with ~15 participants, this is not exercised.

### Cycle Generation

Per cohort, generate a Hamiltonian cycle (every participant sends to exactly one, receives from exactly one, forming a single loop):

1. Start with participant list as nodes
2. Backtracking DFS: try to build a path visiting all nodes, returning to start
3. At each step, choose the next unvisited node that minimizes the edge's social weight (greedy heuristic with backtracking)
4. Score the complete cycle: sum of social weights of all edges
5. Run multiple attempts (randomized starting order), keep the lowest-scoring cycle
6. At N≤15, this completes in milliseconds

### Social Weight Scoring

For an edge (A → B):
- Sum of `known_groups.weight` for all groups containing both A and B
- +1 for each past season where A sent to B or B sent to A
- Higher score = stronger existing connection = less desirable pairing

The algorithm minimizes total cycle score. The constraint is soft — at small pool sizes with dense social graphs, some known-person pairings are inevitable. The algorithm degrades gracefully.

### Organizer Override

After generation, the organizer sees the full cycle(s) and can swap individual pairings. Swaps must preserve single-loop topology — the app validates this before accepting the swap.

Assignments are not visible to participants until the organizer advances the season to Delivery phase.

---

## App Structure

### Leptos Configuration

Single crate. Features `ssr` and `hydrate` are mutually exclusive at compile time. SSR renders the first paint server-side (fast on Ukrainian mobile networks), then WASM hydrates for interactivity.

`Cargo.toml` `[package.metadata.leptos]` section configures cargo-leptos: site root, site address, WASM opt-level, end-to-end test command.

### Entry Points

**`src/main.rs`** (SSR, compiled with `ssr` feature):
- Connect to Postgres, run `sqlx::migrate!()`
- Build `AppState` (pool, config, CSRF secret)
- Set up Axum router with `leptos_routes_with_context` (provides pool + config to all server functions)
- Apply middleware: `TraceLayer`, `CompressionLayer`
- Bind and serve

**`src/lib.rs`** (WASM, compiled with `hydrate` feature):
- `hydrate_body(App)` — attaches reactive graph to server-rendered DOM

**`src/app.rs`**:
- `shell()` — HTML document wrapper with `<HydrationScripts>`, `<AutoReload>`, `<MetaTags>`
- `App` — root component with `<Router>`, `<Routes>`, context providers

### Module Layout

```
src/
  main.rs              — SSR entry point, server setup
  lib.rs               — WASM entry point
  app.rs               — shell(), App component, route definitions

  auth.rs              — OTP flow, session creation/validation, CSRF
  sms.rs               — TurboSMS API client
  config.rs            — Config struct, from_env()
  error.rs             — AppError enum (thiserror + IntoResponse + FromServerFnError)
  phone.rs             — Phone number normalization and validation
  db.rs                — PgPool setup, migration, shared query helpers
  types.rs             — Shared domain types (Phase, UserRole, UserStatus, ReceiptStatus enums)

  pages/
    login.rs           — Phone input + OTP verification
    onboarding.rs      — Nova Poshta city + branch number selection
    season.rs          — Participant season view (enroll, confirm, assignment, receipt)
    home.rs            — Landing / current state

  admin/
    mod.rs             — Admin route tree
    dashboard.rs       — Season overview, confirmed count, action buttons
    participants.rs    — Register new, list, deactivate
    season.rs          — Create, launch, advance phase
    assignments.rs     — View graph, override pairings, release
    sms.rs             — Trigger SMS batches (assignment notifications, nudges, season-open)
```

One file per module. No `mod.rs` pattern (except `admin/mod.rs` for the route subtree). Public types in `types.rs`, re-exported from `lib.rs` where needed.

### Context Injection

```
main.rs:
  PgPool + Config + CsrfSecret provided via leptos_routes_with_context

Any server function:
  use_context::<PgPool>()              → DB access
  use_context::<Config>()              → App configuration
  use_context::<http::request::Parts>() → Read cookies/headers
  expect_context::<ResponseOptions>()   → Set cookies/headers
  leptos_axum::extract::<T>()          → Typed Axum extractors
```

### Route Structure

**Participant routes:**

| Path | Component | Auth | SsrMode | Purpose |
|------|-----------|------|---------|---------|
| `/login` | Login | No | Async | Phone input + OTP |
| `/onboarding` | Onboarding | Yes | Async | Nova Poshta branch (first login only) |
| `/` | Home | Yes | Async | Current season state, next action |
| `/season` | Season | Yes | Async | Enroll, confirm, view assignment, confirm receipt |

**Admin routes:**

| Path | Component | Auth + Admin | SsrMode | Purpose |
|------|-----------|-------------|---------|---------|
| `/admin` | Dashboard | Yes | Async | Season health, counts, actions |
| `/admin/participants` | Participants | Yes | Async | Register, list, deactivate |
| `/admin/season` | SeasonManage | Yes | Async | Create, launch, advance phase |
| `/admin/assignments` | Assignments | Yes | Async | View graph, override, release |
| `/admin/sms` | SmsBatch | Yes | Async | Trigger notification batches |

All routes use `SsrMode::Async` — server resolves all data (including auth state) before sending HTML. No flash of unauthenticated content. No client-side loading spinners for auth checks.

### Auth Guard

Two layers:

1. **Route level:** `<ProtectedRoute>` with `condition=move || auth_resource.get().map(|r| r.unwrap_or(false))`. Redirects to `/login` if unauthenticated. Admin routes additionally check `role = Admin`.

2. **Server function level:** Every protected server function calls `validate_session()` at the top. This is defense-in-depth — the route guard handles the common case, the server function guard prevents direct API calls from bypassing it.

Auth state for SSR: a `get_current_user()` server function called as a `Resource::new()` in `App`. Resolved during SSR, serialized into HTML, hydrated client-side. No separate API call on initial page load.

### Error Handling

`AppError` enum via `thiserror`:

```
AppError
  ├── NotFound
  ├── Unauthorized
  ├── Forbidden
  ├── InvalidInput(String)
  ├── InvalidTransition { from: Phase }
  ├── RateLimited
  ├── SmsFailed(String)
  ├── Database(sqlx::Error)
  └── Internal(anyhow::Error)
```

Implements:
- `thiserror::Error` — for Display and source chain
- `IntoResponse` — maps to HTTP status codes for Axum
- `FromServerFnError` — bridges to Leptos server function error model
- `From<sqlx::Error>` — automatic conversion from DB errors
- `From<anyhow::Error>` — automatic conversion from contextual errors

Server functions return `Result<T, ServerFnError>`. Inside the function, use `?` with `.context("...")` from anyhow. The `From` impls handle conversion.

### Configuration

`Config` struct with `from_env() -> Result<Config, ConfigError>`:

| Variable | Required | Purpose |
|----------|----------|---------|
| `DATABASE_URL` | Yes | Postgres connection string |
| `TURBOSMS_TOKEN` | Yes | TurboSMS API bearer token |
| `TURBOSMS_SENDER` | Yes | Registered alpha-name |
| `CSRF_SECRET` | No | Override for CSRF secret (generated at startup if absent) |

`ConfigError` is a `thiserror` enum with per-variable variants. Fails fast at startup with a clear message naming the missing variable.

No config files. No dotenv crate. Environment variables are environment variables.

### Tracing

```
tracing_subscriber::registry()
    .with(tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "samete=info,tower_http=info".into()))
    .with(tracing_subscriber::fmt::layer())
    .init();
```

`tower_http::trace::TraceLayer` on the Axum router for request/response logging. Structured fields for SMS delivery results, auth events, season transitions.

No external monitoring or alerting in season 1. Stdout logs + manual checks via Coolify log viewer.

---

## Participant Flows

What each screen shows at each season phase. The app's job is to show the right thing at the right time.

### Home Screen (`/`)

The participant sees exactly one thing based on the current season phase and their state within it:

| Season Phase | Participant State | Home Screen Shows |
|-------------|-------------------|-------------------|
| No active season | — | "No season running. We'll SMS you when one opens." |
| enrollment | Not enrolled | Enroll button + season timeline + theme (if any) + content guidelines |
| enrollment | Enrolled | "You're in. Creation period starts after signup deadline." |
| preparation | Enrolled, not confirmed | "Create your mail. Confirm ready by [deadline]." + confirm button + countdown |
| preparation | Confirmed | "You're confirmed. Assignments coming after [deadline]." |
| assignment | Confirmed | "Organizer is preparing assignments." |
| delivery | Assigned, receipt_status = NoResponse | Recipient's name, phone, Nova Poshta city + branch number. Receipt buttons. |
| delivery | Receipt confirmed | "Thanks. Meetup details coming soon." |
| complete | — | "Season complete." |

This is a single component with a match on (phase, participant_state). Not multiple pages — one page that shows the right content.

### Admin Dashboard (`/admin`)

The organizer sees:

| Season Phase | Dashboard Shows |
|-------------|-----------------|
| No active season | "Create new season" button |
| enrollment | Enrolled count. Deadline countdown. |
| preparation | Enrolled count. Confirmed count / enrolled count. "Advance to assignment" button (disabled until confirm deadline passes). Pre-deadline SMS nudge button. |
| assignment | "Generate assignments" button. After generation: cycle visualization, override controls, "Release assignments" button (advances phase to delivery + triggers assignment SMS). |
| delivery | Assignment list. Notification status (notified_at null/set per assignment). Receipt status per participant (NoResponse / Received / NotReceived). Receipt nudge SMS button. "Advance to complete" button. |
| complete | Season summary. "Create new season" button. |

---

## Testing Strategy

### The Feedback Loop

The compiler is your best friend, forever and always. The feedback loop has five layers, each catching what the inner layers cannot:

```
1. rust-analyzer        (instant — types, borrow checker, inline diagnostics)
2. bacon clippy         (continuous — pedantic lints, style, correctness hints)
3. cargo test           (on demand — unit tests, integration tests, business rules)
4. cargo leptos end-to-end  (on demand — full-stack E2E, user-visible flows)
5. CI                   (on push — everything, clean environment)
```

Nothing ships that any layer rejects.

### Compiler Configuration

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = { level = "deny" }
pedantic = { level = "deny" }
```

Pedantic clippy findings are not warnings. They are errors. They are always fixed. No `#[allow(clippy::...)]` without a comment explaining why the lint is wrong for this specific case.

### Development Runner

`bacon clippy` as the continuous background runner. Not `bacon check` — clippy gives the pedantic lints. The developer (or implementing agent) keeps bacon running in a terminal and treats every output line as blocking.

### LSP

rust-analyzer must be active and its diagnostics treated as authoritative. Implementing agents must use LSP tool access to verify type correctness before moving on. Red squiggles are not advisory — they are errors.

### Unit and Integration Tests

**Intelligent TDD based on the spec.** Tests are derived from the product spec's acceptance criteria, not invented. Each test traces back to a story number.

**What to test with `cargo test`:**
- Phase transition logic: valid transitions succeed, invalid transitions return `Err`
- OTP generation, hashing, verification, expiry, rate limiting
- Phone number normalization (various input formats → E.164)
- Assignment algorithm: cycle validity, social weight minimization, cohort splitting
- Session creation, validation, expiry, revocation

**What NOT to test with `cargo test`:**
- Database operations (tested via E2E against real Postgres)
- SMS delivery (tested via E2E with a mock TurboSMS endpoint or manual verification)
- Leptos component rendering (tested via E2E in a real browser)

### E2E Tests: Playwright

E2E tests are written as part of the architecture phase, before implementation. They encode the user stories as executable specifications. The implementing agent works story by story, running E2E tests to see progress. Red → green.

**Setup:**

```
end2end/
  playwright.config.ts    — baseURL: http://127.0.0.1:3000, workers: 1
  package.json            — @playwright/test dependency
  tests/
    fixtures/
      mail_club_page.ts   — Page Object Model
    epic1_join.spec.ts     — Stories 1.1, 1.2, 1.3
    epic2_season.spec.ts   — Stories 2.1, 2.2, 2.3, 2.4
    epic3_assign.spec.ts   — Stories 3.1, 3.2, 3.3
    epic4_manage.spec.ts   — Stories 4.1, 4.2
    epic5_sms.spec.ts      — Stories 5.1, 5.2, 5.3, 5.4
    epic6_account.spec.ts  — Story 6.1
```

**cargo-leptos orchestration:**

```toml
# Cargo.toml [package.metadata.leptos]
end2end-cmd = "npx playwright test"
end2end-dir = "end2end"
```

`cargo leptos end-to-end` builds the app, starts the server, runs Playwright, tears down. The `webServer` block in `playwright.config.ts` is left commented out — cargo-leptos manages the server lifecycle.

**Page Object Model:** `MailClubPage` class with methods mapping to user actions:
- `login(phone)` — enter phone, submit, enter OTP, submit
- `completeOnboarding(city, branchNumber)` — set Nova Poshta city + branch number
- `enrollInSeason()` — click enroll
- `confirmReady()` — click confirm
- `viewAssignment()` — read recipient details
- `confirmReceipt(received)` — click received/not received
- Admin methods: `createSeason(...)`, `launchSeason()`, `generateAssignments()`, `releaseAssignments()`, `triggerSms(type)`

**Test traceability:** Each test case comments its story number:

```typescript
// Story 2.2: Confirm mail is ready
test('confirmed participant enters assignment graph', async ({ page }) => {
  // ...
});
```

**OTP in tests:** Tests need to verify OTP codes. Options:
1. Test mode: when `SAMETE_TEST_MODE=true`, the app uses a fixed OTP code (e.g., `000000`). This env var is only set in test environments. The server logs a warning at startup when test mode is active.
2. Read OTP from DB: Playwright test calls a test-only API endpoint that returns the current OTP for a phone number. This endpoint only exists when compiled with a `test-support` feature.

Option 1 is simpler. Option 2 is more correct (tests the real OTP flow except the SMS delivery). Recommend option 2 with the test-support feature gate — the endpoint cannot exist in production builds.

---

## Development Protocol

### For the Implementer

The implementer may be a human or an AI agent. These rules apply to both.

**The compiler is your best friend, forever and always.**

1. **Model in types first.** Before writing any logic, define the enums, structs, and newtypes that represent the domain. Make invalid states unrepresentable. Then let the compiler tell you what methods those types need.

2. **Strict pedantic clippy, always.** `clippy::pedantic = deny`. Every finding is fixed. No exceptions without a documented reason. This extends the compiler's feedback loop into style and correctness territory.

3. **TDD from the spec.** User stories have Given/When/Then acceptance criteria. These become tests — unit tests for pure logic, E2E tests for user-visible flows. Write the test, watch it fail, implement until it passes. Tests trace back to story numbers.

4. **Use every feedback channel.** rust-analyzer for instant type feedback. bacon clippy for continuous lint checking. `cargo test` for business rules. `cargo leptos end-to-end` for full-stack verification. Treat output from every channel as blocking.

5. **Agents use LSP.** An implementing agent must leverage rust-analyzer diagnostics. LSP output is not advisory — it is a compiler-equivalent feedback channel. Fix diagnostics before moving on.

6. **One story at a time.** Implement story by story in dependency order. Run the relevant E2E test after each story. Do not move to the next story until the current one passes E2E.

7. **No speculation.** Do not build for imagined futures. Do not add configurability. Do not add abstractions for one-time operations. The spec defines what exists. Build exactly that.

### Implementation Order

Follow the story dependency chain:

```
Phase 1 — Foundation (no user-visible features yet):
  DB schema + migrations
  Config + tracing setup
  AppError type
  Phase enum with transition methods
  Phone number normalization

Phase 2 — Auth (Epic 1):
  1.1 Organizer registers participant (admin)
  1.2 SMS OTP sign-in
  1.3 Onboarding (Nova Poshta branch)

Phase 3 — Season lifecycle (Epics 4, 2):
  4.1 Create season (admin)
  4.2 Launch season (admin)
  2.1 Enroll in season
  2.2 Confirm ready

Phase 4 — Assignment (Epic 3):
  3.1 Generate cohort assignments
  3.2 Social-awareness constraints
  3.3 Override assignments (admin)

Phase 5 — Delivery (Epics 2, 5):
  2.3 Receive assignment + 5.1 Assignment SMS
  2.4 Confirm receipt + 5.2 Receipt nudge SMS
  5.3 Season-open SMS
  5.4 Pre-deadline nudge SMS

Phase 6 — Account management (Epic 6):
  6.1 Deactivate account (admin)
```

### Justfile

```
dev:         cargo leptos watch
test:        cargo test
clippy:      cargo clippy --all-targets
e2e:         cargo leptos end-to-end
build:       cargo leptos build --release
db-reset:    sqlx database drop && sqlx database create && sqlx migrate run
db-migrate:  sqlx migrate run
prepare:     cargo sqlx prepare
```

---

## Open Questions (Non-Blocking)

These can be resolved during implementation without architectural impact:

- **Domain name** — needed before deployment, not before development
- **Exact Coolify configuration** — resolved during first deploy
- **Backup schedule** — set up after first deploy, before season 1 launch
- **Alpha-name registration timeline** — start 3+ weeks before launch. The alpha-name text itself can be decided later
- **Season 1 timeline** — when does the first season actually run? Determines the development deadline
