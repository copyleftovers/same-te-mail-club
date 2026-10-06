# Spec Review: worktree-agent-a18ac82e5b9ac2a54

Verdict: FAIL
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a18ac82e5b9ac2a54
Branch: worktree-agent-a18ac82e5b9ac2a54
HEAD SHA: 4d000e7c27d57c9dc254040dfa00056893f8dc6a (from .git/refs/heads/worktree-agent-a18ac82e5b9ac2a54)
Reviewed at: 2026-10-06 (UTC; no shell available, so no clock time)
Files reviewed:
- <worktree>/src/pages/login.rs (lines 560-1193)
- <worktree>/src/admin/page.rs (lines 1655-2298, plus an `.into_any()`/component inventory of the whole file)
- <worktree>/scripts/assert-playwright-ran.sh
- <worktree>/scripts/isolated-capture.sh (build block)
- <worktree>/justfile (E2E recipes)
- <worktree>/.github/workflows/ci.yml (Check job)
- Base proxy: /home/user/same-te-mail-club/src/pages/login.rs and src/admin/page.rs (the main tree; see Reasoning)
- Spec: /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-blockers.md § T0 + common DoD

Binding: yagni + first-principles were loaded from /tmp/claude-manifesto-repo/LLM_MANIFESTOS/manifestos/ before review.

Tooling limit: this reviewer had only Read, Grep, Glob and Write. There was no shell, so I could not run `git diff 57f0109..4d000e7`, the builds, the headroom probe, the HTML-identity gate, E2E or the sabotage gates. The commit list comes from .git/worktrees/agent-a18ac82e5b9ac2a54/logs/HEAD: f01a419 (am: ci), 144de77, 67b139f, 4d000e7.

## Findings

FAIL -- Issues found:

Missing:
- T0.1 item 3, "`LoginPage`: append `.into_any()` to its `view!{…}`". Not implemented. <worktree>/src/pages/login.rs:647-678: the `view!` ends at `}` on line 678 with no `.into_any()`.
- DoD 2 / T0 sabotage gates (a)(b)(c) plus the revert-and-prove-clean step. The implementer says they were not run. I could not run them either (no shell). Required evidence is absent, so the guards (CI codegen steps, assert-playwright-ran.sh, the isolated-capture artifact check) have not been shown to go RED on the real defect.
- DoD 5, "E2E gate green 3 consecutive times". The implementer reports one run (116/2/0).
- No evidence for the HTML-identity gate (`html-identical`) or the headroom-96 gate. The only hint is the commit subject of 4d000e7 ("so builds pass at limit 96"). The report notes given to me do not include either output.

Partial:
- T0.1 item 1, "move each of the four step `<div …>` blocks … into four private `#[component]`s: `PhoneStep`, `OtpStep`, `InviteCodeStep`, `NameStep`". Only `PhoneStep` (login.rs:845-915) and `OtpStep` (login.rs:920-1016) exist. Steps 3 and 4 are still inline `<div data-testid="invite-code-step">` / `<div data-testid="name-collection-step">` blocks inside `LoginStepRouter`'s `view!` (login.rs:804-835). They are only erased one level down through `InviteCodeForm`/`NameCollectionForm` `.into_any()`. The plan lists the four components explicitly; the extension rule adds seams and does not allow dropping listed ones.

Misinterpreted:
- T0.5 CI comment. The spec text is "clippy/test never run codegen, so layout-depth overflows in Leptos views only surface in a real build." What was built, at <worktree>/.github/workflows/ci.yml:46-47: `# clippy/test never run codegen, so recursion/layout overflows … (see src/main.rs recursion_limit WHY).` That WHY does not exist: grep finds no `recursion_limit` in src/. The line is left over from the superseded PLAN-W1 design (raise the limit), carried in by commit f01a419 "am: ci…". It sends readers to a non-existent attribute and contradicts the owner override (root fix, no recursion_limit anywhere).
- T0.1 item 4, `InviteCodeCard(code, revoke_invite_action, hydrated)`, "Pass `revoke_pending` / `i18n` by recomputing them inside the component". Built instead with a 4th prop `revoke_pending: Memo<bool>` (page.rs:2003-2008). Behaviour is the same (same Memo), but the signature differs from the spec. Minor.
- T0.1 item 1, "Pass `resend` as a prop typed `impl Fn() + Send + Sync + 'static`". Built as generic `F: Fn() + 'static`, named `on_resend` (login.rs:921-933). The cfg-dual bindings correctly stay outside `view!`. Minor.

Extra:
- <worktree>/src/admin/page.rs:1900-1958 `InviteCodeStatusCell` and :1960-1997 `InviteCodeRedeemer` are new `#[component]`s. The extension rule says "extracted into a `#[component]` only when it is a `<For>` child or a match/if branch". The status `<span>` wrapper and the redeemer `<dl>` are neither; they are sub-trees of the `<For>` child. Under the rule's "no discretion" wording they should have been `.into_any()` erasures, not new components. `ParticipantRow` (page.rs:2123-2228, the participants `<For>` child) and the section-level `.into_any()` additions (render_sms_report_inline :1045, render_cycle_visualization :1164, SwapFormSection :1656, InviteCodesSection :1897, ParticipantListSection :2295) do fit the extension rule, provided the report lists each as `file:line → erasure` (not visible to me).
- <worktree>/src/pages/login.rs:842-844 WHY comment cites "see conventions: view-type depth". No such entry exists in orchestration_log/reference/ (grep: 0 matches). Dangling reference.

Verified compliant:
- No `recursion_limit` / `crate-attr` in src/, Cargo.toml, justfile, scripts or ci.yml code (the ci.yml comment is the only textual hit, flagged above).
- PhoneStep and OtpStep bodies match the base Step 1/2 markup token-for-token (attributes, classes, testids, closures, comments). Steps 3/4 are unchanged.
- InviteCodeCard + sub-components and ParticipantRow reproduce the base `<li>`/`<tr>` markup with the same elements, attributes, testids and order. `attr:aria-busy` is retained at :1751/:2081/:2190 (A1 owns it, correctly untouched).
- T0.3 `scripts/assert-playwright-ran.sh` matches the spec byte-for-byte.
- T0.4 justfile: 4× `touch target/e2e-start.marker`, 4× `cargo leptos end-to-end`, 4× assert line, plus the header comment above `e2e: e2e-release`.
- T0.5 ci.yml: both codegen build steps follow the hydrate clippy step; `just e2e-release` appears once; no duplicate guard in ci.yml.
- T0.6 isolated-capture.sh: `rm -f` + build + artifact check, exactly as specified.

## Reasoning

I read the T0 section and the common DoD of PLAN-blockers.md in full and applied the owner override (root fix only, no recursion_limit, seams at component/.into_any()). I then checked each What-You-Must-Do item against the worktree code. Without a shell I could not produce the git diff. As the "before" image I used the main checkout's src/pages/login.rs and src/admin/page.rs. Main's reflog shows it at the 57f0109 lineage plus only docs commits and the U3/U4 merges, which do not touch login.rs or the invite/participant regions of page.rs. That is a read outside the worktree, made solely to stand in for the unavailable `git show 57f0109:<file>`. Under that comparison the moved markup is DOM-identical, which is the core safety property.

The FAIL rests on spec-completeness, not on the approach. Two explicitly enumerated items are not done (LoginPage erasure; InviteCodeStep/NameStep extraction). A CI comment hard-codes the superseded recursion_limit design. The plan's mandatory evidence, the sabotage gates, is absent by the implementer's own admission. I did not accept "builds pass at 96" as a substitute for the listed seams: the plan says "no discretion", and the extension rule only adds seams. Equally, I did not demand seams beyond what the plan lists (yagni). The two extra sub-components are flagged because the extension rule forbids component extraction outside `<For>` children and match/if branches.

To pass: add `.into_any()` to LoginPage; extract InviteCodeStep and NameStep; fix the ci.yml:46-47 comment to the spec wording; fold InviteCodeStatusCell/InviteCodeRedeemer back into InviteCodeCard as `.into_any()` erasures, or justify them in the report as forced by a named overflow (`file:line`) under the extension rule. Then run and paste the headroom-96, HTML-identity, sabotage (a)(b)(c) + revert-clean, and 3× E2E outputs. The wasm-clippy-on-1.99 failure the implementer reported belongs to ENV, as claimed. I could not verify that claim.
