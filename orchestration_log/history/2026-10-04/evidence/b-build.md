# b-build — CI-way build/lint/test (2026-10-04, HEAD 1a3c8a5, rustc/cargo 1.97.0)

CI `check` job (ci.yml:20-62): fmt, clippy SSR, clippy wasm, cargo-audit, cargo test. `e2e` job (ci.yml:64-end): postgres:16 svc, cargo-leptos, wasm-opt, sqlx-cli, just, node20, playwright chromium, `just e2e-release`.
CI flags: `cargo clippy ... -- -D warnings`; `cargo test` (bare, no ssr step); `SQLX_OFFLINE=true` job-level.

| Check | Command | Exit | Counts | Log |
|---|---|---|---|---|
| fmt | `cargo fmt --all -- --check` | 0 | clean | logs/fmt.log |
| clippy SSR | `SQLX_OFFLINE=true cargo clippy --features ssr --no-default-features -- -D warnings` | 0 | 0 warnings | logs/clippy-ssr.log |
| clippy wasm | `SQLX_OFFLINE=true cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings` | 0 | 0 warnings (target already installed) | logs/clippy-wasm.log |
| test bare | `SQLX_OFFLINE=true cargo test` | 0 | 68 passed / 0 failed / 0 ignored | logs/test-bare.log |
| test ssr | `SQLX_OFFLINE=true cargo test --features ssr` | 0 | 75 passed / 0 failed / 0 ignored | logs/test-ssr.log |
| cargo audit | (CI step, not in task) | not run | cargo-audit absent (`~/.cargo/bin` lacks it) | — |

## Findings
1. OK — all four CI gates green; `.sqlx/` offline cache (82 entries) sufficient.
2. MINOR — 7 of 75 `#[test]` (grep: 75 `#[test]`, 7 `#[cfg(test)]` mods) excluded from bare `cargo test` (68 vs 75). CI runs bare only → CI never executes the 7 ssr-gated tests. Gate: `#[cfg(feature = "ssr")]` ×57 sites, `#[cfg(any(feature = "ssr", test))]` ×12 sites in src/. 0 `#[ignore]`.
3. MINOR — CLAUDE.md claims "55 bare / 62 ssr"; actual 68 / 75 (doc stale).
4. MINOR — supply-chain audit (CI step) unverified here.
5. INFO — future-incompat warning: proc-macro-error2 v2.0.1 (non-fatal).

## E2E runnability (not run, per instructions)
Blocked here:
- docker daemon absent (`failed to connect to the docker API at unix:///var/run/docker.sock`); `docker` CLI present.
- postgres not running (`pg_isready`: /var/run/postgresql:5432 no response); `psql` client present.
- cargo-leptos, just, sqlx-cli, cargo-audit, brotli not in `~/.cargo/bin`/PATH.
- end2end/node_modules absent (needs `npm ci`); no chromium found (`npx playwright install chromium`).
- node/npx present (/opt/node22).
Needed: postgres 16 (samete/samete@localhost/samete) via native install or docker daemon; `cargo binstall cargo-leptos wasm-opt sqlx-cli just`; `npm ci` + playwright chromium; env `DATABASE_URL SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true`; then `just e2e-release`.
