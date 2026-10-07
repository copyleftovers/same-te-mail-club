# Spec Review: worktree-agent-aa77876d6f145827e

Verdict: PASS
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-aa77876d6f145827e
Branch: worktree-agent-aa77876d6f145827e
HEAD SHA: 4bccd33f53c0ec95b72cd1f13ac53030140cf453
Diff range: 57f0109..4bccd33 (cace3f2 fix(auth) + 4bccd33 style fmt)
Reviewed at: 2026-10-07T00:08:41Z
Spec: /home/user/same-te-mail-club/orchestration_log/history/2026-10-04/plans/PLAN-blockers.md § U1 (lines 727-1170)
Files reviewed:
- migrations/20261004000001_registration_tickets.sql
- src/auth.rs
- src/pages/login.rs
- end2end/tests/fixtures/mail_club_page.ts
- end2end/tests/mail_club.spec.ts
- .sqlx/ (5 new query files)

Re-rooting: none. The verdict path is in main recon, which is exempt per _spec-reviewer.md.

## Findings

PASS. The implementation is spec-compliant. All 5 U1 subsections (U1.1–U1.5) and every U1 verification gate were checked in code and by running them.

U1.1 migration: the file name and content match the spec byte for byte. It applied cleanly to a fresh sibling DB.

U1.2 auth.rs:
- `generate_token` and `extract_cookie` exist (auth.rs:52-67). `extract_session_cookie` was removed, `create_session` uses `generate_token()` (:241), and `current_user` uses `extract_cookie(parts, "session")` (:412).
- `MAX_INVITE_ATTEMPTS=5`, `create_registration_ticket` (tx: purge expired, then insert), `registration_ticket_phone`, `consume_registration_ticket` (DELETE…RETURNING on `&mut PgConnection`) and `record_failed_invite_attempt` all match the spec SQL.
- The 5 new unit tests are present and listed by `--list`.

U1.3 login.rs:
- `REGISTRATION_COOKIE` const, ssr-gated (:479-481).
- `verify_otp_code` `None` branch issues the ticket, sets a 600s cookie and redirects to `/login?pending=1`.
- `validate_invite_code` requires a live ticket, does not count an empty code as an attempt, and records an attempt on invalid, used or revoked codes.
- `register_with_code` reads the phone only from the consumed ticket, inside the tx. There are no `let _ =` lines and no `normalize`. Failed code lookups record an attempt after `drop(tx)`. The three SQL strings are byte-identical to the base. The `phone_mod` import was removed and the clear-cookie uses `REGISTRATION_COOKIE`.
- `extract_pending_phone_cookie` and `check_pending_registration` were deleted, the docs were reworded, and `data-testid="register-form"` was added. That is the only `view!` change.

U1.4/U1.5 E2E: the POM `submitInviteCode` and `forgeRegistration` are verbatim from the spec, and the `APIRequestContext` import was added. EXTRA_PHONES, CODES.FORGE and the 4 tests (2 setup, 2 story) were inserted after "1.1 — used invite code is rejected", as specified.

Cosmetic deviations (not defects):
- `consume_registration_ticket(&mut tx, …)` instead of `&mut *tx`. This is equivalent through deref coercion.
- The `CODES` type annotation is split across lines.

Gates, all re-run by the reviewer in the worktree (local-only `RUSTC_BOOTSTRAP=1 RUSTFLAGS=-Zcrate-attr=recursion_limit="256"`; no file edited; tree clean at the end):
- Grep gates: `pending_phone` in src/ = 0; the deleted-symbol grep over src/ and end2end/tests/ = 0; `fn extract_session_cookie` = 0; `normalize` in `register_with_code` = 0; `let _ =` in `register_with_code` = 0. The last migration is `20261004000001_registration_tickets.sql`.
  - The only `pending_phone` left in the repo is the deliberate decoy cookie inside the spec-mandated `forgeRegistration` POM.
- `cargo fmt --check`: clean. `clippy --features ssr -D warnings`: exit 0; the only warning is a future-incompat notice from the dependency proc-macro-error2.
- `cargo test` (bare): 68 passed, 0 failed. `cargo test --features ssr`: 80 passed, 0 failed. `cargo build --features ssr`: exit 0.
- `.sqlx`: 5 new query files, one per new query (purge, insert, select, delete-returning, update).
- **Manual adversarial gate.** Debug binary on 127.0.0.1:3911 with the fresh, migrated sibling DB `samete_u1`:
  - Plan probe: the forged cookie `registration_ticket=+380671234567; pending_phone=…` with `code=x-y` gave no `Set-Cookie: session`, a redirect to `/login`, and `users` count = 0. The gate's `ACTION` path was non-empty.
  - Strengthened probe (the reviewer's addition, because the plan's `x-y` code would fail regardless of the fix): a seeded REAL unused code `real-code` with the same forged cookie gave no session cookie, users = 0, and the code stayed `unused`.
  - Forged-ticket `validate_invite_code` was refused with "Номер не підтверджено. Почни спочатку."
  - Positive control: a real OTP for +380671111111 issued a 43-char opaque ticket. The cookie is HttpOnly, Max-Age=600 and redirects to `/login?pending=1`. The DB stores a 64-hex hash only.
  - Throttle: 5 wrong codes gave 5 "invalid" errors and `invite_attempts=5`. After that, the valid code `good-one` was refused with "not verified", and `register_with_code` also refused (redirect to `/login`, no user).
  - Happy path: a fresh OTP ticket plus `good-one`, sent with a decoy `pending_phone=+380679999999` cookie, registered +380671111111 (not the decoy phone), set the session cookie, cleared `registration_ticket` (Max-Age=0), redirected to `/onboarding`, marked the code `used` and deleted the ticket row.
  - Replay of the consumed ticket was refused and the code stayed `unused`.
- E2E, isolated harness `mode=full`, run 3 times (browser path `/opt/pw-browsers`; the pwb shim failed to launch):
  - Run 1: 99 passed / 1 failed / 20 did not run. The failure was visual-audit "admin confirm phase", where advance-button was missing. It finished at 23:49 UTC, the exact moment the disk filled (the coordinator confirmed this), and there was no server panic. Environmental.
  - Run 2: harness setup failed with "No space left on device". Environmental.
  - Run 3: 115 passed / 1 failed / 4 did not run. The server aborted with the reactive_graph "already disposed" panic under release `panic = "abort"`, which is the T1 defect (a separate lane). Not U1.
  - Run 4 (clean): **120 passed, 2 skipped, 0 failed, exit=0**. That is the baseline of 116 plus 4, as the spec expects.
  - Both new titles, "1.1 — forged registration cookie cannot create an account" and "1.1 — invite code attempts are throttled per verification", passed in all 3 runs that reached them. All pre-existing 1.1 and 1.6 titles passed in every run.
- wasm clippy fails with 31 errors, all outside U1 scope:
  - 29 are rustc 1.99's `unused_async_trait_impl` lint firing on every `#[server]` macro expansion in 8 files, untouched ones included.
  - 2 are `.ok().is_some_and` at login.rs:588 and :608, the `is_pending` detection that U1 did not touch.
  - This is ENV / toolchain scope, as the implementer reported.

## Reasoning

I read the U1 section of the plan in full, plus the Preamble, the Design decisions and the shared gate definitions. I then compared the merge-base diff `57f0109..4bccd33` against each U1 step line by line. Every required function, SQL string, doc rewording, deletion, testid, POM method and spec test is present and matches the spec text, with only the two cosmetic deviations listed. I found no extra features, abstractions or files beyond the declared U1 write-set.

I did not trust the implementer's report. I re-ran every gate myself: the greps, fmt, SSR clippy, both test suites, the debug build and isolated E2E. I also ran the manual adversarial gate that the implementer skipped. The plan's own curl probe uses a non-existent code (`x-y`), so it cannot tell a fixed server from a broken one. I therefore added a probe with a real unused invite code, and a positive control (real OTP to ticket to successful registration). Together these show that the ticket is the sole authority: forged cookies fail even with a valid code, the decoy `pending_phone` is ignored, tickets are single-use, and the 5-attempt throttle exhausts the ticket.

The E2E trail needed attribution. The two non-clean runs failed in visual-audit tests unrelated to registration. One coincided with the disk-full event; the other was a server abort with the documented T1 leptos_i18n disposed-signal panic. A fourth run on a healthy disk was fully green at exactly the expected count, and U1's own tests passed in every run that reached them. wasm clippy failures are a toolchain lint on macro output, spread uniformly across untouched server functions. U1 introduced none of them.

Housekeeping: I created and then removed the gitignored `target/debug` (3.9G) to free disk. I dropped the sibling DB `samete_u1` after use. No worktree files were edited or committed (`git status` is empty, HEAD = 4bccd33).
