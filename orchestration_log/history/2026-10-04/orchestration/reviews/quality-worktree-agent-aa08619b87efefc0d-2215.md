# Code Quality Review: worktree-agent-aa08619b87efefc0d

Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-aa08619b87efefc0d
Branch: worktree-agent-aa08619b87efefc0d
Diff range: aa7b45f..b8b8a34 (A1 commits: 9a0d8e0 test, b8b8a34 fix; 0a28c3e = base merge, not reviewed)
Reviewed at: 2026-10-07T21:23Z
Spec verdict (A3): PASS, so the full review ran.
Bound to: self-documenting-code, correct-by-construction, kiss (decomplect).

Gates I ran myself in the worktree:
- `cargo fmt --check`: exit 0.
- `SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr -- -D warnings`: exit 0. The only warning is cargo's future-incompat note for the dependency proc-macro-error2.
- `cargo test`: 79 passed.
- `cargo test --features ssr`: 97 passed, 2 ignored.
- Source gate `grep -rn "attr:" src --include=*.rs`: 0 matches.

The worktree's `target/` existed before this review. I only built into it incrementally and did not create it, so I left it in place for the orchestrator to delete.

## Code Quality Review

### Summary
The fix is minimal: exactly one token changes on each of 13 lines (`attr:aria-busy` → `aria-busy`), with every closure untouched. The test adds one deterministic E2E check that holds the `generate_invite_code` response with `page.route`, plus a behavior-preserving extraction of the distributor-selection logic. Quality is good. The one structural gap is that nothing automated stops `attr:` from coming back on native elements, which would be its fourth occurrence. That guard sits outside A1's planned write-set, so I record it as a follow-up and it does not block this merge.

### Strengths
- **Smallest possible fix.** In `src/admin/page.rs` (497, 675, 705, …), the diff has exactly 13 `-attr:aria-busy` and 13 `+aria-busy` lines with identical closures. There is no collateral change.
- **Deterministic pending state, no timing guesses.** In `end2end/tests/fixtures/mail_club_page.ts:372-380`, the route handler is registered before the click and awaits a promise that is only released after both assertions. The busy state therefore cannot end mid-assertion. There is no `waitForTimeout` and no `networkidle`.
- **Correct glob.** The glob `**/*generate_invite_code*` matches the snake_case server-fn URL segment, the same convention existing URL hints rely on (`request_otp`, `verify_otp_code`, `complete_onboarding`). It does not match the `list_invite_codes` refetch. `route.continue()` sends to the network and does not collide with the GET-only static-asset handler in `cached-context.ts:54`.
- **The non-retrying read is safe.** `getAttribute("attr:aria-busy")` (mail_club_page.ts:380) runs while the response is held, so the DOM is frozen. It is the plan's single sanctioned exception, because Playwright has no web-first assertion that an attribute is absent by name.
- **Both directions asserted.** The test checks `aria-busy="true"` while pending and its removal after resolution (`not.toHaveAttribute`, line 383). That proves the reactive closure, not just the attribute name.
- **Clean POM extraction.** `selectDistributor` (mail_club_page.ts:347) moves identical logic out of `generateInviteCode` (line 249 now calls it), so there is no duplicated selection logic. The extraction is needed because `distributor-select` is `required=true` (page.rs:1732): without a selection, HTML5 validation would block the submit and no POST would ever be held. The method is private, named after what it does, and its early `return` flattens the old nested if/else.
- **Selectors are testids only** (`generate-code-button`, `distributor-select`, `generated-code-display`). The test title traces to Story 1.5.

### Critical Issues (Must Fix)
None.

### Important Issues (Should Fix)
1. **No automated guard against `attr:` returning on native elements (follow-up, outside A1's write-set)**
   - File: CI / `justfile` (no gate exists). `grep -rn "attr:" .github justfile scripts` gives 0 matches.
   - Issue: The one runtime check covers 1 of 13 sites (`generate-code-button`). The compiled-artifact grep was a one-time manual gate. If any of the other 12 sites, or a new button, regresses to `attr:`, it compiles cleanly, passes clippy, and passes E2E.
   - Impact: This defect class keeps coming back: `attr:aria-invalid` in 2026-07-04 (it broke every error border), `attr:aria-current` in 2026-07-12, and `attr:aria-busy` now. Leptos silently emits a literal attribute named `attr:...`, so nothing fails loudly. Correct-by-construction requires the check to be mechanical, not something people have to remember.
   - Fix: Add a gate to the `check` CI job (and `just check`): `! grep -rnE '^\s*attr:' src --include=*.rs`. Today there are zero legitimate `attr:` uses in `src/`, so a blanket gate is exact. If `attr:` is ever needed for attribute pass-through on a component, switch to an allowlist. The plan (§ Scope, "No changes to: … CI workflow") keeps CI out of this batch, so record this in `deferred_items.md` or schedule it as its own unit. It does not block A1.

### Minor Issues (Nice to Have)
1. **If the route ever stops matching, the test silently becomes a race**
   - File: end2end/tests/fixtures/mail_club_page.ts:374-379
   - Issue: If the server fn is renamed or prefixed, the glob stops matching. The request then goes through unheld, and `toHaveAttribute("aria-busy","true")` may still pass by catching the short real pending window, which makes the test flaky rather than failing.
   - Fix: Record that the handler ran, e.g. `let intercepted = false;` set inside the handler and `expect(intercepted).toBe(true)` before `release()`. Alternatively, start a `page.waitForRequest("**/*generate_invite_code*")` promise before the click and await it before the busy assertion.
2. **`selectDistributor` silently does nothing when there are no distributor options**
   - File: end2end/tests/fixtures/mail_club_page.ts:353-359
   - Issue: With no non-placeholder options, the method returns without selecting anything. The click is then blocked by `required` and the failure shows up as an aria-busy timeout, which points the debugger away from the real cause. This behavior predates A1 and was carried over unchanged, but the extraction now gives it a second caller.
   - Fix: Assert that an option exists, e.g. `await expect(select.locator("option").nth(1)).toBeAttached()`, instead of the `if (options.length > 1)` no-op.
3. **The route is not cleaned up on the failure path**
   - File: end2end/tests/fixtures/mail_club_page.ts:384
   - Issue: `unroute` and `release()` only run on the success path. This has no practical impact because each test gets a fresh page, so the handler dies with it. Noted only for completeness; no change needed unless the POM ever reuses pages.

### Assessment
**Ready to merge:** Yes
**Reasoning:** The fix is exact (13 single-token changes, source gate 0, all standard gates green) and the new E2E test makes the pending state deterministic in line with end2end/README.md. The recurrence guard is a real gap but is outside A1's sanctioned write-set, so it should be a follow-up rather than block this merge.
