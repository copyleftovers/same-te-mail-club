# Implementation Plan: The Mail Club

## Preamble

This plan prescribes the exact implementation of the Mail Club app across 6 phases. The codebase is a bare Leptos 0.8 scaffold with dependencies declared but zero domain code. The implementer (human or agent) follows this plan story-by-story.

**Every step specifies exactly what to create, what types to define, and how to verify. No decisions are left to the implementer.**

### Constitutional Constraints

Two manifestos govern all implementation:

1. **Correct By Construction** — Make invalid states unrepresentable. Trust the compiler. Pay upfront at compile time.
2. **Simple Made Easy** — Simple means untangled. More clean parts > fewer tangled parts. Optimize for the artifact.

**Tension resolution:** Type richness when it eliminates entanglement (enum replacing boolean flags). Not when it introduces entanglement (generic trait hierarchies braiding concerns).

### Current State

```
src/main.rs   — Axum server, basic Leptos route setup
src/lib.rs    — WASM hydrate entry point, declares app module
src/app.rs    — shell(), App component, placeholder HomePage
```

All crates in Cargo.toml. No migrations, no domain types, no modules beyond `app`.

### Dev Environment

- Postgres via `docker compose up -d` (db=samete, user=samete, pass=samete, port=5432)
- `DATABASE_URL=postgres://samete:samete@localhost:5432/samete`
- `bacon clippy-ssr` as continuous background runner
- Pre-commit: fmt, cargo-check (ssr), clippy (ssr)

### Reference Files

Read before starting any phase:
- `spec/Architecture.md` — authoritative HOW (§Development Protocol and §Testing Strategy are binding)
- `spec/Product Spec.md` — authoritative WHAT
- `spec/User Stories.md` — acceptance criteria (Given/When/Then for every story)

### Development Protocol

**The compiler is your best friend, forever and always.**

Five-layer feedback loop — ALL layers are BLOCKING:

```
1. rust-analyzer / LSP    (instant — types, borrow checker, inline diagnostics)
2. bacon clippy-ssr       (continuous — pedantic lints, style, correctness hints)
3. cargo test             (on demand — unit tests, business rules)
4. cargo leptos end-to-end (on demand — full-stack E2E, user-visible flows)
5. pre-commit             (on commit — fmt, cargo check, clippy)
```

**Rules:**

1. **Model in types first.** Define enums, structs, newtypes BEFORE any logic. Make invalid states unrepresentable. Then let the compiler tell you what methods those types need.
2. **Strict pedantic clippy, always.** Already configured: `clippy::pedantic = deny`. Every finding is fixed. No `#[allow(clippy::...)]` without a comment explaining why the lint is wrong for this specific case.
3. **TDD from the spec.** Tests derive from acceptance criteria in `spec/User Stories.md`. Write test → watch fail → implement → watch pass. Every test traces to a story number.
4. **Use LSP.** Implementing agents MUST use rust-analyzer diagnostics (via LSP tool). LSP output is not advisory — it is blocking. Red squiggles are errors. Fix diagnostics before moving on.
5. **One story at a time.** Implement in dependency order per this plan. Run the relevant E2E test after each story. Do NOT move to the next story until the current one passes.
6. **No speculation.** Do not build for imagined futures. Do not add configurability. Do not add abstractions for one-time operations. The spec defines what exists. Build exactly that.
7. **`cargo sqlx prepare --workspace` after every phase** that adds or changes `sqlx::query!()` calls. Commit `.sqlx/` — it is NOT in .gitignore.

### E2E Tests

E2E test stubs exist in `end2end/tests/` — they encode user stories as executable specifications. They are **FAILING by design**. The implementer makes them pass story by story. Red → green.

| Test file | Stories | Implementation phase |
|-----------|---------|---------------------|
| `epic1_join.spec.ts` | 1.1, 1.2, 1.3 | Phase 2 |
| `epic4_manage.spec.ts` | 4.1, 4.2 | Phase 3 |
| `epic2_season.spec.ts` (enrollment/confirm tests) | 2.1, 2.2 | Phase 3 |
| `epic3_assign.spec.ts` | 3.1, 3.2, 3.3 | Phase 4 |
| `epic2_season.spec.ts` (assignment/receipt tests) | 2.3, 2.4 | Phase 5 |
| `epic5_sms.spec.ts` | 5.1, 5.2, 5.3, 5.4 | Phase 5 |
| `epic6_account.spec.ts` | 6.1 | Phase 6 |

Page Object Model: `end2end/tests/fixtures/mail_club_page.ts` — centralizes all selectors. Update selectors here when UI changes, not in test files.

E2E test environment requires:
- `SAMETE_TEST_MODE=true` — fixed OTP code "000000"
- `SAMETE_SMS_DRY_RUN=true` — log SMS instead of sending
- Postgres running with migrated DB (`just db-reset` before test run)

### What to Test Where

| Test with `cargo test` (unit) | Test with `just e2e` (E2E) |
|------|------|
| Phase transition logic | Database operations |
| Phone number normalization | Leptos component rendering |
| OTP hashing/verification logic | Full user flows (login, enroll, confirm) |
| Assignment algorithm (cycle validity, scoring, cohort splitting) | Auth guards and redirects |
| Session token generation logic | Admin workflows end-to-end |

**What NOT to test with `cargo test`:** Database operations, SMS delivery, component rendering. Those are E2E.

---

## Phase 1: Foundation

No user-visible features. Establishes the domain model, database schema, configuration, and error handling.

### Prerequisites

- `docker compose up -d` — Postgres running
- `DATABASE_URL` exported

### 1.1 Database Migrations

**Create directory:** `migrations/`

**Create file:** `migrations/20260312000001_create_types.sql`

```sql
CREATE TYPE season_phase AS ENUM (
    'signup',
    'creating',
    'confirming',
    'assigning',
    'sending',
    'receiving',
    'complete',
    'cancelled'
);
```

**Create file:** `migrations/20260312000002_create_tables.sql`

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    nova_poshta_branch TEXT,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    onboarded BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sessions (
    token_hash TEXT PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE otp_codes (
    phone TEXT PRIMARY KEY,
    code_hash TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE seasons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phase season_phase NOT NULL DEFAULT 'signup',
    signup_deadline TIMESTAMPTZ NOT NULL,
    confirm_deadline TIMESTAMPTZ NOT NULL,
    theme TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX one_active_season
    ON seasons ((true))
    WHERE phase NOT IN ('complete', 'cancelled');

CREATE TABLE enrollments (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    season_id UUID NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    nova_poshta_branch TEXT NOT NULL,
    confirmed_ready BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, season_id)
);

CREATE TABLE assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    season_id UUID NOT NULL REFERENCES seasons(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id),
    recipient_id UUID NOT NULL REFERENCES users(id),
    released BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (season_id, sender_id),
    UNIQUE (season_id, recipient_id)
);

CREATE TABLE receipts (
    assignment_id UUID PRIMARY KEY REFERENCES assignments(id) ON DELETE CASCADE,
    received BOOLEAN NOT NULL,
    note TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE known_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    weight INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE known_group_members (
    group_id UUID NOT NULL REFERENCES known_groups(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, user_id)
);

CREATE TABLE past_pairings (
    sender_id UUID NOT NULL REFERENCES users(id),
    recipient_id UUID NOT NULL REFERENCES users(id),
    season_id UUID NOT NULL REFERENCES seasons(id),
    PRIMARY KEY (sender_id, recipient_id, season_id)
);
```

**Gate:**
```bash
sqlx database create && sqlx migrate run
# REQUIRED: exits 0, no errors
```

### 1.2 Domain Types (`src/types.rs`)

This module compiles on BOTH server and client (no feature gate). All types: `Serialize + Deserialize + Clone`.

**Create file:** `src/types.rs`

```rust
use serde::{Deserialize, Serialize};

/// Season phase — mirrors `season_phase` Postgres enum.
/// Transition rules gathered here, not scattered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::Type))]
#[cfg_attr(feature = "ssr", sqlx(type_name = "season_phase", rename_all = "lowercase"))]
pub enum Phase {
    Signup,
    Creating,
    Confirming,
    Assigning,
    Sending,
    Receiving,
    Complete,
    Cancelled,
}

impl Phase {
    /// Next phase in sequence. Err for terminal or cancelled.
    pub fn try_advance(self) -> Result<Self, InvalidTransition> {
        match self {
            Self::Signup => Ok(Self::Creating),
            Self::Creating => Ok(Self::Confirming),
            Self::Confirming => Ok(Self::Assigning),
            Self::Assigning => Ok(Self::Sending),
            Self::Sending => Ok(Self::Receiving),
            Self::Receiving => Ok(Self::Complete),
            Self::Complete | Self::Cancelled => Err(InvalidTransition { from: self }),
        }
    }

    pub fn can_advance(self) -> bool {
        self.try_advance().is_ok()
    }

    /// Cancel from any non-terminal phase.
    pub fn cancel(self) -> Result<Self, InvalidTransition> {
        if self.is_terminal() {
            Err(InvalidTransition { from: self })
        } else {
            Ok(Self::Cancelled)
        }
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Complete | Self::Cancelled)
    }
}

impl std::fmt::Display for Phase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Signup => "signup",
            Self::Creating => "creating",
            Self::Confirming => "confirming",
            Self::Assigning => "assigning",
            Self::Sending => "sending",
            Self::Receiving => "receiving",
            Self::Complete => "complete",
            Self::Cancelled => "cancelled",
        };
        f.write_str(s)
    }
}

/// Error for invalid phase transitions. Kept in types.rs (shared)
/// so Phase methods can use it without SSR dependencies.
#[derive(Debug, Clone)]
pub struct InvalidTransition {
    pub from: Phase,
}

impl std::fmt::Display for InvalidTransition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no valid transition from {}", self.from)
    }
}

impl std::error::Error for InvalidTransition {}
```

**Unit tests — add `#[cfg(test)] mod tests` in same file:**

Test every valid transition chain: `Signup→Creating→...→Complete`. Test invalid: `Complete.try_advance()` is `Err`, `Cancelled.try_advance()` is `Err`. Test `cancel()` from every non-terminal returns `Ok(Cancelled)`. Test `cancel()` from `Complete` and `Cancelled` returns `Err`. Test `is_terminal()`. Test `can_advance()` matches `try_advance().is_ok()`. Test `Display` output matches Postgres enum values exactly.

**Gate:**
```bash
cargo test -- types
# REQUIRED: all pass
```

### 1.3 Error Types (`src/error.rs`)

SSR-only module. Contains `AppError` enum.

**Create file:** `src/error.rs`

```rust
use crate::types::{InvalidTransition, Phase};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("invalid phase transition from {from}")]
    InvalidTransition {
        from: Phase,
        #[source]
        source: InvalidTransition,
    },

    #[error("SMS delivery failed: {0}")]
    SmsFailed(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<InvalidTransition> for AppError {
    fn from(t: InvalidTransition) -> Self {
        Self::InvalidTransition {
            from: t.from,
            source: t,
        }
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let status = match &self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::InvalidInput(_) => StatusCode::BAD_REQUEST,
            Self::InvalidTransition { .. } => StatusCode::CONFLICT,
            Self::SmsFailed(_) => StatusCode::BAD_GATEWAY,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

impl From<AppError> for leptos::prelude::ServerFnError {
    fn from(e: AppError) -> Self {
        Self::new(e.to_string())
    }
}
```

**Note:** If `ServerFnError::new()` does not exist in the installed Leptos version, use whichever constructor accepts a `String` or `impl Display`. The compiler will guide you. Check `ServerFnError::ServerError(String)` as fallback.

### 1.4 Configuration (`src/config.rs`)

SSR-only module.

**Create file:** `src/config.rs`

```rust
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub turbosms_token: String,
    pub turbosms_sender: String,
    pub csrf_secret: u128,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing DATABASE_URL")]
    MissingDatabaseUrl,
    #[error("missing TURBOSMS_TOKEN")]
    MissingTurbosmsToken,
    #[error("missing TURBOSMS_SENDER")]
    MissingTurbosmsSender,
}

impl Config {
    /// Read from environment. Fails fast naming the missing variable.
    pub fn from_env() -> Result<Self, ConfigError> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| ConfigError::MissingDatabaseUrl)?;
        let turbosms_token = std::env::var("TURBOSMS_TOKEN")
            .map_err(|_| ConfigError::MissingTurbosmsToken)?;
        let turbosms_sender = std::env::var("TURBOSMS_SENDER")
            .map_err(|_| ConfigError::MissingTurbosmsSender)?;

        let csrf_secret = std::env::var("CSRF_SECRET")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(rand::random::<u128>);

        Ok(Self {
            database_url,
            turbosms_token,
            turbosms_sender,
            csrf_secret,
        })
    }
}
```

### 1.5 Phone Normalization (`src/phone.rs`)

SSR-only module. Normalize diverse Ukrainian phone inputs to E.164 `+380XXXXXXXXX`.

**Create file:** `src/phone.rs`

```rust
/// Normalize a phone input to E.164 (+380XXXXXXXXX).
/// Strips whitespace, hyphens, parentheses.
/// Replaces leading 0 with +380, prepends + if starts with 380.
/// Validates with phonenumber crate.
/// Returns Err for non-Ukrainian or structurally invalid numbers.
pub fn normalize(raw: &str) -> Result<String, PhoneError> {
    // 1. Strip whitespace, hyphens, parentheses, dots
    // 2. If starts with "0", replace with "+380"
    // 3. If starts with "380" (no +), prepend "+"
    // 4. Parse with phonenumber crate (default region UA)
    // 5. Validate: must be valid, country code 380
    // 6. Return E.164 format string
    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum PhoneError {
    #[error("invalid phone number format")]
    InvalidFormat,
    #[error("only Ukrainian (+380) numbers are supported")]
    NotUkrainian,
}
```

**Unit tests in same file:**

Test normalization:
- `"0671234567"` → `"+380671234567"`
- `"+380671234567"` → `"+380671234567"` (already normalized)
- `"380671234567"` → `"+380671234567"`
- `"067-123-45-67"` → `"+380671234567"`
- `"(067) 123 45 67"` → `"+380671234567"`
- `"+1234567890"` → `Err(NotUkrainian)`
- `"abc"` → `Err(InvalidFormat)`
- `""` → `Err(InvalidFormat)`

### 1.6 Database Helpers (`src/db.rs`)

SSR-only module. Pool creation and migration.

**Create file:** `src/db.rs`

```rust
use sqlx::PgPool;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPool::connect(database_url).await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!().run(pool).await
}
```

### 1.7 Main.rs Rewrite

Replace the current `main.rs` with the full server setup.

**Rewrite file:** `src/main.rs`

```rust
#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::get_configuration;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use samete::app::{App, shell};
    use tower_http::compression::CompressionLayer;
    use tower_http::trace::TraceLayer;

    // 1. Tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "samete=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Config
    let config = samete::config::Config::from_env()
        .expect("configuration error");

    // 3. Database
    let pool = samete::db::create_pool(&config.database_url)
        .await
        .expect("database connection failed");
    samete::db::run_migrations(&pool)
        .await
        .expect("migrations failed");

    // 4. Leptos
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    // 5. Router with context injection
    let csrf_secret = config.csrf_secret;
    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                let config = config.clone();
                move || {
                    leptos::context::provide_context(pool.clone());
                    leptos::context::provide_context(config.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(leptos_options);

    tracing::info!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
```

**Note:** The exact `leptos_routes_with_context` signature may differ. Consult Leptos 0.8 docs. The key requirement: `PgPool` and `Config` are available via `use_context::<PgPool>()` and `use_context::<Config>()` in all server functions.

### 1.8 Module Declarations (`src/lib.rs`)

**Update file:** `src/lib.rs`

```rust
#![allow(clippy::must_use_candidate)]

pub mod app;
pub mod types;

#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod error;
#[cfg(feature = "ssr")]
pub mod phone;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

### 1.9 SQLx Offline Mode

After migrations run and code compiles with `sqlx::query!()` macros:

```bash
cargo sqlx prepare --workspace
```

This generates `.sqlx/` directory with query metadata. Commit `.sqlx/` to git (it is NOT in `.gitignore`). This enables Docker builds and CI without a running database.

**Run `cargo sqlx prepare` after EVERY phase that adds or modifies `sqlx::query!()` calls.**

### Phase 1 Verification Gates

```bash
# Gate 1: Migrations
sqlx migrate run
# REQUIRED: exits 0

# Gate 2: SSR compiles
cargo build --features ssr
# REQUIRED: exits 0, zero warnings

# Gate 3: WASM compiles
cargo build --features hydrate --target wasm32-unknown-unknown
# REQUIRED: exits 0

# Gate 4: Clippy
cargo clippy --features ssr -- -D warnings
# REQUIRED: exits 0

# Gate 5: Tests
cargo test
# REQUIRED: all phase transition tests pass

# Gate 6: No TODO in production code
grep -rn "todo!()" src/ --include="*.rs" | grep -v "test" | grep -v "#\[cfg(test)\]"
# REQUIRED: only phone.rs normalize() has todo!() (to be implemented)
```

---

## Phase 2: Auth (Epic 1)

Implements: user registration (admin), SMS OTP sign-in, onboarding. After this phase, users can authenticate and access the app.

### Prerequisites

- Phase 1 complete and all gates pass
- `TURBOSMS_TOKEN` and `TURBOSMS_SENDER` env vars set (use dummy values for dev; SMS won't actually send until TurboSMS account is live)

### 2.1 SMS Client (`src/sms.rs`)

SSR-only module. ~30 lines of reqwest + serde_json.

**Create file:** `src/sms.rs`

```rust
use crate::config::Config;

/// Send an SMS via TurboSMS API.
/// POST https://api.turbosms.ua/message/send.json
/// Body: { "recipients": [phone], "sms": { "sender": alpha_name, "text": message } }
/// Bearer token auth.
pub async fn send_sms(
    config: &Config,
    phone: &str,
    message: &str,
) -> Result<(), SmsError> {
    // 1. Build request body as serde_json::json!()
    // 2. POST with reqwest::Client, Bearer token from config
    // 3. Parse response, check for success
    // 4. Return Ok(()) or Err with message from API
    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum SmsError {
    #[error("TurboSMS API error: {0}")]
    ApiError(String),
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}
```

**Add a dev bypass:** When env var `SAMETE_SMS_DRY_RUN=true`, log the SMS content instead of sending. This is for development only.

### 2.2 Auth Module (`src/auth.rs`)

SSR-only module. Session and OTP logic. NO server functions here — just helper functions called by server functions in page modules.

**Create file:** `src/auth.rs`

Functions to implement:

```rust
use sqlx::PgPool;
use uuid::Uuid;

/// Generate a 6-digit OTP, hash it, store in otp_codes (upsert).
/// Returns the raw code (for SMS sending).
pub async fn create_otp(pool: &PgPool, phone: &str) -> Result<String, crate::error::AppError> {
    // 1. Generate 6-digit code: rand::random::<u32>() % 1_000_000, zero-pad
    // 2. SHA-256 hash the code
    // 3. INSERT INTO otp_codes (phone, code_hash, attempts, created_at, expires_at)
    //    ON CONFLICT (phone) DO UPDATE — upsert replaces previous code
    //    expires_at = now() + 10 minutes
    // 4. Return raw code string
    todo!()
}

/// Verify an OTP code against stored hash.
pub async fn verify_otp(pool: &PgPool, phone: &str, code: &str) -> Result<Uuid, crate::error::AppError> {
    // 1. SELECT from otp_codes WHERE phone = $1 AND expires_at > now()
    // 2. If no row: return Err(Unauthorized)
    // 3. If attempts >= 3: DELETE the row, return Err(Unauthorized)
    // 4. SHA-256 hash the submitted code
    // 5. Compare to stored hash
    // 6. If mismatch: INCREMENT attempts, return Err(Unauthorized)
    // 7. If match: DELETE the otp_codes row
    // 8. Look up user by phone: SELECT id FROM users WHERE phone = $1 AND is_active = true
    // 9. If no user: return Err(Unauthorized) — must be registered
    // 10. Create session: call create_session(pool, user_id)
    // 11. Return user_id
    todo!()
}

/// Create a new session. Returns raw token (for cookie).
pub async fn create_session(pool: &PgPool, user_id: Uuid) -> Result<String, crate::error::AppError> {
    // 1. Generate 32 random bytes via rand::OsRng (use rand::Rng::random)
    // 2. base64url encode → raw token
    // 3. SHA-256 hash → token_hash
    // 4. INSERT INTO sessions (token_hash, user_id, expires_at)
    //    expires_at = now() + 90 days
    // 5. Return raw token
    todo!()
}

/// Validate session from cookie value. Returns user_id if valid.
pub async fn validate_session(pool: &PgPool, raw_token: &str) -> Result<Uuid, crate::error::AppError> {
    // 1. SHA-256 hash the raw token
    // 2. SELECT user_id, expires_at FROM sessions WHERE token_hash = $1
    // 3. If no row: Err(Unauthorized)
    // 4. If expires_at < now(): DELETE the row, Err(Unauthorized)
    // 5. Return user_id
    todo!()
}

/// Delete session (logout).
pub async fn delete_session(pool: &PgPool, raw_token: &str) -> Result<(), crate::error::AppError> {
    // 1. SHA-256 hash the raw token
    // 2. DELETE FROM sessions WHERE token_hash = $1
    todo!()
}

/// Get current user from request cookies. Returns (user_id, is_admin, onboarded).
/// Used by server functions and route guards.
pub async fn current_user(pool: &PgPool, parts: &http::request::Parts) -> Result<CurrentUser, crate::error::AppError> {
    // 1. Extract "session" cookie from parts.headers
    // 2. Call validate_session(pool, token)
    // 3. SELECT is_admin, onboarded, name FROM users WHERE id = $1
    // 4. Return CurrentUser struct
    todo!()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CurrentUser {
    pub id: Uuid,
    pub name: String,
    pub is_admin: bool,
    pub onboarded: bool,
}
```

**Rate limiting:** Implement the rate limits from Architecture spec (1 OTP/60s per phone, 5/hour, etc.) by checking timestamps in `otp_codes` and a counter. Use database-level checks — no in-memory state.

### 2.3 Login Page (`src/pages/login.rs`) — Stories 1.2

**Create file:** `src/pages.rs` (module declaration)
```rust
pub mod login;
```

**Create file:** `src/pages/login.rs`

Two-step flow:
1. Phone input form → calls `request_otp` server function
2. OTP code input form → calls `verify_otp_code` server function

**Server functions in this file:**

```rust
#[server]
pub async fn request_otp(phone: String) -> Result<(), ServerFnError> {
    // 1. Normalize phone (phone::normalize)
    // 2. Check user exists and is_active: SELECT FROM users WHERE phone = $1 AND is_active
    // 3. Check rate limit (1 per 60s, 5 per hour)
    // 4. Create OTP (auth::create_otp)
    // 5. Send SMS (sms::send_sms) with message "Ваш код: {code}"
    // 6. Return Ok(()) — do NOT reveal whether phone is registered
    todo!()
}

#[server]
pub async fn verify_otp_code(phone: String, code: String) -> Result<bool, ServerFnError> {
    // 1. Normalize phone
    // 2. Call auth::verify_otp(pool, phone, code)
    // 3. On success: get raw session token from auth::create_session
    // 4. Set session cookie via ResponseOptions
    //    Cookie: session={token}; HttpOnly; Secure; SameSite=Strict; Max-Age=7776000; Path=/
    //    (In dev, omit Secure flag)
    // 5. Return Ok(true) for success
    // 6. Return Ok(false) for failure (wrong code, expired, etc.)
    // Note: Do NOT return Err for wrong code — that's a normal flow, not an error
    todo!()
}
```

**Component:** `LoginPage` with two states (phone entry, code entry). Use reactive signals to toggle between states. On successful verification, redirect to `/` (or `/onboarding` if not onboarded).

**Test-support feature:** Add to `Cargo.toml`:
```toml
test-support = ["ssr"]
```

Add a `#[cfg(feature = "test-support")]` server function:
```rust
#[cfg(feature = "test-support")]
#[server]
pub async fn get_test_otp(phone: String) -> Result<Option<String>, ServerFnError> {
    // SELECT code_hash FROM otp_codes WHERE phone = $1
    // This won't return the raw code (it's hashed). Instead:
    // In test mode, store the raw code in a separate column or use a fixed code.
    // Decision: Use fixed code "000000" when SAMETE_TEST_MODE=true
    todo!()
}
```

**Simpler test approach:** When `SAMETE_TEST_MODE=true` env var is set, `create_otp` always generates `"000000"`. Log a warning at startup.

### 2.4 Admin: Register Participant (`src/admin/participants.rs`) — Story 1.1

**Create file:** `src/admin/mod.rs`
```rust
pub mod participants;
```

**Create file:** `src/admin/participants.rs`

**Server function:**
```rust
#[server]
pub async fn register_participant(phone: String, name: String) -> Result<(), ServerFnError> {
    // 1. Validate admin session (current_user must have is_admin = true)
    // 2. Normalize phone
    // 3. Validate name is non-empty, trimmed
    // 4. INSERT INTO users (phone, name) VALUES ($1, $2)
    // 5. Handle unique violation (phone already exists) → InvalidInput
    todo!()
}

#[server]
pub async fn list_participants() -> Result<Vec<ParticipantSummary>, ServerFnError> {
    // 1. Validate admin session
    // 2. SELECT id, phone, name, is_active, onboarded FROM users WHERE is_admin = false
    //    ORDER BY created_at DESC
    todo!()
}
```

**Component:** Admin page with a form (phone + name) and a list of existing participants.

### 2.5 Onboarding Page (`src/pages/onboarding.rs`) — Story 1.3

**Add to `src/pages.rs`:**
```rust
pub mod onboarding;
```

**Create file:** `src/pages/onboarding.rs`

**Server function:**
```rust
#[server]
pub async fn complete_onboarding(nova_poshta_branch: String) -> Result<(), ServerFnError> {
    // 1. Validate session (must be authenticated)
    // 2. Validate nova_poshta_branch is non-empty, trimmed
    // 3. UPDATE users SET nova_poshta_branch = $1, onboarded = true WHERE id = $2
    todo!()
}
```

**Component:** Single form collecting Nova Poshta branch. On submit, redirect to `/`.

### 2.6 App.rs: Auth Routes + Guard

**Update `src/app.rs`:**

Add an auth resource that fetches current user state. Use this for route guards.

```rust
#[server]
pub async fn get_current_user() -> Result<Option<auth::CurrentUser>, ServerFnError> {
    // Try to read session cookie and validate
    // Return None if no session / invalid / expired
    // Return Some(CurrentUser) if valid
    todo!()
}
```

**Route structure after Phase 2:**
- `/login` → `LoginPage` (no auth required)
- `/onboarding` → `OnboardingPage` (auth required, only shown if `!onboarded`)
- `/` → `HomePage` (auth required, redirect to `/onboarding` if `!onboarded`)
- `/admin/participants` → `ParticipantsPage` (auth + admin required)

**Update `src/lib.rs`** to add module declarations:
```rust
pub mod pages;
pub mod admin;

#[cfg(feature = "ssr")]
pub mod auth;
#[cfg(feature = "ssr")]
pub mod sms;
```

### Phase 2 Verification Gates

```bash
# Gate 1: SSR compiles
cargo build --features ssr
# REQUIRED: exits 0

# Gate 2: WASM compiles
cargo build --features hydrate --target wasm32-unknown-unknown
# REQUIRED: exits 0

# Gate 3: Clippy
cargo clippy --features ssr -- -D warnings
# REQUIRED: exits 0

# Gate 4: SQLx prepare
cargo sqlx prepare --workspace
# REQUIRED: exits 0, .sqlx/ updated

# Gate 5: No scattered auth checks
grep -rn "is_admin" src/pages/ --include="*.rs"
# REQUIRED: zero matches (admin checks live in admin/ modules and auth.rs only)

# Gate 6: No raw SQL outside sqlx::query!()
grep -rn "execute\|fetch" src/ --include="*.rs" | grep -v "sqlx::query" | grep -v "test" | grep -v ".execute(pool)"
# Advisory: review any matches for raw string SQL

# Gate 7: E2E — Epic 1 tests pass
SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end -- --grep "Epic 1"
# REQUIRED: all Story 1.1, 1.2, 1.3 tests pass
```

---

## Phase 3: Season Lifecycle (Epics 4, 2)

Implements: season creation, launch, enrollment, ready-confirm. After this phase, the full pre-assignment participant flow works.

### Prerequisites

- Phase 2 complete and all gates pass
- Admin user exists in database

### 3.1 Admin: Season Management (`src/admin/season.rs`) — Stories 4.1, 4.2

**Add to `src/admin/mod.rs`:**
```rust
pub mod season;
pub mod dashboard;
```

**Create file:** `src/admin/season.rs`

**Server functions:**

```rust
#[server]
pub async fn create_season(
    signup_deadline: String,  // ISO 8601 datetime string
    confirm_deadline: String,
    theme: Option<String>,
) -> Result<(), ServerFnError> {
    // 1. Validate admin session
    // 2. Parse deadlines to OffsetDateTime
    // 3. Validate signup_deadline < confirm_deadline
    // 4. Validate both deadlines are in the future
    // 5. INSERT INTO seasons (phase, signup_deadline, confirm_deadline, theme)
    //    VALUES ('signup', $1, $2, $3)
    // 6. The one_active_season index prevents creating a second active season
    //    Handle unique violation → InvalidInput("an active season already exists")
    todo!()
}

#[server]
pub async fn advance_season() -> Result<(), ServerFnError> {
    // 1. Validate admin session
    // 2. SELECT id, phase FROM seasons WHERE phase NOT IN ('complete', 'cancelled')
    // 3. If no active season: NotFound
    // 4. Call Phase::try_advance() on the current phase
    // 5. UPDATE seasons SET phase = $1 WHERE id = $2
    todo!()
}

#[server]
pub async fn cancel_season() -> Result<(), ServerFnError> {
    // 1. Validate admin session
    // 2. SELECT active season
    // 3. Call Phase::cancel()
    // 4. UPDATE seasons SET phase = 'cancelled' WHERE id = $1
    todo!()
}
```

**Note on Story 4.2 (Launch):** The Architecture spec says "Season is not open for enrollment until the organizer explicitly launches it." The phase model handles this: a season starts in `signup` phase. The organizer creates it (it exists but could be in a pre-launch state). However, the spec's phase enum starts at `signup` which IS the enrollment-open state. Resolution: creation itself IS the launch. The `create_season` function creates a season in `signup` phase. The SMS notification (Story 5.3) is triggered separately from `admin/sms.rs` in Phase 5. Until Phase 5, the season is visible to participants but they won't receive an SMS about it.

### 3.2 Season Enrollment (`src/pages/season.rs`) — Story 2.1

**Add to `src/pages.rs`:**
```rust
pub mod season;
pub mod home;
```

**Create file:** `src/pages/season.rs`

**Server functions:**

```rust
#[server]
pub async fn get_season_info() -> Result<Option<SeasonInfo>, ServerFnError> {
    // 1. Validate session
    // 2. SELECT from seasons WHERE phase NOT IN ('complete', 'cancelled')
    // 3. If no active season: return None
    // 4. Check enrollment status for current user:
    //    SELECT FROM enrollments WHERE user_id = $1 AND season_id = $2
    // 5. Return SeasonInfo { phase, signup_deadline, confirm_deadline, theme,
    //    is_enrolled, is_confirmed, assignment (if released) }
    todo!()
}

#[server]
pub async fn enroll_in_season(nova_poshta_branch: String) -> Result<(), ServerFnError> {
    // 1. Validate session
    // 2. Get active season, verify phase is Signup
    // 3. Verify signup_deadline > now()
    // 4. Validate nova_poshta_branch non-empty
    // 5. UPDATE users SET nova_poshta_branch = $1 WHERE id = $2 (update their default)
    // 6. INSERT INTO enrollments (user_id, season_id, nova_poshta_branch)
    //    Handle duplicate → already enrolled
    todo!()
}
```

### 3.3 Confirm Ready (same file) — Story 2.2

**Server function in `src/pages/season.rs`:**

```rust
#[server]
pub async fn confirm_ready() -> Result<(), ServerFnError> {
    // 1. Validate session
    // 2. Get active season, verify phase is Creating or Confirming
    // 3. Verify confirm_deadline > now()
    // 4. UPDATE enrollments SET confirmed_ready = true
    //    WHERE user_id = $1 AND season_id = $2 AND confirmed_ready = false
    // 5. If no row updated: either not enrolled or already confirmed
    todo!()
}
```

### 3.4 Home Page (`src/pages/home.rs`) — Participant State View

**Create file:** `src/pages/home.rs`

Single component that shows the right content based on (season phase, participant state). This is the table from Architecture spec's "Home Screen" section.

**Server function:**

```rust
#[server]
pub async fn get_home_state() -> Result<HomeState, ServerFnError> {
    // 1. Validate session → get user_id
    // 2. Get active season (if any)
    // 3. If no active season: return HomeState::NoSeason
    // 4. Check enrollment + confirmation + assignment + receipt status
    // 5. Return the appropriate variant
    todo!()
}
```

`HomeState` is a serializable enum matching the home screen table:
```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum HomeState {
    NoSeason,
    SignupOpen { deadline: String, theme: Option<String> },
    Enrolled { creation_start: String },
    Creating { confirm_deadline: String },
    Confirming { confirm_deadline: String },
    Confirmed { assignment_eta: String },
    Assigning,
    Assigned { recipient_name: String, recipient_phone: String, recipient_branch: String },
    Receiving,
    ReceiptConfirmed,
    Complete,
}
```

The component matches on this enum and renders the appropriate content. One component, one match, no scattered conditionals.

### 3.5 Admin Dashboard (`src/admin/dashboard.rs`)

**Create file:** `src/admin/dashboard.rs`

Shows season health: enrolled count, confirmed count, phase, action buttons. The dashboard table from Architecture spec's "Admin Dashboard" section.

**Server function:**

```rust
#[server]
pub async fn get_dashboard() -> Result<DashboardState, ServerFnError> {
    // 1. Validate admin session
    // 2. Get active season (if any)
    // 3. COUNT enrollments, COUNT confirmed, etc.
    // 4. Return dashboard state
    todo!()
}
```

### 3.6 Route Updates

**Update `src/app.rs` routes:**
- `/` → `HomePage` (auth required)
- `/season` → `SeasonPage` (auth required)
- `/admin` → `Dashboard` (auth + admin required)
- `/admin/season` → `SeasonManage` (auth + admin required)

### Phase 3 Verification Gates

```bash
# Gate 1: SSR + WASM compile
cargo build --features ssr && cargo build --features hydrate --target wasm32-unknown-unknown
# REQUIRED: exits 0

# Gate 2: Clippy
cargo clippy --features ssr -- -D warnings
# REQUIRED: exits 0

# Gate 3: Phase transition is centralized
grep -rn "phase" src/ --include="*.rs" | grep -i "update\|set" | grep -v "types.rs" | grep -v "test" | grep -v "admin/season.rs"
# REQUIRED: no direct phase manipulation outside admin/season.rs (transition logic is in types.rs, DB update is in admin/season.rs)

# Gate 4: SQLx prepare
cargo sqlx prepare --workspace
# REQUIRED: exits 0

# Gate 5: Unit tests
cargo test
# REQUIRED: all pass

# Gate 6: E2E — Epic 4 and enrollment/confirm tests pass
SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end -- --grep "Epic 4|Story 2.1|Story 2.2"
# REQUIRED: season creation, enrollment, confirm-ready tests pass
```

---

## Phase 4: Assignment (Epic 3)

Implements: cohort splitting, Hamiltonian cycle generation, social-awareness scoring, organizer override. Pure algorithm + admin UI.

### Prerequisites

- Phase 3 complete
- Understand the algorithm from Architecture spec's "Assignment Algorithm" section

### 4.1 Assignment Algorithm (`src/assignment.rs`)

**Create file:** `src/assignment.rs` — PURE LOGIC, no database, no framework dependencies. SSR-only.

This module is the algorithm from Architecture spec. It takes data in, returns assignments out. No side effects.

**Add to `src/lib.rs`:**
```rust
#[cfg(feature = "ssr")]
pub mod assignment;
```

**Types:**

```rust
use uuid::Uuid;

/// Input to the assignment algorithm.
pub struct AssignmentInput {
    /// Confirmed participant IDs.
    pub participants: Vec<Uuid>,
    /// Social weight between two participants. Higher = stronger existing connection.
    /// Keyed by (min(a,b), max(a,b)) for canonical ordering.
    pub social_weights: std::collections::HashMap<(Uuid, Uuid), u32>,
}

/// A single cohort's assignment cycle.
pub struct Cycle {
    /// Ordered list of participant IDs forming the cycle.
    /// participants[0] sends to participants[1], ..., participants[N-1] sends to participants[0].
    pub participants: Vec<Uuid>,
    /// Total social weight score (lower is better).
    pub score: u32,
}

/// Full assignment result.
pub struct AssignmentResult {
    pub cohorts: Vec<Cycle>,
}
```

**Functions:**

```rust
/// Split N participants into cohorts of 11-15.
/// For N <= 15: single cohort.
/// For N > 15: find the partition minimizing max deviation from mean group size.
/// All groups must be in [3, 15] range (organizer approves small cohorts).
pub fn split_cohorts(participants: &[Uuid]) -> Vec<Vec<Uuid>> {
    todo!()
}

/// Generate a Hamiltonian cycle for a cohort that minimizes total social weight.
/// Backtracking DFS with greedy heuristic. Multiple random restarts, keep best.
pub fn generate_cycle(
    participants: &[Uuid],
    social_weights: &std::collections::HashMap<(Uuid, Uuid), u32>,
    attempts: usize,
) -> Cycle {
    todo!()
}

/// Top-level: split into cohorts, generate cycle for each.
pub fn generate_assignments(input: AssignmentInput) -> AssignmentResult {
    let cohorts = split_cohorts(&input.participants);
    let cycles = cohorts
        .iter()
        .map(|cohort| generate_cycle(cohort, &input.social_weights, 100))
        .collect();
    AssignmentResult { cohorts: cycles }
}

/// Compute social weight between two participants.
/// Canonical key: (min, max).
pub fn weight_key(a: Uuid, b: Uuid) -> (Uuid, Uuid) {
    if a < b { (a, b) } else { (b, a) }
}

/// Validate that a set of assignments forms valid cycles.
/// Every participant sends to exactly one, receives from exactly one.
/// Each cohort forms a single connected loop.
pub fn validate_cycles(result: &AssignmentResult) -> Result<(), String> {
    todo!()
}
```

**Unit tests — EXTENSIVE:**

- 3 participants → single cycle of length 3
- 15 participants → single cycle of length 15
- 25 participants → two cohorts (13+12 or 12+13)
- 30 participants → two cohorts of 15
- Social weights respected: with two participants having high weight, they should be non-adjacent in the cycle (when possible)
- `validate_cycles` catches invalid topologies
- Cohort splitting edge cases: N=3, N=11, N=16, N=31

### 4.2 Story 3.2: Social Weight Computation

**In `src/assignment.rs`:**

```rust
/// Build the social weight matrix from DB data.
/// Called by admin/assignments.rs before running the algorithm.
pub fn build_weight_matrix(
    group_memberships: &[(Uuid, Uuid, u32)],  // (user_id, group_id, group_weight)
    past_pairings: &[(Uuid, Uuid)],            // (sender, recipient) from all past seasons
) -> std::collections::HashMap<(Uuid, Uuid), u32> {
    // For each pair of users:
    // - Sum weights of groups they share
    // - +1 for each past pairing (either direction)
    // Return the weight map
    todo!()
}
```

### 4.3 Admin: Assignments (`src/admin/assignments.rs`) — Story 3.3

**Add to `src/admin/mod.rs`:**
```rust
pub mod assignments;
```

**Create file:** `src/admin/assignments.rs`

**Server functions:**

```rust
#[server]
pub async fn generate_assignments() -> Result<AssignmentPreview, ServerFnError> {
    // 1. Validate admin session
    // 2. Get active season (must be in Assigning phase)
    // 3. Get confirmed participants: SELECT user_id FROM enrollments
    //    WHERE season_id = $1 AND confirmed_ready = true
    // 4. Build social weight matrix from known_groups + past_pairings
    // 5. Run assignment::generate_assignments()
    // 6. Store assignments in DB (released = false)
    // 7. Return preview with cycle visualization data
    todo!()
}

#[server]
pub async fn swap_assignment(
    season_id: String,
    sender_a: String,
    sender_b: String,
) -> Result<(), ServerFnError> {
    // 1. Validate admin session
    // 2. Parse UUIDs
    // 3. Load both assignments
    // 4. Swap recipients: A's recipient becomes B's, B's becomes A's
    // 5. Validate the resulting cycles still form valid topology
    //    (call assignment::validate_cycles)
    // 6. If invalid: return Err(InvalidInput("swap breaks cycle topology"))
    // 7. UPDATE both assignment rows
    todo!()
}

#[server]
pub async fn release_assignments() -> Result<(), ServerFnError> {
    // 1. Validate admin session
    // 2. Get active season (must be in Assigning or Sending phase)
    // 3. UPDATE assignments SET released = true WHERE season_id = $1
    // 4. Advance phase to Sending if still in Assigning
    todo!()
}
```

### Phase 4 Verification Gates

```bash
# Gate 1: Compile
cargo build --features ssr
# REQUIRED: exits 0

# Gate 2: Clippy
cargo clippy --features ssr -- -D warnings
# REQUIRED: exits 0

# Gate 3: Algorithm tests
cargo test -- assignment
# REQUIRED: all pass, including cycle validity, cohort splitting, social weight tests

# Gate 4: Algorithm is pure
grep -rn "sqlx\|PgPool\|use_context\|leptos" src/assignment.rs
# REQUIRED: zero matches (algorithm has no framework/DB dependencies)

# Gate 5: SQLx prepare
cargo sqlx prepare --workspace
# REQUIRED: exits 0

# Gate 6: E2E — Epic 3 tests pass
SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end -- --grep "Epic 3"
# REQUIRED: assignment generation, override, release tests pass
```

---

## Phase 5: Delivery + SMS (Epics 2, 5)

Implements: assignment view, receipt confirmation, all SMS notification batches.

### Prerequisites

- Phase 4 complete
- SMS client functional (or dry-run mode)

### 5.1 Stories 2.3 + 5.1: Assignment View + Notification SMS

**Update `src/pages/home.rs`** (or `season.rs`):

The `HomeState::Assigned` variant already carries recipient details. The component shows:
- Recipient's real name
- Recipient's phone number
- Recipient's Nova Poshta branch

This data comes from the `get_home_state` server function (Phase 3), which now includes assignment lookup:

```rust
// In get_home_state, when phase is Sending and assignment is released:
// SELECT u.name, u.phone, e.nova_poshta_branch
// FROM assignments a
// JOIN users u ON u.id = a.recipient_id
// JOIN enrollments e ON e.user_id = a.recipient_id AND e.season_id = a.season_id
// WHERE a.sender_id = $1 AND a.season_id = $2 AND a.released = true
```

### 5.2 Stories 2.4 + 5.2: Receipt Confirmation + Nudge SMS

**Server function in `src/pages/season.rs`:**

```rust
#[server]
pub async fn confirm_receipt(received: bool, note: Option<String>) -> Result<(), ServerFnError> {
    // 1. Validate session → user_id
    // 2. Get active season (must be in Receiving phase)
    // 3. Find assignment where user is RECIPIENT:
    //    SELECT a.id FROM assignments a
    //    WHERE a.recipient_id = $1 AND a.season_id = $2
    // 4. INSERT INTO receipts (assignment_id, received, note) VALUES ($1, $2, $3)
    //    Handle duplicate → already confirmed
    todo!()
}
```

### 5.3 Admin: SMS Batch Triggers (`src/admin/sms.rs`) — Stories 5.1-5.4

**Add to `src/admin/mod.rs`:**
```rust
pub mod sms;
```

**Create file:** `src/admin/sms.rs`

**Server functions — one per SMS type:**

```rust
/// Story 5.3: Season-open notification to ALL active users
#[server]
pub async fn send_season_open_sms() -> Result<SmsReport, ServerFnError> {
    // 1. Validate admin session
    // 2. Get active season
    // 3. SELECT phone FROM users WHERE is_active = true
    // 4. Send SMS to each: "Новий сезон Mail Club відкрито! Зайди в додаток для реєстрації."
    // 5. Return report: sent count, failed count, failed phones
    todo!()
}

/// Story 5.1: Assignment notification to released participants
#[server]
pub async fn send_assignment_sms() -> Result<SmsReport, ServerFnError> {
    // 1. Validate admin session
    // 2. Get active season (Sending phase)
    // 3. SELECT u.phone FROM assignments a JOIN users u ON u.id = a.sender_id
    //    WHERE a.season_id = $1 AND a.released = true
    // 4. Send SMS: "Твоє призначення готове! Зайди в додаток щоб побачити адресата."
    // 5. Return report
    todo!()
}

/// Story 5.4: Pre-deadline nudge to non-confirmers
#[server]
pub async fn send_confirm_nudge_sms() -> Result<SmsReport, ServerFnError> {
    // 1. Validate admin session
    // 2. Get active season (Confirming phase)
    // 3. SELECT u.phone FROM enrollments e JOIN users u ON u.id = e.user_id
    //    WHERE e.season_id = $1 AND e.confirmed_ready = false
    // 4. Send SMS: "Нагадування: підтверди готовність листа до [deadline]."
    // 5. Return report
    todo!()
}

/// Story 5.2: Receipt nudge to non-responders
#[server]
pub async fn send_receipt_nudge_sms() -> Result<SmsReport, ServerFnError> {
    // 1. Validate admin session
    // 2. Get active season (Receiving phase)
    // 3. SELECT u.phone FROM assignments a
    //    JOIN users u ON u.id = a.recipient_id
    //    LEFT JOIN receipts r ON r.assignment_id = a.id
    //    WHERE a.season_id = $1 AND a.released = true AND r.assignment_id IS NULL
    // 4. Send SMS: "Ти отримав/ла лист? Підтверди в додатку."
    // 5. Return report
    todo!()
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SmsReport {
    pub sent: u32,
    pub failed: u32,
    pub failed_phones: Vec<String>,
}
```

**All SMS sends are inline (await per message).** At ≤50 users this is fast enough. The organizer sees a "sending..." state and then the report.

### 5.4 Route Updates

Add `/admin/sms` → SMS batch trigger page.

### Phase 5 Verification Gates

```bash
# Gate 1: Compile
cargo build --features ssr
# REQUIRED: exits 0

# Gate 2: Clippy
cargo clippy --features ssr -- -D warnings
# REQUIRED: exits 0

# Gate 3: SMS messages are Ukrainian
grep -rn "send_sms\|Ваш код\|Новий сезон\|Нагадування\|Твоє призначення" src/ --include="*.rs" | head -20
# ADVISORY: verify all user-facing SMS text is in Ukrainian

# Gate 4: All SMS sends go through sms::send_sms
grep -rn "turbosms\|api.turbosms" src/ --include="*.rs" | grep -v "sms.rs"
# REQUIRED: zero matches (all SMS goes through sms.rs)

# Gate 5: SQLx prepare
cargo sqlx prepare --workspace
# REQUIRED: exits 0

# Gate 6: E2E — Epic 2 (assignment/receipt) + Epic 5 tests pass
SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end -- --grep "Epic 2|Epic 5"
# REQUIRED: assignment view, receipt, all SMS trigger tests pass
```

---

## Phase 6: Account Management (Epic 6)

### Prerequisites

- Phase 5 complete

### 6.1 Story 6.1: Deactivate Account

**Update `src/admin/participants.rs`:**

```rust
#[server]
pub async fn deactivate_participant(user_id: String) -> Result<(), ServerFnError> {
    // 1. Validate admin session
    // 2. Parse user_id UUID
    // 3. UPDATE users SET is_active = false WHERE id = $1
    // 4. DELETE FROM sessions WHERE user_id = $1 (revoke all sessions)
    todo!()
}
```

**Effects of deactivation (enforced by existing code, verify):**
- `is_active = false` → login rejected in `request_otp` (Phase 2)
- Sessions deleted → existing sessions invalidated
- `is_active = false` → excluded from `send_season_open_sms` (Phase 5)
- Cannot enroll: enrollment checks `is_active` (add this check if not present in Phase 3)

**Verification:**
```bash
# Gate: Deactivation blocks all access
grep -rn "is_active" src/ --include="*.rs"
# REQUIRED: appears in user lookup queries (auth, enrollment, SMS recipient lists)
```

### Phase 6 Verification Gates

```bash
# Gate 1: Full compile
cargo build --features ssr && cargo build --features hydrate --target wasm32-unknown-unknown
# REQUIRED: exits 0

# Gate 2: Full clippy
cargo clippy --features ssr -- -D warnings
# REQUIRED: exits 0

# Gate 3: Full unit test suite
cargo test
# REQUIRED: all pass

# Gate 4: SQLx prepare
cargo sqlx prepare --workspace
# REQUIRED: exits 0

# Gate 5: E2E — ALL tests pass (full regression)
SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end
# REQUIRED: all epic tests pass — full season lifecycle E2E
```

---

## Post-Implementation: Past Pairings

After a season completes, past pairings must be recorded for future social-awareness. Add to the `advance_season` function:

When advancing from `Receiving` → `Complete`:
```sql
INSERT INTO past_pairings (sender_id, recipient_id, season_id)
SELECT sender_id, recipient_id, season_id FROM assignments
WHERE season_id = $1
ON CONFLICT DO NOTHING;
```

This is triggered automatically during phase advancement. No separate action needed.

---

## Forbidden Patterns

### BANNED: Scattered phase checks
```rust
// BANNED — phase logic must live in Phase::try_advance / Phase::cancel
if phase == Phase::Signup {
    // do something
} else if phase == Phase::Creating {
    // ...
}
```
Phase transition logic is in `types.rs`. Server functions call `try_advance()` or `cancel()`. They do NOT implement transition rules themselves.

### BANNED: Raw SQL strings outside sqlx::query!()
```rust
// BANNED
pool.execute("UPDATE seasons SET phase = 'creating'").await;
```
All queries use `sqlx::query!()` or `sqlx::query_as!()` for compile-time checking.

### BANNED: Unwrap on user input
```rust
// BANNED
let uuid = Uuid::parse_str(&input).unwrap();
```
User input parsing returns `Result`. Use `?` with `AppError::InvalidInput`.

### BANNED: Boolean flags for states
```rust
// BANNED — use Phase enum
struct Season {
    is_active: bool,
    is_complete: bool,
    is_cancelled: bool,
}
```

### BANNED: Auth checks only at route level
```rust
// BANNED — every server function must validate session independently
#[server]
pub async fn admin_action() -> Result<(), ServerFnError> {
    // Missing: validate_admin_session()
    // This function is callable via direct API request
}
```

### BANNED: In-memory state for rate limiting or session data
```rust
// BANNED — all state in Postgres
static RATE_LIMITS: LazyLock<Mutex<HashMap<String, Instant>>> = ...;
```

### BANNED: Generic trait hierarchies
```rust
// BANNED — violates tension resolution rule
trait Repository<T> { ... }
impl Repository<User> for PgRepo { ... }
impl Repository<Season> for PgRepo { ... }
```
Direct `sqlx::query!()` calls. No abstraction layer over the database.

### BANNED: `#[allow(clippy::...)]` without explanatory comment
```rust
// BANNED
#[allow(clippy::too_many_lines)]
fn big_function() { ... }

// ALLOWED (with reason)
#[allow(clippy::too_many_lines)] // View macro generates many lines; splitting would fragment the template
fn big_component() -> impl IntoView { ... }
```

---

## Definition of Done

Every item is binary pass/fail. All must pass.

1. `cargo build --features ssr` exits 0 with zero warnings
2. `cargo build --features hydrate --target wasm32-unknown-unknown` exits 0
3. `cargo clippy --features ssr -- -D warnings` exits 0
4. `cargo test` — all tests pass
5. `cargo sqlx prepare --workspace` exits 0 and `.sqlx/` is committed
6. `cargo fmt --check` exits 0
7. All migration files exist and `sqlx migrate run` succeeds on clean database
8. Phase enum unit tests cover every valid and invalid transition
9. Phone normalization tests cover all input formats from spec
10. Assignment algorithm tests verify cycle validity for N=3, N=15, N=25
11. No `todo!()` remains in production code (search: `grep -rn "todo!()" src/ | grep -v test`)
12. Every `#[server]` function validates auth at the top
13. Every admin server function checks `is_admin`
14. All SMS text is in Ukrainian
15. No in-memory state — all data in Postgres
16. Pre-commit hooks pass: `pre-commit run --all-files`
