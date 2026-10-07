# Code Quality Review: worktree-agent-aa77876d6f145827e

Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-aa77876d6f145827e
Branch: worktree-agent-aa77876d6f145827e
Diff range: 57f0109..f89d773 (cace3f2 fix(auth), 4bccd33 style fmt, f89d773 fix(auth) atomic claim). The re-review delta is 4bccd33..f89d773.
Reviewed at: 2026-10-07 (round 2, re-review of f89d773)
Spec verdict (A3): PASS (spec-worktree-agent-aa77876d6f145827e-2330.md)
Bound: self-documenting-code, correct-by-construction, kiss (decomplect)

## Round history
- **Round 1 (HEAD 4bccd33): With fixes.**
  - Important #1: the throttle was a non-atomic check-then-increment.
  - Minors 1–4: DB errors were reported as "not verified"; a DB error was counted as a guess; a phone return value was discarded; a string literal was used for the status compare.
- **Round 2 (HEAD f89d773): this report.** All round-1 findings are resolved, and they were verified, not taken on the implementer's word.

## Gates (round 2, foreground, HEAD f89d773; the recursion limit was set locally only and nothing was committed)

Env: `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3 SQLX_OFFLINE=true RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zcrate-attr=recursion_limit="256"`

| Gate | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --no-default-features --features ssr -- -D warnings` | exit 0, 0 errors |
| `cargo test` | exit 0, 68 passed |
| `cargo test --features ssr` | exit 0, 80 passed |
| `.sqlx` | The stale `SELECT phone` and `record_failed_invite_attempt` query files are removed. Claim and refund have new files, and consume was regenerated. SQLX_OFFLINE clippy compiling is the proof that they are current. |
| E2E | Not re-run by this reviewer. The implementer reports 3x 121 passed / 2 skipped / 0 failed, and the new burst test was red before the fix (11 > 4) and green after. |

### Concurrency probe (reviewer's own, on a throwaway sibling DB `samete_u1q`, dropped afterwards)
The ticket table came from the U1 migration, and the probe used the exact claim and refund SQL from `src/auth.rs`.
- 40 concurrent claims on one fresh ticket: exactly **5** succeeded and the final count was **5**. The cap holds under a burst.
- Refund race: 3 claims were left in flight (valid-code submissions). Then 30 concurrent wrong-guess claims ran at the same time as their 3 refunds.
  - Wrong-guess claims kept: **5**. Final counter: **5**. Wrong guesses never exceeded the cap.
- Reasoning for why this holds in general:
  - Every refund is paired 1:1 with its own request's successful claim, and wrong guesses never refund.
  - So at all times `counter = kept_wrong_guesses + in_flight_non_guess_claims ≤ 5`, which means `kept_wrong_guesses ≤ 5`.
  - A refund cannot take back a claim it did not make. The `invite_attempts > 0` guard stops underflow.
  - Under READ COMMITTED, a claim UPDATE that was blocked on the row lock re-checks `invite_attempts < 5` against the newly committed row (EvalPlanQual). That re-check is what closes the burst window.
- **No refund-above-cap race was found.**

Housekeeping:
- The `target/debug` this review created was deleted again. The other `target/` subdirs existed before.
- The tree is clean at f89d773.

## Code Quality Review

### Summary
U1 swaps the client-trusted `pending_phone=<phone>` cookie for a hashed, single-use server-side registration ticket, and caps invite-code guessing at 5 per OTP verification.
- Round 2 replaces the check-then-increment with an atomic claim-before-lookup and a refund for submissions that are not wrong guesses. Probing confirms the cap holds under concurrency.
- Round 2 also pulls the registration transaction into `redeem_invite_code`, which returns an explicit `RedemptionFailure` outcome enum.

### Strengths
- **The atomic claim is the single place the cap is enforced.** `claim_invite_attempt` (src/auth.rs, one `UPDATE … WHERE invite_attempts < $2`, success decided by `rows_affected() == 1`). `consume_registration_ticket` no longer re-checks the cap, and its doc says so.
- **The refund semantics are exact.** Valid, empty, and DB-failed lookups refund, so only genuine wrong guesses keep their attempt. A code accepted on the 5th try can still register: the counter is 4 after the refund, so the register claim succeeds.
- **The failure classes are typed.** `RedemptionFailure::{TicketGone, WrongCode, Internal}` replaces the old `let Ok(..) else` ladder. `register_with_code` matches on it, and only `Internal` refunds. The status compare uses `InviteCodeStatus::Unused` (round-1 Minor #4).
- **DB errors are no longer disguised.** `validate_invite_code` maps `AppError::Unauthorized` to "not verified" and sends every other error through `into_server_fn_error` (round-1 Minor #1). A DB error on the lookup refunds and surfaces as a server error instead of costing a guess (round-1 Minor #2).
- **The dead return value is gone.** `registration_ticket_phone` became `claim_invite_attempt -> Result<(), AppError>` (round-1 Minor #3).
- **The comments are WHY comments.** The claim-then-refund rationale explains a race the code alone cannot show.
- **The regression test is behavioural.** The burst E2E fires 12 concurrent POSTs and asserts the oracle answered at most 4 times. The implementer reports it red before the fix.

### Critical Issues (Must Fix)
None.

### Important Issues (Should Fix)
None.

### Minor Issues (Nice to Have)
1. **`redeem_invite_code` throws away the cause of `Internal` failures.**
   - File: src/pages/login.rs (`redeem_invite_code`, each `.map_err(|_| RedemptionFailure::Internal)` on begin, consume, lookup, insert, update and commit)
   - Issue: The sqlx error is dropped and never logged. `register_with_code` then refunds and redirects silently.
   - Impact: A production registration failure, such as a unique violation on a racing second ticket for the same phone or a pool outage, leaves no trace.
   - Fix: Make it `Internal(sqlx::Error)` or `AppError`, and `tracing::warn!` it in the `Internal` arm of `register_with_code`.
2. **The burst-probe POM depends on UI copy in the response body.**
   - File: end2end/tests/fixtures/mail_club_page.ts (`burstInviteCodeGuesses`, `b.includes("Недійсний код")`)
   - Issue: It counts oracle answers by matching localized copy. This is an API-level probe, so the testid rule does not apply directly, but a copy change would silently turn the count into 0 and the `≤ 4` assertion would then pass vacuously.
   - Impact: The regression guard can stop guarding without any visible failure.
   - Fix: Also assert a lower bound (for example `answeredAsInvalid >= 1`), or count the "not verified" refusals and assert that the two counts sum to `count`.
3. **The cap constant is duplicated in the spec.**
   - File: end2end/tests/mail_club.spec.ts (local `const MAX_INVITE_ATTEMPTS = 5`)
   - Issue: It mirrors `auth::MAX_INVITE_ATTEMPTS` by hand.
   - Impact: The test drifts if the cap changes. This is acceptable across the language boundary.
   - Fix: Optional. Move it to `capture-constants.ts`-style shared E2E constants with a pointer to `src/auth.rs`.

### Assessment
**Ready to merge:** Yes
**Reasoning:**
- The forged-cookie hole is closed by construction.
- The invite throttle is now atomic. The reviewer's concurrency probe confirms that 40 parallel claims yield exactly 5, and that refunds cannot lift kept wrong guesses above the cap.
- fmt, SSR clippy with `-D warnings`, and both test suites are green.
- The remaining items are Minor and can be deferred.
