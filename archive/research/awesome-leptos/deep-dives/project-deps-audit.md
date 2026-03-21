# Project Dependencies Audit

**Codebase**: `/Users/ryzhakar/pp/same-te-mail-club`
**Audit Date**: 2026-03-19
**Leptos Version**: 0.8 (stable)

---

## Leptos Ecosystem Versions

| Crate | Specified | Resolved | Status |
|-------|-----------|----------|--------|
| `leptos` | `0.8` | `0.8.17` | ✓ latest 0.8.x |
| `leptos_router` | `0.8` | `0.8.12` | ✓ latest 0.8.x |
| `leptos_meta` | `0.8` | `0.8.6` | ✓ latest 0.8.x |
| `leptos_axum` | `0.8` (SSR only) | `0.8.8` | ✓ latest 0.8.x |
| `leptos_i18n` | `0.6` | `0.6.1` | ✓ fully enabled, configured |

**Transitive:**
- `leptos-use`: `0.18.3` (reactive utilities, pulled by leptos_i18n)

---

## Full Dependency List

### Shared (All Targets)

| Crate | Version | Category | Purpose |
|-------|---------|----------|---------|
| `serde` | `1.0.228` | Serialization | Struct ser/de for client↔server communication |
| `serde_json` | `1.0.149` | Serialization | JSON handling |
| `uuid` | `1.22.0` | Utilities | Unique IDs (v4), with `serde` feature enabled |
| `time` | `0.3.47` | Utilities | Temporal types (timestamps, formatting), with `serde`, `formatting`, `macros` features |
| `tracing` | `0.1.44` | Logging | Core tracing API (shared facade) |
| `thiserror` | `2.0.18` | Error handling | Error trait derives and context macros |

### SSR-Only (Server Runtime)

| Crate | Version | Features | Purpose |
|-------|---------|----------|---------|
| `leptos_axum` | `0.8.8` | — | SSR integration layer for Axum |
| `axum` | `0.8.8` | — | HTTP framework (routing, handlers) |
| `tokio` | `1.50.0` | `["full"]` | Async runtime (all features: sync, time, io, macros, etc.) |
| `tower-http` | `0.6.8` | `["compression-br", "trace"]` | Middleware (Brotli compression, tracing) |
| `tracing-subscriber` | `0.3.22` | `["env-filter", "fmt"]` | Tracing setup (env filtering, formatted logging) |
| `sqlx` | `0.8.6` | `["runtime-tokio", "postgres", "macros", "uuid", "time"]` | Async SQL toolkit (Tokio runtime, PostgreSQL driver, query macros, type integration) |
| `reqwest` | `0.13.2` | `["json"]` | HTTP client (for outbound SMS, third-party APIs) |
| `phonenumber` | `0.3.9+9.0.21` | — | International phone number parsing (E.164 format) |
| `rand` | `0.8.5` | — | Random number generation (OTP codes) |
| `blake2` | `0.10.6` | — | BLAKE2 hash (password/OTP hashing) |
| `sha2` | `0.10.9` | — | SHA-256 hash (crypto primitives) |
| `base64` | `0.22.1` | — | Encoding/decoding (OTP handling) |
| `http` | `1.0.0` | — | HTTP types (status, headers, methods) |
| `anyhow` | `1.0.x` | — | Error context and ad-hoc error handling |

### Hydrate-Only (Client WASM)

| Crate | Version | Purpose |
|--------|---------|---------|
| `wasm-bindgen` | `0.2.114` | JavaScript/WASM interop |
| `console_error_panic_hook` | `0.1.7` | Better panic logging in browser console (dev aid) |

---

## Feature Flags Configuration

### Default Features: None
```toml
[features]
default = []
```

### Feature Matrix

#### `hydrate` (Client-Side WASM)
Enabled for client bundle only.
```
- leptos/hydrate
- console_error_panic_hook
- wasm-bindgen
- uuid/js                  (WASM-compatible UUID v4)
- leptos_i18n/hydrate      (i18n WASM support)
```

#### `ssr` (Server Runtime)
Enabled for binary build via `cargo-leptos --bin-features ssr`.
```
- leptos/ssr
- leptos_meta/ssr
- leptos_router/ssr
- leptos_axum
- axum
- tokio
- tower-http
- tracing-subscriber
- sqlx
- reqwest
- phonenumber
- rand
- blake2
- sha2
- base64
- http
- anyhow
- leptos_i18n/ssr          (server-side string translation)
- leptos_i18n/axum         (Axum middleware for i18n)
```

---

## Cargo.toml Metadata

### [package.metadata.leptos]
```toml
output-name = "samete"
site-root = "target/site"
site-pkg-dir = "pkg"
style-file = "style/main.css"
tailwind-input-file = "style/tailwind.css"
assets-dir = "public"
site-addr = "127.0.0.1:3000"
reload-port = 3001
end2end-cmd = "npx playwright test"
end2end-dir = "end2end"
browserquery = "defaults"
env = "DEV"
bin-features = ["ssr"]
bin-default-features = false
lib-features = ["hydrate"]
lib-default-features = false
lib-profile-release = "wasm-release"
```

**Key Settings:**
- SSR binary uses `ssr` feature; client lib uses `hydrate` feature
- Hot reload on port 3001
- Tailwind v4 input configured
- WASM release profile uses aggressive optimization (`opt-level = 'z'`, LTO, single codegen unit)

### [package.metadata.leptos-i18n]
```toml
default = "uk"
locales = ["uk"]
```

**Localization:**
- Single locale: Ukrainian (`uk`)
- Default locale is `uk`
- No fallback or multi-locale setup

---

## Linting Configuration

### Rust Lints
```toml
[lints.rust]
unsafe_code = "forbid"
```
No `unsafe` code is permitted. All unsafe blocks will cause compilation failure.

### Clippy Lints
```toml
[lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
```

**Severity:** All clippy findings (all + pedantic) are **errors**, not warnings. No exceptions without explicit `#[allow]` comments justifying the exception.

---

## Build Profiles

### [profile.release]
```toml
strip = true
lto = "thin"
panic = "abort"
```

**Server binary optimizations:**
- Strip symbols
- Thin LTO (faster compilation than full LTO, still aggressive)
- Abort on panic (smaller binary)

### [profile.wasm-release]
```toml
inherits = "release"
opt-level = 'z'
lto = true
codegen-units = 1
```

**WASM client optimizations (inherits release, then overrides):**
- `opt-level = 'z'`: Smallest possible binary size
- Full LTO: Maximum optimization (slower build, smaller output)
- Single codegen unit: Sequential compilation, better optimization
- **Purpose:** Minimize WASM download size for hydration critical path

---

## Tracing Setup

### Tracing Facade
- **Core API**: `tracing 0.1.44` (shared, no SSR feature gate)
- **Subscriber**: `tracing-subscriber 0.3.22` (SSR only)

### Current Configuration (src/main.rs)

Server-side tracing initialization:
```rust
use tracing_subscriber::prelude::*;

tracing_subscriber::registry()
    .with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "samete=info,tower_http=info".into()),
    )
    .with(tracing_subscriber::fmt::layer())
    .init();
```

**Details:**
- Uses `tracing_subscriber::registry()` (modern layered approach)
- Environment filter: respects `RUST_LOG` env var, falls back to `samete=info,tower_http=info`
- Formatter: structured text output (default fmt layer)
- Initialization: global static subscriber (only called once in main)

### Actual Usage (7 call sites)
```
src/main.rs:1
  tracing::info!("listening on http://{}");

src/pages/login.rs:1
  tracing::warn!("SMS send failed for {}: {}");

src/admin/sms.rs:4
  tracing::warn!(phone = %phone, error = %e, "SMS send failed");
  tracing::warn!(phone = %target.phone, error = %e, "SMS send failed");
  tracing::warn!(phone = %phone, error = %e, "SMS send failed");
  tracing::warn!(phone = %phone, error = %e, "SMS send failed");

src/sms.rs:1
  tracing::info!(
    phone = %phone,
    duration = ?duration,
    status_code = status.as_u16(),
    "sent SMS"
  );
```

**Pattern**: Structured logging with field-value pairs using `%` (display), `?` (debug), and `@` (span) sigils.

### Client-Side Tracing
**Not configured.** The `tracing` crate exists in dependencies but is not used client-side. Browser console fallback would require separate subscriber setup (not implemented).

---

## i18n (Internationalization) Setup

### Dependency
- **leptos_i18n 0.6.1** with no optional features
  - Features enabled via SSR/hydrate gates: `leptos_i18n/ssr`, `leptos_i18n/axum`, `leptos_i18n/hydrate`

### Configuration
```toml
[package.metadata.leptos-i18n]
default = "uk"
locales = ["uk"]
```

**Scope:**
- Single active locale: Ukrainian (`uk`)
- Default: Ukrainian
- No multi-locale switching or fallback chain defined

### Current Status
- **Enabled**: Yes, crate is included, features are gated
- **Implemented**: Minimal (single locale only, no UI for language selection)
- **Expansion headroom**: Moderate (add more locale codes to `locales` list, add UI toggle, would require no structural changes)

---

## Database

### SQLx Configuration
- **Driver**: PostgreSQL (`sqlx` feature `postgres`)
- **Async Runtime**: Tokio (`sqlx` feature `runtime-tokio`)
- **Compile-time Verification**: `sqlx::query!()` macros supported
- **Type Integration**:
  - `uuid` feature: seamless `uuid::Uuid` → PostgreSQL `UUID`
  - `time` feature: seamless `time::*` types → PostgreSQL `timestamp`

### Migration System
- Managed via `sqlx-cli` (separate tool, not in Cargo.toml)
- Migrations stored in `migrations/` directory (SQLx convention)
- Offline mode: `.sqlx/` generated files committed (allows `cargo sqlx prepare` without live DB)

---

## Version Constraints & Compatibility

### Hard Pins (Will Break if Changed)

| Constraint | Impact | Risk Level |
|-----------|--------|-----------|
| Leptos 0.8.x required | All framework logic, routing, signals, SSR | **Critical** — version incompatible |
| Axum 0.8 (tracks Leptos) | HTTP routing, middleware tower-http | **Critical** — must match Leptos |
| Tokio 1.x required | Async runtime, task spawning | **Critical** — 2.x is breaking |
| SQLx 0.8 (tracks Leptos) | SQL macros, type mapping | **Critical** — sqlx offline data keyed to version |
| Serde 1.x required | Shared struct ser/de | **High** — serialization contract |

### Semi-Flexible

| Constraint | Impact | Risk Level |
|-----------|--------|-----------|
| Tracing 0.1.x | Logging, no breaking changes expected | **Low** — stable API |
| Reqwest 0.13 | HTTP client for SMS delivery | **Low** — minor version can bump safely |
| UUID 1.x | ID generation, stable API | **Low** — minor version can bump safely |
| Time 0.3.x | Timestamp handling, stable API | **Low** — minor version can bump safely |

### Adding New Dependencies

When adding new Leptos crates, ensure they target **0.8.x**:
- `leptos_actix` (if switching from Axum)
- `leptos_axum` — already at 0.8.8 ✓
- Community crates (check GitHub branches for 0.8 support)

**Avoid:**
- Crates pinned to Leptos 0.7 or 0.9
- Incompatible async runtimes (use Tokio, not smol/async-std without adapters)

---

## Justfile Commands

All commands in `/Users/ryzhakar/pp/same-te-mail-club/justfile`:

| Command | Purpose | Dependencies |
|---------|---------|--------------|
| `just dev` | Start dev server with hot reload | `cargo leptos watch` |
| `just test` | Run unit tests | `cargo test` |
| `just clippy` | Run clippy on all targets | `cargo clippy --all-targets` |
| `just _kill-stale` | Kill processes on port 3000 | `lsof`, `xargs`, `kill` |
| `just e2e` | Full E2E pipeline | `_kill-stale`, `db-reset`, `db-seed`, `cargo leptos end-to-end` (Playwright) |
| `just e2e-single pattern` | Single E2E test by grep | Requires `just e2e` to have run first (DB state dependency) |
| `just e2e-rerun` | Re-run E2E without DB reset | `_kill-stale`, assumes DB is in correct state |
| `just build` | Release build | `cargo leptos build --release` |
| `just db-reset` | Drop, create, migrate database | `sqlx`, PostgreSQL CLI |
| `just db-seed` | Insert test admin user | `psql`, `seed/test_admin.sql` |
| `just db-migrate` | Run pending migrations | `sqlx` |
| `just db-new name` | Create a new migration | `sqlx` |
| `just prepare` | Generate sqlx offline query data | `cargo sqlx prepare` |
| `just fmt` | Format all code | `cargo fmt` |
| `just fmt-check` | Check formatting | `cargo fmt -- --check` |
| `just check` | Full validation (fmt-check + clippy + test) | Runs all three verification gates |

**Environment Variables Expected:**
- `DATABASE_URL` — PostgreSQL connection string (used by db-* commands)
- `SAMETE_TEST_MODE=true` — Enable test mode for E2E (SMS dry-run, test data)
- `SAMETE_SMS_DRY_RUN=true` — Prevent actual SMS sending during E2E

---

## Dependency Tree Summary

### Direct Dependencies (Top Level)
- **Framework**: leptos, leptos_router, leptos_meta, leptos_i18n
- **Serialization**: serde, serde_json
- **Error Handling**: thiserror
- **Utilities**: uuid, time, tracing
- **SSR-only (11 crates)**: leptos_axum, axum, tokio, tower-http, tracing-subscriber, sqlx, reqwest, phonenumber, rand, blake2, sha2, base64, http, anyhow
- **WASM-only (2 crates)**: wasm-bindgen, console_error_panic_hook

### Total Resolved (via cargo tree)
~666 lines of dependency tree (100+ transitive crates)

**Heaviest Dependencies:**
- `tokio` — 100+ sub-dependencies (async runtime ecosystem)
- `sqlx` — 30+ sub-dependencies (database driver + types)
- `leptos` — 50+ sub-dependencies (framework + utilities)

---

## Conflict Prevention Checklist

When adding or updating dependencies:

- [ ] **Check Leptos version**: Is the new crate compatible with Leptos 0.8.x?
- [ ] **Check Axum version**: Does it require a specific Axum version? (Must be 0.8 to match leptos_axum)
- [ ] **Check Tokio version**: Does it require Tokio 2.x+? (Project locked to 1.x)
- [ ] **Check SQLx version**: If using SQL macros, does it match sqlx 0.8?
- [ ] **Feature intersection**: Does the new crate enable conflicting features on shared deps (e.g., `serde` vs `serde` with different feature sets)?
- [ ] **WASM compatibility**: If client-side use intended, does the crate have WASM support? (Check `wasm-bindgen`, check for `#[cfg(target_arch = "wasm32")]`)
- [ ] **Size impact**: For WASM, check estimated binary impact. Run `cargo build --lib --target wasm32-unknown-unknown --release` and compare output size.

---

## Notable Decisions

1. **Single Locale (uk only)**: The i18n infrastructure is in place but unused for multi-locale. Expansion would be straightforward (add locale codes, add UI toggle), but currently not needed.

2. **Feature Gating Discipline**: The codebase perfectly separates SSR, hydrate, and shared dependencies via feature gates. Adding a feature-gated dependency is safe and idiomatic.

3. **No Custom Async Executor**: Tokio with `features = ["full"]` is included wholesale. This is fine for a small project but could be trimmed to specific modules (time, io, sync) for binary size savings if needed later.

4. **Comprehensive Cryptography**: Three hash algorithms (BLAKE2, SHA-256, base64) for OTP and password handling. Appropriate for the security-sensitive auth/delivery logic.

5. **Strict Clippy**: All findings are errors. This enforces high code quality but means no lint-suppression exceptions without comment justification.

6. **No Custom Error Type**: Using `thiserror` for derives + `anyhow` for context. No bespoke `AppError` enum yet (though one exists in the codebase — see `src/error.rs` for context-aware server function error types).

---

## Audit Verification

**Audited via:**
- `Cargo.toml` (direct reading)
- `Cargo.lock` (exact resolved versions)
- `cargo tree --depth 1` and `cargo tree -i leptos` (dependency graph)
- Source code grep for actual usage (tracing, i18n configuration)
- `justfile` (build and test commands)

**Data Current As Of:**
- Cargo.lock snapshot: valid for reproducible builds
- Leptos version: 0.8.17 (latest stable 0.8.x as of 2026-03-19)
