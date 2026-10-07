# Implementation Plan: Launch Blockers ENV, T0, T1, U1–U4, A1

**CANONICAL COPY:** `orchestration_log/history/2026-10-04/plans/PLAN-blockers.md` (tracked). `orchestration_log/recon/2026-10-04/fix/PLAN.md` is a byte-identical mirror for convenience; if they ever differ, the tracked file wins — edit only the tracked file, then re-copy.

HEAD at planning: `1a3c8a5`. Evidence: `readiness/l-toolchain.md` + `readiness/logs/l-*.log` (T0, ENV), `readiness/f-e2e.md` + `readiness/logs/f-*.log` (ENV), `readiness/n-panic.md` + `logs/n-*.log` (T1), `readiness/f-e2e.md` § attr: leak check (A1), `readiness/d-ops.md` (F1/F2/F5), `readiness/g-authverify.md` (1b), `readiness/i-swapverify.md`, `readiness/a-scope.md` (3.3 rows). Every claim below re-verified against source at HEAD.

## Preamble

| Unit | Blocker | Verified at |
|---|---|---|
| ENV | E2E only ran in this container with hand-made shims: the musl tailwind picked by a musl-built cargo-leptos, a Playwright browser-revision mismatch, `npm ci` lockfile drift, unpinned tools (sqlx-cli 0.9 vs sqlx 0.8.6), and no session bootstrap. | `cargo-leptos-0.3.7/src/ext/util.rs:27`, `exe.rs:225,353`; `end2end/package.json:11` vs `package-lock.json:12`; `.gitignore:1` |
| T0 | HEAD does not build: the SSR bin's login view tree and the wasm hydrate lib's invite-code list are each one concrete nested view type, and their layout overflows rustc's default query depth (128) at codegen. CI is false-green since 2026-07-10: Check never runs codegen; `cargo leptos end-to-end` exits 0 on build failure and skips Playwright. | `readiness/l-toolchain.md`; `src/pages/login.rs` (`LoginPage`/`LoginStepRouter`/`NameCollectionForm`), `src/admin/page.rs:1841-2033` (`InviteCodesSection` `<For>`); `.github/workflows/ci.yml:39-56` (no build step), `:125` |
| T1 | Concurrent SSR load kills the server: leptos_i18n 0.6.1 `context.rs:213` reads a disposed signal in a detached tokio task; `[profile.release] panic = "abort"` turns the task panic into process death (prod builds the same profile). | `Cargo.toml:95-104,127`; `src/app.rs:73`; `n-panic.md` |
| U1 | Self-registration skips OTP: `register_with_code` / `validate_invite_code` trust a raw, unsigned `pending_phone=<phone>` cookie. curl with any phone + one invite code creates the account. Invite redemption is unthrottled (~39,800-code space). | `src/pages/login.rs:208-219` (cookie = normalized phone), `:320-328`, `:391-399`, `:485-496` |
| U2 | `SAMETE_TEST_MODE=true` = OTP `000000` for every phone incl. admin + OTP rate limit off. No boot guard. Read ad hoc at 5 sites. Undocumented. | `src/auth.rs:77,121`, `src/pages/home.rs:180,398,521`, `src/config.rs` (no check), `README.md:185-192` |
| U3 | No production path to the first admin; invite codes require an admin to exist (circular). | `migrations/20260314000003…:5` (`distributor_id NOT NULL`), `src/admin/invite_codes.rs:90`, only admin insert = `seed/test_admin.sql` |
| A1 | `attr:aria-busy` on 13 native `<button>`s emits a literal `attr:aria-busy` attribute during pending; `aria-busy` never reaches assistive tech. | `src/admin/page.rs:497,675,705,753,865,906,955,992,1102,1647,1748,2016,2171`; `f-e2e.md` attr: leak check |
| U4 | `swap_assignment` exchanges two senders' recipients. Math: in one cycle this always splits it (or self-loops); first UPDATE also violates non-deferrable `UNIQUE(season_id, recipient_id)`; three statements on the pool, no tx, validation after writes; `validate_swap_topology` treats all rows as one cohort; `advance_season` never re-validates; E2E 3.3 asserts only that the viz renders. | `src/admin/assignments.rs:184-226,318-416,435-488`, `migrations/20260314000002…:56-67`, `src/admin/season.rs:144-174`, `end2end/tests/mail_club.spec.ts:649-654`, `fixtures/mail_club_page.ts:483-498` |

Story 3.3 (spec/technical/User Stories.md:228-241): "swap individual sender→recipient pairings while maintaining cycle integrity"; AC "Swaps must preserve the single-loop topology"; "organizer sees the full graph for each cohort". The only swap of two people that preserves every loop is **exchanging their positions in the cycle** (conjugating the sender→recipient map by the transposition (a b)). U4 implements exactly that.

## Design decisions (final — do not revisit)

- **ENV:** one idempotent `scripts/bootstrap-toolchain.sh` holding every pin (cargo-leptos 0.3.7, sqlx-cli 0.8.6, just 1.58.0, wasm-opt 0.116.1, cargo-audit, tailwind v4.2.1 sha256-pinned in `.tools/bin` on `PATH`, Playwright exact 1.58.2 + its browser revision, Postgres+role). CI, a committed Claude Code cloud SessionStart hook and the README call it.
- **T0:** a root fix with no `recursion_limit` anywhere. Cut view-type depth at existing UI seams: login steps become sub-components, `.into_any()` erasure goes at component and branch boundaries, and the invite-code `<li>` becomes an `InviteCodeCard` component. No DOM change. Headroom proof: both crates still build with a temporary limit of 96. No toolchain pin (unpinned stable surfaced the defect; a pin would hide it). Check job gains real codegen builds of the SSR bin and wasm lib. One guard script `scripts/assert-playwright-ran.sh` (fresh `end2end/results.json` with >0 executed tests and 0 failures) called by every `cargo leptos end-to-end` just recipe — CI's E2E job runs `just e2e-release`, so one home covers local and CI. `isolated-capture.sh` deletes the release artifacts before building and requires them after.
- **T1:** `panic = "unwind"` in `[profile.release]` (server bin) + explicit `panic = "abort"` in `[profile.wasm-release]` (WASM size unchanged). No `[patch.crates-io]`: conventions forbid it without the crate's own suite, and upstream 0.6.2 has the same code. Regression probe `scripts/ssr-stress.sh`, shown RED with the abort binary and GREEN with the unwind binary.
- **A1:** bare `aria-busy` at all 13 sites. Proof: a compiled-artifact string gate plus one E2E test that holds the server response with `page.route`.
- **U1:** Server-side `registration_tickets` table is the sole authority. On OTP success for an unknown phone the server inserts `(sha256(token), phone, expires_at = now()+10min, invite_attempts = 0)` and sets cookie `registration_ticket=<random 32-byte base64url token>`. The phone is read ONLY from the ticket row. Registration consumes the row (`DELETE … RETURNING phone`) inside the registration transaction. Throttle falls out of the same row: each invalid/used/revoked code submission increments `invite_attempts`; a ticket with `invite_attempts >= 5` resolves to nothing → user must re-verify via OTP (itself limited to 5/h/phone). No U1b.
- **U2:** `Config.sms: SmsMode { Live{token,sender} | DryRun{test_mode} }` — test mode is unrepresentable with live SMS. Boot refuses `SAMETE_TEST_MODE=true` without `SAMETE_SMS_DRY_RUN=true`, and refuses test mode on a non-loopback bind address (prod container binds `0.0.0.0`; `just e2e`, CI, `isolated-capture.sh`, `just dev` all bind `127.0.0.1`). All 5 env reads replaced by `Config::test_mode()`.
- **U3:** Idempotent startup bootstrap from `SAMETE_ADMIN_PHONE` + `SAMETE_ADMIN_NAME` (both or neither; phone normalized via `phone::normalize`; invalid → boot refuses). Upsert: insert admin, or promote existing user with that phone to admin. Documented in README + `.env.example`.
- **U4:** Position-exchange swap as a pure function in `src/assignment.rs`; cohorts recovered from edges by walking (multi-cohort-correct); swap = one transaction: lock season row, load edges, compute, validate, `DELETE` season rows, bulk `INSERT`, commit. `advance_season` re-validates when leaving Assignment. Preview renders every cohort.

## Write-sets and lanes

| Unit | Files written |
|---|---|
| ENV | `scripts/bootstrap-toolchain.sh` (new), `.claude/settings.json` (new), `.claude/hooks/session-start.sh` (new), `.gitignore`, `justfile`, `scripts/isolated-capture.sh`, `end2end/package.json`, `end2end/package-lock.json`, `.github/workflows/ci.yml`, `README.md` |
| T0 | `src/pages/login.rs`, `src/admin/page.rs`, `scripts/assert-playwright-ran.sh` (new), `scripts/isolated-capture.sh`, `justfile`, `.github/workflows/ci.yml` |
| T1 | `Cargo.toml`, `scripts/ssr-stress.sh` (new), `orchestration_log/reference/deferred_items.md` |
| A1 | `src/admin/page.rs`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/mail_club.spec.ts` |
| U1 | `migrations/20261004000001_registration_tickets.sql` (new), `src/auth.rs`, `src/pages/login.rs`, `.sqlx/*`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/mail_club.spec.ts` |
| U2 | `src/config.rs`, `src/main.rs`, `src/sms.rs`, `src/auth.rs`, `src/pages/login.rs`, `src/pages/home.rs`, `README.md`, `.env.example` |
| U3 | `src/config.rs`, `src/main.rs`, `src/db.rs`, `.sqlx/*`, `README.md`, `.env.example` |
| U4 | `src/assignment.rs`, `src/admin/assignments.rs`, `src/admin/season.rs`, `src/admin/page.rs`, `locales/uk.json`, `.sqlx/*`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/visual-audit.spec.ts` |

Overlaps: U1∩U2 = `auth.rs`, `login.rs`. U2∩U3 = `config.rs`, `main.rs`, `README.md`, `.env.example`. U1∩U4 = POM + `mail_club.spec.ts` (disjoint regions: Epic 1 vs Epic 3 / invite vs assignment POM sections) + `.sqlx/` (distinct query files). U3∩U1/U4 = `.sqlx/` only. T1 shares no file with any unit. A1∩U4 = `src/admin/page.rs` (U4: `render_cycle_ring`; A1: 13 button attribute tokens, one inside U4-untouched `SwapFormSection`). A1∩U1/U4 = POM + `mail_club.spec.ts` (A1: invite-codes POM section + one Epic 1 test).

Shared files are not a reason to sequence. Every lane has its own worktree, and conflicts are resolved at integration: union edits, regenerate `.sqlx/`, re-run that unit's gates. Further overlaps: ENV∩T0 = `justfile`, `scripts/isolated-capture.sh`, `.github/workflows/ci.yml` (different lines). T0∩U1 = `src/pages/login.rs` (T0 moves the step blocks into components; U1 edits server fns and adds one testid inside `NameCollectionForm`). T0∩U4/A1 = `src/admin/page.rs` (T0: `InviteCodesSection` `<li>`; U4: `render_cycle_ring`; A1: 13 attribute tokens, one of them inside the `<li>` that T0 moves).

**ENV addendum (2026-10-07, PRB-102).** Pin the Rust toolchain in a committed `rust-toolchain.toml` (channel = the exact version CI's `dtolnay/rust-toolchain` resolves to; add `wasm32-unknown-unknown` and the clippy/rustfmt components). CI must read the same file. Separately, make `cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings` green on the pinned toolchain AND on current stable 1.99.0:
- fix the 2 `.ok().is_some_and` sites in `src/pages/login.rs`;
- for the `unused_async_trait_impl` hits on `#[server]` expansions, use a crate-level `#![allow]` with a WHY comment only if the lint fires inside macro-generated code the crate cannot change; prove that with the lint's span output in the report.
Gate: both clippy targets exit 0 on the pin, and the report pastes the output.

**Lanes (binding: `recon/2026-10-04/fix/prompts/_parallelism.md`).** ENV, T0, T1, U1, U2, U3, U4 and A1 are concurrent lanes. Each starts from the working branch `claude/loving-johnson-7l8hn5`, and each is integrated and reviewed on its own when it finishes. Proven symbol dependencies:

| Dependent | Symbol | Introduced by / file | Base for the dependent lane |
|---|---|---|---|
| U2 | `Config.admin_bootstrap: Option<AdminBootstrap>` (U2's `config_with` test constructs `Config` with it; U2's `from_env` keeps it) | U3, `src/config.rs` — exists on branch `worktree-agent-a1fe18df1d59b4c14` | working branch + `git merge worktree-agent-a1fe18df1d59b4c14` |

There are no other symbol dependencies. U2 changes `create_otp`/`check_otp_rate_limit`/`request_otp`, which exist at HEAD and which U1 does not change. A1's 13 sites and `generate-code-button` exist at HEAD. If T0 moved a site, the integrator keeps both edits (the token change applies inside `InviteCodeCard`).

**Build at the default limit (applies to every lane except T0):** HEAD does not compile its SSR bin or wasm lib at rustc's default limit until T0's view refactor is in the tree. That is a build precondition, not a symbol. Any lane whose gates need a release build (T1, U1–U4, A1, ENV) does this: if a branch already containing T0's commit exists, merge it into the worktree for the gate runs, then `git reset --hard` back to the lane's own commit before reporting (the lane's commits never contain T0's). Otherwise export `RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Zcrate-attr=recursion_limit="256"'` in the gate shell only. Never commit it or write it into any file. The integrator re-runs every gate on the integrated branch without it.

**Playwright-ran check without T0's script:** where a gate below calls `bash scripts/assert-playwright-ran.sh <marker>` and the file is absent in the lane's tree, run instead:
```bash
[ end2end/results.json -nt <marker> ] && node -e 'const s=require("./end2end/results.json").stats;console.log(`playwright ran: expected=${s.expected} unexpected=${s.unexpected} flaky=${s.flaky} skipped=${s.skipped}`);process.exit(s.expected+s.unexpected+s.flaky>0&&s.unexpected===0?0:1)'; echo "guard=$?"
```
Same REQUIRED output.

After every lane is integrated, the orchestrator runs the Global Gates on the working branch.

## Shared environment (every unit)

Commit style (CLAUDE.md, binding): one-line conventional commit, no body, no `Co-Authored-By`, no AI attribution.

Postgres (once per container):
```bash
pg_isready -h localhost -p 5432 || docker compose -f /home/user/same-te-mail-club/docker-compose.yml up -d
until pg_isready -h localhost -p 5432; do sleep 1; done
```
**REQUIRED OUTPUT:** `localhost:5432 - accepting connections`.

Per-unit sibling DB (NEVER the shared `samete` DB; NEVER `just db-reset`, `just e2e*`, `_kill-stale`, port 3000 — the isolated harness is the e2e path):
```bash
sqlx --version || cargo binstall --no-confirm sqlx-cli --no-default-features --features postgres,rustls
export DATABASE_URL=postgres://samete:samete@localhost:5432/samete_<unit>   # <unit> = u1|u2|u3|u4
sqlx database drop -y; sqlx database create && sqlx migrate run
```

sqlx cache regeneration (after ANY `query!`/`query_as!`/`query_scalar!` change, including whitespace):
```bash
cargo sqlx prepare --workspace -- --features ssr
git status --short .sqlx | head
```
**REQUIRED:** command exits 0; `.sqlx/` has added/modified files for every new query. `ls .sqlx | wc -l` is NOT 0.

Standard gates (run from the worktree root, in this order, after the unit's last code change):
```bash
cargo fmt --all -- --check
SQLX_OFFLINE=true cargo clippy --features ssr --no-default-features -- -D warnings
SQLX_OFFLINE=true cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings
SQLX_OFFLINE=true cargo build --no-default-features --features ssr --bin samete
SQLX_OFFLINE=true cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate
cargo test 2>&1 | grep -E "^test result"
SQLX_OFFLINE=true cargo test --features ssr 2>&1 | grep -E "^test result"
```
**REQUIRED OUTPUT:** fmt prints nothing, exit 0. Both clippy runs and both builds end with `Finished` and contain zero `warning:`/`error:` lines. Every `test result:` line reads `ok.` with `0 failed`.

E2E container recipe (this cloud container; verified by the readiness E2E agent — apply before any E2E command):
```bash
# Playwright 1.58 expects the chrome-headless-shell layout; /opt/pw-browsers lacks it.
# A shim dir with chromium_headless_shell-1208 (+ ffmpeg symlink) already exists:
export PLAYWRIGHT_BROWSERS_PATH=/tmp/claude-0/-home-user-same-te-mail-club/eeccd659-50b4-54c7-87fc-f4a8a8b2e54c/scratchpad/pwb
ls "$PLAYWRIGHT_BROWSERS_PATH"            # REQUIRED: chromium_headless_shell-1208  ffmpeg-1011
# cargo-leptos runs the musl tailwind binary, which does not execute here; it was
# replaced by the glibc build. Verify (re-apply with the cp if the check fails):
TW=/root/.cache/cargo-leptos/tailwindcss-v4.2.1/tailwindcss-v4.2.1
cmp -s "$TW/tailwindcss-linux-x64" "$TW/tailwindcss-linux-x64-musl" && echo tailwind-shim-ok \
  || cp "$TW/tailwindcss-linux-x64" "$TW/tailwindcss-linux-x64-musl"
```
Browser-path check (`readiness/n-panic.md` reports the `pwb` shim pointing at a non-existent `/opt/.../1208/chrome-linux`, while its runs passed with `/opt/pw-browsers`). Run this once per container and use whichever path prints `launch-ok`, trying the shim first:
```bash
(cd end2end && node -e 'require("@playwright/test").chromium.launch().then(b=>{console.log("launch-ok");return b.close()})')
```
If the shim fails, `export PLAYWRIGHT_BROWSERS_PATH=/opt/pw-browsers` and rerun. If neither prints `launch-ok`, report BLOCKED. Never run `npx playwright install`. Baseline for `bash scripts/isolated-capture.sh <suffix> full` on a buildable tree: **116 passed, 2 skipped, 0 failed**. Each unit's E2E counts are relative to this.

E2E gate (isolated harness — own port + own DB `samete_e2e_<unit>`; requires release build; run in background, output to file, never pipe through `tail`/`head`):
```bash
# PLAYWRIGHT_BROWSERS_PATH + tailwind shim from the container recipe above
[ -d end2end/node_modules ] || (cd end2end && npm ci)
bash scripts/isolated-capture.sh e2e_<unit> full > /tmp/e2e_<unit>.log 2>&1; echo "exit=$?" >> /tmp/e2e_<unit>.log
grep -E "exit=|[0-9]+ (passed|failed|flaky|skipped)" /tmp/e2e_<unit>.log
```
**REQUIRED OUTPUT:** `exit=0`; a `N passed` line; NO `failed` line; NO `flaky` line; `2 skipped`. Every new/changed test title named in the unit appears with a pass mark in the list output (`grep -F "<title>" /tmp/e2e_<unit>.log`). If `orchestration_log/recon/2026-10-04/readiness/f-e2e.md` exists and records a different working recipe for this container, use that recipe's environment deltas (e.g. browser path) — the pass criteria above stay identical. Stability: run the E2E gate 3 times; all 3 must meet the criteria.

---

# ENV — Reproducible toolchain: a fresh session runs `just e2e-release` with no manual shims

Lane: concurrent, base = working branch `claude/loving-johnson-7l8hn5`. Consumes no symbol introduced by another unit (see the gate note on the build).

## Why This Matters
Every readiness E2E run needed hand-made shims, and several produced false or misleading results:

| Defect | Evidence (verified) | Root cause |
|---|---|---|
| Tailwind fails | f-e2e/readiness logs; `file $(which cargo-leptos)` = static-pie (musl build from `cargo binstall`) | cargo-leptos 0.3.7 picks the tailwind download by its OWN build target (`ext/util.rs:27` `cfg!(target_env = "musl")` → `exe.rs:353,377` downloads `tailwindcss-linux-x64-musl`), which does not run on this glibc host. cargo-leptos prefers a `tailwindcss` already on `PATH` (`exe.rs:225` `from_global_path`). |
| Playwright browser mismatch | `@playwright/test` resolves to 1.58.2 → chromium-headless-shell rev 1208 (`playwright-core/browsers.json`); container images ship whatever revisions they ship; readiness used a scratchpad `pwb` shim that later broke (`n-panic.md`) | Nothing in the repo ensures the matching browser revision is installed. |
| `npm ci` lockfile drift | `end2end/package.json` `"@playwright/test": "^1.58.2"` vs `package-lock.json` root `packages[""]` `"^1.44.1"` | Commit `968fe66` edited package.json without regenerating the lock root. |
| `cargo leptos end-to-end` exit-code blindness | `l-toolchain.md` | Handled by T0 (`scripts/assert-playwright-ran.sh` in every just E2E recipe). ENV does not edit it. ENV's own gate checks `end2end/results.json` directly, so it needs nothing from T0. |
| Unpinned tools | CI `cargo binstall cargo-leptos` / `sqlx-cli` / `just` / `cargo-audit` unpinned; this container has `sqlx-cli 0.9.0` while `Cargo.lock` has `sqlx 0.8.6` | A cli/lib major mismatch for `cargo sqlx prepare` writes `.sqlx` data the 0.8 macros may not read. cargo-leptos 0.3.11 vs 0.3.7 drift was observed. |
| No session bootstrap | `.claude` is gitignored wholesale (`.gitignore:1`); no SessionStart hook | Each cloud session starts without the toolchain. |

## Decisions (final)
- **One bootstrap script** `scripts/bootstrap-toolchain.sh` holds every pin. It is idempotent, non-interactive, holds no secrets, and is the ONLY place versions live. CI, the SessionStart hook and the README call it. Local macOS developers run it too; its Postgres step uses Docker when available.
- **Tailwind:** a repo-controlled pinned binary at `.tools/bin/tailwindcss` (v4.2.1, glibc linux-x64, sha256-verified; on macOS the matching `macos-arm64`/`macos-x64` asset with its own pinned sha256). `.tools/bin` is prepended to `PATH` by the `justfile` (`export PATH := …`) and by `scripts/isolated-capture.sh`, so cargo-leptos never downloads one.
- **Playwright:** exact pin `"@playwright/test": "1.58.2"` (no caret) + regenerated lock, so the browser revision is deterministic. The bootstrap installs exactly that revision (`npx playwright install chromium`) unless `PLAYWRIGHT_BROWSERS_PATH` already holds `chromium_headless_shell-<rev>`. Rejected: a config-level `executablePath` fallback, which braids host layout into test config and masks revision drift.
- **Pins:** cargo-leptos `0.3.7` (last version CI ran green and the one the readiness agents verified), sqlx-cli `0.8.6` (= `Cargo.lock` sqlx), just `1.58.0`, wasm-opt `0.116.1`, cargo-audit latest-at-plan `0.21.x`. The implementer pins the exact version `cargo binstall --dry-run cargo-audit` resolves today and records it.
- **SessionStart hook:** `.claude/settings.json` + `.claude/hooks/session-start.sh`, synchronous (no async: a racing agent must not see a half-installed toolchain). Remote-only (`$CLAUDE_CODE_REMOTE == true`). It calls the bootstrap and appends `PATH`/`PLAYWRIGHT_BROWSERS_PATH`/`DATABASE_URL` exports to `$CLAUDE_ENV_FILE`.

## What You Must Do

### ENV.1 `scripts/bootstrap-toolchain.sh` (new, `chmod +x`)
```bash
#!/usr/bin/env bash
# Idempotent toolchain bootstrap — the single home of every tool pin.
# Used by: CI (.github/workflows/ci.yml), the Claude Code cloud SessionStart hook
# (.claude/hooks/session-start.sh), and developers (README "Setup").
# Safe to re-run: every step checks before installing. No secrets.
set -euo pipefail
cd "$(dirname "$0")/.."

CARGO_LEPTOS_VERSION="0.3.7"
SQLX_CLI_VERSION="0.8.6"
JUST_VERSION="1.58.0"
WASM_OPT_VERSION="0.116.1"
CARGO_AUDIT_VERSION="<pinned by implementer, see ENV decisions>"
TAILWIND_VERSION="v4.2.1"
TAILWIND_SHA256_LINUX_X64="39e8d4e24b3c83b0a6e69e100a972fbc75d5fef8dce47b3ddac3cf92dea81fe3"
TAILWIND_SHA256_MACOS_ARM64="<implementer: sha256 of the release asset>"
TAILWIND_SHA256_MACOS_X64="<implementer: sha256 of the release asset>"

log() { echo "[bootstrap] $*"; }

# --- Rust targets ---
rustup target list --installed | grep -qx wasm32-unknown-unknown || rustup target add wasm32-unknown-unknown

# --- cargo-binstall ---
command -v cargo-binstall >/dev/null || cargo install --locked cargo-binstall

# --- pinned cargo tools: install only if missing or wrong version ---
ensure_tool() { # <crate> <version> <version-probe-cmd...>
    local crate="$1" version="$2"; shift 2
    if "$@" 2>/dev/null | grep -qF "$version"; then log "$crate $version present"; return; fi
    log "installing $crate $version"
    cargo binstall --no-confirm --locked "$crate@$version"
}
ensure_tool cargo-leptos "$CARGO_LEPTOS_VERSION" cargo leptos --version
ensure_tool sqlx-cli "$SQLX_CLI_VERSION" sqlx --version
ensure_tool just "$JUST_VERSION" just --version
ensure_tool wasm-opt "$WASM_OPT_VERSION" wasm-opt --version   # probe prints "version 116"; ensure_tool matches the crate version string "116"
ensure_tool cargo-audit "$CARGO_AUDIT_VERSION" cargo audit --version

# --- tailwind: pinned binary on a repo-controlled PATH entry ---
mkdir -p .tools/bin
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64)  tw_asset=tailwindcss-linux-x64;   tw_sha="$TAILWIND_SHA256_LINUX_X64" ;;
    Darwin-arm64)  tw_asset=tailwindcss-macos-arm64; tw_sha="$TAILWIND_SHA256_MACOS_ARM64" ;;
    Darwin-x86_64) tw_asset=tailwindcss-macos-x64;   tw_sha="$TAILWIND_SHA256_MACOS_X64" ;;
    *) echo "[bootstrap] FATAL: no pinned tailwind for $(uname -s)-$(uname -m)"; exit 1 ;;
esac
if ! echo "$tw_sha  .tools/bin/tailwindcss" | sha256sum -c --status 2>/dev/null; then
    log "installing tailwindcss $TAILWIND_VERSION ($tw_asset)"
    curl -fsSL -o .tools/bin/tailwindcss \
        "https://github.com/tailwindlabs/tailwindcss/releases/download/${TAILWIND_VERSION}/${tw_asset}"
    echo "$tw_sha  .tools/bin/tailwindcss" | sha256sum -c --status \
        || { echo "[bootstrap] FATAL: tailwindcss sha256 mismatch"; exit 1; }
    chmod +x .tools/bin/tailwindcss
fi

# --- Node deps + the exact Playwright browser revision ---
(cd end2end && npm ci)
pw_rev="$(node -e 'const b=require("./end2end/node_modules/playwright-core/browsers.json").browsers.find(x=>x.name==="chromium-headless-shell");console.log(b.revision)')"
pw_dir="${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"
if [ -f "$pw_dir/chromium_headless_shell-$pw_rev/INSTALLATION_COMPLETE" ]; then
    log "playwright chromium-headless-shell $pw_rev present in $pw_dir"
else
    log "installing playwright chromium $pw_rev into $pw_dir"
    (cd end2end && PLAYWRIGHT_BROWSERS_PATH="$pw_dir" npx playwright install chromium)
fi

# --- Postgres (role samete/samete on localhost:5432) ---
if ! pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
    if docker info >/dev/null 2>&1; then
        docker compose up -d db
    elif command -v pg_ctlcluster >/dev/null; then
        pg_ctlcluster "$(ls /usr/lib/postgresql | sort -n | tail -1)" main start
    else
        echo "[bootstrap] FATAL: no Postgres on :5432 and neither docker nor pg_ctlcluster available"; exit 1
    fi
    until pg_isready -h localhost -p 5432 >/dev/null 2>&1; do sleep 1; done
fi
if ! psql "postgres://samete:samete@localhost:5432/postgres" -tAc 'select 1' >/dev/null 2>&1; then
    log "creating role samete"
    su postgres -c "psql -c \"CREATE ROLE samete LOGIN CREATEDB PASSWORD 'samete'\""
fi

log "done — export PATH=\"$PWD/.tools/bin:\$PATH\" PLAYWRIGHT_BROWSERS_PATH=\"$pw_dir\""
```
Implementer obligations:
- Replace each `<…>` placeholder with the real value (sha256 from `curl -fsSL <url> | sha256sum`; cargo-audit from `cargo binstall --dry-run cargo-audit`). A placeholder left in = FAIL.
- Make the `wasm-opt` probe correct: `wasm-opt --version` prints `version 116`, so call `ensure_tool wasm-opt "$WASM_OPT_VERSION"` with a probe that greps `116`. Implement it as a dedicated line `wasm-opt --version 2>/dev/null | grep -q "version 116" || cargo binstall --no-confirm --locked "wasm-opt@$WASM_OPT_VERSION"`, and delete the generic call for wasm-opt.
- `docker compose up -d db`: the service name must equal the one in `docker-compose.yml` (verify; use that exact name).
- The `su postgres` branch runs only on the native-cluster path; on Docker the compose env already creates the role.

### ENV.2 `.gitignore`
Replace the line `.claude` with:
```
.claude/*
!.claude/settings.json
!.claude/hooks/
.tools/
```

### ENV.3 `.claude/settings.json` (new)
```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "\"$CLAUDE_PROJECT_DIR\"/.claude/hooks/session-start.sh"
          }
        ]
      }
    ]
  }
}
```

### ENV.4 `.claude/hooks/session-start.sh` (new, `chmod +x`)
```bash
#!/usr/bin/env bash
# Claude Code cloud sessions only: install the pinned toolchain so `just e2e-release`
# works in a fresh container. Synchronous on purpose: no agent turn may race a
# half-installed toolchain. Idempotent; no secrets.
set -euo pipefail
[ "${CLAUDE_CODE_REMOTE:-}" = "true" ] || exit 0
cd "$CLAUDE_PROJECT_DIR"
bash scripts/bootstrap-toolchain.sh >&2
{
    echo "export PATH=\"$CLAUDE_PROJECT_DIR/.tools/bin:\$HOME/.cargo/bin:\$PATH\""
    echo "export PLAYWRIGHT_BROWSERS_PATH=\"${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}\""
    echo 'export DATABASE_URL="postgres://samete:samete@localhost/samete"'
} >> "$CLAUDE_ENV_FILE"
```
(Bootstrap output goes to stderr so the hook's stdout stays clean for the harness.)

### ENV.5 `justfile`
Directly after the header comment line add:
```just
# Repo-pinned tools (tailwindcss) take precedence; installed by scripts/bootstrap-toolchain.sh.
export PATH := justfile_directory() + "/.tools/bin:" + env_var("PATH")

# Install/verify the pinned toolchain (idempotent).
bootstrap:
    bash scripts/bootstrap-toolchain.sh
```
No other justfile change. T0 edits the E2E recipe bodies; any textual conflict is resolved at integration (keep both edits).

### ENV.6 `scripts/isolated-capture.sh`
After `set -euo pipefail` add:
```bash
# Repo-pinned tools (tailwindcss) — see scripts/bootstrap-toolchain.sh.
export PATH="$(cd "$(dirname "$0")/.." && pwd)/.tools/bin:$PATH"
```

### ENV.7 `end2end/package.json` + `end2end/package-lock.json`
In package.json, change `"@playwright/test": "^1.58.2"` to `"@playwright/test": "1.58.2"`. Then run:
```bash
(cd end2end && npm install --package-lock-only --ignore-scripts)
```
No other dependency change (`git diff end2end/package-lock.json` must touch only the root `packages[""]` entry).

### ENV.8 `.github/workflows/ci.yml`
In the E2E job, replace the five steps `Install cargo-leptos`, `Install wasm-opt`, `Install sqlx-cli`, `Install just`, `Install Playwright dependencies` with:
```yaml
      - name: Bootstrap pinned toolchain
        run: bash scripts/bootstrap-toolchain.sh

      - name: Playwright system dependencies
        working-directory: end2end
        run: npx playwright install-deps chromium
```
(`Install Node.js` and `Install cargo-binstall` stay before it. Postgres is the job's service container, so bootstrap's `pg_isready` passes and it skips the start. The `samete` role exists via service env.) In the Check job, replace `cargo binstall --no-confirm cargo-audit && cargo audit` with `cargo binstall --no-confirm --locked cargo-audit@<same pin as bootstrap> && cargo audit`. Leave the Check job's build steps and the `just e2e-release` line exactly as they are (T0 edits nearby; integration keeps both).

### ENV.9 `README.md` — Prerequisites + Setup only
Replace the Prerequisites bullets for cargo-leptos and sqlx-cli with one bullet: "- Toolchain pins (cargo-leptos, sqlx-cli, just, wasm-opt, cargo-audit, tailwindcss, Playwright browser): `just bootstrap` (or `bash scripts/bootstrap-toolchain.sh` before `just` exists). Claude Code cloud sessions run it automatically (`.claude/hooks/session-start.sh`)." In Setup, insert `bash scripts/bootstrap-toolchain.sh` as the first command. Do not touch the Environment Variables table or Deployment (U2/U3 own them).

## Verification Gates (ENV)
Static:
```bash
grep -c "<" scripts/bootstrap-toolchain.sh | xargs -I{} sh -c 'grep -n "<[a-z]" scripts/bootstrap-toolchain.sh || echo no-placeholders'
grep -n '"@playwright/test"' end2end/package.json end2end/package-lock.json
git check-ignore -q .claude/settings.json && echo IGNORED || echo tracked-ok
git check-ignore -q .claude/hooks/session-start.sh && echo IGNORED || echo tracked-ok
git check-ignore -q .claude/worktrees && echo still-ignored
python3 -c "import json;json.load(open('.claude/settings.json'))" && echo json-ok
python3 -c "import yaml;yaml.safe_load(open('.github/workflows/ci.yml'))" && echo yaml-ok
bash -n scripts/bootstrap-toolchain.sh .claude/hooks/session-start.sh && echo syntax-ok
```
**REQUIRED:** `no-placeholders`; package.json `"1.58.2"` and lock root `"1.58.2"` (both exact); `tracked-ok` ×2; `still-ignored`; `json-ok`; `yaml-ok`; `syntax-ok`.

Idempotence: run `bash scripts/bootstrap-toolchain.sh` twice. **REQUIRED:** both exit 0. The second run prints `present` for every pinned tool and the playwright line, with no `installing` lines (`grep -c installing` = `0` on run 2).

Clean-shell end-to-end (the acceptance gate: no manual shims). Use a login shell with NO readiness exports. The working branch does not build at rustc's default recursion limit, and the root fix is T0's view refactor in `src/pages/login.rs`/`src/admin/page.rs`. If an existing T0 branch (`git branch -a --list '*t0*'`) has that commit, `git merge` it into this worktree for the gate run only, then `git reset --hard` back to ENV's commit before committing. Otherwise set `RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Zcrate-attr=recursion_limit="256"'` inside the gate shell. That compile-only flag is never committed and never added to any script; the integrator re-runs this gate on the integrated branch without it.
```bash
env -i HOME="$HOME" PATH="$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin:/opt/node22/bin" TERM=dumb bash -lc '
  set -e
  cd <worktree>
  mv /root/.cache/cargo-leptos /root/.cache/cargo-leptos.bak-env 2>/dev/null || true   # prove no reliance on cached/shimmed tailwind
  CLAUDE_CODE_REMOTE=true CLAUDE_PROJECT_DIR="$PWD" CLAUDE_ENV_FILE=/tmp/env-hook.env bash .claude/hooks/session-start.sh
  source /tmp/env-hook.env
  export SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true
  command -v tailwindcss
  rm -f end2end/results.json
  just e2e-release || true
  node -e "const s=require(\"./end2end/results.json\").stats; console.log(\"playwright ran: expected=\"+s.expected+\" unexpected=\"+s.unexpected+\" skipped=\"+s.skipped)"
' > /tmp/env-clean.log 2>&1; echo "exit=$?"
mv /root/.cache/cargo-leptos.bak-env /root/.cache/cargo-leptos 2>/dev/null || true
grep -E "tailwindcss$|playwright ran:|FATAL" /tmp/env-clean.log
```
**REQUIRED:** `exit=0`; the `command -v` line ends in `<worktree>/.tools/bin/tailwindcss`; `playwright ran: expected=116 unexpected=0 skipped=2` (if T0's guard is present it also passes); no `FATAL`; no line mentioning `tailwindcss-linux-x64-musl`. `just e2e-release` resets the `samete` DB and uses :3000. This gate is the ONE sanctioned exception to the Shared-environment ban, for ENV only. First run `ss -ltnp | grep -c ':3000 '` → `0`; if it is non-zero, STOP and report BLOCKED instead of killing anything.

Standard gates (Rust unchanged; must still pass).

Commit: `build(env): pin toolchain and add idempotent bootstrap + cloud SessionStart hook`. Write-set exactly: `scripts/bootstrap-toolchain.sh` (new), `.claude/settings.json` (new), `.claude/hooks/session-start.sh` (new), `.gitignore`, `justfile`, `scripts/isolated-capture.sh`, `end2end/package.json`, `end2end/package-lock.json`, `.github/workflows/ci.yml`, `README.md`.

---

# T0 — Build actually compiles at rustc's default limit; CI and harnesses cannot go false-green

## Why This Matters
HEAD does not build. `cargo build` of the SSR bin and of the wasm hydrate lib fail with `error: queries overflow the depth limit!` (layout of Leptos `into_any` async blocks over one deeply nested view type per page exceeds rustc's default 128; trigger commit `ee816fd`; reproduced on rustc 1.97.0/1.97.1/1.99.0 — `readiness/l-toolchain.md`, `readiness/logs/l-debug-bin-1.99.0.log`, `l-wasm-lib-debug-1.99.0.log`). `cargo clippy`/`cargo test` never run codegen, so the Check job stays green. `cargo leptos end-to-end` exits 0 on build failure and skips Playwright, so the E2E job has reported success with zero tests since 2026-07-10 (runs 29118276831 … 30162383205). Raising the limit would mask the cause, and the views would keep growing past any number. The fix removes the depth.

## What You Must Do

### T0.1 Root fix: cut view-type depth at natural seams (no `recursion_limit` anywhere)
Cause, from the overflow traces in `readiness/logs/l-debug-bin-1.99.0.log:365` and `l-wasm-lib-debug-1.99.0.log:225`: each failing type is ONE concrete nested `View<…>` type spanning a whole page tree. rustc lays out the `into_any::resolve` / `hydrate_async` async block over that type at codegen, and its depth exceeds 128. `.into_any()` erases a subtree to `AnyView`, so nesting stops at that boundary. Insert erasure at seams that already exist as UI units; markup stays byte-identical.

**Bin tree (`src/pages/login.rs`), named in the trace as `__component_login_step_router` + `__component_name_collection_form` inside LoginPage's `page-frame > auth-card` divs:**
1. `LoginStepRouter`: move each of the four step `<div …>` blocks (Step 1 phone, Step 2 OTP, Step 3 invite-code wrapper, Step 4 name-collection wrapper) out of the single `view!` into four private `#[component]`s in the same file: `PhoneStep`, `OtpStep`, `InviteCodeStep`, `NameStep`. Each component's body is the exact existing `<div …>…</div>` (same attributes, classes, `style:display` closures, testids, children, comments) and ends with `.into_any()`. Props are exactly the signals/actions/closures each block uses (`is_pending: bool`, `is_otp_error: bool`, `otp_step: Memo<bool>`, `hydrated: ReadSignal<bool>`, `request_action`, `submitted_phone`, `entered_code`, `set_entered_code`, `register_action`, `resend_cooldown: RwSignal<u32>`, and the `resend` callback). Pass `resend` as a prop typed `impl Fn() + Send + Sync + 'static`; the dual-cfg `resend` bindings stay where they are, outside every `view!` (conventions: no `#[cfg]` inside `view!`). `LoginStepRouter`'s `view!` becomes `<PhoneStep …/> <OtpStep …/> <InviteCodeStep …/> <NameStep …/>` followed by `.into_any()`.
2. `NameCollectionForm` and `InviteCodeForm`: append `.into_any()` to their `view!{…}`.
3. `LoginPage`: append `.into_any()` to its `view!{…}`.
No other change in login.rs. Names `InviteCodeForm` and `NameCollectionForm` stay, because U1 edits `NameCollectionForm`'s `<form>`.

**Lib tree (`src/admin/page.rs`), named in the trace as the `<ul class="invite-code-list">` `<For>` at :1841–1872 inside `InviteCodesSection`:**
4. Extract the `<For>` child, the whole `<li class="invite-code-card" data-testid="invite-code-row">…</li>`, into a private `#[component] fn InviteCodeCard(code: InviteCodeRow, revoke_invite_action: ServerAction<RevokeInviteCode>, hydrated: ReadSignal<bool>) -> impl IntoView`, whose body is the existing `<li>` verbatim, ending in `.into_any()`. The `<For>` child becomes `<InviteCodeCard code=code revoke_invite_action=revoke_invite_action hydrated=hydrated/>`. Pass `revoke_pending` / `i18n` by recomputing them inside the component exactly as the current code computes them; do not change what they read.
5. Append `.into_any()` to the `Ok(codes) => view!{…}` branch view, if it is not already present.

**Gate-driven extension (exhaustive rule, no discretion):** if either build (T0 gates) still overflows, the new error names the next closure (`file:line`). Apply the same move to the innermost enclosing existing UI unit at that location: a step/section/card/branch `view!` gets `.into_any()`, extracted into a `#[component]` only when it is a `<For>` child or a match/if branch. Repeat until both builds pass at the default limit and at the headroom limit (gates below). Record every extra seam in the report as `file:line → component/erasure`.

**Zero DOM change:** no element, attribute, class, testid, text, or order change. The isolated E2E (116 passed) and the HTML-diff gate below prove it.

**Lint:** new components follow the existing `#[component]` conventions. No `#[allow]` without a WHY comment. `#[allow(clippy::too_many_arguments)]` on a new component carries the same WHY as `LoginStepRouter`'s existing one, and only if clippy demands it.

### T0.3 `scripts/assert-playwright-ran.sh` (new, `chmod +x`)
```bash
#!/usr/bin/env bash
# Fail unless Playwright actually ran during this E2E invocation.
#
# WHY: `cargo leptos end-to-end` exits 0 when the cargo build fails and then never
# starts Playwright — CI was false-green from 2026-07-10 to 2026-07-25 with zero
# tests executed. Playwright's JSON reporter (end2end/playwright.config.ts) writes
# end2end/results.json on every run; a fresh file with >0 executed tests is the proof.
#
# Usage (from repo root): scripts/assert-playwright-ran.sh <start-marker>
#   <start-marker> — a file touched BEFORE the E2E command started.
set -euo pipefail

marker="${1:?usage: scripts/assert-playwright-ran.sh <start-marker>}"
results="end2end/results.json"

[ -f "$results" ] \
    || { echo "FATAL: $results missing — Playwright never ran (build failure?)"; exit 1; }
[ "$results" -nt "$marker" ] \
    || { echo "FATAL: $results predates this run — Playwright never ran (build failure?)"; exit 1; }

node -e '
const s = require(process.argv[1]).stats;
console.log(`playwright ran: expected=${s.expected} unexpected=${s.unexpected} flaky=${s.flaky} skipped=${s.skipped}`);
if (s.expected + s.unexpected + s.flaky === 0) {
    console.error("FATAL: Playwright executed 0 tests");
    process.exit(1);
}
if (s.unexpected > 0) {
    console.error("FATAL: Playwright reported failures");
    process.exit(1);
}
' "$PWD/$results"
```

### T0.4 `justfile` — guard every `cargo leptos end-to-end` recipe
Each of `e2e-dev`, `e2e-single`, `e2e-rerun`, `e2e-release` gets one line before and one line after its `cargo leptos end-to-end …` line (the end-to-end line itself is unchanged). Exact final form of `e2e-release` (apply the same two lines to the other three):
```just
e2e-release: _kill-stale db-reset db-seed
    mkdir -p target && touch target/e2e-start.marker
    SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end --release
    bash scripts/assert-playwright-ran.sh target/e2e-start.marker
```
Add above `e2e: e2e-release` a comment line: `# All end-to-end recipes assert Playwright actually ran: cargo leptos end-to-end exits 0 on build failure.`

### T0.5 `.github/workflows/ci.yml`
1. Check job: insert directly after the step `cargo clippy (hydrate / WASM)` (features match `[package.metadata.leptos]` `bin-features = ["ssr"]`, `bin-default-features = false`, `lib-features = ["hydrate"]`):
```yaml
      # clippy/test never run codegen, so layout-depth overflows in Leptos views
      # only surface in a real build.
      - name: cargo build (SSR bin — codegen)
        run: cargo build --no-default-features --features ssr --bin samete

      - name: cargo build (hydrate lib, wasm — codegen)
        run: cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate
```
2. E2E job: the step `Run E2E tests (release)` stays `run: just e2e-release` — the guard lives in the recipe (T0.4), one home for local and CI. Do NOT add a second copy of the check in ci.yml.

### T0.6 `scripts/isolated-capture.sh` — stale-binary blind spot
The harness runs Playwright directly (exit code already propagates; screenshot floor exists), but it serves whatever `target/release/samete` exists: if `cargo leptos build --release` fails without a non-zero exit, a stale binary from an earlier build would be tested. Replace the line `cargo leptos build --release` with:
```bash
# Remove the artifacts first so a failed build cannot leave a stale binary/WASM to be
# served: cargo leptos has been observed to exit 0 on cargo build failure.
rm -f target/release/samete target/site/pkg/samete.wasm
cargo leptos build --release
[ -x target/release/samete ] && [ -f target/site/pkg/samete.wasm ] \
    || { echo "[isolated-capture] FATAL: release build produced no binary/WASM — build failed"; exit 1; }
```

## Verification Gates (T0)
Build gates (the CI Check steps, run exactly):
```bash
SQLX_OFFLINE=true cargo build --no-default-features --features ssr --bin samete > /tmp/t0-bin.log 2>&1; echo "exit=$?"
SQLX_OFFLINE=true cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate > /tmp/t0-wasm.log 2>&1; echo "exit=$?"
grep -c "overflow the depth limit" /tmp/t0-bin.log /tmp/t0-wasm.log
```
**REQUIRED OUTPUT:** `exit=0`, `exit=0`, both counts `0`.
```bash
grep -rn "recursion_limit" src/ Cargo.toml .cargo 2>/dev/null; echo "count=$(grep -rn recursion_limit src/ Cargo.toml 2>/dev/null | wc -l)"
echo "${RUSTFLAGS:-}${CARGO_ENCODED_RUSTFLAGS:-}" | grep -c "recursion_limit\|crate-attr"
rustc --version
```
**REQUIRED:** `count=0`; second line `0` (no env masking: unset any `RUSTFLAGS='-Zcrate-attr=…'` / `RUSTC_BOOTSTRAP` left over from readiness work before ALL T0 gates); `rustc 1.99.0`.
Release builds at the default limit:
```bash
SQLX_OFFLINE=true cargo build --release --no-default-features --features ssr --bin samete > /tmp/t0-rel-bin.log 2>&1; echo "exit=$?"
cargo leptos build --release > /tmp/t0-leptos-rel.log 2>&1; echo "exit=$?"
grep -c "overflow the depth limit\|^error" /tmp/t0-rel-bin.log /tmp/t0-leptos-rel.log
ls -la target/release/samete target/site/pkg/samete.wasm
```
**REQUIRED:** `exit=0`, `exit=0`, both counts `0`, both artifacts listed with a timestamp from this run.
Headroom proof (temporary, then removed): add `#![recursion_limit = "96"]` as line 1 of both `src/main.rs` and `src/lib.rs`, then:
```bash
SQLX_OFFLINE=true cargo build --no-default-features --features ssr --bin samete > /tmp/t0-h96-bin.log 2>&1; echo "exit=$?"
SQLX_OFFLINE=true cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate > /tmp/t0-h96-wasm.log 2>&1; echo "exit=$?"
sed -i '/^#!\[recursion_limit = "96"\]$/d' src/main.rs src/lib.rs
git diff --quiet -- src/main.rs src/lib.rs && echo roots-untouched
```
**REQUIRED:** `exit=0`, `exit=0`, `roots-untouched`. The views fit with ≥25% margin below rustc's default. If 96 fails, add the next seam (extension rule) and repeat. Lowering the probe value is BANNED.
HTML identity (SSR output unchanged; hydration comment markers excluded):
```bash
# run once with STAGE=before (the lane's base tree, before the view edits) and once with STAGE=after; same DB state, same port
for path in /login "/login?pending=1"; do curl -s "http://127.0.0.1:3963$path" | sed -E 's/<!--[^>]*-->//g' > "/tmp/t0-html-$(echo $path | tr -c 'a-z0-9' _)-$STAGE.html"; done
curl -s -b "session=$ADMIN_TOKEN" http://127.0.0.1:3963/admin | sed -E 's/<!--[^>]*-->//g' > /tmp/t0-html-admin-$STAGE.html
diff -q /tmp/t0-html-_login-before.html /tmp/t0-html-_login-after.html && diff -q /tmp/t0-html-_login_pending_1-before.html /tmp/t0-html-_login_pending_1-after.html && diff -q /tmp/t0-html-admin-before.html /tmp/t0-html-admin-after.html && echo html-identical
```
(Server: release binary on 127.0.0.1:3963 with `SAMETE_SMS_DRY_RUN=true SAMETE_TEST_MODE=true` against a sibling DB seeded with `seed/test_admin.sql`, plus ≥1 invite code inserted via `INSERT INTO invite_codes (code, distributor_id) VALUES ('alpha-beta', '00000000-0000-0000-0000-000000000001')`. `ADMIN_TOKEN` comes from logging in with curl: POST `request_otp` then `verify_otp_code` with `000000`, and read the `session=` cookie. The "before" binary is the base tree built with an env-only `RUSTFLAGS='-Zcrate-attr=recursion_limit="256"' RUSTC_BOOTSTRAP=1`, which is the ONLY permitted use of that flag: a throwaway comparison build, never committed, unset afterwards.) **REQUIRED:** `html-identical`.
```bash
grep -c "assert-playwright-ran.sh target/e2e-start.marker" justfile
grep -c "touch target/e2e-start.marker" justfile
grep -c "cargo leptos end-to-end" justfile
```
**REQUIRED:** `4`, `4`, `4`.
```bash
grep -n "cargo build --no-default-features --features ssr --bin samete\|cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate\|just e2e-release" .github/workflows/ci.yml
```
**REQUIRED:** exactly three matches (two in the Check job, `just e2e-release` once in the E2E job).
```bash
python3 -c "import yaml,sys; yaml.safe_load(open('.github/workflows/ci.yml'))" && echo yaml-ok
```
**REQUIRED:** `yaml-ok`.
Standard gates (Shared environment).

Positive E2E gates (both paths, with T0.1–T0.6 applied):
1. Isolated harness: the E2E gate from Shared environment with `<unit>=t0`. **REQUIRED:** `exit=0`, `116 passed`, `2 skipped`, no `failed`/`flaky`.
2. end-to-end path with the guard, on an own port and own DB (do NOT use `just e2e*`: they kill :3000 and reset `samete`):
```bash
export T0DB=postgres://samete:samete@localhost:5432/samete_t0e2e
DATABASE_URL=$T0DB sqlx database drop -y; DATABASE_URL=$T0DB sqlx database create
DATABASE_URL=$T0DB sqlx migrate run && psql $T0DB -f seed/test_admin.sql
mkdir -p target && touch target/e2e-start.marker
DATABASE_URL=$T0DB LEPTOS_SITE_ADDR=127.0.0.1:3961 CAPTURE_BASE_URL=http://127.0.0.1:3961 \
  SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true \
  cargo leptos end-to-end --release > /tmp/t0-e2e-pos.log 2>&1
bash scripts/assert-playwright-ran.sh target/e2e-start.marker; echo "guard=$?"
DATABASE_URL=$T0DB sqlx database drop -y
```
**REQUIRED:** `playwright ran: expected=116 unexpected=0 flaky=0 skipped=2` (if the line reports a different `expected` with `unexpected=0`, paste it — the isolated-harness count above is the authority) and `guard=0`.

Sabotage gates (prove each guard turns RED on a broken build; run AFTER committing T0 so the revert is `git checkout`):
```bash
git checkout HEAD~1 -- src/pages/login.rs src/admin/page.rs   # reintroduce the real defect (pre-refactor views)
git diff --cached --stat                                      # must show exactly those two files
```
(a) CI Check steps:
```bash
SQLX_OFFLINE=true cargo build --no-default-features --features ssr --bin samete > /tmp/t0-sab-bin.log 2>&1; echo "exit=$?"
SQLX_OFFLINE=true cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate > /tmp/t0-sab-wasm.log 2>&1; echo "exit=$?"
grep -c "overflow the depth limit" /tmp/t0-sab-bin.log /tmp/t0-sab-wasm.log
```
**REQUIRED:** `exit=101`, `exit=101`, both counts ≥ `1`.
(b) end-to-end guard (the CI E2E path):
```bash
touch target/e2e-start.marker
SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true DATABASE_URL=postgres://unused LEPTOS_SITE_ADDR=127.0.0.1:3962 \
  cargo leptos end-to-end --release > /tmp/t0-sab-e2e.log 2>&1; echo "leptos_exit=$?"
bash scripts/assert-playwright-ran.sh target/e2e-start.marker; echo "guard=$?"
```
**REQUIRED:** `leptos_exit=0` is acceptable (that is the defect being guarded), the guard prints `FATAL: end2end/results.json predates this run — Playwright never ran (build failure?)` (or `… missing …`), and `guard=1`.
(c) isolated harness:
```bash
bash scripts/isolated-capture.sh t0sab full > /tmp/t0-sab-iso.log 2>&1; echo "exit=$?"
grep -c "release build produced no binary/WASM" /tmp/t0-sab-iso.log
```
**REQUIRED:** `exit=` non-zero and count `1` (or the build step itself exits non-zero first; then paste the failing line — either way exit ≠ 0 and no Playwright line `passed` appears: `grep -c " passed" /tmp/t0-sab-iso.log` = `0`).
Revert and prove clean:
```bash
git checkout HEAD -- src/pages/login.rs src/admin/page.rs
git status --short
grep -rc "recursion_limit" src/ | grep -v ":0$" | wc -l
```
**REQUIRED:** `git status --short` prints nothing; last count `0`. Then rerun the T0 build gates once more: `exit=0`, `exit=0`.

## T0 Definition of Done (in addition to the common DoD)
Commit message: `fix(build): reduce view type depth at component seams and make CI/E2E fail on build failure`. Write-set exactly: `src/pages/login.rs`, `src/admin/page.rs`, `scripts/assert-playwright-ran.sh` (new), `scripts/isolated-capture.sh`, `justfile`, `.github/workflows/ci.yml`. Report all sabotage outputs verbatim, plus every extra seam added under the extension rule.

---

# T1 — Server survives the leptos_i18n disposed-signal panic

## Why This Matters
Under concurrent load the production server process dies. `leptos_i18n` 0.6.1 `context.rs:213` (reached from `src/app.rs:73` `provide_i18n_context()`) reads `locale_signal.get()` inside `Effect::new_isomorphic`. On the server that effect's first run happens in a detached tokio task, which can be polled after `leptos_integration_utils` force-disposes the request Owner. The read then panics with `Tried to access a reactive value that has already been disposed`. Under `[profile.release] panic = "abort"` (Cargo.toml:98) that task panic aborts the whole server. The Dockerfile builds `cargo leptos build --release`, so production has the same exposure. Evidence: `readiness/n-panic.md`, `logs/n-stress-abort-debuginfo-backtrace.log` (8/8 server lifetimes killed), `logs/n-stress-unwind.log` (unwind: 4 panics absorbed, server keeps serving 200s), E2E crashes 4/5 on the cargo-leptos path.

## Decision (final)
`[profile.release] panic = "unwind"` for the server, with `panic = "abort"` set explicitly in `[profile.wasm-release]`. Verified wiring: `[package.metadata.leptos]` has `lib-profile-release = "wasm-release"` (Cargo.toml:127) and no `bin-profile-release`, so the server bin uses `release` and the WASM lib uses `wasm-release`, which `inherits = "release"`. Without the explicit override the WASM would inherit unwind and grow. Rejected: `[patch.crates-io]` of leptos_i18n with `try_get()`. conventions.md forbids a patch without running the patched crate's own suite, it is a vendored fork to maintain, and 0.6.2 upstream has the same code. Unwind confines every future task panic of this class, and it is a two-line change. The upstream fix stays out of scope.

## What You Must Do

### T1.0 Record the RED baseline and WASM size BEFORE editing (in the T1 worktree; build precondition per Lanes)
```bash
cargo leptos build --release > /tmp/t1-build-before.log 2>&1; echo "build=$?"
brotli -c -q 11 target/site/pkg/samete.wasm | wc -c > /tmp/t1-wasm-br-before.txt
sha256sum target/site/pkg/samete.wasm | cut -d' ' -f1 > /tmp/t1-wasm-sha-before.txt
cp target/release/samete /tmp/t1-samete-abort
cat /tmp/t1-wasm-br-before.txt
```
**REQUIRED:** `build=0` and a byte count (≈471 KB, i.e. 460000–490000). Keep `/tmp/t1-samete-abort` for the RED gate.

### T1.1 `Cargo.toml`
Exact final form of the two profiles:
```toml
[profile.release]
strip = true
lto = "thin"
# WHY: unwind on the server. leptos_i18n 0.6.x's SSR isomorphic effect
# (context.rs:213) can read a disposed signal in a detached tokio task after the
# request Owner is dropped; under abort that panic kills the whole server, under
# unwind tokio confines it to the task. WASM keeps abort (wasm-release below).
panic = "unwind"

[profile.wasm-release]
inherits = "release"
opt-level = 'z'
lto = true
codegen-units = 1
# WHY: explicit — `inherits = "release"` would otherwise pull in unwind and grow the WASM.
panic = "abort"
```
No other Cargo.toml change. `Cargo.lock` must not change.

### T1.2 `scripts/ssr-stress.sh` (new, `chmod +x`) — the regression probe
```bash
#!/usr/bin/env bash
# SSR concurrency stress: rounds of client-aborted requests at SSR routes, then a
# liveness check after every round.
#
# WHY: leptos_i18n 0.6.x's server-side isomorphic effect (context.rs:213) can read a
# disposed signal in a detached tokio task once the request Owner is disposed. With
# panic = "abort" that kills the server; with unwind the panic stays in the task.
# This is the fast repro (orchestration_log/recon/2026-10-04/readiness/n-panic.md).
#
# Usage: scripts/ssr-stress.sh <port> <server-log-file> [rounds=80]
# Exit 0 = server alive after all rounds; exit 1 = server died.
set -uo pipefail

port="${1:?usage: scripts/ssr-stress.sh <port> <server-log-file> [rounds]}"
log="${2:?usage: scripts/ssr-stress.sh <port> <server-log-file> [rounds]}"
rounds="${3:-80}"

panics() { grep -c "panicked" "$log" || true; }

for round in $(seq 1 "$rounds"); do
    pids=()
    for path in / /admin /onboarding /login; do
        for timeout in 0.003 0.006 0.01 0.02 0.04 0.08; do
            curl -s -o /dev/null -b session=stresstoken123 --max-time "$timeout" "http://127.0.0.1:${port}${path}" & pids+=($!)
            curl -s -o /dev/null --max-time "$timeout" "http://127.0.0.1:${port}${path}" & pids+=($!)
        done
    done
    wait "${pids[@]}" 2>/dev/null
    if ! curl -sf -o /dev/null --max-time 5 "http://127.0.0.1:${port}/login"; then
        echo "DEAD at round ${round}; panics logged: $(panics)"
        exit 1
    fi
done
echo "ALIVE after ${rounds} rounds; panics logged: $(panics)"
```

### T1.3 Build after the change
```bash
cargo leptos build --release > /tmp/t1-build-after.log 2>&1; echo "build=$?"
brotli -c -q 11 target/site/pkg/samete.wasm | wc -c > /tmp/t1-wasm-br-after.txt
sha256sum target/site/pkg/samete.wasm | cut -d' ' -f1 > /tmp/t1-wasm-sha-after.txt
paste /tmp/t1-wasm-br-before.txt /tmp/t1-wasm-br-after.txt
```

### T1.4 `orchestration_log/reference/deferred_items.md`
ONLY after every T1 gate below is green: delete the whole line beginning `- Leptos SSR reactive-disposal panic` (resolve = delete, per conventions 2026-07-13). No other edit to that file.

## Verification Gates (T1)
Profile wiring:
```bash
awk '/^\[profile.release\]/,/^$/' Cargo.toml | grep -c 'panic = "unwind"'
awk '/^\[profile.wasm-release\]/,/^$/' Cargo.toml | grep -c 'panic = "abort"'
grep -c '^bin-profile-release' Cargo.toml; grep -c '^lib-profile-release = "wasm-release"' Cargo.toml
git diff --quiet Cargo.lock && echo lock-unchanged
```
**REQUIRED:** `1`, `1`, `0`, `1`, `lock-unchanged`.

WASM size (must not regress):
```bash
echo "before=$(cat /tmp/t1-wasm-br-before.txt) after=$(cat /tmp/t1-wasm-br-after.txt)"
[ "$(cat /tmp/t1-wasm-br-after.txt)" -le "$(cat /tmp/t1-wasm-br-before.txt)" ] && echo wasm-ok
```
**REQUIRED:** `wasm-ok`. Paste both numbers in the report. The expected outcome is identical sizes, and identical `sha256` (`cmp /tmp/t1-wasm-sha-before.txt /tmp/t1-wasm-sha-after.txt`), since the effective wasm profile did not change. Report a sha mismatch as a concern, not a failure.

Stress server launcher (used by RED and GREEN; `<BIN>` and `<N>` vary):
```bash
export T1DB=postgres://samete:samete@localhost:5432/samete_t1
DATABASE_URL=$T1DB sqlx database drop -y; DATABASE_URL=$T1DB sqlx database create
DATABASE_URL=$T1DB sqlx migrate run && psql $T1DB -f seed/test_admin.sql
LEPTOS_SITE_ADDR=127.0.0.1:3971 LEPTOS_SITE_ROOT=target/site LEPTOS_SITE_PKG_DIR=pkg LEPTOS_OUTPUT_NAME=samete \
  DATABASE_URL=$T1DB SAMETE_SMS_DRY_RUN=true <BIN> > /tmp/t1-server-<N>.log 2>&1 &
SRV=$!; until curl -sf -o /dev/null http://127.0.0.1:3971/login; do sleep 0.5; done
bash scripts/ssr-stress.sh 3971 /tmp/t1-server-<N>.log 80; echo "stress=$?"
kill $SRV 2>/dev/null; wait $SRV 2>/dev/null
```
RED (before; `<BIN>=/tmp/t1-samete-abort`, `<N>=red1..red3`): run up to 3 times and stop at the first `DEAD`.
**REQUIRED:** at least one run prints `DEAD at round … panics logged: ≥1` with `stress=1`, and `grep -m1 "already been disposed" /tmp/t1-server-redK.log` matches. If all 3 runs print ALIVE, STOP: report BLOCKED (the probe did not reproduce, so the GREEN gate would prove nothing).
GREEN (after; `<BIN>=./target/release/samete`, `<N>=green1..green3`): run all 3.
**REQUIRED:** all 3 print `ALIVE after 80 rounds` with `stress=0`. The sum of `panics logged` across the 3 runs must be ≥ 1, which proves the trigger fired and was absorbed. If the sum is 0, run 3 more. If the sum is still 0, report BLOCKED.

E2E on the path that crashes most (cargo-leptos end-to-end, own port + own DB; Playwright-ran check per Lanes). Run 3 times:
```bash
export T1E=postgres://samete:samete@localhost:5432/samete_t1e2e
DATABASE_URL=$T1E sqlx database drop -y; DATABASE_URL=$T1E sqlx database create
DATABASE_URL=$T1E sqlx migrate run && psql $T1E -f seed/test_admin.sql
touch target/e2e-start.marker
DATABASE_URL=$T1E LEPTOS_SITE_ADDR=127.0.0.1:3972 CAPTURE_BASE_URL=http://127.0.0.1:3972 \
  SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true cargo leptos end-to-end --release > /tmp/t1-e2e-K.log 2>&1
bash scripts/assert-playwright-ran.sh target/e2e-start.marker; echo "guard=$?"
```
**REQUIRED (each of 3 runs, fresh DB each time):** `guard=0` and `playwright ran: … unexpected=0`. `grep -c ERR_CONNECTION_REFUSED /tmp/t1-e2e-K.log` = `0`. Plus the isolated-harness E2E gate: 116 passed / 2 skipped.
Standard gates.
After green: T1.4, then `grep -c "reactive-disposal" orchestration_log/reference/deferred_items.md` → **REQUIRED:** `0`.

Commit: `fix(build): unwind panics on the server so a task panic cannot kill it`. Write-set: `Cargo.toml`, `scripts/ssr-stress.sh` (new), `orchestration_log/reference/deferred_items.md`.

---

# U1 — Server-verified registration tickets + invite throttle

## Why This Matters
Any person with one invite code can register any phone number they do not own (no SMS reaches the victim). OTP exists to prove phone ownership; the current cookie proves nothing. Invite codes can be brute-forced with no limit.

## What You Must Do

### U1.1 Migration (new file, exact name and content)
`migrations/20261004000001_registration_tickets.sql`:
```sql
-- Server-side proof that a phone passed OTP verification and may register.
-- The client holds only a random token; the phone is never read from the client.
CREATE TABLE registration_tickets (
    token_hash TEXT PRIMARY KEY,
    phone TEXT NOT NULL,
    invite_attempts INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL
);
```
Then: `sqlx migrate run` against `samete_u1`.

### U1.2 `src/auth.rs` — TDD, pure helpers first
Step 1 — add failing tests to the existing `#[cfg(test)] mod tests` (change its `use` to `use super::{constant_time_hash_eq, extract_cookie, generate_token, sha256_hex};`):
```rust
    fn parts_with_cookie(header: &str) -> http::request::Parts {
        http::Request::builder()
            .header(http::header::COOKIE, header)
            .body(())
            .expect("request builds")
            .into_parts()
            .0
    }

    #[test]
    fn generate_token_is_43_url_safe_characters() {
        let token = generate_token();
        assert_eq!(token.len(), 43);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn generate_token_is_distinct_per_call() {
        assert_ne!(generate_token(), generate_token());
    }

    #[test]
    fn extract_cookie_finds_named_cookie_among_several() {
        let parts = parts_with_cookie("session=abc; registration_ticket=xyz");
        assert_eq!(extract_cookie(&parts, "registration_ticket").as_deref(), Some("xyz"));
        assert_eq!(extract_cookie(&parts, "session").as_deref(), Some("abc"));
    }

    #[test]
    fn extract_cookie_does_not_match_a_longer_name() {
        let parts = parts_with_cookie("registration_ticket_old=zzz; xsession=1");
        assert_eq!(extract_cookie(&parts, "registration_ticket"), None);
        assert_eq!(extract_cookie(&parts, "session"), None);
    }

    #[test]
    fn extract_cookie_returns_none_without_cookie_header() {
        let parts = http::Request::builder().body(()).expect("request builds").into_parts().0;
        assert_eq!(extract_cookie(&parts, "session"), None);
    }
```
Step 2 — run `SQLX_OFFLINE=true cargo test --features ssr auth::tests 2>&1 | grep -E "error\[|test result"`. **REQUIRED:** compile error naming `generate_token` / `extract_cookie` (red).

Step 3 — implement (replace `extract_session_cookie` entirely; its one caller `current_user` becomes `extract_cookie(parts, "session")`):
```rust
/// Generate a random 32-byte URL-safe token (43 chars, no padding).
pub(crate) fn generate_token() -> String {
    use rand::RngCore as _;
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// Read cookie `name` from the request's `Cookie` header (exact name match).
pub(crate) fn extract_cookie(parts: &http::request::Parts, name: &str) -> Option<String> {
    let cookie_header = parts.headers.get(http::header::COOKIE)?.to_str().ok()?;
    cookie_header.split(';').find_map(|pair| {
        let (key, value) = pair.trim().split_once('=')?;
        (key == name).then(|| value.to_owned())
    })
}
```
`create_session`: replace its first four lines (`use rand::RngCore`, `bytes`, `fill_bytes`, `raw_token = …encode`) with `let raw_token = generate_token();`.

Step 4 — add the ticket API (place after `create_session`):
```rust
/// Maximum invalid invite-code submissions per registration ticket.
/// After this many, the ticket stops resolving and the user must re-verify by OTP.
pub const MAX_INVITE_ATTEMPTS: i32 = 5;

/// Record that `phone` passed OTP verification and may self-register.
///
/// Returns the raw token for the `registration_ticket` cookie; only its
/// SHA-256 is stored. Ticket lives 10 minutes.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn create_registration_ticket(pool: &PgPool, phone: &str) -> Result<String, AppError> {
    let raw_token = generate_token();
    let token_hash = sha256_hex(&raw_token);
    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM registration_tickets WHERE expires_at < now()")
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        r#"
        INSERT INTO registration_tickets (token_hash, phone, expires_at)
        VALUES ($1, $2, now() + interval '10 minutes')
        "#,
        token_hash,
        phone,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(raw_token)
}

/// Resolve a live ticket to its OTP-verified phone without consuming it.
///
/// # Errors
///
/// Returns `Err(AppError::Unauthorized)` if the ticket is unknown, expired,
/// or exhausted; `Err(AppError::Database(_))` on DB failure.
pub async fn registration_ticket_phone(pool: &PgPool, raw_token: &str) -> Result<String, AppError> {
    let token_hash = sha256_hex(raw_token);
    sqlx::query_scalar!(
        r#"
        SELECT phone FROM registration_tickets
        WHERE token_hash = $1 AND expires_at > now() AND invite_attempts < $2
        "#,
        token_hash,
        MAX_INVITE_ATTEMPTS,
    )
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Unauthorized)
}

/// Consume a live ticket inside the caller's transaction, returning its phone.
/// Rolling back the transaction restores the ticket.
///
/// # Errors
///
/// Returns `Err(AppError::Unauthorized)` if the ticket is unknown, expired,
/// or exhausted; `Err(AppError::Database(_))` on DB failure.
pub async fn consume_registration_ticket(
    conn: &mut sqlx::PgConnection,
    raw_token: &str,
) -> Result<String, AppError> {
    let token_hash = sha256_hex(raw_token);
    sqlx::query_scalar!(
        r#"
        DELETE FROM registration_tickets
        WHERE token_hash = $1 AND expires_at > now() AND invite_attempts < $2
        RETURNING phone
        "#,
        token_hash,
        MAX_INVITE_ATTEMPTS,
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(AppError::Unauthorized)
}

/// Count one invalid invite-code submission against the ticket.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn record_failed_invite_attempt(pool: &PgPool, raw_token: &str) -> Result<(), AppError> {
    let token_hash = sha256_hex(raw_token);
    sqlx::query!(
        "UPDATE registration_tickets SET invite_attempts = invite_attempts + 1 WHERE token_hash = $1",
        token_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}
```
Step 5 — `sqlx prepare` (shared env), then rerun step-2 command. **REQUIRED:** `test result: ok.` with the 5 new test names passing (`cargo test --features ssr auth::tests -- --list` lists `generate_token_is_43_url_safe_characters`, `generate_token_is_distinct_per_call`, `extract_cookie_finds_named_cookie_among_several`, `extract_cookie_does_not_match_a_longer_name`, `extract_cookie_returns_none_without_cookie_header`).

### U1.3 `src/pages/login.rs`
Define once near `set_cookie_header`:
```rust
/// Cookie carrying the opaque registration-ticket token (never the phone).
#[cfg(feature = "ssr")]
const REGISTRATION_COOKIE: &str = "registration_ticket";
```
1. `verify_otp_code`, `None =>` branch: replace the body with:
```rust
        None => {
            // Unknown phone passed OTP: record server-side proof and hand the
            // client an opaque token. The phone is never read back from the client.
            let raw_ticket = auth::create_registration_ticket(&pool, &normalized)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            let response_options =
                leptos::prelude::expect_context::<leptos_axum::ResponseOptions>();
            let cookie = set_cookie_header(REGISTRATION_COOKIE, &raw_ticket, 600);
            response_options.append_header(
                axum::http::header::SET_COOKIE,
                axum::http::HeaderValue::from_str(&cookie)
                    .map_err(|e| ServerFnError::new(format!("invalid cookie: {e}")))?,
            );
            leptos_axum::redirect("/login?pending=1");
            Ok(false)
        }
```
Update the function's doc bullet "No account → set a short-lived `pending_phone` cookie…" to: "No account → store a registration ticket server-side, set the opaque `registration_ticket` cookie (HttpOnly, 10 min), redirect to `/login?pending=1`". Update the inline comment block above `match` (case 3) the same way; delete the "Deactivated user … pending_phone" wording → "without issuing a registration ticket".

2. `validate_invite_code`: replace the "Require that the phone was OTP-verified" block with:
```rust
    let parts = leptos::context::use_context::<http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;
    let not_verified = || ServerFnError::new(td_string!(Locale::uk, auth_phone_not_verified));
    let ticket = crate::auth::extract_cookie(&parts, REGISTRATION_COOKIE).ok_or_else(not_verified)?;
    crate::auth::registration_ticket_phone(&pool, &ticket)
        .await
        .map_err(|_| not_verified())?;
```
Keep the empty-code check unchanged (empty code does NOT count as an attempt). Replace the final `match status { … }` with:
```rust
    let rejection = match status {
        Some(InviteCodeStatus::Unused) => return Ok(code),
        None => td_string!(Locale::uk, auth_invite_code_invalid),
        Some(InviteCodeStatus::Used) => td_string!(Locale::uk, auth_invite_code_used),
        Some(InviteCodeStatus::Revoked) => td_string!(Locale::uk, auth_invite_code_revoked),
    };
    crate::auth::record_failed_invite_attempt(&pool, &ticket)
        .await
        .map_err(crate::error::AppError::into_server_fn_error)?;
    Err(ServerFnError::new(rejection))
```
(If `td_string!` returns a type that does not unify across arms, bind each arm with `.to_string()`; the `ServerFnError::new` argument stays the localized message.) Doc comment: replace every `pending_phone` mention with "registration ticket"; add bullet "- the ticket is unknown, expired, or exhausted (5 invalid codes)".

3. `register_with_code`: replace the body from `let parts = …` through `let _ = tx.commit().await;` with:
```rust
    let parts = leptos::context::use_context::<http::request::Parts>()
        .ok_or_else(|| ServerFnError::new("no request parts in context"))?;

    let Some(ticket) = auth::extract_cookie(&parts, REGISTRATION_COOKIE) else {
        leptos_axum::redirect("/login");
        return Ok(());
    };

    let name = name.trim().to_owned();
    if name.is_empty() {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    }

    let Ok(mut tx) = pool.begin().await else {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };

    // The phone comes ONLY from the server-side ticket. Consumed in this tx;
    // any early return drops the tx, rolling the ticket back.
    let Ok(phone) = auth::consume_registration_ticket(&mut *tx, &ticket).await else {
        leptos_axum::redirect("/login");
        return Ok(());
    };

    let code_row = sqlx::query!(
        r#"SELECT id, status AS "status: String" FROM invite_codes WHERE code = $1 FOR UPDATE"#,
        code,
    )
    .fetch_optional(&mut *tx)
    .await;
    let Ok(Some(code_row)) = code_row.map(|row| row.filter(|r| r.status == "unused")) else {
        drop(tx);
        if let Err(e) = auth::record_failed_invite_attempt(&pool, &ticket).await {
            tracing::warn!(error = %e, "failed to record invite attempt");
        }
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };

    let Ok(user_id) = sqlx::query_scalar!(
        r#"INSERT INTO users (phone, name) VALUES ($1, $2) RETURNING id"#,
        phone,
        name,
    )
    .fetch_one(&mut *tx)
    .await
    else {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };

    let Ok(_) = sqlx::query!(
        r#"UPDATE invite_codes SET status = 'used', redeemer_id = $1, redeemed_at = now() WHERE id = $2"#,
        user_id,
        code_row.id,
    )
    .execute(&mut *tx)
    .await
    else {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };

    let Ok(()) = tx.commit().await else {
        leptos_axum::redirect("/login?pending=1");
        return Ok(());
    };
```
Note: the `SELECT … invite_codes … FOR UPDATE`, `INSERT INTO users` and `UPDATE invite_codes` SQL strings are byte-identical to HEAD (keep them so — do not re-indent). The DB-error case of the code lookup also counts as an attempt; accept that. Remove `phone as phone_mod` from this fn's `use` (phone already normalized at ticket creation). In the cookie-clearing block replace `set_cookie_header("pending_phone", "", 0)` with `set_cookie_header(REGISTRATION_COOKIE, "", 0)`. Rewrite the doc comment steps 1/4/7 to "registration ticket" wording.

4. Delete `extract_pending_phone_cookie` entirely. Delete `check_pending_registration` entirely (server fn, its doc comment and `#[allow]`) — it has zero callers (`grep -rn check_pending_registration src end2end` = only its definition). In the `LoginPage` doc comment delete the paragraph "Cookie detection for step 3/4 is done via a server Resource (`check_pending_registration`) …" and change "server sets `pending_phone` cookie" → "server issues a registration ticket".

5. `NameCollectionForm`: add `data-testid="register-form"` to the `<form method="post" action=RegisterWithCode::url()>` element. No other view! change.

### U1.4 E2E — POM (`end2end/tests/fixtures/mail_club_page.ts`)
- Comments: replace every `pending_phone` with `registration_ticket` (lines ~297, ~313).
- Add after `selfRegister` (add `APIRequestContext` to the existing `@playwright/test` type import in this file):
```ts
  /**
   * Submit one invite code on the invite-code step (reached via
   * reachInviteCodeStep). Waits for the server response; caller asserts.
   */
  async submitInviteCode(code: string) {
    await this.page.getByTestId("invite-code-input").fill(code);
    await this.clickAndWaitForResponse(
      this.page.getByTestId("submit-invite-code-button"),
      "validate_invite_code",
    );
  }

  /**
   * Attack probe (Story 1.1 security AC): POST register_with_code from a
   * cookieless API context with a forged ticket cookie whose value is a phone.
   * Returns the response's Set-Cookie header ("" if none).
   */
  async forgeRegistration(request: APIRequestContext, phone: string, code: string): Promise<string> {
    await this.page.goto("/login");
    const action = await this.page.getByTestId("register-form").getAttribute("action");
    expect(action).toBeTruthy();
    const response = await request.post(action as string, {
      form: { code, name: "Підробка" },
      headers: { Cookie: `registration_ticket=${phone}; pending_phone=${phone}` },
      maxRedirects: 0,
    });
    return response.headers()["set-cookie"] ?? "";
  }
```

### U1.5 E2E — spec (`end2end/tests/mail_club.spec.ts`)
- `EXTRA_PHONES`: add `FORGED_COOKIE_TEST: "+380670000008",` and `THROTTLE_TEST: "+380670000009",`.
- `CODES` type and initializer: add `FORGE: string` / `FORGE: ""`.
- Insert immediately after the test `"1.1 — used invite code is rejected"` (inside the same Epic 1 describe):
```ts
    test("setup — generate invite code for registration-security tests", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      CODES.FORGE = await app.generateInviteCode();
      expect(CODES.FORGE.length).toBeGreaterThan(0);
    });

    // Story 1.1 security AC: registration requires a server-verified OTP for that phone
    test("1.1 — forged registration cookie cannot create an account", async ({ page, request }) => {
      const app = new MailClubPage(page);
      const setCookie = await app.forgeRegistration(request, EXTRA_PHONES.FORGED_COOKIE_TEST, CODES.FORGE);
      expect(setCookie).not.toMatch(/session=[^;]+/);
      // The phone is still unregistered: a real OTP verify routes it to the invite step.
      await app.reachInviteCodeStep(EXTRA_PHONES.FORGED_COOKIE_TEST);
      await expect(page.getByTestId("invite-code-step")).toBeVisible();
      // The code was not consumed.
      await app.login(ADMIN_PHONE);
      await app.expectInviteCodeStatus(CODES.FORGE, "unused");
    });

    // Story 1.1 security AC: invite-code guessing is throttled per OTP verification
    test("1.1 — invite code attempts are throttled per verification", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.reachInviteCodeStep(EXTRA_PHONES.THROTTLE_TEST);
      for (let i = 0; i < 5; i++) {
        await app.submitInviteCode(`wrong-guess-${i}`);
        await expect(page.getByTestId("invite-code-error")).not.toBeEmpty();
      }
      // A valid unused code is now refused: the ticket is exhausted.
      await app.submitInviteCode(CODES.FORGE);
      await expect(page.getByTestId("invite-code-error")).toContainText("Почни спочатку");
      await expect(page.getByTestId("legal-name-input")).not.toBeVisible();
    });

    test("setup — revoke registration-security invite code", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.revokeInviteCode(CODES.FORGE);
      await app.expectInviteCodeStatus(CODES.FORGE, "revoked");
    });
```
(The code is revoked so no extra active participant or unused code leaks into later count assertions such as 5.3's "3 active participants".)

## Verification Gates (U1)
```bash
grep -rn "pending_phone" src/
```
**REQUIRED:** zero matches.
```bash
grep -rn "extract_pending_phone_cookie\|check_pending_registration\|CheckPendingRegistration" src/ end2end/tests/
```
**REQUIRED:** zero matches.
```bash
grep -n "fn extract_session_cookie" src/auth.rs
```
**REQUIRED:** zero matches.
```bash
awk '/pub async fn register_with_code/,/^}/' src/pages/login.rs | grep -c "normalize"
```
**REQUIRED:** `0` (the registered phone comes only from the ticket).
```bash
awk '/pub async fn register_with_code/,/^}/' src/pages/login.rs | grep -c "let _ ="
```
**REQUIRED:** `0`.
```bash
ls migrations/ | tail -1
```
**REQUIRED:** `20261004000001_registration_tickets.sql`.
Standard gates, sqlx regeneration, E2E gate (titles: `1.1 — forged registration cookie cannot create an account`, `1.1 — invite code attempts are throttled per verification`, plus all pre-existing `1.1`/`1.6` titles passing). Expected passed count = baseline + 4.

Manual adversarial gate (debug binary on its own port + `samete_u1`; `samete_u1` must be migrated):
```bash
SQLX_OFFLINE=true cargo build --features ssr
DATABASE_URL=postgres://samete:samete@localhost:5432/samete_u1 SAMETE_SMS_DRY_RUN=true LEPTOS_SITE_ADDR=127.0.0.1:3911 ./target/debug/samete > /tmp/u1-server.log 2>&1 &
SRV=$!; until curl -sf http://127.0.0.1:3911/login >/dev/null; do sleep 1; done
ACTION=$(curl -s http://127.0.0.1:3911/login | grep -o 'action="[^"]*register_with_code[^"]*"' | head -1 | cut -d'"' -f2)
echo "ACTION=$ACTION"   # REQUIRED: non-empty path
curl -s -o /dev/null -D - -X POST "http://127.0.0.1:3911$ACTION" -H 'Cookie: registration_ticket=+380671234567; pending_phone=+380671234567' --data-urlencode 'code=x-y' --data-urlencode 'name=X' | grep -i "set-cookie: session=[^;]"
psql postgres://samete:samete@localhost:5432/samete_u1 -tAc "SELECT count(*) FROM users WHERE phone='+380671234567'"
kill $SRV
```
**REQUIRED:** the `grep -i set-cookie: session=` prints nothing; psql prints `0`.

---

# U2 — Test mode structurally confined to dry-run + loopback

Lane base: working branch + `git merge worktree-agent-a1fe18df1d59b4c14` (U3; provides `Config.admin_bootstrap`). `auth.rs`/`login.rs` edits target HEAD's `create_otp`/`check_otp_rate_limit`/`request_otp`, which U1 does not change.

## Why This Matters
One stray env var in Coolify turns OTP into `000000` for every account including admin. The flag must be impossible to combine with real SMS and impossible on a network-exposed bind.

## What You Must Do

### U2.1 `src/config.rs` — TDD
Step 1 — append a test module (failing: types do not exist):
```rust
#[cfg(test)]
mod tests {
    use super::{Config, ConfigError, SmsMode, sms_mode_from_vars};

    fn config_with(sms: SmsMode) -> Config {
        Config { database_url: String::new(), sms, admin_bootstrap: None }
    }

    #[test]
    fn dry_run_without_test_mode() {
        assert_eq!(sms_mode_from_vars(Some("true"), None, None, None).ok(), Some(SmsMode::DryRun { test_mode: false }));
    }

    #[test]
    fn dry_run_with_test_mode() {
        assert_eq!(sms_mode_from_vars(Some("true"), Some("true"), None, None).ok(), Some(SmsMode::DryRun { test_mode: true }));
    }

    #[test]
    fn test_mode_without_dry_run_is_refused_even_with_credentials() {
        let result = sms_mode_from_vars(None, Some("true"), Some("tok".into()), Some("snd".into()));
        assert!(matches!(result, Err(ConfigError::TestModeRequiresDryRun)));
    }

    #[test]
    fn live_requires_token() {
        assert!(matches!(sms_mode_from_vars(None, None, None, Some("snd".into())), Err(ConfigError::MissingTurbosmsToken)));
    }

    #[test]
    fn live_rejects_empty_sender() {
        assert!(matches!(sms_mode_from_vars(None, None, Some("tok".into()), Some(String::new())), Err(ConfigError::EmptyTurbosmsSender)));
    }

    #[test]
    fn live_with_credentials() {
        assert_eq!(
            sms_mode_from_vars(None, None, Some("tok".into()), Some("snd".into())).ok(),
            Some(SmsMode::Live { token: "tok".into(), sender: "snd".into() })
        );
    }

    #[test]
    fn only_literal_true_enables_flags() {
        assert!(matches!(sms_mode_from_vars(Some("1"), Some("TRUE"), None, None), Err(ConfigError::MissingTurbosmsToken)));
    }

    #[test]
    fn test_mode_refuses_non_loopback_bind() {
        let config = config_with(SmsMode::DryRun { test_mode: true });
        assert!(matches!(config.check_bind_addr("0.0.0.0:3000".parse().expect("addr")), Err(ConfigError::TestModeRequiresLoopback(_))));
        assert!(config.check_bind_addr("127.0.0.1:3000".parse().expect("addr")).is_ok());
    }

    #[test]
    fn non_test_mode_allows_any_bind() {
        let config = config_with(SmsMode::DryRun { test_mode: false });
        assert!(config.check_bind_addr("0.0.0.0:3000".parse().expect("addr")).is_ok());
    }
}
```
(`admin_bootstrap: None` comes from U3's branch merged into this lane's base; the field is named `admin_bootstrap` there, verified.)
Step 2 — `SQLX_OFFLINE=true cargo test --features ssr config::tests` → **REQUIRED:** compile errors (red).
Step 3 — implement. Add `#[derive(PartialEq, Eq)]`-capable types:
```rust
/// How SMS is delivered. Test mode exists only inside `DryRun`, so a server
/// that sends real SMS can never run with the fixed test OTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SmsMode {
    /// Real `TurboSMS` delivery.
    Live { token: String, sender: String },
    /// SMS is logged, never sent. `test_mode` = fixed OTP `000000`, no OTP
    /// rate limits, no deadline gates (local dev / E2E only).
    DryRun { test_mode: bool },
}
```
`Config`: remove fields `turbosms_token`, `turbosms_sender`, `sms_dry_run` and the stray CSRF comment; add `pub sms: SmsMode`. `ConfigError`: add
```rust
    #[error("SAMETE_TEST_MODE=true requires SAMETE_SMS_DRY_RUN=true")]
    TestModeRequiresDryRun,
    #[error("SAMETE_TEST_MODE=true requires a loopback bind address, got {0}")]
    TestModeRequiresLoopback(std::net::SocketAddr),
```
`from_env` body after `database_url`:
```rust
        let sms = sms_mode_from_vars(
            std::env::var("SAMETE_SMS_DRY_RUN").ok().as_deref(),
            std::env::var("SAMETE_TEST_MODE").ok().as_deref(),
            std::env::var("TURBOSMS_TOKEN").ok(),
            std::env::var("TURBOSMS_SENDER").ok(),
        )?;
```
and construct `Self { database_url, sms, <U3 field> }`. Rewrite the `from_env` doc to describe both flags and both refusals (keep `# Errors`). Add:
```rust
impl Config {
    /// True only in dry-run SMS mode with `SAMETE_TEST_MODE=true`.
    #[must_use]
    pub fn test_mode(&self) -> bool {
        matches!(self.sms, SmsMode::DryRun { test_mode: true })
    }

    /// Refuse test mode on any non-loopback bind address.
    ///
    /// # Errors
    ///
    /// Returns `Err(ConfigError::TestModeRequiresLoopback)` when test mode is
    /// on and `addr` is not loopback.
    pub fn check_bind_addr(&self, addr: std::net::SocketAddr) -> Result<(), ConfigError> {
        if self.test_mode() && !addr.ip().is_loopback() {
            return Err(ConfigError::TestModeRequiresLoopback(addr));
        }
        Ok(())
    }
}

/// Parse SMS delivery mode from raw env values. Only the literal `"true"` enables a flag.
fn sms_mode_from_vars(
    dry_run: Option<&str>,
    test_mode: Option<&str>,
    token: Option<String>,
    sender: Option<String>,
) -> Result<SmsMode, ConfigError> {
    let test_mode = test_mode == Some("true");
    if dry_run == Some("true") {
        return Ok(SmsMode::DryRun { test_mode });
    }
    if test_mode {
        return Err(ConfigError::TestModeRequiresDryRun);
    }
    let token = token.ok_or(ConfigError::MissingTurbosmsToken)?;
    if token.is_empty() {
        return Err(ConfigError::EmptyTurbosmsToken);
    }
    let sender = sender.ok_or(ConfigError::MissingTurbosmsSender)?;
    if sender.is_empty() {
        return Err(ConfigError::EmptyTurbosmsSender);
    }
    Ok(SmsMode::Live { token, sender })
}
```
(Merge `check_bind_addr`/`test_mode` into the existing `impl Config` block — one impl block.)
Step 4 — rerun step-2 command: **REQUIRED:** `test result: ok.` with all 9 tests.

### U2.2 `src/sms.rs`
`send_sms`: replace `if config.sms_dry_run { … return Ok(()); }` with
```rust
    let (token, sender) = match &config.sms {
        SmsMode::DryRun { .. } => {
            tracing::info!(phone = phone, message = message, "[DRY RUN] SMS would be sent");
            return Ok(());
        }
        SmsMode::Live { token, sender } => (token.as_str(), sender.as_str()),
    };
```
JSON body `"sender": config.turbosms_sender` → `"sender": sender`. `post_to_turbosms(client, config, &body)` (both call sites) → `post_to_turbosms(client, token, &body)`; its signature `config: &Config` → `token: &str`; `.bearer_auth(&config.turbosms_token)` → `.bearer_auth(token)`. Import `use crate::config::{Config, SmsMode};`. Doc line "When `config.sms_dry_run` is true" → "In `SmsMode::DryRun`".

### U2.3 `src/auth.rs`
- `create_otp(pool, phone)` → `create_otp(pool: &PgPool, phone: &str, test_mode: bool)`; first statement `let code = if test_mode { "000000".to_owned() } else { … };`. Doc: "When `test_mode` (see `Config::test_mode`) is true, always returns `"000000"`."
- `check_otp_rate_limit(pool, phone)` → `check_otp_rate_limit(pool: &PgPool, phone: &str, test_mode: bool)`; first statement `if test_mode { return Ok(()); }`. Doc: add "Skipped entirely when `test_mode` is true."
- SQL strings unchanged.

### U2.4 `src/pages/login.rs` (`request_otp` only)
`auth::check_otp_rate_limit(&pool, &normalized)` → `auth::check_otp_rate_limit(&pool, &normalized, config.test_mode())`; `auth::create_otp(&pool, &normalized)` → `auth::create_otp(&pool, &normalized, config.test_mode())`. Comment "(test mode returns "000000")" stays.

### U2.5 `src/pages/home.rs`

**CORRECTION (2026-10-07, U2 spec review):** read the test-mode flag via `use_context::<Config>()` ONLY at the top of each server fn (`enroll_in_season`, `confirm_ready`, `get_home_state`), BEFORE the first `.await`, and pass `test_mode: bool` down (e.g. into `resolve_preparation_state`). A helper that calls `use_context` after an `.await` loses the reactive owner on SSR intermittently: the home state becomes Err("no config in context") and the island never hydrates. Any prescription below that conflicts with this is superseded.
Add (next to the other `#[cfg(feature = "ssr")]` helpers, before `resolve_preparation_state`):
```rust
/// Test mode (deadline gates bypassed) from the server `Config` context.
#[cfg(feature = "ssr")]
fn test_mode() -> Result<bool, ServerFnError> {
    leptos::context::use_context::<crate::config::Config>()
        .map(|config| config.test_mode())
        .ok_or_else(|| ServerFnError::new("no config in context"))
}
```
Replace each of the three lines `let test_mode = std::env::var("SAMETE_TEST_MODE").as_deref() == Ok("true");` (in `resolve_preparation_state`, `enroll_in_season`, `confirm_ready`) with `let test_mode = test_mode()?;`. `is_past_deadline` and its tests unchanged.

### U2.6 `src/main.rs`
Move these lines from step 4 to directly after step 2 (before the database step):
```rust
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    config.check_bind_addr(addr).expect("configuration error");
    if config.test_mode() {
        tracing::warn!(
            "SAMETE_TEST_MODE=true: fixed OTP 000000, OTP rate limits and deadline gates disabled (local/E2E only)"
        );
    }
```
Step 4 keeps `let leptos_options = conf.leptos_options;` and `let routes = generate_route_list(App);`.

### U2.7 Docs
`README.md` Environment Variables table — final rows (replace the whole table body; keep U3's two rows, which already exist):
```
| `DATABASE_URL` | Yes | Postgres connection string |
| `TURBOSMS_TOKEN` | Yes, unless `SAMETE_SMS_DRY_RUN=true` | TurboSMS API bearer token |
| `TURBOSMS_SENDER` | Yes, unless `SAMETE_SMS_DRY_RUN=true` | Registered alpha-name |
| `SAMETE_SMS_DRY_RUN` | No (dev/E2E only) | `true` = SMS are logged (including OTP codes), never sent. Never set in production |
| `SAMETE_TEST_MODE` | No (dev/E2E only) | `true` = fixed OTP `000000`, no OTP rate limits, no deadline gates. Boot is refused unless `SAMETE_SMS_DRY_RUN=true` AND the server binds a loopback address. Never set in production |
| `SAMETE_ADMIN_PHONE` / `SAMETE_ADMIN_NAME` | (U3 rows, unchanged) | |
| `RUST_LOG` | No | Log filter; default `samete=info,tower_http=info` |
```
Delete the `CSRF_SECRET` row (the variable does not exist; `grep -rn CSRF_SECRET src` = 0).
`.env.example` — final content (keep U3's admin lines below):
```sh
export DATABASE_URL="postgres://samete:samete@localhost/samete"
# Local dev / E2E only. SAMETE_TEST_MODE=true (fixed OTP 000000, no rate limits,
# no deadline gates) is refused at boot unless SAMETE_SMS_DRY_RUN=true and the
# server binds a loopback address. Production: set TURBOSMS_TOKEN and
# TURBOSMS_SENDER and leave both SAMETE_* flags unset.
export SAMETE_TEST_MODE=true
export SAMETE_SMS_DRY_RUN=true
```

## Verification Gates (U2)
```bash
grep -rn "SAMETE_TEST_MODE\|SAMETE_SMS_DRY_RUN" src/
```
**REQUIRED:** matches only in `src/config.rs` and `src/main.rs`.
```bash
grep -rn "std::env::var" src/ | grep -v "src/config.rs" | grep -v "SAMETE_LOG_POOL"
```
**REQUIRED:** zero matches.
```bash
grep -rn "sms_dry_run\|turbosms_token\|turbosms_sender\|CSRF_SECRET" src/ README.md
```
**REQUIRED:** zero matches.
Boot-refusal gates (no DB needed — refusal precedes DB connect):
```bash
SQLX_OFFLINE=true cargo build --features ssr
env -u TURBOSMS_TOKEN -u SAMETE_SMS_DRY_RUN DATABASE_URL=postgres://x SAMETE_TEST_MODE=true LEPTOS_SITE_ADDR=127.0.0.1:3921 ./target/debug/samete 2>&1 | grep -c "TestModeRequiresDryRun"
DATABASE_URL=postgres://x SAMETE_TEST_MODE=true SAMETE_SMS_DRY_RUN=true LEPTOS_SITE_ADDR=0.0.0.0:3922 ./target/debug/samete 2>&1 | grep -c "TestModeRequiresLoopback"
```
**REQUIRED:** each prints `1`; both processes exit non-zero immediately.
Standard gates. E2E gate (passed count equal to the lane base's count; proves `isolated-capture.sh` still boots test mode on 127.0.0.1).

---

# U3 — First-admin bootstrap from env

## Why This Matters
A fresh production DB has no admin; the only ways in are hand-written SQL or running the test seed (which installs a publicly known phone as admin).

## What You Must Do

### U3.1 `src/config.rs` — TDD
Step 1 — append a `#[cfg(test)] mod tests` to `src/config.rs` (none exists at HEAD):
```rust
#[cfg(test)]
mod tests {
    use super::{AdminBootstrap, ConfigError, admin_bootstrap_from_vars};

    #[test]
    fn neither_var_means_no_bootstrap() {
        assert_eq!(admin_bootstrap_from_vars(None, None).ok(), Some(None));
    }

    #[test]
    fn blank_vars_count_as_unset() {
        assert_eq!(admin_bootstrap_from_vars(Some("  ".into()), Some(String::new())).ok(), Some(None));
    }

    #[test]
    fn both_vars_normalize_phone_and_trim_name() {
        assert_eq!(
            admin_bootstrap_from_vars(Some("067 123 45 67".into()), Some("  Організатор ".into())).ok(),
            Some(Some(AdminBootstrap { phone: "+380671234567".into(), name: "Організатор".into() }))
        );
    }

    #[test]
    fn phone_without_name_is_refused() {
        assert!(matches!(admin_bootstrap_from_vars(Some("+380671234567".into()), None), Err(ConfigError::AdminBootstrapIncomplete)));
    }

    #[test]
    fn name_without_phone_is_refused() {
        assert!(matches!(admin_bootstrap_from_vars(None, Some("Організатор".into())), Err(ConfigError::AdminBootstrapIncomplete)));
    }

    #[test]
    fn invalid_phone_is_refused() {
        assert!(matches!(admin_bootstrap_from_vars(Some("12345".into()), Some("A".into())), Err(ConfigError::InvalidAdminPhone(_))));
    }
}
```
Step 2 — `SQLX_OFFLINE=true cargo test --features ssr config::tests` → **REQUIRED:** compile errors (red).
Step 3 — implement:
```rust
/// First admin ensured at every boot (idempotent).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminBootstrap {
    /// E.164-normalized phone.
    pub phone: String,
    pub name: String,
}
```
`Config`: add `pub admin_bootstrap: Option<AdminBootstrap>,`. `ConfigError`: add
```rust
    #[error("SAMETE_ADMIN_PHONE and SAMETE_ADMIN_NAME must be set together")]
    AdminBootstrapIncomplete,
    #[error("SAMETE_ADMIN_PHONE is not a valid Ukrainian phone: {0}")]
    InvalidAdminPhone(String),
```
In `from_env`, before `Ok(Self { … })`:
```rust
        let admin_bootstrap = admin_bootstrap_from_vars(
            std::env::var("SAMETE_ADMIN_PHONE").ok(),
            std::env::var("SAMETE_ADMIN_NAME").ok(),
        )?;
```
and include it in `Self { … }`. Add to `from_env` doc `# Errors`: "or if exactly one of `SAMETE_ADMIN_PHONE`/`SAMETE_ADMIN_NAME` is set, or the phone is invalid."
```rust
/// Parse the optional first-admin bootstrap. Blank values count as unset.
fn admin_bootstrap_from_vars(
    phone: Option<String>,
    name: Option<String>,
) -> Result<Option<AdminBootstrap>, ConfigError> {
    let present = |v: Option<String>| v.map(|s| s.trim().to_owned()).filter(|s| !s.is_empty());
    match (present(phone), present(name)) {
        (None, None) => Ok(None),
        (Some(phone), Some(name)) => {
            let phone = crate::phone::normalize(&phone)
                .map_err(|_| ConfigError::InvalidAdminPhone(phone.clone()))?;
            Ok(Some(AdminBootstrap { phone, name }))
        }
        _ => Err(ConfigError::AdminBootstrapIncomplete),
    }
}
```
Step 4 — rerun: **REQUIRED:** `test result: ok.` with the 6 tests.

### U3.2 `src/db.rs`
```rust
/// Ensure the configured first admin exists: insert it, or promote the
/// existing user with that phone to admin. Idempotent; runs at every boot.
///
/// # Errors
///
/// Returns `Err` on database failure.
pub async fn ensure_admin(
    pool: &PgPool,
    admin: &crate::config::AdminBootstrap,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO users (phone, name, role, onboarded)
        VALUES ($1, $2, 'admin', true)
        ON CONFLICT (phone) DO UPDATE SET role = 'admin'
        "#,
        admin.phone,
        admin.name,
    )
    .execute(pool)
    .await?;
    Ok(())
}
```
(`onboarded = true` mirrors `seed/test_admin.sql`; existing users keep their name/onboarded/status.)

### U3.3 `src/main.rs`
Directly after the `run_migrations(...).expect("migrations failed");` statement:
```rust
    if let Some(admin) = &config.admin_bootstrap {
        samete::db::ensure_admin(&pool, admin)
            .await
            .expect("admin bootstrap failed");
        tracing::info!(phone = %admin.phone, "admin bootstrap ensured");
    }
```

### U3.4 Docs
`README.md` Environment Variables table: add two rows after `TURBOSMS_SENDER`:
```
| `SAMETE_ADMIN_PHONE` | No (set both or neither) | Phone of the first organizer. At every boot the app ensures this user exists with role `admin` (creates it, or promotes an existing account). Any Ukrainian format; stored as E.164 |
| `SAMETE_ADMIN_NAME` | No (set both or neither) | Display name used only when the admin user is created |
```
Add subsection at the end of `## Deployment`:
```markdown
### First admin

A fresh database has no organizer, and invite codes can only be created by one. Set `SAMETE_ADMIN_PHONE` and `SAMETE_ADMIN_NAME` in the Coolify environment and deploy: on boot the app creates that admin (or promotes the existing account with that phone). The operation is idempotent — leaving the variables set is safe. Then sign in at `/login` with that phone (real SMS OTP) and generate invite codes from `/admin`. Never run `seed/test_admin.sql` against production: it installs a fixed, publicly known test phone as admin.
```
`.env.example`: append
```sh
# First organizer (optional; both or neither). Ensured at every boot.
# export SAMETE_ADMIN_PHONE="+380XXXXXXXXX"
# export SAMETE_ADMIN_NAME="Організатор"
```

## Verification Gates (U3)
sqlx regeneration (new query in `db.rs`), standard gates.
Behavioral gate:
```bash
export DATABASE_URL=postgres://samete:samete@localhost:5432/samete_u3
sqlx database drop -y; sqlx database create
SQLX_OFFLINE=true cargo build --features ssr
for run in 1 2; do
  SAMETE_SMS_DRY_RUN=true SAMETE_ADMIN_PHONE="067 123 45 67" SAMETE_ADMIN_NAME="Організатор" LEPTOS_SITE_ADDR=127.0.0.1:3931 ./target/debug/samete > /tmp/u3-boot-$run.log 2>&1 &
  SRV=$!; until grep -q "listening on" /tmp/u3-boot-$run.log || ! kill -0 $SRV 2>/dev/null; do sleep 1; done
  grep -c "admin bootstrap ensured" /tmp/u3-boot-$run.log; kill $SRV; wait $SRV 2>/dev/null
done
psql "$DATABASE_URL" -tAc "SELECT phone, role, count(*) OVER () FROM users"
SAMETE_SMS_DRY_RUN=true SAMETE_ADMIN_PHONE="+380671234567" LEPTOS_SITE_ADDR=127.0.0.1:3932 ./target/debug/samete 2>&1 | grep -c AdminBootstrapIncomplete
```
**REQUIRED OUTPUT:** `1`, `1` (one per boot); psql prints exactly `+380671234567|admin|1`; last command prints `1`.
E2E gate: passed count equal to baseline (bootstrap vars unset in harness → no behavior change).

---

# U4 — Swap as position exchange, transactional, re-validated at release

## Why This Matters
Story 3.3 swap never succeeds (UNIQUE violation on the first UPDATE), and if it ever wrote, it would persist a split graph with no rollback that `advance_season` would release to participants. The E2E test is vacuous.

## What You Must Do

### U4.1 `src/assignment.rs` — TDD (pure, ungated, runs under bare `cargo test`)
Step 1 — add to `mod tests` (failing):
```rust
    // ── swap_positions / cycles_from_edges ───────────────────────────────────

    fn ring_edges(ids: &[Uuid]) -> Vec<(Uuid, Uuid)> {
        (0..ids.len()).map(|i| (ids[i], ids[(i + 1) % ids.len()])).collect()
    }

    #[test]
    fn swap_positions_preserves_single_loop_for_every_pair() {
        for n in 3..=11 {
            let ids = make_uuids(n);
            for i in 0..n {
                for j in (i + 1)..n {
                    let swapped = swap_positions(&ring_edges(&ids), ids[i], ids[j]).expect("valid swap");
                    let result = validated_cycles_from_edges(&swapped).expect("single loop");
                    assert_eq!(result.cohorts.len(), 1, "n={n} i={i} j={j}");
                    assert_eq!(result.cohorts[0].participants.len(), n);
                }
            }
        }
    }

    #[test]
    fn swap_positions_exchanges_the_two_positions() {
        let ids = make_uuids(4); // a→b→c→d→a
        let (a, b, c, d) = (ids[0], ids[1], ids[2], ids[3]);
        let mut swapped = swap_positions(&ring_edges(&ids), a, c).expect("valid swap");
        swapped.sort();
        let mut expected = vec![(c, b), (b, a), (a, d), (d, c)]; // c→b→a→d→c
        expected.sort();
        assert_eq!(swapped, expected);
    }

    #[test]
    fn swap_positions_adjacent_pair_never_self_assigns() {
        let ids = make_uuids(3);
        let swapped = swap_positions(&ring_edges(&ids), ids[0], ids[1]).expect("valid swap");
        assert!(swapped.iter().all(|(s, r)| s != r));
        assert!(validated_cycles_from_edges(&swapped).is_ok());
    }

    #[test]
    fn swap_positions_across_cohorts_keeps_both_loops() {
        let left = make_uuids(3);
        let right = make_uuids(4);
        let mut edges = ring_edges(&left);
        edges.extend(ring_edges(&right));
        let swapped = swap_positions(&edges, left[0], right[2]).expect("valid swap");
        let result = validated_cycles_from_edges(&swapped).expect("two loops");
        let mut sizes: Vec<usize> = result.cohorts.iter().map(|c| c.participants.len()).collect();
        sizes.sort_unstable();
        assert_eq!(sizes, vec![3, 4]);
    }

    #[test]
    fn swap_positions_rejects_same_participant() {
        let ids = make_uuids(3);
        assert_eq!(swap_positions(&ring_edges(&ids), ids[0], ids[0]), Err(SwapError::SameParticipant));
    }

    #[test]
    fn swap_positions_rejects_unassigned_participant() {
        let ids = make_uuids(3);
        let stranger = Uuid::new_v4();
        assert_eq!(swap_positions(&ring_edges(&ids), ids[0], stranger), Err(SwapError::NotAssigned(stranger)));
    }

    #[test]
    fn cycles_from_edges_recovers_each_cohort() {
        let mut edges = ring_edges(&make_uuids(3));
        edges.extend(ring_edges(&make_uuids(5)));
        let result = cycles_from_edges(&edges).expect("decomposes");
        assert_eq!(result.cohorts.len(), 2);
    }

    #[test]
    fn cycles_from_edges_rejects_dangling_recipient() {
        let ids = make_uuids(4);
        let edges = vec![(ids[0], ids[1]), (ids[1], ids[2]), (ids[2], ids[3])];
        assert!(cycles_from_edges(&edges).is_err());
    }

    #[test]
    fn cycles_from_edges_rejects_duplicate_sender() {
        let ids = make_uuids(3);
        let mut edges = ring_edges(&ids);
        edges.push((ids[0], ids[2]));
        assert!(cycles_from_edges(&edges).is_err());
    }

    #[test]
    fn recipient_exchange_splits_the_loop() {
        // Regression: the old swap semantics (exchange two senders' recipients).
        let ids = make_uuids(5);
        let mut edges = ring_edges(&ids);
        edges[0].1 = ids[3];
        edges[2].1 = ids[1];
        assert!(validated_cycles_from_edges(&edges).is_err());
    }

    #[test]
    fn validated_cycles_from_edges_rejects_empty() {
        assert!(validated_cycles_from_edges(&[]).is_err());
    }
```
Step 2 — `cargo test assignment::tests 2>&1 | grep -E "error\[|test result"` → **REQUIRED:** compile errors (red).
Step 3 — implement (after `validate_cycles`, before `mod tests`; no `cfg` gate):
```rust
/// Why a position swap was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum SwapError {
    /// Both slots name the same participant.
    SameParticipant,
    /// The participant has no assignment in this set.
    NotAssigned(Uuid),
}

/// Exchange the positions of `a` and `b` in the assignment graph.
///
/// Every edge `s → r` becomes `σ(s) → σ(r)` where σ swaps `a` and `b`.
/// Relabelling preserves the cycle structure exactly: every loop stays one
/// loop of the same length, within or across cohorts.
///
/// # Errors
///
/// `SwapError::SameParticipant` if `a == b`; `SwapError::NotAssigned` if
/// either is not a sender in `edges`.
pub fn swap_positions(
    edges: &[(Uuid, Uuid)],
    a: Uuid,
    b: Uuid,
) -> Result<Vec<(Uuid, Uuid)>, SwapError> {
    if a == b {
        return Err(SwapError::SameParticipant);
    }
    for participant in [a, b] {
        if !edges.iter().any(|&(sender, _)| sender == participant) {
            return Err(SwapError::NotAssigned(participant));
        }
    }
    let sigma = |u: Uuid| {
        if u == a {
            b
        } else if u == b {
            a
        } else {
            u
        }
    };
    Ok(edges.iter().map(|&(s, r)| (sigma(s), sigma(r))).collect())
}

/// Recover the ordered cycles from sender → recipient edges.
///
/// Does not check cohort size or receiver uniqueness — see `validated_cycles_from_edges`.
///
/// # Errors
///
/// Returns `Err` if a sender appears twice, a recipient has no outgoing
/// edge, or a walk re-enters a participant before closing its loop.
pub fn cycles_from_edges(edges: &[(Uuid, Uuid)]) -> Result<AssignmentResult, String> {
    let mut next: HashMap<Uuid, Uuid> = HashMap::with_capacity(edges.len());
    for &(sender, recipient) in edges {
        if next.insert(sender, recipient).is_some() {
            return Err(format!("participant {sender} has more than one assignment"));
        }
    }
    let mut visited = std::collections::HashSet::with_capacity(edges.len());
    let mut cohorts = Vec::new();
    for &(start, _) in edges {
        if visited.contains(&start) {
            continue;
        }
        let mut participants = Vec::new();
        let mut current = start;
        loop {
            if !visited.insert(current) {
                return Err(format!("participant {current} is reached twice"));
            }
            participants.push(current);
            let Some(&recipient) = next.get(&current) else {
                return Err(format!("participant {current} has no assignment"));
            };
            if recipient == start {
                break;
            }
            current = recipient;
        }
        cohorts.push(Cycle { participants, score: 0 });
    }
    Ok(AssignmentResult { cohorts })
}

/// Recover cycles from edges and validate them as a releasable assignment set.
///
/// # Errors
///
/// Returns `Err` if `edges` is empty, does not decompose into loops, or any
/// loop fails `validate_cycles`.
pub fn validated_cycles_from_edges(edges: &[(Uuid, Uuid)]) -> Result<AssignmentResult, String> {
    if edges.is_empty() {
        return Err("no assignments".to_owned());
    }
    let result = cycles_from_edges(edges)?;
    validate_cycles(&result)?;
    Ok(result)
}
```
Step 4 — rerun step-2: **REQUIRED:** `test result: ok.` including the 11 new names.

### U4.2 `src/admin/assignments.rs`
1. `AssignmentLink`: add field `pub recipient_id: String,` (between `sender_name` and `recipient_name`). In `store_and_build_preview` add `recipient_id: recipient_id.to_string(),` to the `AssignmentLink { … }` literal.
2. Delete `validate_swap_topology` entirely.
3. Replace `swap_assignment` body (signature and `#[server(SwapAssignment)]` unchanged) with:
```rust
    use crate::{
        assignment::{self, SwapError},
        auth,
        i18n::i18n::{Locale, td_string},
        types::Phase,
    };

    let (pool, _user) = auth::require_admin().await?;

    let sid: uuid::Uuid = season_id
        .parse()
        .map_err(|_| ServerFnError::new("invalid season_id"))?;
    let sa: uuid::Uuid = sender_a
        .parse()
        .map_err(|_| ServerFnError::new("invalid sender_a"))?;
    let sb: uuid::Uuid = sender_b
        .parse()
        .map_err(|_| ServerFnError::new("invalid sender_b"))?;

    let mut tx = pool.begin().await.map_err(db_err)?;

    // Lock the season row: serializes concurrent swaps and advance_season.
    let phase = sqlx::query_scalar!(
        r#"
        SELECT phase AS "phase: Phase"
        FROM seasons
        WHERE id = $1 AND launched_at IS NOT NULL
        FOR UPDATE
        "#,
        sid,
    )
    .fetch_optional(&mut *tx)
    .await
    .map_err(db_err)?;
    if phase != Some(Phase::Assignment) {
        return Err(ServerFnError::new(td_string!(Locale::uk, assignments_error_wrong_phase)));
    }

    let edges: Vec<(uuid::Uuid, uuid::Uuid)> = sqlx::query!(
        r#"SELECT sender_id, recipient_id FROM assignments WHERE season_id = $1"#,
        sid,
    )
    .fetch_all(&mut *tx)
    .await
    .map_err(db_err)?
    .into_iter()
    .map(|row| (row.sender_id, row.recipient_id))
    .collect();

    let swapped = assignment::swap_positions(&edges, sa, sb).map_err(|e| {
        ServerFnError::new(match e {
            SwapError::SameParticipant => td_string!(Locale::uk, assignments_error_swap_same_participant),
            SwapError::NotAssigned(id) if id == sa => td_string!(Locale::uk, assignments_error_sender_a_not_found),
            SwapError::NotAssigned(_) => td_string!(Locale::uk, assignments_error_sender_b_not_found),
        })
    })?;

    // Validate BEFORE any write. Holds by construction; checked anyway.
    assignment::validated_cycles_from_edges(&swapped).map_err(|_| {
        ServerFnError::new(td_string!(Locale::uk, assignments_error_swap_breaks_cycle))
    })?;

    // Rewrite the season's rows in one statement pair: UNIQUE(season_id,
    // sender_id) and UNIQUE(season_id, recipient_id) are non-deferrable, so
    // per-row UPDATEs would collide mid-swap. Safe to recreate rows: in the
    // Assignment phase no SMS/receipt state exists yet (notified_at,
    // receipt_* stay at defaults).
    sqlx::query!("DELETE FROM assignments WHERE season_id = $1", sid)
        .execute(&mut *tx)
        .await
        .map_err(db_err)?;
    let (senders, recipients): (Vec<uuid::Uuid>, Vec<uuid::Uuid>) = swapped.into_iter().unzip();
    sqlx::query!(
        r#"
        INSERT INTO assignments (season_id, sender_id, recipient_id)
        SELECT $1, s, r FROM UNNEST($2::uuid[], $3::uuid[]) AS t(s, r)
        "#,
        sid,
        &senders[..],
        &recipients[..],
    )
    .execute(&mut *tx)
    .await
    .map_err(db_err)?;

    tx.commit().await.map_err(db_err)?;
    Ok(())
```
(If `td_string!` arms do not unify in the `match`, append `.to_string()` to each arm.) Doc comment of `swap_assignment` becomes:
```
/// Exchange two participants' positions in the assignment cycle (admin only).
///
/// Every loop keeps its members and length (see `assignment::swap_positions`).
/// Runs in one transaction: lock season, read, compute, validate, rewrite, commit.
/// Any error rolls back; nothing partial is ever persisted.
///
/// # Errors
///
/// Returns `Err` if caller is not admin, the season is not launched and in the
/// Assignment phase, both senders are the same, either has no assignment, or
/// the result fails validation.
```
4. `get_assignment_preview`: keep the query (append `, a.sender_id` to its `ORDER BY a.created_at` → `ORDER BY a.created_at, a.sender_id`); replace everything from `// Build chain from next_map` to the final `Ok(Some(AssignmentPreview { … }))` with:
```rust
    let by_sender: std::collections::HashMap<uuid::Uuid, &AssignmentRow> =
        assignments.iter().map(|a| (a.sender_id, a)).collect();
    let edges: Vec<(uuid::Uuid, uuid::Uuid)> = assignments
        .iter()
        .map(|a| (a.sender_id, a.recipient_id))
        .collect();
    let result = crate::assignment::cycles_from_edges(&edges).map_err(|_| {
        ServerFnError::new(td_string!(Locale::uk, season_error_assignments_invalid))
    })?;

    let cohorts = result
        .cohorts
        .iter()
        .map(|cycle| CohortPreview {
            score: cycle.score,
            chain: cycle
                .participants
                .iter()
                .filter_map(|id| by_sender.get(id))
                .map(|a| AssignmentLink {
                    sender_id: a.sender_id.to_string(),
                    sender_name: a.sender_name.clone(),
                    recipient_id: a.recipient_id.to_string(),
                    recipient_name: a.recipient_name.clone(),
                })
                .collect(),
        })
        .collect();

    Ok(Some(AssignmentPreview {
        season_id: season.id.to_string(),
        cohorts,
        phase: season.phase,
    }))
```
and change the fn's `use crate::auth;` to `use crate::{auth, i18n::i18n::{Locale, td_string}};`.

### U4.3 `src/admin/season.rs` — release guard in `advance_season`
Insert after `let next_phase = …?;` and before the `UPDATE seasons`:
```rust
    // Release guard: leaving Assignment publishes the graph to participants.
    if season.phase == Phase::Assignment {
        let edges: Vec<(uuid::Uuid, uuid::Uuid)> = sqlx::query!(
            r#"SELECT sender_id, recipient_id FROM assignments WHERE season_id = $1"#,
            season.id,
        )
        .fetch_all(&pool)
        .await
        .map_err(db_err)?
        .into_iter()
        .map(|row| (row.sender_id, row.recipient_id))
        .collect();
        crate::assignment::validated_cycles_from_edges(&edges).map_err(|_| {
            ServerFnError::new(td_string!(Locale::uk, season_error_assignments_invalid))
        })?;
    }
```
(Same SQL string as in `swap_assignment` → one `.sqlx` entry.) Add to the doc `# Errors`: "or, when leaving the Assignment phase, the stored assignments are missing or not valid loops."

### U4.4 `src/admin/page.rs` — accessible link list carrying the graph (testid contract)
In `render_cycle_ring`, inside `<figure class="cycle-viz-container" data-testid="cycle-visualization">`, directly after `</svg>` and before `</figure>`, insert:
```rust
            <ol class="sr-only" data-testid="cycle-link-list">
                {chain
                    .iter()
                    .map(|link| view! {
                        <li
                            data-testid="cycle-link"
                            data-sender-id=link.sender_id.clone()
                            data-sender-name=link.sender_name.clone()
                            data-recipient-id=link.recipient_id.clone()
                        >
                            {link.sender_name.clone()} " → " {link.recipient_name.clone()}
                        </li>
                    })
                    .collect_view()}
            </ol>
```
No `#[cfg]` tokens and no bare `>` comparisons in view! attributes. `chain` is the existing `&[AssignmentLink]` parameter. No other page.rs change (swap form keeps `season_id` hidden input and both selects).

### U4.5 `locales/uk.json`
- Add `"assignments_error_swap_same_participant": "Обери двох різних учасників.",` after `assignments_error_swap_breaks_cycle`.
- Add `"season_error_assignments_invalid": "Призначення некоректні — згенеруй їх знову.",` after `season_error_no_active_season`.
- Change value of `assignments_swap_description` to `"Обери двох учасників, щоб поміняти їх місцями в циклі."`.
- Delete key `assignments_error_broken_cycle` (only user was `validate_swap_topology`; verify `grep -rn assignments_error_broken_cycle src` = 0 before deleting).

### U4.6 E2E
POM (`end2end/tests/fixtures/mail_club_page.ts`), assignments section:
- `swapAssignment`: replace the final `await expect(this.page.getByTestId("cycle-visualization")).toBeVisible();` and its preceding comment with:
```ts
    // The caller asserts the graph change via readCycleEdges() + expect.poll.
    await expect(this.page.getByTestId("action-error")).toBeEmpty();
```
- Add:
```ts
  /**
   * Read the assignment graph from the admin cycle visualization's link list.
   * Returns "senderId>recipientId" strings, sorted (order-independent).
   */
  async readCycleEdges(): Promise<string[]> {
    const links = this.page.getByTestId("cycle-link");
    await expect(links.first()).toBeAttached();
    const count = await links.count();
    const edges: string[] = [];
    for (let i = 0; i < count; i++) {
      const link = links.nth(i);
      edges.push(`${await link.getAttribute("data-sender-id")}>${await link.getAttribute("data-recipient-id")}`);
    }
    return edges.sort();
  }

  /** Sender UUID for a participant name, from the cycle link list. */
  async cycleSenderId(name: string): Promise<string> {
    const link = this.page.getByTestId("cycle-link").and(this.page.locator(`[data-sender-name="${name}"]`));
    await expect(link).toBeAttached();
    return (await link.getAttribute("data-sender-id")) as string;
  }
```
Spec (`end2end/tests/mail_club.spec.ts`), replace the body of `"3.3 — admin swaps two assignments"` with:
```ts
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      const before = await app.readCycleEdges();
      expect(before).toHaveLength(3);
      const idA = await app.cycleSenderId(NAMES.A);
      const idB = await app.cycleSenderId(NAMES.B);
      const sigma = (u: string) => (u === idA ? idB : u === idB ? idA : u);
      const expected = before
        .map((edge) => edge.split(">"))
        .map(([s, r]) => `${sigma(s)}>${sigma(r)}`)
        .sort();
      expect(expected).not.toEqual(before);

      await app.swapAssignment(NAMES.A, NAMES.B);

      await expect.poll(() => app.readCycleEdges()).toEqual(expected);
      // Still one loop through all three: no self-assignment.
      for (const edge of expected) {
        const [s, r] = edge.split(">");
        expect(s).not.toEqual(r);
      }
```
(For 3 participants position-exchange of A and B is the reverse orientation — `expected !== before` always holds.)
`end2end/tests/visual-audit.spec.ts`: replace the `// ── Note: admin swap form error (A40) is unreachable via UI ──` comment block (6 lines) with:
```ts
  // ── Note: admin swap form error (A40) not captured ──────────────────────────
  // Choosing the same participant in both slots is now rejected server-side
  // (assignments_error_swap_same_participant → action-error). The state is
  // reachable but not yet part of the capture set.
```

## Verification Gates (U4)
```bash
grep -rn "validate_swap_topology\|assignments_error_broken_cycle" src/ locales/
```
**REQUIRED:** zero matches.
```bash
awk '/pub async fn swap_assignment/,/^}/' src/admin/assignments.rs | grep -c "&pool)"
```
**REQUIRED:** `0` (every statement in swap runs on `&mut *tx`).
```bash
awk '/pub async fn swap_assignment/,/^}/' src/admin/assignments.rs | grep -n "validated_cycles_from_edges\|DELETE FROM assignments"
```
**REQUIRED:** the `validated_cycles_from_edges` line number is lower than the `DELETE FROM assignments` line number.
```bash
grep -n "validated_cycles_from_edges" src/admin/season.rs
```
**REQUIRED:** exactly one match, inside `advance_season`.
```bash
cargo test assignment::tests 2>&1 | grep -E "^test result"
```
**REQUIRED:** `test result: ok.` and `0 failed`; `cargo test assignment::tests -- --list` lists all 11 names from U4.1.
sqlx regeneration, standard gates, E2E gate (titles `3.3 — admin swaps two assignments`, `phase — advance assignment → delivery`, `2.3 — participant sees recipient details` pass; passed count equals baseline).

---

# A1 — `aria-busy` instead of a leaked `attr:aria-busy`

## Why This Matters
`attr:`-prefixed attributes on NATIVE elements emit a literal attribute named `attr:aria-busy` (Leptos 0.8; empirically confirmed in `readiness/f-e2e.md` § "attr: leak check": `"attr:aria-busy=true"` on `create-season-button` in pending state). Screen readers never get `aria-busy` during any admin loading state.

## What You Must Do
Exhaustive site list (verified `grep -rn "attr:" src --include=*.rs` = these 13 lines, all `<button>` native elements in `src/admin/page.rs`; no other `attr:` exists app-wide). Line numbers are at HEAD `1a3c8a5`; after U4 they shift, so locate each by its signal name:

| # | testid (button) | line @1a3c8a5 | signal |
|---|---|---|---|
| 1 | `create-season-button` | 497 | `pending` |
| 2 | `launch-button` | 675 | `launch_pending` |
| 3 | `advance-button` | 705 | `advance_pending` |
| 4 | cancel confirm | 753 | `cancel_pending` |
| 5 | season-open SMS | 865 | `season_open_pending` |
| 6 | confirm-nudge SMS | 906 | `confirm_nudge_pending` |
| 7 | assignment SMS | 955 | `assignment_pending` |
| 8 | receipt-nudge SMS | 992 | `receipt_nudge_pending` |
| 9 | `generate-button` | 1102 | `generate_pending` |
| 10 | `swap-button` | 1647 | `swap_pending` |
| 11 | `generate-code-button` | 1748 | `generate_pending` |
| 12 | `invite-code-revoke-button` | 2016 | `revoke_pending` |
| 13 | deactivate | 2171 | `deactivate_pending` |

At each site replace the prefix only:
```rust
attr:aria-busy=move || X_pending.get().then_some("true")
// becomes
aria-busy=move || X_pending.get().then_some("true")
```
Exactly one token changes per line (`attr:aria-busy` → `aria-busy`); the closure is untouched. Before editing, re-run `grep -rn "attr:" src --include=*.rs`. If it now lists a site not in the table (added by U1–U4), fix it the same way and add it to the report.

### A1 E2E — the pending state is held deterministically
POM (`end2end/tests/fixtures/mail_club_page.ts`), admin invite-codes section:
```ts
  /**
   * Story 1.5 a11y: while generate_invite_code is in flight the button carries a
   * bare aria-busy="true" (never a literal "attr:aria-busy"). The server response
   * is held via page.route until the assertion is done, then released.
   */
  async expectGenerateCodeBusyWhilePending() {
    await this.page.goto("/admin");
    const button = this.page.getByTestId("generate-code-button");
    await expect(button).toBeEnabled();
    let release: () => void = () => {};
    const held = new Promise<void>((resolve) => (release = resolve));
    await this.page.route("**/*generate_invite_code*", async (route) => {
      await held;
      await route.continue();
    });
    await button.click();
    await expect(button).toHaveAttribute("aria-busy", "true");
    expect(await button.getAttribute("attr:aria-busy")).toBeNull();
    release();
    await expect(this.page.getByTestId("generated-code-display")).toBeVisible();
    await expect(button).not.toHaveAttribute("aria-busy", "true");
    await this.page.unroute("**/*generate_invite_code*");
  }
```
(The `getAttribute` call runs while the response is held, so the DOM cannot change under it. Playwright has no web-first assertion for an attribute's absence by name. This is the one sanctioned non-retrying read.)
Spec (`end2end/tests/mail_club.spec.ts`), in Epic 1 directly after `"1.6 — admin revokes an unused code"`:
```ts
    test("1.5 — generate button exposes aria-busy while pending", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.expectGenerateCodeBusyWhilePending();
    });
```
That test generates one extra unused code. No later test counts unused codes; it is left as is.

## Verification Gates (A1)
```bash
grep -rn "attr:" src --include=*.rs | wc -l
grep -rn "aria-busy=move" src/admin/page.rs | wc -l
```
**REQUIRED:** `0`, then `13` (or 13 + any extra sites found and reported).
Compiled-artifact gate (the attribute name is a string literal in both the server binary and the WASM; deterministic, no timing):
```bash
cargo leptos build --release > /tmp/a1-build.log 2>&1; echo "build=$?"
grep -c -a "attr:aria-busy" target/release/samete target/site/pkg/samete.wasm
grep -c -a "aria-busy" target/release/samete target/site/pkg/samete.wasm
```
**REQUIRED:** `build=0`; first grep `0` for both files; second grep ≥ `1` for both files. Run the first grep on the lane base BEFORE the edit too, and paste it: it is expected ≥ `1` in at least one file (RED proof).
Standard gates; isolated E2E gate (baseline + U1's 4 + this 1). The new title `1.5 — generate button exposes aria-busy while pending` must pass in all 3 runs.

Commit: `fix(a11y): emit bare aria-busy on admin submit buttons`. Write-set: `src/admin/page.rs`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/mail_club.spec.ts`.

---

## Testing Decisions
- Behavior tested through public pure interfaces; DB-bound server fns are covered by E2E (project rule: DB = E2E).
- `auth`: `generate_token`, `extract_cookie` (unit). Ticket lifecycle (create/resolve/consume/attempts) — E2E (forged cookie, throttle, existing registration tests).
- `config`: `sms_mode_from_vars`, `Config::check_bind_addr`, `admin_bootstrap_from_vars` (unit, `--features ssr`; never mutate process env in tests — `unsafe_code = "forbid"` and edition 2024 make `set_var` unsafe).
- `assignment`: `swap_positions`, `cycles_from_edges`, `validated_cycles_from_edges` (unit, bare `cargo test`), exhaustive over n=3..=11 and all pairs.
- Boot behavior (U2 refusals, U3 bootstrap) — executable binary gates above.
- Runner: `cargo test` / `cargo test --features ssr`; names describe behavior in snake_case (prior art: `src/auth.rs` tests, `src/assignment.rs` tests). E2E titles trace to story numbers.

## Forbidden Patterns

### BANNED: reading identity from the client
```rust
// BANNED — any phone, user id, or role taken from a cookie/form/query value
let phone = extract_cookie(&parts, "registration_ticket"); // then used as a phone
```
The cookie value is only ever hashed and looked up.

### BANNED: signed-cookie / HMAC / secret-key schemes for U1
No new secret, no `CSRF_SECRET`, no `blake2`/`hmac` usage. Server state is the authority.

### BANNED: `let _ =` on writes in auth/registration/swap paths
```rust
let _ = tx.commit().await;          // BANNED
let _ = sqlx::query!(...).execute(&mut *tx).await; // BANNED
```

### BANNED: per-row UPDATE of `recipient_id` for swaps, or writes before validation
Also BANNED: making `UNIQUE(season_id, recipient_id)` DEFERRABLE or dropping it.

### BANNED: reading `SAMETE_TEST_MODE` / `SAMETE_SMS_DRY_RUN` outside `src/config.rs`
### BANNED: weakening the guards
No "allow test mode in prod" override var, no warning-only fallback instead of refusing boot, no `0.0.0.0` exception.

### BANNED: E2E shortcuts
`waitForTimeout`, `networkidle`, `waitForLoadState`, `force: true`, `page.evaluate`, `getByText`, `getByRole` with name, CSS-class selectors, non-retrying assertions on DOM state where a web-first/`expect.poll` form exists. Import `test`/`expect` only from `./fixtures/cached-context`.

### BANNED: environment-touching commands
`just e2e`, `just e2e-release`, `just db-reset`, `just _kill-stale`, anything on port 3000 or DB `samete`, bare `cargo sqlx prepare` without `-- --features ssr`, `git push`, `git merge` into main, commits outside your worktree.

### BANNED: lint escapes
No new `#[allow(clippy::…)]` without a one-line WHY comment. No `#[cfg]` tokens inside `view!`. No `attr:`-prefixed attributes on native elements. Every new `pub fn` returning `Result` has a `# Errors` doc section (clippy pedantic `missing_errors_doc`).

### BANNED: scope creep
No changes to: `request_otp` outcome enum, IP rate limiting, OTP hashing, Dockerfile, CI workflow, assignment generation, swap form markup (A1's `aria-busy` token is the only swap-form change), visual-audit captures (only the A40 comment text changes), `[patch.crates-io]` or any dependency version (T1). Out-of-scope findings → report in DONE_WITH_CONCERNS, do not fix.

## Definition of Done (per unit; binary)
1. Every "What You Must Do" step applied; no step skipped or substituted.
2. Every unit-specific gate shows its REQUIRED OUTPUT (paste the command + output in the report).
3. Standard gates: fmt clean; both clippy runs zero warnings; both `cargo test` runs `ok`, `0 failed`.
4. `.sqlx/` regenerated with `-- --features ssr` (U1, U3, U4) and committed; `SQLX_OFFLINE=true` clippy passes.
5. E2E gate green 3 consecutive times via `scripts/isolated-capture.sh e2e_<unit> full`; new/changed titles listed as passed.
6. `git status --short` clean after commit; exactly the unit's write-set files changed (`git diff --stat main...HEAD`).
7. One-line conventional commit(s) in the worktree; report the SHA(s). Suggested messages: `build(env): pin toolchain and add idempotent bootstrap + cloud SessionStart hook` (ENV), `fix(build): reduce view type depth at component seams and make CI/E2E fail on build failure` (T0), `fix(build): unwind panics on the server so a task panic cannot kill it` (T1), `fix(a11y): emit bare aria-busy on admin submit buttons` (A1), `fix(auth): require server-side registration ticket for self-registration` (U1), `fix(config): confine test mode to dry-run SMS and loopback bind` (U2), `feat(admin): bootstrap first admin from env at startup` (U3), `fix(assignments): make swap a transactional position exchange and guard release` (U4).
8. Report status: DONE / DONE_WITH_CONCERNS / BLOCKED with gate evidence. A gate failing twice → STOP and report BLOCKED (task, failing output, attempts).

## Global Gates (orchestrator, on the working branch once every lane is integrated)
```bash
SQLX_OFFLINE=true cargo build --no-default-features --features ssr --bin samete
SQLX_OFFLINE=true cargo build --lib --target wasm32-unknown-unknown --no-default-features --features hydrate
SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr -- -D warnings
cargo test && SQLX_OFFLINE=true cargo test --features ssr
bash scripts/isolated-capture.sh e2e_final full > /tmp/e2e_final.log 2>&1; echo "exit=$?"
grep -rn "pending_phone\|SAMETE_TEST_MODE" src/ | grep -v "src/config.rs\|src/main.rs"
grep -rn "attr:" src --include=*.rs | wc -l
```
**REQUIRED:** both builds exit 0; clippy clean; tests ok; `exit=0` with `121 passed`, `2 skipped`, 0 failed (T0 baseline 116 + U1's 4 + A1's 1); the pending_phone/TEST_MODE grep has zero matches; the `attr:` count is `0`.
