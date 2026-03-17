# Development Protocol

Extracted from `spec/technical/Architecture.md`. These rules are binding for all implementation — human or agent.

---

## The Feedback Loop

**The compiler is your best friend, forever and always.** The feedback loop has five layers, each catching what the inner layers cannot:

```
1. rust-analyzer        (instant — types, borrow checker, inline diagnostics)
2. bacon clippy         (continuous — pedantic lints, style, correctness hints)
3. cargo test           (on demand — unit tests, integration tests, business rules)
4. cargo leptos end-to-end  (on demand — full-stack E2E, user-visible flows)
5. CI                   (on push — everything, clean environment)
```

Nothing ships that any layer rejects.

## Compiler Configuration

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = { level = "deny" }
pedantic = { level = "deny" }
```

Pedantic clippy findings are not warnings. They are errors. They are always fixed. No `#[allow(clippy::...)]` without a comment explaining why the lint is wrong for this specific case.

## Continuous Runner

`bacon clippy` as the continuous background runner. Not `bacon check` — clippy gives the pedantic lints. Keep bacon running and treat every output line as blocking.

## LSP

rust-analyzer must be active and its diagnostics treated as authoritative. Implementing agents must use LSP tool access to verify type correctness before moving on. Red squiggles are not advisory — they are errors.

## What to Test Where

**Unit tests (`cargo test`):**
- Phase transition logic: valid transitions succeed, invalid transitions return `Err`
- OTP generation, hashing, verification, expiry, rate limiting
- Phone number normalization (various input formats → E.164)
- Assignment algorithm: cycle validity, social weight minimization, cohort splitting
- Session creation, validation, expiry, revocation

**E2E tests (`cargo leptos end-to-end`):**
- Database operations
- SMS delivery (dry-run)
- Leptos component rendering
- Full user flows (login, enroll, confirm, assign, deliver)
- Auth guards and redirects

For E2E conventions, wait strategies, and POM contract: see `end2end/README.md`.

## Rules for Implementers

1. **Model in types first.** Before writing any logic, define the enums, structs, and newtypes that represent the domain. Make invalid states unrepresentable. Then let the compiler tell you what methods those types need.

2. **Strict pedantic clippy, always.** `clippy::pedantic = deny`. Every finding is fixed. No exceptions without a documented reason.

3. **TDD from the spec.** User stories have Given/When/Then acceptance criteria. These become tests — unit tests for pure logic, E2E tests for user-visible flows. Write the test, watch it fail, implement until it passes. Tests trace back to story numbers in `spec/technical/User Stories.md`.

4. **Use every feedback channel.** rust-analyzer for instant type feedback. bacon clippy for continuous lint checking. `cargo test` for business rules. `cargo leptos end-to-end` for full-stack verification. Treat output from every channel as blocking.

5. **Agents use LSP.** An implementing agent must leverage rust-analyzer diagnostics. LSP output is not advisory — it is a compiler-equivalent feedback channel. Fix diagnostics before moving on.

6. **One story at a time.** Implement story by story in dependency order. Run the relevant E2E test after each story. Do not move to the next story until the current one passes E2E.

7. **No speculation.** Do not build for imagined futures. Do not add configurability. Do not add abstractions for one-time operations. The spec defines what exists. Build exactly that.

8. **`cargo sqlx prepare --workspace` after every change** to `sqlx::query!()` calls. Commit `.sqlx/` — it is NOT in .gitignore.

## Story Dependency Chain

```
Phase 1 — Foundation:
  DB schema + migrations, Config + tracing, AppError, Phase enum, phone normalization

Phase 2 — Auth (Epic 1):
  1.1 register participant → 1.2 sign in → 1.3 onboarding

Phase 3 — Season lifecycle (Epics 4, 2):
  4.1 create season → 4.2 launch → 2.1 enroll → 2.2 confirm ready

Phase 4 — Assignment (Epic 3):
  3.1 generate → 3.2 social constraints → 3.3 override

Phase 5 — Delivery (Epics 2, 5):
  2.3 receive assignment + 5.1 SMS → 2.4 confirm receipt + 5.2 SMS
  5.3 season-open SMS, 5.4 pre-deadline SMS

Phase 6 — Account management (Epic 6):
  6.1 deactivate account
```

All 6 phases are complete. This chain is a reference for future phases or debugging.
