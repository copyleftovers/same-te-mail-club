# c-ci — CI status (copyleftovers/same-te-mail-club), 2026-10-04

Manifesto binding: /tmp manifests exist; commitments — stop-yapping: tables only; first-principles: GitHub API as source, docs as claims; kiss: minimal queries; dry: no repeated data.

## 1. Last 15 runs on main (workflow "CI", total_count 39; only 15 returned by perPage)
| run id | sha | title | event | conclusion | date |
|---|---|---|---|---|---|
| 30162383205 | c472bfc | ci: run validation for RV5 residual round | push | success | 2026-07-25 |
| 29189498352 | 6fc68ca | fix(deps): upgrade transitive CVEs via lockfile update; ignore unfixable rsa advisory (FU-19b) | push | success | 2026-07-12 |
| 29188964277 | 6600644 | perf(sms): batch N+1 DB writes (FU-27) | push | FAILURE | 2026-07-12 |
| 29177651904 | 36e62e5 | fix(devx): add SQLX_OFFLINE clippy pass to check | push | success | 2026-07-12 |
| 29151064861 | 80d1c70 | fix(i18n): align cancelled-season copy | push | success | 2026-07-11 |
| 29149491267 | 4a9792f | doc(guidance): fix z-index map | push | success | 2026-07-11 |
| 29148792346 | 2e001a5 | fix(admin): em-dash dark contrast | push | success | 2026-07-11 |
| 29132991848 | d6a17f6 | fix(visual-audit): distinct stateId | push | success | 2026-07-11 |
| 29118276831 | 443efdd | fix(admin): cycle-viz aspect-ratio | push | success | 2026-07-10 |
| 28719895709 | f3291e5 | refactor(onboarding): FIELD_DISCRIMINANT_SEPARATOR | push | success | 2026-07-04 |
| 28717592269 | d4a9396 | test(e2e): onboarding validation capture | push | success | 2026-07-04 |
| 28678927584 | 5ba82ec | doc: session 2026-06-25 LEAVE | push | success | 2026-07-03 |
| 28172280159 | 405b955 | fix(sqlx): regenerate query cache | push | success | 2026-06-25 |
| 28171533086 | e94cfe2 | docs(frontend-protocol): drop stale .stat-card refs | push | FAILURE | 2026-06-25 |
| 28104967929 | 1012dfe | fix(auth): propagate SMS delivery failure | push | success | 2026-06-24 |

All events = push. Trigger: ci.yml `on: push/pull_request` branches [main]; no workflow_dispatch.

## 2. Latest code-validating run
Run 30162383205 @ c472bfc (2026-07-25, ~10m20s), attempt 1, success. Jobs:
| job | conclusion | notable steps |
|---|---|---|
| Check (fmt + clippy + test) | success | fmt, clippy SSR, clippy hydrate/WASM, supply-chain audit, cargo test all success |
| E2E (Playwright — release) | success | "Run E2E tests (release)" 14:54:02-14:59:18 success |

## 3. HEAD coverage
| commit | run? | note |
|---|---|---|
| 1a3c8a5 (HEAD) | NO run | message has skip directive; docs-only |
| c472bfc | run 30162383205 success | empty trigger commit; validates tree incl. all code up to acee94f |
| fc322ed | NO run | empty commit; skip token substring in message suppressed it |
| 4f47a95 | NO run | skip token, docs |
- `git diff --stat c472bfc HEAD`: 3 files, +46 — only orchestration_log/history/2026-07-15/session.md, reference/codebase_state.md, reference/conventions.md. Zero code after the last green run. OK.
- Last green run covers the full RV5 code tree (acee94f code + docs). Last code commit = 027586b/acee94f (<= c472bfc).
- Local clone is shallow/truncated (50 commits; `git cat-file` rejects 6fc68ca, 6600644, 36e62e5, 80d1c70) so run SHAs of 07-12 and earlier cannot be matched locally; origin main (`git ls-remote`) = 1a3c8a5 = HEAD.

## 4. Red runs in last 15
| run | job | first failing step | evidence |
|---|---|---|---|
| 29188964277 (6600644, 07-12) | Check (fmt+clippy+test) | Supply-chain audit (cargo audit, exit 1 on RUSTSEC warnings: rand 0.8.5/0.9.2 RUSTSEC-2026-0097, RUSTSEC-2026-0190 `Error::downcast_mut` unsound) | log tail: `##[error]Process completed with exit code 1.` after advisory list. Step name inferred from the log content (audit output), not from step metadata. Fixed next run 29189498352 (lockfile upgrade + ignore rsa). |
| 28171533086 (e94cfe2, 06-25) | Check | clippy SSR (per conventions.md claim: sqlx offline cache stale) | logs 410 Gone (expired); cause UNVERIFIED from API. Fixed by 28172280159. |
Trend: 13/15 green; both reds followed by green on the next push; latest 2 runs of record green... note most recent run is green.

## 5. Open PRs / issues
- Open PRs: none (`[]`).
- Open issues: none (totalCount 0).

## Findings
| sev | finding |
|---|---|
| OK | HEAD tree == last green CI tree; no unvalidated code |
| MINOR | HEAD 1a3c8a5 and fc322ed have no run (skip-token); harmless but "CI green at HEAD" is by inheritance, not by run |
| MINOR | Supply-chain audit is a hard CI gate; new RUSTSEC advisories since 2026-07-25 would turn the next push red (CI last ran 2026-07-25, >2 months ago; freshness unverified) |
| MINOR | Local clone truncated; cannot verify older run SHAs locally |
