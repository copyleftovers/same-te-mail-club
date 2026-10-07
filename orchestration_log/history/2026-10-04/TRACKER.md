# Solution Tracker — 2026-10-04 readiness campaign

Problems live in `PROBLEMS.md` (same dir). This file tracks only solutions: which unit fixes what, where the work is, and its state. Update on every state change; commit + push immediately.

## Resume from scratch (orchestrator dead)
1. Branch `claude/loving-johnson-7l8hn5` = integration branch. Plans: `plans/PLAN-blockers.md`, `plans/PLAN-product.md`, `plans/PLAN-findings.md` (unit specs; every unit is a concurrent lane).
2. Contracts/bindings (tracked copies): `orchestration/prompts/_implementer.md`, `_spec-reviewer.md`, `_code-quality-reviewer.md`, `_parallelism.md`, `lane-<UNIT>.md`; `orchestration/binding/<role>.md`. Live originals sit in gitignored `orchestration_log/recon/2026-10-04/` — if absent, copy these back there (lane prompts reference those absolute paths).
3. Manifestos: `git clone --depth 1 https://github.com/ryzhakar/LLM_MANIFESTOS /tmp/claude-manifesto-repo/LLM_MANIFESTOS`. Skills/agent defs: `git clone --depth 1 https://github.com/ryzhakar/claude-skills /home/user/ryzhakar/claude-skills` (then `claude plugin marketplace add` + install manifesto, orchestration, dev-discipline, agent-conduct, product-craft, qa-automation).
4. Cycle per unit (no exceptions): dev-discipline:implementer (worktree; dispatch = lane file) -> dev-discipline:spec-reviewer (verdict file) -> dev-discipline:code-quality-reviewer (report file, `Ready to merge:`) -> integrate (merge into branch, union `locales/uk.json` keys, `cargo sqlx prepare --workspace -- --features ssr`, fmt/clippy ssr+wasm/test gates, one-line commit, push, remove worktree).
5. Unit status below. A unit "implementing" whose worktree is gone = relaunch from its lane file. Branch names `worktree-agent-<id>`; check `git worktree list`, `git branch -a | grep worktree-agent`.
6. Environment gotchas until unit ENV lands: cargo-leptos install; Playwright browser path (`/opt/pw-browsers`); tailwind glibc binary; local-only `#![recursion_limit="256"]` until T0 lands; build env `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`; disk: dedupe `hardlink -c */target/*/deps` across worktrees; max 20 concurrent subagents.

## Recovery state (2026-10-05 20:40 UTC)
All implementers were killed by a session limit at ~09:20 UTC. Their work is preserved on this branch as patches in `wip/` (`INDEX.txt` lists per worktree: branch, base, commit count, uncommitted size; `<id>.commits.patch` = committed work, `<id>.uncommitted.patch` = working-tree diff). Apply with `git am` / `git apply` onto the listed base to recover a lane without the remote machine. Worktree target/ dirs were deleted (disk). Lane agents resumed 20:45 UTC. A background loop snapshots wip/ patches every 15 min onto this branch. Order of recovery: land U3+U4 (quality reviews resumed), then resume the lanes (SendMessage to the same agent ids — resume, not relaunch).

## Recovery log
- 2026-10-06 00:50 UTC: machine restored from an old disk snapshot (local branch at 5360af1, worktrees at ~09:20 state). Branch fast-forwarded from origin; every worktree rebuilt from wip/ patches of 2026-10-05 21:15 (old state kept in git stash entries `pre-restore-<id>`); Postgres restarted; all 18 agents resumed; loop scripts now tracked in orchestration/scripts/.

- 2026-10-06 10:50 UTC: all 18 agents resumed; all killed by the weekly limit (reset 17:00 UTC).
- 2026-10-06 17:55 UTC: resume policy changed (owner): SMALL BATCHES of 3, each batch finishes (implement -> spec -> quality -> merge) before the next. Order: U3 fix + U4 minors + T0; then T1, U1, U2; then ENV, A1, P lanes; then P-DOC; then F lanes. Loop scripts live at orchestration_log/history/2026-10-04/orchestration/scripts/.
- 2026-10-06 18:00 UTC: batch 1 = U3-fix (DONE, quality re-review sent to ae5b62914242ed4a1), U4-minors, T0; T1 resumed into U3's freed slot.
- 2026-10-06 18:25 UTC: U4-minors DONE @781241f, quality re-review sent to aae408a3826c4c153. U1 resumed into the slot. rustc drifted to 1.99 and wasm clippy fails in untouched code (32 errors); this is ENV's scope.
- 2026-10-06 19:00 UTC: U3 merged (gates: fmt, ssr clippy, ssr tests 92/0) + pushed; U4 minors merged (same gates green) + pushed. Running: T0, T1, U1.
- 2026-10-06 ~20:00 UTC: T0, T1 and U1 were killed by a session limit (reset 22:40 UTC). The worker restarted. 22:50 UTC: Postgres and the loops restarted, T0/T1/U1 resumed.
- 2026-10-06 23:05 UTC: T0 DONE_WITH_CONCERNS @4d000e7 (no recursion_limit; E2E 116/2/0 once; sabotage gates not run). Spec review dispatched (a4164fc74e49884c1, prompt orchestration/prompts/spec-T0.md). T1 and U1 still running.
- 2026-10-06 23:15 UTC: T0 spec FAIL. Missing: LoginPage .into_any(), the InviteCodeStep and NameStep extractions, sabotage gates, 3x E2E and pasted gate output; the stale CI comment must go; the 2 extra components need justifying. The findings went back to the T0 implementer. Next spec reviewer: give it a shell (general-purpose + role file); the dev-discipline:spec-reviewer type has no Bash.
- 2026-10-06 23:30 UTC: U1 DONE_WITH_CONCERNS @4bccd33 (3x E2E 120/2/0 reported; the curl gate was skipped). Spec review (general-purpose + role file, with shell) dispatched: a2360fe04bb536092, prompt spec-U1.md. U2 is held so no more than 3 agents run at once.
- 2026-10-06 23:45 UTC: T1 DONE_WITH_CONCERNS @a6bb8be. The stress test went red on the abort build and green on the unwind build. E2E was green on 3 of 4 runs; run 2 flaked at visual-audit:800 (inactive-status matched 2 rows). Spec review dispatched: a8dd9d6cf2adcb3b0 (spec-T1.md); it must root-cause the flake.
- 2026-10-07 00:05 UTC: U1 spec PASS. The reviewer ran the curl attack tests itself, all refused; E2E 120/2/0 clean. Quality review dispatched: a28e91d30492f1a82 (quality-U1.md). Note: the reviewer saw a reactive-disposal server abort in one E2E run, which T1 fixes. 23:49 UTC: disk hit 100%; freed idle target/ dirs; disk monitor armed.
- 2026-10-07 ~00:30 UTC: the T0 implementer, the T1 spec reviewer and the U1 quality reviewer were all killed by a session limit (reset 03:40); the container restarted. 03:50: Postgres and the loops restarted and all three resumed. T0 is at 3d8f468 (spec fixes committed; evidence steps pending).
- 2026-10-07 04:20 UTC: T1 spec PASS. The reviewer re-ran RED/GREEN and 1x E2E 116/2/0. Quality review dispatched: aaced95f49cc2fd50 (quality-T1.md). New problems: PRB-101 (unscoped `inactive-status` wait in the POM: the flake's root cause) and PRB-102 (rustc 1.99 drift, wasm clippy red). PRB-101 goes to F-COV, PRB-102 to ENV. PLAN-blockers stress-script wording changed burst -> round to match the script.
- 2026-10-07 04:25 UTC: order changed. ENV now takes the next free slot, ahead of U2, because PRB-102 keeps wasm clippy red on every lane. ENV's resume message must point to the PLAN-blockers ENV addendum; F-COV's resume message must point to its addendum (PRB-101).
- 2026-10-07 04:45 UTC: U1 quality: With fixes. Important: the invite-attempt throttle is not atomic (concurrent submits bypass the 5-attempt cap); plus 4 minors. Everything went back to the U1 implementer (aa77876d6f145827e); next is a quality re-review by a28e91d30492f1a82.
- 2026-10-07 04:40 UTC: T1 quality Yes (4 minors). Merged; gates fmt, ssr clippy, ssr tests 92/0. Pushed. Minors 1 (stress script counts any panic; grep the disposed panic) and 3 (no CI guard for panic=unwind) go to the T1 implementer when a slot frees. Minor 2: narrowed deferred item added to reference/deferred_items.md. ENV resumed with the PRB-102 addendum.
- 2026-10-07 ~05:00 UTC: T0, U1 and ENV were killed by a session limit (reset 08:50); the worker restarted. 09:50: Postgres and the loops restarted; idle target/ dirs freed (5.4G -> 12G); all three resumed. T0 @d09cbe7 (release builds green, 3x E2E pending). ENV @c82b4eb (is_ok_and fix + macro-lint allow; gates pending). U1 had 97 uncommitted files (.sqlx churn), told to regenerate properly and commit first.
- 2026-10-07 10:10 UTC: ENV DONE_WITH_CONCERNS @c82b4eb (toolchain pinned to 1.97.1 via rust-toolchain.toml; wasm clippy green on 1.97.1 and 1.99; SessionStart hook + bootstrap; macOS tailwind pins missing). Spec review dispatched: ac1b973e5c3b40e78 (spec-ENV.md). Running: T0 impl, U1 impl, ENV spec reviewer.
- 2026-10-07 10:20 UTC: T0 round-2 DONE @d09cbe7: all 5 spec items fixed and the evidence pasted (limit 96 ok, HTML identical, sabotage a/b/c red, 3x E2E 116/2/0). It touched home.rs, outside the plan's write-set, citing the limit-96 trace as the reason. Spec review round 2 (with shell) dispatched: a2403a6d25eb9ef07 (spec-T0-r2.md).
- 2026-10-07 10:55 UTC: ENV spec FAIL on one item only: the macOS tailwind sha256 pins are missing and the checksum command is not portable. Everything else passes, incl. the clean-shell gate 116/2/0 on 1.97.1 and the lint allow proven to cover only macro code. Sent back to the ENV implementer. Next: spec re-review by ac1b973e5c3b40e78.
- 2026-10-07 11:30 UTC: U1 quality fixes DONE @f89d773 (atomic claim before lookup + refund; new concurrent-cap E2E test red→green; 3x E2E 121/2/0). Quality re-review sent to a28e91d30492f1a82. Running: ENV impl (macOS pins), T0 spec r2, U1 quality r2.
- 2026-10-07 11:50 UTC: ENV spec PASS @3adddfc (macOS pins verified, portable sha256). Quality review dispatched: adbd58c3ed66841ad (quality-ENV.md). Running: ENV quality, T0 spec r2, U1 quality r2.
- 2026-10-07 12:00 UTC: T0 spec round 2 PASS @d09cbe7, minors M1-M5 non-blocking; the reviewer reproduced the limit-96 check and sabotage (a), and confirmed home.rs was forced. Quality review dispatched: a4da7633e863cdd01 (quality-T0.md). Running: T0 quality, ENV quality, U1 quality r2.
- 2026-10-07 12:10 UTC: ENV quality: With fixes. Three to fix: the crate-wide unknown_lints allow (scope it to hydrate); a bootstrap that can hang when pg_isready/psql are missing; the hook writes no env on network failure and has no timeout. Plus 7 minors. All went back to the ENV implementer; next is a quality re-review by adbd58c3ed66841ad.
- 2026-10-07 12:30 UTC: U1 quality Yes; merged b93bbf5. Gates: .sqlx regenerated (unchanged), fmt, ssr clippy, ssr tests 97/0. T0 quality Yes (4 minors); merged 7ebeb03 with no conflicts. Gates on the integrated branch, no shim: fmt, ssr clippy, tests, release SSR bin + wasm-release lib, 0 depth overflows. Both pushed; worktrees of T0, T1 and U1 removed.
- 2026-10-07 ~13:00 UTC: ENV killed by a session limit (reset 14:40). 14:50: ENV, U2 and A1 resumed, each told to merge the integrated branch first.
- Follow-up minors to bundle into one small fix unit (FOLLOWUP-1): T1 stress script greps any panic (should match the disposed panic) and has no CI guard for panic=unwind; U1 swallows the Internal DB error without logging, the burst-probe count has no lower bound, and MAX_INVITE_ATTEMPTS is duplicated in the spec; T0 is missing WHY comments on the inline into_any cells and has odd step ordering in login.rs.
- 2026-10-07 15:45 UTC: ENV quality fixes DONE @96fbdec + base merge ded4c39. Clean-shell E2E WITHOUT the recursion shim: 121/2/0; both clippies green on the pin and on 1.99. Quality re-review sent to adbd58c3ed66841ad. Running: U2 impl, A1 impl, ENV quality r2.
- 2026-10-07 16:15 UTC: ENV quality Yes. Merged 215f8b1 and pushed before its gates finished (the gate run was killed); gates re-running on the integrated branch. A1 DONE_WITH_CONCERNS @b8b8a34 (13 aria-busy tokens, new E2E test, 1x E2E 122/2/0, RED proof missing). A1 spec review dispatched: a4202533451a66739 (spec-A1.md). P-COPY resumed into ENV's slot. Running: U2 impl, P-COPY impl, A1 spec.
- 2026-10-07 16:40 UTC: ENV post-merge gates GREEN on the integrated branch with the pinned rustc 1.97.1: fmt, ssr clippy, wasm hydrate clippy, ssr tests 97/0. PRB-102 resolved. ENV worktree removed. ENV leftover minors go to FOLLOWUP-1: CI cargo-binstall action v1.18.1 vs script pin 1.25.1; trap set mid-script.
- 2026-10-07 17:20 UTC: U2 DONE_WITH_CONCERNS @bb69736 (SmsMode, boot refusals, config-centralised test mode; E2E run 1 had 2 failures, a hydration stall; run 2 green 121/2/0). Spec review dispatched: a955d1ac5f06c0385 (spec-U2.md), told to root-cause the run-1 failure. Running: P-COPY impl, A1 spec, U2 spec.
- 2026-10-07 ~18:30 UTC: the A1 spec reviewer, U2 spec reviewer and P-COPY implementer were killed by a session limit (reset 19:50); the worker restarted. 20:50: disk at 2.6G free; freed main target/ and tmp build dirs (now 8.8G). Loops restarted; all three resumed.
- 2026-10-07 21:30 UTC: P-COPY DONE @bbb9a13 + base merge 31b9460 (10 uk.json values; gates green; 1x E2E 121/2/0). The implementer ran `pkill -f cargo`, which may have killed other lanes' builds; the reviewers were told. Spec review dispatched: aeae408cb21d97114 (spec-P-COPY.md). Running: A1 spec, U2 spec, P-COPY spec.
- 2026-10-07 21:45 UTC: P-COPY spec PASS. Proven dependency: the 3 SMS bodies now end in ':' and expect the link that P-S adds, so P-COPY must NOT merge before P-S (merge together, or P-S first). Quality review dispatched: a78dfbe4a3dc32d73. Also: the dead regex alternatives at mail_club.spec.ts:660 go to F-COV.
- 2026-10-07 22:00 UTC: P-COPY quality Yes, gated: merge only together with or after P-S (SMS link; after both, `grep -c "let message = with_site_link" src/admin/sms.rs` = 4) AND P-H2 (home_reported_label points to the 'arrived after all' button). The narrowing of mail_club.spec.ts:660 goes to P-H1. Copy nits (duplicate 'створи щось своє', «призначення» at uk.json:135, the ~100 UAH price going stale) go to FOLLOWUP-1. P-S resumed (ac57401dbc39b3668); P-H2 is next.

## Units
| Unit | Plan | Problems (PRB) | State | Branch / worktree | Spec | Quality | Integrated |
|---|---|---|---|---|---|---|---|
| 2026-10-05 20:45 | U4 | 823905e | reviewer gates green in worktree; post-merge gates running |
| T0 view-type depth root fix + CI build/E2E guards | blockers | TBD | implementing | worktree-agent-a18ac82e5b9ac2a54 | - | - | - |
| T1 server panic=unwind | blockers | TBD | implementing | worktree-agent-a4d2369a7e8477b86 | - | - | - |
| ENV reproducible toolchain + SessionStart hook | blockers | TBD | implementing | worktree-agent-a9394f7e60a701d45 | - | - | - |
| U1 registration tickets (OTP bypass) | blockers | TBD | implementing | worktree-agent-aa77876d6f145827e | - | - | - |
| U2 test-mode confinement | blockers | TBD | implementing | worktree-agent-ae294272a7d5f6210 (base + U3) | - | - | - |
| U3 first-admin bootstrap | blockers | TBD | INTEGRATED | worktree-agent-a1fe18df1d59b4c14 @ 067b9bd | PASS | Yes | merged 2026-10-06 |
| U4 swap position exchange | blockers | TBD | INTEGRATED; minors fixed @781241f, quality Yes, merged 2026-10-06 | worktree-agent-a46c4c68440f8d984 @ 066ebf2 | PASS | Yes (4 minors) | 823905e |
| A1 attr:aria-busy leak | blockers | TBD | implementing | worktree-agent-aa08619b87efefc0d | - | - | - |
| P-COPY copy fixes | product | TBD | REVIEWED (spec PASS, quality Yes); merge ONLY with or after P-S and P-H2 | worktree-agent-ad7158d9abfc86219 | - | - | - |
| P-S SMS site link | product | TBD | implementing | worktree-agent-ac57401dbc39b3668 | - | - | - |
| P-V real E2E assertions | product | TBD | implementing | worktree-agent-ab7ca384dc0b558ce | - | - | - |
| P-H1 participation home states | product | TBD | implementing | worktree-agent-a17d3d793043b76c0 | - | - | - |
| P-R organizer roster | product | TBD | implementing | worktree-agent-a82424857366c0f92 | - | - | - |
| P-F forwarding requests | product | TBD | implementing | worktree-agent-a36ce1f5f8b4f3c4b (base + U4) | - | - | - |
| P-H2 delivery card persists | product | TBD | implementing | worktree-agent-a8f818e1aa0d43eaa | - | - | - |
| P-H3 branch editable | product | TBD | implementing | worktree-agent-adb44670e29cec0d1 | - | - | - |
| P-M1 meetup storage + form | product | TBD | implementing | worktree-agent-ab0d6b0268101da57 | - | - | - |
| P-M2 meetup + timeline on home | product | TBD | implementing | worktree-agent-a961a61a4015cfb42 | - | - | - |
| P-DOC badge doc sync | product | TBD | not launched | - | - | - | - |
| F-* (17 code lanes + DOC-OPUS + ORCH) | findings | TBD | planned, not launched | - | - | - | - |

## Integration log
| Time (UTC) | Unit | Merge commit | Gates |
|---|---|---|---|
