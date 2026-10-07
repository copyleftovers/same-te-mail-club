# f-e2e — full E2E suite in this container

Binding: stop-yapping / first-principles / kiss / dry read in full. Commitments: cut noise -> tables+fragments only; derive from source -> every claim cites a command/log/file:line; keep simple -> one verdict table, one proof section per finding; no duplication -> logs hold raw output, this file holds verdicts.

## Setup

| Step | Command | Result |
|---|---|---|
| cargo-binstall | curl prebuilt from GH releases | OK — no cargo-binstall preinstalled |
| cargo-leptos | `cargo binstall cargo-leptos` → fell back to source build of 0.3.11; GH GraphQL 403'd via proxy for binary fetch | **0.3.11 built but incompatible** (see Finding 1). Fixed: downloaded `cargo-leptos-x86_64-unknown-linux-musl.tar.gz` v0.3.7 directly (matches `ci.yml`'s resolved version) and placed at `/root/.cargo/bin/cargo-leptos` |
| just, wasm-opt, sqlx-cli | `cargo binstall` | OK (prebuilt binaries fetched) |
| brotli | `apt-get install -y brotli` | OK |
| wasm32-unknown-unknown target | `rustup target add` | OK, preinstalled by default toolchain |
| `npm ci` in `end2end/` | — | OK |
| rustc toolchain | default was 1.97.0; `rustup update stable` jumped to 1.99.0 | **Neither built the release binary** (see Finding 1). Used `rustup toolchain install 1.97.1` (CI's exact pinned version per `gh run view 30162383205 --log`) via `RUSTUP_TOOLCHAIN=1.97.1` env override per invocation (no repo file touched) |
| Chromium browser dirs | Preinstalled payload is at `chromium-1194`/`chromium_headless_shell-1194`; pinned `@playwright/test@1.58.2` expects revision `1208` | Fixed outside repo: created `/opt/pw-browsers/chromium-1208/chrome-linux64` and `/opt/pw-browsers/chromium_headless_shell-1208/chrome-headless-shell-linux64` as directories containing symlinks into the 1194 payload (+ a `chrome-headless-shell` symlink to `headless_shell` inside the 1194 dir, since the binary name differs between revisions) |
| Postgres 16 | docker daemon absent (`/var/run/docker.sock` missing, confirmed `docker info`) | Used the native `postgresql-16` cluster already installed in the image instead: `pg_ctlcluster 16 main start`; created role/db matching `.env.example` (`samete`/`samete`/`samete`) via `psql` as the `postgres` OS user |

## Environment deltas vs CI

| Axis | CI (`ci.yml` / `gh run view 30162383205 --log`) | This container | Note |
|---|---|---|---|
| rustc | 1.97.1 (dtolnay/rust-toolchain@stable at that CI run's resolve time) | 1.99.0 default; pinned to 1.97.1 via `rustup toolchain install` + `RUSTUP_TOOLCHAIN` env (no toolchain file in repo, so nothing to edit) | `recursion_limit` overflow on both 1.97.0 and 1.99.0 (Finding 1) |
| cargo-leptos | 0.3.7 (binstall resolved that version in the CI run inspected) | 0.3.7 (manually fetched; binstall itself resolves 0.3.11 — version drift in the registry/releases since that CI run) | 0.3.11 is also incompatible standalone (Finding 1) regardless of rustc |
| Postgres | Docker service container `postgres:16` | Native `postgresql-16` package, docker daemon unavailable in this container | Functionally identical (pg 16, same role/db/pw) |
| Playwright browser | installed fresh via `npx playwright install --with-deps chromium` (revision resolved by pinned `@playwright/test`) | Preinstalled revision 1194 symlinked to the expected 1208 path (binary-compatible Chrome for Testing build) | Worked after symlink fix; no `playwright install` run (per prompt's hard constraint) |
| Disk | Ephemeral CI runner, presumably ample | **This container ran out of disk twice mid-build** (`target/debug` reached 14G; `/` hit 100%) | Required `rm -rf target/debug/incremental` and `rm -rf /tmp/cargo-install*` / `~/.cache/uv` mid-task; this is a container sizing issue, not a CI config difference |

## Runs

| Run | Passed | Failed | Skipped | Did-not-run | Duration | Log |
|---|---|---|---|---|---|---|
| 1 | 102 | 1 | 2 | 12 | 2.9m | `logs/f-e2e-run1.log` |
| 2 | 14 | 4 | 1 | 98 | 1.0m | `logs/f-e2e-run2.log` |
| 3 | not run | — | — | — | — | Stopped after 2 runs: both crashed the server process via the same pre-existing panic (see Finding 2); a 3rd green run is not achievable without a source fix, and the prompt forbids editing source/tests |

Two earlier attempts (`logs/f-e2e-run1-rustc1.97.0-FAIL.log`, `-rustc1.99.0-FAIL.log`, `-leptos0.3.11-FAIL.log`, `-leptos0.3.7-FAIL.log`, `-enospc.log`, `-enospc2.log`) failed purely on toolchain/disk setup before any test ran — captured for the setup table, not counted as suite runs.

## Failures

| Test | Error | Classification | Evidence |
|---|---|---|---|
| `visual-audit.spec.ts:676` capture home — assignment visible to participant (run1) | `thread 'tokio-rt-worker' panicked at reactive_graph-0.2.13/src/traits.rs:394:39: Tried to access a reactive value that has already been disposed.` → server process crash → `Resource 'tokio_process_tools::ProcessHandle' should not have been dropped` → cascading `ERR_CONNECTION_REFUSED`/`ERR_INSUFFICIENT_RESOURCES` on every subsequent test in the suite | **Product bug (pre-existing, known)** | `logs/f-e2e-run1.log` lines ~271-275; matches `orchestration_log/reference/deferred_items.md`: "Leptos SSR reactive-disposal panic (intermittent `tower_http` 500s) — no fix commit exists" |
| `mail_club.spec.ts:251` 4.1 admin creates a season (run2) + full cascade (98 did-not-run) | Same panic signature, same process-crash cascade, triggered earlier in run2's serial sequence (nondeterministic timing — "intermittent" per the existing deferred-item description) | **Product bug (pre-existing, known)**, confirmed reproducible across 2 independent runs | `logs/f-e2e-run2.log` line 75: `thread 'tokio-rt-worker' (27382) panicked at .../reactive_graph-0.2.13/src/traits.rs:394:39` |
| `tests/visual-audit.spec.ts:249` capture login — phone input (both runs, retry too) | Downstream of the crashed server (`ERR_INSUFFICIENT_RESOURCES`/`ERR_CONNECTION_REFUSED`/cache-write `ENOSPC`) | Downstream of the above / environment (disk) — not independent | same logs |
| `mail_club.spec.ts:854` "generate invite code for deactivation target" (run1 only, first enospc-affected attempt, superseded) | `ENOSPC: no space left on device` in `cached-context.ts:91` | **Environment** (container disk exhaustion mid-build, fixed before the counted run1/run2 above) | `logs/f-e2e-run1-enospc.log` |

**Root-cause note on the panic:** it is the same `reactive_graph` disposal panic already tracked as an open item in `orchestration_log/reference/deferred_items.md` ("Leptos SSR reactive-disposal panic (intermittent `tower_http` 500s)"). It reproduced in both runs attempted here, at different points in the serial test sequence — consistent with "intermittent." This confirms the deferred item is still live; no new root cause found in this session (no source edits made, per the read-only constraint).

## Finding 1 — toolchain-version sensitivity (environment-setup hazard)

`cargo-leptos` 0.3.11 built against rustc 1.97.0 or 1.99.0 fails the release build of `samete` with:
```
error: queries overflow the depth limit!
  = help: consider increasing the recursion limit by adding a `#![recursion_limit = "256"]` attribute to your crate (`samete`)
```
(Full output: `logs/f-e2e-run1-rustc1.97.0-FAIL.log`, `-rustc1.99.0-FAIL.log`.) Pinning rustc to CI's exact 1.97.1 did not fix it alone — cargo-leptos 0.3.11 (source-built, since binstall's GitHub GraphQL check 403'd through the proxy) still overflowed (`logs/f-e2e-run1-rustc1.97.1-leptos0.3.11-FAIL.log`). Downgrading to cargo-leptos **0.3.7** (CI's resolved version at the time of the last green run, confirmed via `gh run view 30162383205 --log`) still overflowed under plain `just e2e` (`logs/f-e2e-run1-rustc1.97.1-leptos0.3.7-FAIL.log`) — tracked down via `strace -f -e trace=execve` to the WASM-target (`hydrate` feature) compile step of `src/admin/page.rs`'s large nested view! tree (table/list construction, `page.rs:1841-1872`). Workaround applied (env-only, no repo file touched): `RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Zcrate-attr=recursion_limit="1024"'` on every build/run invocation. Builds then succeeded and tests ran. This is an environment-reproducibility gap (a newer `rustc`/toolchain patch triggers a type-nesting/query-depth sensitivity that CI's pinned `dtolnay/rust-toolchain@stable` snapshot apparently did not hit) — not something introduced by this session, but worth a `#![recursion_limit = "..."]` crate attribute or dependency bump if CI ever re-resolves a newer stable.

## Verdict

**BLOCKER** (environment): the full suite does not run out-of-the-box in this container — three separate toolchain-compatibility gaps had to be worked around (cargo-leptos version resolution, rustc version sensitivity triggering a recursion-limit overflow, Playwright browser revision mismatch) plus a disk-space ceiling that crashed two build attempts mid-flight.

**MAJOR** (product): once running, the suite is not 3x-green-stable — a pre-existing Leptos reactive-disposal panic (already tracked in `deferred_items.md`) crashed the server process in both completed runs, at different points in the serial sequence, cascading into dozens of false "did not run"/network-error failures downstream. This is not a new defect; it reconfirms a known, still-unfixed issue.

No repo file was left modified: `git status --short` is empty; `git diff --stat` is empty. All installs/fixes (cargo-leptos binary swap, Playwright browser symlinks, rustc 1.97.1 toolchain, Postgres role/db, env-only `RUSTFLAGS`/`RUSTUP_TOOLCHAIN`) are container-level, not repo-level.

---

## attr: leak check

Method: built release binary (rustc 1.97.1, cargo-leptos 0.3.7, `RUSTFLAGS='-Zcrate-attr=recursion_limit="1024"'`), served it on an isolated port against a sibling DB (`samete_fattr`), fetched SSR HTML (`curl`/`ctx.request.get`) for `/login`, `/onboarding`, `/admin`, `/` anonymously and as the seeded admin, and the **hydrated DOM** (`page.content()` after WASM init + a route with the button in its pending state) for `/admin`, `/`, `/onboarding`.

Grep commands (token-spacing-tolerant) run against every captured file:
```
grep -oE 'attr ?: ?[a-z-]+' <file>        # catches "attr:x", "attr : x", "attr:  x"
grep -oE 'attr[^a-z]{0,3}:' <file>        # catches any "attr" immediately followed by ":"
grep -o 'aria-busy' <file>                # raw occurrence count
```

| File | `attr ?: ?[a-z-]+` matches | bare `aria-busy` count |
|---|---|---|
| `ssr_login.html` (anon) | 0 | 0 |
| `ssr_onboarding.html` (anon) | 0 | 0 |
| `ssr_admin.html` (admin SSR, idle) | 0 | 0 |
| `ssr_home.html` (admin SSR, idle) | 0 | 0 |
| `dom_admin.html` (hydrated, idle) | 0 | 0 |
| `dom_home.html` (hydrated, idle) | 0 | 0 |
| `dom_onboarding.html` (hydrated, idle) | 0 | 0 |
| `dom_admin_pending.html` (hydrated, **mid-submit** — `create-season-button` in its `pending` state, POST artificially delayed 4s) | 1 | — (see below) |

The one pending-state capture: the live button's `getAttributeNames()` (evaluated in-page, not regexed from a serialized string) returned:
```
["type=submit","data-testid=create-season-button","class=btn","attr:aria-busy=true","disabled="]
```
**Confirmed: `attr:aria-busy` leaks as a literal DOM attribute named `attr:aria-busy`** (value `"true"`) instead of a bare `aria-busy="true"` — but **only while the signal driving it is truthy** (i.e. only during the pending/loading state of a submit). In every idle-state capture (SSR and hydrated, all 4 pages) the attribute is simply absent (the `move || pending.get().then_some("true")` closure returns `None` when idle, so Leptos emits nothing at all — not even a leaked name). This matches the known Leptos 0.8 hazard already logged in `orchestration_log/reference/conventions.md` ("Added 2026-07-04... `attr:aria-invalid` ... on a NATIVE element emits a literal attribute NAMED `attr:...`") but documents it is `aria-busy`-specific here and that it is invisible unless the pending branch is actually exercised (hence 0/0 at every one of the 13 call sites in `src/admin/page.rs` under idle conditions).

App-wide `attr:` leak grep across all SSR/DOM captures (login, onboarding, home, admin — anonymous and authenticated, idle and one pending case): **0 occurrences except the single pending-state `attr:aria-busy` above.** No other `attr:`-prefixed attribute name was found leaking in any captured page.

**Severity: MAJOR** (accessibility regression, silent — assistive tech reads a non-standard attribute name instead of `aria-busy` during every loading state on every one of admin page.rs's 13 submit buttons; not a BLOCKER since it only affects the transient pending window, not steady-state rendering).

---

## Auth bypass empirical check

Same running release server + sibling DB `samete_fattr`, with `SAMETE_SMS_DRY_RUN=true` and **`SAMETE_TEST_MODE` unset** for this specific check (server restarted without it). Steps taken exactly per `g-authverify.md`'s 1b proof:

1. Inserted one unused invite code directly via `psql` (admin UI's own generate-code flow was not used, to keep this fully independent of TEST_MODE):
```
PGPASSWORD=samete psql -h localhost -U samete -d samete_fattr -c \
  "insert into invite_codes (code, distributor_id) select 'bypass-test', id from users where phone='+380670000001' returning code,status;"
```
→ `bypass-test | unused` (1 row).

2. Resolved the real server-fn endpoint path (leptos appends a build-specific numeric hash) by capturing the actual rendered form via a one-off Playwright script (`getform.cjs`, outside the repo) that drove the UI up to the name-collection step: `action="/api/register_with_code4575314452317009345"`.

3. Sent the registration request **cold** — no `/login` visit, no `request_otp` call, no `verify_otp_code` call ever made by this client — with a hand-set `pending_phone` cookie carrying a fresh, never-before-seen phone number:
```
curl -sS -i -X POST "http://127.0.0.1:<port>/api/register_with_code4575314452317009345" \
  -H "Cookie: pending_phone=+380999000222" \
  --data-urlencode "code=bypass-test" --data-urlencode "name=Bypass Victim"
```

**Result:**
```
HTTP/1.1 200 OK
content-type: application/json
set-cookie: session=uGQPcaL93lQGeIc7RjE9EXvLgefcS9ra6JseD8dqxtU; HttpOnly; Secure; SameSite=Strict; Max-Age=7776000; Path=/
set-cookie: pending_phone=; HttpOnly; Secure; SameSite=Strict; Max-Age=0; Path=/
location: /onboarding
```
DB verification (same `psql`):
```
select code,status,redeemer_id from invite_codes where code='bypass-test';
 bypass-test | used | 51a2289e-1830-44e9-ba4a-89fa7b17a28d

select u.phone, s.token_hash is not null as has_session from sessions s join users u on u.id=s.user_id where u.phone='+380999000222';
 +380999000222 | t
```
Pre-check confirmed the phone had zero `users` rows before the request (`select count(*) from users where phone='+380999000222'` → `0`).

**Confirmed empirically: self-registration fully skips OTP.** A client that never requested or entered any OTP code, holding only a forged `pending_phone` cookie (a raw, unsigned, unencrypted phone string — trivially settable by `curl`) plus one valid invite code, obtained a complete authenticated session (`users` row created, invite code marked `used` and bound to the attacker's chosen name, `sessions` row created, `Set-Cookie: session=...` returned) for a phone number the attacker does not own. This matches `g-authverify.md`'s static finding 1b exactly, now confirmed live against a running server rather than by source trace alone.

**Severity: BLOCKER** (confirms the prior static-analysis BLOCKER with a live reproduction; anyone holding one valid invite code can register/takeover any unclaimed phone number without ever possessing that phone, defeating the entire purpose of SMS-OTP auth).
