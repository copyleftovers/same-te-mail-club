# Code Quality Review: worktree-agent-aa77876d6f145827e

Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-aa77876d6f145827e
Branch: worktree-agent-aa77876d6f145827e
Diff range: 57f0109..4bccd33 (cace3f2 fix(auth), 4bccd33 style fmt)
Reviewed at: 2026-10-07T04:11:00Z
Spec verdict (A3): PASS (spec-worktree-agent-aa77876d6f145827e-2330.md)
Bound: self-documenting-code, correct-by-construction, kiss (decomplect)

## Gates (re-run in the foreground after the container restart; recursion limit set locally only, nothing committed)

Env: `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3 SQLX_OFFLINE=true RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zcrate-attr=recursion_limit="256"`

| Gate | Result |
|---|---|
| `cargo fmt --check` | exit 0 |
| `cargo clippy --no-default-features --features ssr -- -D warnings` | exit 0. The only warning is the proc-macro-error2 future-incompat note from a dependency. |
| `cargo test` | exit 0, 68 passed |
| `cargo test --features ssr` | exit 0, 80 passed |
| wasm clippy | Not run. It is ENV scope (rustc 1.99 `#[server]` lint) per the dispatch. |

Housekeeping:
- The `target/debug` this review created was deleted. The other `target/` subdirs existed before and were left alone.
- The tree is clean and HEAD is 4bccd33. No edits and no commits.

## Code Quality Review

### Summary
U1 replaces the client-trusted `pending_phone=<phone>` cookie with a server-side, hashed, single-use registration ticket, and adds a 5-attempt invite-code throttle per ticket.
- The authority model is now correct by construction. `register_with_code` can only learn the phone from `consume_registration_ticket` inside the registering transaction.
- The change also removes the old silent `let _ =` on the code-mark UPDATE and the commit.
- One weakness remains. The throttle is a check-then-increment that is not atomic, so concurrent requests on one ticket can bypass it.

### Strengths
- **The ticket is the single source of truth for the phone.** The phone comes from `auth::consume_registration_ticket(&mut tx, &ticket)` (src/pages/login.rs:405). The `phone_mod::normalize` call on client input was deleted, so a forged cookie cannot be turned into a phone.
- **The consume is transactional, so rollback restores the ticket.** It is a DELETE…RETURNING inside the registration tx (src/auth.rs:316-334), and any early return drops the tx. The WHY comment at login.rs:403-404 is the right kind of comment.
- **Silent error swallowing was removed.** `let _ = …execute` and `let _ = tx.commit()` became `let Ok(..) else { redirect }` (login.rs:437-452). Before this, a failed commit still went on to issue a session for a user who did not exist.
- **Only a hash of the token is stored.** `generate_token` was extracted and reused by `create_session` (auth.rs:52-58, :241), and only `sha256_hex` of it reaches the DB.
- **`extract_cookie` generalises two copy-pasted prefix parsers.** It matches the exact name through `split_once('=')`, which fixes a latent prefix-collision class (for example `xsession=`). Unit tests cover the multi-cookie, longer-name and no-header cases (auth.rs tests).
- **The throttle limit is a named constant.** `MAX_INVITE_ATTEMPTS` is documented and used in both SQL predicates, with no magic 5 inline.
- **The forge E2E test uses a REAL unused code (`CODES.FORGE`).** It proves that a valid code plus a forged cookie still fails, and it cleans up afterwards by revoking the code.
- **Dead code was deleted.** `check_pending_registration` (and its `#[allow]`) and `extract_pending_phone_cookie` are gone.

### Critical Issues (Must Fix)
None.

### Important Issues (Should Fix)
1. **The invite throttle is not atomic: concurrent requests on one ticket bypass the 5-attempt cap.**
   - File:
     - src/pages/login.rs:326-328: the liveness check `registration_ticket_phone`, which reads `invite_attempts < 5`.
     - src/pages/login.rs:352: `record_failed_invite_attempt`, a separate unconditional `+1` after the code lookup.
     - src/auth.rs:298-310 and :344-353.
   - Issue: The read and the increment are two separate statements with the code lookup in between.
     - N concurrent `validate_invite_code` POSTs carrying the same ticket all pass the `< 5` read before any increment lands.
     - Each one then gets a full code-existence oracle. The cap only applies to sequential requests.
   - Impact: The unit exists to close an unthrottled guess oracle over a ~39,800-code space (PLAN U1 row, line 14). A single OTP-verified ticket plus a burst of a few hundred parallel requests gets back to near-unthrottled guessing. The E2E throttle test is sequential, so it cannot detect this.
   - Fix: Reserve the attempt atomically before the oracle runs. Make `registration_ticket_phone` a single `UPDATE registration_tickets SET invite_attempts = invite_attempts + 1 WHERE token_hash=$1 AND expires_at>now() AND invite_attempts < $2 RETURNING phone`.
     - The row lock serialises concurrent requests, and the predicate refuses the sixth.
     - This counts every validation, successful ones included. So either allow `<= $2` in `consume_registration_ticket`, or decrement on the `Unused` branch, so that a code accepted on the 5th try can still register.
     - Add a unit- or E2E-level concurrent probe, for example `Promise.all` of 10 submits, then assert that `invite_attempts` stays capped and later valid codes are refused.

### Minor Issues (Nice to Have)
1. **DB failures are reported as "phone not verified" in `validate_invite_code`.**
   - File: src/pages/login.rs:326-328 (`.map_err(|_| not_verified())`)
   - Issue: `AppError::Database` and `AppError::Unauthorized` collapse into the same user message.
   - Impact: An outage tells the user to restart verification, and the error is never logged.
   - Fix: Match `AppError::Unauthorized` to `not_verified()`, and route everything else through `AppError::into_server_fn_error`, which is already used at :354.
2. **`register_with_code` counts a DB error on the code lookup as a failed guess.**
   - File: src/pages/login.rs:416 (`code_row.map(|row| row.filter(..))` collapses `Err` and `Ok(None)`/non-unused into one branch)
   - Issue: A transient DB error burns one of the user's 5 attempts.
   - Impact: Low. A legitimate user loses an attempt during an outage.
   - Fix: Split the `Err` arm. Redirect without recording the attempt and log it at `tracing::warn!`.
3. **`registration_ticket_phone` returns a phone that its only caller discards.**
   - File: src/pages/login.rs:326
   - Issue: The name and return type promise "phone lookup", but the call site uses it purely as a liveness predicate.
   - Impact: A reader has to work out why the value is dropped. The atomic-reservation fix in Important #1 resolves this naturally (rename it to something like `reserve_invite_attempt`).
   - Fix: Fold this into the fix for Important #1.
4. **The status comparison uses a string literal.**
   - File: src/pages/login.rs:416 (`r.status == "unused"`)
   - Issue: This is carried over from the base `AS "status: String"`. The `InviteCodeStatus` enum already exists and is used in `validate_invite_code`.
   - Impact: Cosmetic and pre-existing in shape. The restructuring touched this line.
   - Fix: On the next touch, select `status AS "status: InviteCodeStatus"` and compare against the variant.

### Assessment
**Ready to merge:** With fixes
**Reasoning:**
- The forge vulnerability, which was the primary blocker, is closed by construction, and all gates are green (fmt, SSR clippy -D warnings, 68 and 80 tests).
- The second half of U1's purpose, throttling invite-code guessing, can be bypassed by concurrent requests because check and increment are not atomic. That one SQL-level change (Important #1) should land before merge.
- The Minor items can be deferred.
