# Spec Review: worktree-agent-a4d2369a7e8477b86

Verdict: PASS
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a4d2369a7e8477b86
Branch: worktree-agent-a4d2369a7e8477b86
HEAD SHA: a6bb8be1beb8c3a5c3e8735d192760dca9355b1e
Diff range: 57f0109..a6bb8be (e1736a6 fix(build), a6bb8be docs(deferred))
Reviewed at: 2026-10-07T04:10Z
Unit: T1, orchestration_log/history/2026-10-04/plans/PLAN-blockers.md (the implementer's authoritative copy was the worktree's PLAN-W1.md)
Files reviewed:
- Cargo.toml
- scripts/ssr-stress.sh
- orchestration_log/reference/deferred_items.md
- end2end/tests/fixtures/mail_club_page.ts, end2end/tests/visual-audit.spec.ts, end2end/tests/mail_club.spec.ts, src/admin/page.rs (flake root-cause only)
- scripts/isolated-capture.sh (flake hypothesis check)

## Findings

PASS. The implementation matches the spec, and all 5 T1 requirements are verified in code and by gates I re-ran myself.

| Req | Evidence |
|---|---|
| T1.1 Cargo.toml profiles | The diff matches the plan's exact TOML form verbatim. Gate output: `1 1 0 1 lock-unchanged`. No other Cargo.toml change, and Cargo.lock is untouched. |
| T1.2 scripts/ssr-stress.sh | The file is mode 755 and `bash -n` is clean. It is byte-identical to the script block in the implementer's plan copy (PLAN-W1.md). |
| T1.3 / WASM size | The wasm-release profile still resolves to `panic = "abort"`: it inherited abort from release before, and sets it explicitly now. So the wasm build is unchanged by construction. brotli -q11 of the built samete.wasm = 533602, matching the implementer's before/after figure. The plan's "≈471 KB / 460–490k" expectation is stale (a pre-existing size drift, not a T1 regression). |
| T1.4 deferred_items | Exactly one line was removed, the `- Leptos SSR reactive-disposal panic` line. `grep -c reactive-disposal` = 0. |
| Commits | Two one-line conventional commits, allowed by DoD item 7 ("commit(s)"). The write-set is exactly the 3 planned files. |

**RED gate (my run).** I built a release ssr bin with `CARGO_PROFILE_RELEASE_PANIC=abort`; it has the same source as HEAD, so it is equivalent to base. Run red1 printed `DEAD at round 64; panics logged: 1` and `stress=1`. The server log contains `Tried to access a reactive value that has already been disposed.`, followed by shell `Aborted`.

**GREEN gate (my run, HEAD unwind release bin).**
- green1: `ALIVE after 80 rounds; panics logged: 3`
- green2: `ALIVE after 80 rounds; panics logged: 2`
- green3: `ALIVE after 80 rounds; panics logged: 1`
- All three had `stress=0`, and every panic was the disposed-signal one. The sum is 6 (required ≥1): the trigger fired and was absorbed.

**E2E (my run).** I ran the full suite (`npx playwright test`, workers=1) against the HEAD unwind binary on its own port (3983) and sibling DB `samete_t1spec_e2e`, with TEST_MODE and DRY_RUN set:
- `116 passed, 2 skipped, 0 failed`, `pw=0`.
- The server was alive at the end, and there were 0 ERR_CONNECTION_REFUSED.
- This matches the 116/2 baseline.
- Harness note: I ran the binary directly rather than through cargo-leptos. A bin built with plain `cargo build` references `pkg/samete_bg.wasm` (a compile-time name cargo-leptos normally sets), so I served a scratch copy of the worktree's `target/site` with a `samete_bg.wasm` alias. A first attempt without the alias failed 4 tests on hydration. That was my harness setup, not the code.

**Standard gates (my run, with the gate-shell-only recursion_limit=256 crate-attr per Lanes, since T0 is not on this branch).**
- fmt: clean.
- ssr clippy `-D warnings`: exit 0.
- `cargo test`: ok, 68 passed.
- `cargo test --features ssr`: ok, 75 passed.
- ssr release bin build: Finished.
- wasm32 hydrate clippy: FAILS with 32 errors (rustc/clippy 1.99 `unused async` on every `#[server]` client expansion across 8 files, plus 2 `.ok().is_some_and` at login.rs). This is **pre-existing and not attributable to T1**:
  - `git diff 57f0109..a6bb8be -- src build.rs Cargo.lock` is empty.
  - The only Cargo.toml change is to release-profile `panic`, which clippy's dev profile does not use.
  - The U1 spec review (spec-worktree-agent-aa77876d6f145827e-2330.md) found the identical failure independently.
  - It is a toolchain-drift item for the orchestrator, not a T1 defect.

**E2E flake root cause (implementer run 2, visual-audit.spec.ts:800, strict-mode violation on `inactive-status`).** The flake predates T1 and T1 did not cause it. It is a stale-match race in the POM, not leftover DB state:
- `end2end/tests/fixtures/mail_club_page.ts:415`: `deactivateParticipant` waits on the page-wide `getByTestId("inactive-status")`, not on the clicked row.
- `src/admin/page.rs:2194` renders that testid on every inactive row.
- In full mode, files run alphabetically with workers=1. So `mail_club.spec.ts:880` (Account Management 6.1) has already deactivated "Деактивований Учасник" before visual-audit Pass B runs. One `inactive-status` therefore already exists when `deactivateParticipant(AUDIT_NAMES.C)` clicks.
- Playwright 1.58.2 `frames.js` (`_expectInternal`) throws `strictModeViolationError` inside evaluate when more than one element matches. That error is non-retriable.
- So there are two outcomes, depending on timing:
  - Usually the first one-shot check sees exactly 1 (stale) element and passes immediately. The wait never actually waited for the deactivation.
  - When the POST plus the refetch land before that first check, it sees 2 elements and fails at once.
- The DB-leftover hypothesis is ruled out: a reused DB would make it fail deterministically, not 1 run in 4.
- T1 only changes panic strategy and has no causal path to this request/refetch timing.
- Fix, out of T1 scope: scope the wait to the row (`row.getByTestId("inactive-status")`), or wait on the row's own state. Route it as a separate E2E unit; mail_club.spec.ts:881 has the same unscoped assertion.

Missing: none. Partial: none. Extra: none. Misinterpreted: none.

Plan-text drift (not an implementer defect): the tracked PLAN-blockers.md T1.2/T1 gates say "burst/bursts". The implementer's worktree copy PLAN-W1.md says "round/rounds", and the script follows that copy verbatim. The orchestrator should reconcile the canonical plan text: REQUIRED strings are now `DEAD at round …` / `ALIVE after 80 rounds`.

## Reasoning

I read the T1 section of the canonical plan, then compared it to the implementer's PLAN-W1.md copy in the worktree. The two differ only in burst→round wording, and the script matches the copy byte for byte. The diff range touches exactly the 3 write-set files. I checked the Cargo.toml hunk against the exact TOML block and ran the profile-wiring greps; cargo-leptos metadata confirms that the bin uses `release` and the lib uses `wasm-release`, with no `bin-profile-release`.

I did not trust the implementer's gate claims. I rebuilt both an abort binary and an unwind binary myself, from the same source with only the profile varied. I reproduced RED (a disposed-signal panic followed by process abort) and GREEN (3/3 alive with 6 absorbed panics). That directly proves the unit's purpose. An earlier "DEAD at round 1, panics 0" result was invalid: the borrowed binary had been deleted during the disk-full cleanup, so the server never started. I discarded it and rebuilt. A full E2E run on the unwind binary matched the 116/2 baseline with the server alive throughout.

On WASM size I reasoned from the profile resolution: both before and after resolve to abort, so the plan's ≤ check holds by construction. The absolute number (533602) confirms the implementer's figure and shows that the plan's range is stale. I root-caused the reported flake from Playwright's own source plus deterministic test ordering, rather than accepting the implementer's DB-leftover guess. I attributed the wasm-clippy failure as pre-existing because T1 has an empty src diff and a separate review found the same failure independently.

I made no edits in the worktree (`git status --short` is empty). All builds, binaries and site copies live in my scratchpad.
