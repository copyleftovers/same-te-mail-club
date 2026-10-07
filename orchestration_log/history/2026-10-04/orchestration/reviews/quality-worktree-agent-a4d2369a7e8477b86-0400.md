# Code Quality Review: worktree-agent-a4d2369a7e8477b86

Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a4d2369a7e8477b86
Branch: worktree-agent-a4d2369a7e8477b86
Diff range: 57f0109..a6bb8be (e1736a6 fix(build), a6bb8be docs(deferred))
Reviewed at: 2026-10-07T04:19Z
Unit: T1 (PLAN-blockers.md). Spec verdict A3: PASS (spec-worktree-agent-a4d2369a7e8477b86-2340.md)
Bound: self-documenting-code, correct-by-construction, kiss (full reads from /tmp/claude-manifesto-repo/LLM_MANIFESTOS/manifestos/)

Gates I ran myself in the worktree:
- Env: CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3 SQLX_OFFLINE=true.
- T0 is not on this branch, so the gate shell alone carried `RUSTC_BOOTSTRAP=1 RUSTFLAGS='-Zcrate-attr=recursion_limit="256"'`. It is not committed and not in any file.
- `cargo fmt --check` → 0
- `cargo clippy --no-default-features --features ssr -- -D warnings` → 0
- `cargo test` → 68 passed
- `cargo test --features ssr` → 75 passed
- `bash -n scripts/ssr-stress.sh` → ok (shellcheck is not installed)
- `git status --short` → clean. I made no edits. The `target/` already existed before my run and I left it in place.

I did not re-run wasm32 clippy. Its failure on rustc 1.99 is ENV scope: `git diff 57f0109..a6bb8be -- src` is empty.

## Code Quality Review

### Summary
T1 is a two-line profile change plus a 34-line manual regression probe and a one-line deferred-items deletion. The change is minimal and correctly wired:
- The server bin uses `release`, which is now unwind.
- The WASM lib uses `wasm-release`, which is now explicitly abort.
- There is no `bin-profile-release`, and Cargo.lock is unchanged.

The rationale comments are WHY-class and justified. I found no defects in the code. The findings concern how honest the probe's signal is and whether the debt record is complete.

### Strengths
- Cargo.toml:102 / :110. The fix is the smallest change that removes the whole failure class: any detached-task panic no longer kills the process. The design rejected a vendored `[patch.crates-io]` fork, which conventions forbid without the crate's own suite. That is the KISS choice.
- Cargo.toml:109-110. The explicit `panic = "abort"` on `wasm-release` encodes the WASM-size invariant in config rather than relying on inheritance. Remove that line and the WASM silently grows, and the comment says exactly that. This is a correct WHY comment.
- Cargo.toml:98-101. The WHY comment names the upstream site (`leptos_i18n` context.rs:213) and the mechanism (detached task after the Owner is disposed). It is a non-obvious, legitimate exception under self-documenting-code.
- scripts/ssr-stress.sh. The probe has bounded parameters: `${1:?usage}` guards, `set -u`, and an explicit exit contract (0 = alive, 1 = dead). It also has a per-round liveness check with a 5s cap, so the loop cannot hang unbounded.
- The spec reviewer's independent evidence carries the poisoning question. Under unwind, a panic mid-reactive-graph could in principle poison shared locks and cascade into later requests. The evidence shows no cascade:
  - GREEN 3/3: alive after 80 rounds with 6 absorbed panics, and `/login` still served after every round.
  - Full E2E: 116 passed / 2 skipped on the unwind binary.

### Critical Issues (Must Fix)
None.

### Important Issues (Should Fix)
None.

### Minor Issues (Nice to Have)
1. **The probe's panic counter matches any panic, not the disposed-signal one**
   - File: scripts/ssr-stress.sh:18
   - Issue: `grep -c "panicked" "$log"` counts every panic line. The plan's GREEN gate uses "sum of panics logged ≥ 1" as proof that the trigger fired and was absorbed. Any unrelated panic would satisfy that proof. For example, src/pages/login.rs:771 documents an SSR pre-warm panic class ("caught by tokio catch_unwind, but generates startup noise") that has existed in this codebase.
   - Impact: GREEN could pass vacuously if the disposed-signal path stopped firing while other panics remained. The spec reviewer checked the log text by hand this time; the script does not enforce it.
   - Fix: `grep -c "already been disposed" "$log"`, which is the same string the RED gate already greps. Optionally guard the missing-log case: today `grep` on an absent file prints an empty count instead of 0.

2. **The deferred-items deletion loses the record that the upstream defect is still live**
   - File: orchestration_log/reference/deferred_items.md (removed line 7)
   - Issue: T1 contains the blast radius; it does not fix the defect. The disposed-signal read in leptos_i18n 0.6.x still panics under concurrent load: the GREEN runs logged 6 panics. The plan states "the upstream fix stays out of scope." After deletion, nothing in the live debt list says that per-request panics are expected noise in production logs, or that a leptos_i18n bump should be checked against `scripts/ssr-stress.sh`.
   - Impact: Someone who later sees "panicked … already been disposed" in prod logs has no pointer to the cause. The rationale survives only in the Cargo.toml comment.
   - Fix: The plan mandated the deletion and conventions say resolve = delete, so the deletion itself is correct for the old item. The orchestrator may add one narrower open item instead, for example: "leptos_i18n 0.6.x disposed-signal read still panics (absorbed by unwind since T1); re-run scripts/ssr-stress.sh on dependency bump; drop when upstream fixes." This is an orchestrator call, not an implementer defect.

3. **Nothing automated guards the regression**
   - File: scripts/ssr-stress.sh (not referenced from justfile or .github/workflows/ci.yml)
   - Issue: A future edit that reverts `panic = "unwind"` (for example, "restore abort for binary size") would pass every CI gate. The probe exists but nothing runs it.
   - Impact: The fix holds only through the Cargo.toml comment.
   - Fix (optional, cheap): a one-line CI or `just check` assertion on profile wiring that reuses the plan's awk gate, `awk '/^\[profile.release\]/,/^$/' Cargo.toml | grep -q 'panic = "unwind"'`. The full stress run in CI is not justified. Defer if judged YAGNI.

4. **The rationale is duplicated in two places**
   - File: Cargo.toml:98-101 and scripts/ssr-stress.sh:5-8
   - Issue: The same WHY paragraph appears twice and can drift. The script's copy is justified because it explains why the probe exists. Trimming it to a pointer ("see Cargo.toml [profile.release] WHY") would give the rationale one home.
   - Impact: Negligible today.
   - Fix: Optional.

### Assessment
**Ready to merge:** Yes
**Reasoning:** The change is minimal and correct by construction (the profile split is verified by wiring, not by convention), and fmt, ssr clippy `-D warnings`, and both test suites pass. All findings are Minor: probe-signal precision and debt-record completeness, none blocking.
