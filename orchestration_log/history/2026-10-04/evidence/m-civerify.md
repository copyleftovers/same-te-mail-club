# CI E2E verification (repo copyleftovers/same-te-mail-club, ci.yml)

Source: GitHub Actions job logs via MCP; logs older than ~90 days return 410 Gone.

| run | date | sha | rustc | build errors? | playwright summary | conclusion |
|---|---|---|---|---|---|---|
| 30162383205 | 2026-07-25 | c472bfc | 1.97.1 (8bab26f4f 2026-07-14) | YES: `queries overflow the depth limit!` (lib at 14:56:48, bin at 14:59:18) -> `could not compile samete (bin "samete") due to 1 previous error` | none (no "passed"/"failed"/"Running N tests") | success (E2E step 13 `success`) |
| 29189498352 | 2026-07-12 (latest other green) | 6fc68ca | 1.97.0 (2d8144b78 2026-07-07) | YES: same recursion-limit error, bin | none | success |
| 29132991848 | 2026-07-11 | d6a17f6 | 1.97.0 | YES: same | none | success |
| 29118276831 | 2026-07-10 | 443efdd | 1.97.0 | YES: same | none | success |
| 28678927584 | 2026-07-03 (earliest green in window) | 5ba82ec | unknown, log 410 Gone | unknown, log 410 | unknown, log 410 | success |
| 28717592269 | 2026-07-04 | d4a9396 | unknown, log 410 Gone | unknown | unknown | success |

Error text (identical in all 4 readable logs):
`error: queries overflow the depth limit!` / `help: consider increasing the recursion limit by adding a #![recursion_limit = "256"] attribute to your crate (samete)` / note: query depth increased by 130 when computing layout of async block ... `samete::pages::login::__component_login_step_router` closures (Leptos view type nesting in src/pages/login.rs).

## Findings

1. Most recent run where Playwright demonstrably executed tests: none provable. All 4 readable logs (2026-07-10 .. 2026-07-25) contain no Playwright output and end at the build error. Runs 2026-07-03 and 2026-07-04 have expired logs (410), so whether Playwright ran then is unverifiable via logs. 
2. Onset: first readable failing build is 2026-07-10 (rustc 1.97.0, released 2026-07-07; runner moved from 1.96.1 -> 1.97.0 in that log). Consistent with a toolchain-triggered regression (stable is unpinned in ci.yml), but the 07-03/07-04 logs are gone, so onset cannot be proven from logs.
3. Step durations: E2E step ~5.5 min on 07-10/07-11/07-25 (long, consistent with a full release build ending in failure), ~1.7 min on 07-12 (cached build).

## ci.yml E2E gate analysis

- Only E2E test step: `just e2e-release` -> `cargo leptos end-to-end --release` (justfile line 51-52). Preceding steps are installs only.
- Observed: the build failed yet the step concluded `success` and the job concluded `success`. So `cargo leptos end-to-end` (or the just recipe) exits 0 when the server/lib build fails; no other step checks that Playwright ran. No step asserts a test count, reads results.json, or checks for the binary.
- Answer: NO step in the E2E job fails when the build fails. The E2E job is a false green. The `check` job (clippy SSR + hydrate, cargo test) passes: clippy does not do the layout computation that trips the recursion limit; a full `cargo build --release` does.
