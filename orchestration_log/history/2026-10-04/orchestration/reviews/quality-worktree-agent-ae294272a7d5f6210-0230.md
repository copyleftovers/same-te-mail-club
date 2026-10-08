# Code Quality Review: worktree-agent-ae294272a7d5f6210

Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-ae294272a7d5f6210
Branch: worktree-agent-ae294272a7d5f6210
Diff range: 5df6509..46f10df (net U2: 8 files, +234/-68)
Reviewed at: 2026-10-08T02:30Z
Spec verdict (A3): PASS (round 2)
Bound: self-documenting-code, correct-by-construction, kiss (full primary texts from /tmp/claude-manifesto-repo/LLM_MANIFESTOS/manifestos/)

Gates (run by this reviewer in the worktree on the existing target/; no new target/ was created):
- `cargo fmt --check`: exit 0
- `SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr -- -D warnings`: exit 0. The only warning is the external proc-macro-error2 future-incompat note.
- `cargo test`: 79 passed
- `cargo test --features ssr`: 106 passed, 2 ignored, 0 failed. All 15 config::tests pass, 9 of them new.
- Grep: `std::env::var` outside config.rs gives only `SAMETE_LOG_POOL` (main.rs:80). `SAMETE_TEST_MODE|SAMETE_SMS_DRY_RUN` appear only in config.rs and main.rs. `sms_dry_run|turbosms_token|turbosms_sender|CSRF_SECRET` in src/ and README.md gives 0.

## Code Quality Review

### Summary
U2 replaces three loose Config fields and five ad-hoc env reads with one `SmsMode` sum type. Test mode now lives only inside `DryRun`, so live SMS combined with the fixed OTP cannot be represented. The two boot refusals are typed `ConfigError` variants and are checked before the DB connects. Plain functions take the flag as an explicit `bool`. The change is small and well tested. The remaining notes are minor.

### Strengths
- src/config.rs:10-17: `SmsMode { Live{token,sender} | DryRun{test_mode} }` makes "test mode with real SMS" unrepresentable by construction. The live token and sender exist only in the `Live` variant, so dry-run can no longer carry empty-string placeholder credentials (the old `unwrap_or_default()` branches are gone).
- src/config.rs:109-132: `sms_mode_from_vars` is a pure function over raw `Option` values. It is fully testable without touching process env and replaces about 25 lines of duplicated token/sender branching. The ordering is correct: dry-run returns early, and test-mode-without-dry-run is refused before any credential check (the `test_mode_without_dry_run_is_refused_even_with_credentials` test covers this).
- src/sms.rs:40-50, 72-79: `send_sms` destructures `SmsMode` with an exhaustive `match`, and `post_to_turbosms` narrows its input from `&Config` to `token: &str`, the one value it needs.
- src/auth.rs:81,127-132: `create_otp` / `check_otp_rate_limit` no longer read process env. The flag comes in as a parameter, which makes both functions deterministic and unit-testable. Call sites read clearly (`config.test_mode()`, login.rs:75,83).
- src/main.rs:24-31: the bind check and the test-mode WARN log run before `create_pool`. A misconfigured prod container therefore fails immediately without touching the DB, and the bind check uses the same `addr` the listener later binds (main.rs:94). The loopback check (`ip().is_loopback()`) correctly refuses both `0.0.0.0` and `[::]`.
- src/pages/home.rs:297,396,519: the flag is read once, as the first statement before any `.await`, and passed down by value. `resolve_preparation_state` does no context lookup (home.rs:173).
- README.md:191-198 and .env.example:2-5 match the code: credentials are required unless dry-run, both refusals are documented, the nonexistent `CSRF_SECRET` row is removed, and the `RUST_LOG` default matches main.rs:16. Dockerfile:22 (`0.0.0.0`), Cargo.toml:123 (`127.0.0.1:3000`), isolated-capture.sh:88 and the justfile/CI env are consistent with the loopback rule.

### Critical Issues (Must Fix)
None.

### Important Issues (Should Fix)
None.

### Minor Issues (Nice to Have)
1. **The hazard in the home.rs helper is documented, not prevented. This is acceptable, but the WARNING overstates how complete its coverage is.**
   - File: src/pages/home.rs:154-164
   - Issue: whether `test_mode()` runs before the first `.await` is enforced only by the WARNING comment, so a future caller can repeat the regression from round 1. Rust has no cheap way to type-forbid "called after an await". Folding the flag into `require_auth`'s return would braid auth with config. A documented helper plus the call-first convention is therefore the right KISS trade-off, and I do not ask for a structural change. Separately, outside this diff (no action required in U2): src/admin/sms.rs:163-164, 246-247, 335-336 and 433-434 call `use_context::<Config>()` after `require_admin().await`. That is the exact pattern this WARNING names, and it carries no warning. The round-2 spec review says admin/sms.rs reads context "before their first await", which contradicts the source. Those are ActionForm POST handlers, not SSR Resources, and their E2E tests pass, so the risk is lower. It is still an unexamined instance of the same hazard.
   - Impact: the convention lives only in a comment in one file.
   - Fix: none required for U2. Orchestrator: add the admin/sms.rs ordering to deferred_items, or hoist those four lookups above `require_admin().await` in a follow-up.
2. **The helper's name is shadowed by its own result binding.**
   - File: src/pages/home.rs:161 (fn `test_mode`), :297, :396, :519 (`let test_mode = test_mode()?;`)
   - Issue: the function and the local `bool` share a name, so a reader has to work out which `test_mode` each line means.
   - Impact: readability only.
   - Fix: rename to `test_mode_from_context()`.
3. **Boot refusals print `Debug`, not the `#[error]` text.**
   - File: src/main.rs:22,26 (`.expect("configuration error")`)
   - Issue: `Result::expect` formats the error with `{:?}`. The operator sees `configuration error: TestModeRequiresDryRun` / `TestModeRequiresLoopback(0.0.0.0:3000)`, not the Display messages that name the env vars (`SAMETE_TEST_MODE=true requires SAMETE_SMS_DRY_RUN=true`). The variant names are still understandable. The new line at :26 copies the existing pattern at :22.
   - Impact: the carefully written messages in config.rs:39-42 never reach the operator.
   - Fix: `.unwrap_or_else(|e| panic!("configuration error: {e}"))` at both sites.
4. **Two `sms_mode_from_vars` error branches are untested.**
   - File: src/config.rs:124-130 vs tests :214-283
   - Issue: `EmptyTurbosmsToken` (empty token) and `MissingTurbosmsSender` (absent sender) have no test. These branches were moved, not newly written, but the function under test is new.
   - Impact: a reordering regression in these checks would go unnoticed.
   - Fix: add `live_rejects_empty_token` and `live_requires_sender` cases next to the existing ones.

### Assessment
**Ready to merge:** Yes
**Reasoning:** The type change removes the dangerous configuration by construction, both refusals are typed, tested and executed before the DB connects, no ad-hoc env reads remain, and all gates pass. The four minor items are readability, diagnostics and test-gap improvements that can be deferred.
