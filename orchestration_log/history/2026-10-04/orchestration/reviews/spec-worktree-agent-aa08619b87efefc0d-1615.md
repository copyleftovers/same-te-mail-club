# Spec Review: worktree-agent-aa08619b87efefc0d

Verdict: PASS
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-aa08619b87efefc0d
Branch: worktree-agent-aa08619b87efefc0d
HEAD SHA: b8b8a34
Reviewed at: 2026-10-07T16:50Z
Diff scope: aa7b45f..b8b8a34 (A1 commits 9a0d8e0 test, b8b8a34 fix; 0a28c3e = integration merge)
Files reviewed:
- src/admin/page.rs
- end2end/tests/fixtures/mail_club_page.ts
- end2end/tests/mail_club.spec.ts
- PLAN-blockers.md § A1 (main repo, orchestration_log/history/2026-10-04/plans/)

## Findings

PASS. The implementation matches the spec. I checked all 6 A1 requirements in code and against the built artifacts.

1. All 13 sites changed from `attr:aria-busy` to bare `aria-busy`, one token per line, with the closures untouched. Every site in the plan table is covered by signal name: create-season, launch, advance, cancel-confirm, 4 SMS, generate, swap (SwapFormSection), generate-code (InviteCodesSection), revoke (InviteCodeCard, after T0's move), and deactivate (ParticipantRow). No extra sites were found.
2. Source gates: `grep -rn "attr:" src --include=*.rs | wc -l` = 0. That includes InviteCodeCard (page.rs:1919), so T0 is present and no recursion_limit is needed. `grep -rn "aria-busy=move" src/admin/page.rs | wc -l` = 13.
3. Compiled-artifact gate, built by me with `cargo leptos build --release` in a scratch clone:
   - Pre-fix, at 0a28c3e (the fix commit's direct parent = base aa7b45f + test commit 9a0d8e0): build=0. `attr:aria-busy` count: samete 1, samete.wasm 1. That is the RED proof.
   - Fix, at b8b8a34: build=0. `attr:aria-busy` count: samete 0, samete.wasm 0. `aria-busy` count: samete 1, samete.wasm 1. That is GREEN. Both artifacts were relinked fresh.
   - I built 0a28c3e rather than bare 9a0d8e0 because 9a0d8e0 predates T0 and would need the forbidden recursion_limit flags. 0a28c3e is the exact pre-edit source of the fix commit.
4. The POM method `expectGenerateCodeBusyWhilePending` matches the plan verbatim, except for one inserted `await this.selectDistributor();`. The new spec test `"1.5 — generate button exposes aria-busy while pending"` is placed directly after `"1.6 — admin revokes an unused code"` (mail_club.spec.ts:136).
5. E2E, one isolated run (`isolated-capture.sh e2e_a1rev full` at b8b8a34): exit=0, 122 passed, 2 skipped, no failed or flaky tests. The new title passed (2.1s). A1's diff adds exactly one test, so the count above 121 comes from the base, not from A1.
6. Standard gates at b8b8a34:
   - fmt --check: clean.
   - SSR clippy -D warnings: exit 0. The only `warning` line is cargo's future-incompat note for the dependency proc-macro-error2, not from project code.
   - `cargo test`: 79 passed, 0 failed.
   - `cargo test --features ssr`: 97 passed, 0 failed, 2 ignored.
   - wasm clippy: out of scope per the dispatch. The branch predates the 1.97.1 pin.

Extra (justified, not a defect):
- mail_club_page.ts: the private `selectDistributor` helper was not in the plan. It is necessary: `distributor-select` is `required=true` (page.rs:1732), so without a selection HTML5 validation would block the plan's literal click, and no POST would ever be held. The helper is extracted from the identical existing logic in `generateInviteCode`, which now calls it, so there is no behavior change and no duplication. Removing it would break the test. It is not scope creep.

Banned-pattern scan of the diff: no `waitForTimeout`, `networkidle`, `force`, `page.evaluate`, getByText, getByRole, or CSS selectors. The single non-retrying `getAttribute` read is the one the plan sanctions. No `attr:` remains in the code. Write-set = exactly the 3 planned files.

Not independently verified: 3-run E2E stability. I ran it once and it was green, and the implementer also reported one green run. Per the Definition of Done, the orchestrator or integrator owns the remaining two runs.

## Reasoning

I read the A1 section of the plan and compared each of the 13 table rows to the merge-base diff hunks by signal name. Each hunk changes only the attribute token. I re-ran the source grep gates in the worktree rather than trusting the report. The reported RED-proof gap was real: the implementer produced no pre-edit artifact grep. I closed it by building the fix's parent and the fix in a scratch clone and grepping both artifacts. That showed the literal string disappears from both the server binary and the wasm, while bare `aria-busy` survives.

For the one deviation, I checked whether the plan's verbatim POM could work. The select's `required=true` means it could not. The helper is a minimal, behavior-preserving extraction that both call sites now share. That fits build-for-today: it is the smallest change that makes the specified test functional.

All gate logs are in the session scratchpad (a1-pre.log, a1-post.log, a1-e2e.log, a1-clippy.log, a1-test.log, a1-testssr.log). The scratch clone's target/ was deleted and the scratch worktree removed. The reviewed worktree was not edited.
