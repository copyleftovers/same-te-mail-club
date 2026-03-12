
Upon starting, check this index of manifestos first: 'https://raw.githubusercontent.com/ryzhakar/LLM_MANIFESTOS/refs/heads/main/README.md'.
Use the 'manifest-oath' skill to swear to any of those manifestos when prompted.
This project being a rust project and striving to be idiomatic, use the 'correctness-by-construction' manifesto by default.
If continuing a session after context compaction - reswear to the active manifestos anew.
If swearing to more than 1 manifesto, figure out their interplay and interdependencies early: hierarchy, governance, conflict resolution, interference, amplification.
Upon figuring out the graph of manifesto interdependence and multiactivation, write it down in the most natural way accessible to you.

Delegate often and well.
Generally, you would want to use simpler models for any subagents, unless there's a good reason to do otherwise.
For any given delegation, you need to make an explicit decision whether to retain the conversation or now.
Rely on externalized context for delegation as a first-class citizen, prefering it to the handing-down the conversation history whenever possible.
Context, instructions and preferences are externalized as manifestos, plans, artifacts, operational notes, etc.

Plans must survive handoff to agents who lack your context. Use defensive-planning skill to do so.

If anything can be delegated and done in parallell, use multiple parallell agents.
One of the workflows where this pattern lends itself beautifully is objective fault analysis based on each of the active manifestos by separate agents.

---

# CLAUDE NOTES

## Active Manifestos

Bind to both on session start (treat as unified framework):
1. **Correct By Construction** — fetch from LLM_MANIFESTOS repo: `Manifesto, rust - "correct by construction".md`
2. **Simple Made Easy** — fetch from LLM_MANIFESTOS repo: `Manifesto, "simple made easy".md`

Tension resolution: type richness warranted when it eliminates entanglement (enum replacing boolean flags). Not warranted when it introduces entanglement (generic trait hierarchies braiding concerns).

## Project Phase

**Implementation.** Scaffold complete. Implementation plan written. E2E test stubs written (failing by design).

## Authoritative Documents (read in this order)

1. `spec/Implementation Plan.md` — prescriptive, phase-by-phase. No decisions for implementer. Start here.
2. `spec/Architecture.md` — technical architecture, development protocol, testing strategy. §Development Protocol and §Testing Strategy are binding.
3. `spec/User Stories.md` — acceptance criteria (Given/When/Then) for all epics
4. `spec/Product Spec.md` — product decisions, season structure, failure protocols

## Development Protocol

**The compiler is your best friend, forever and always.**

Five-layer feedback loop. ALL layers are BLOCKING — nothing moves forward while any layer reports errors:

```
1. rust-analyzer / LSP    (instant — types, borrow checker, inline diagnostics)
2. bacon clippy-ssr       (continuous — pedantic lints, style, correctness hints)
3. cargo test             (on demand — unit tests, business rules)
4. cargo leptos end-to-end (on demand — full-stack E2E, user-visible flows)
5. pre-commit             (on commit — fmt, cargo check, clippy)
```

### Rules for Implementing Agents

1. **Model in types first.** Define enums, structs, newtypes before any logic. Make invalid states unrepresentable. Let the compiler tell you what methods those types need.
2. **Strict pedantic clippy, always.** Already configured: `clippy::pedantic = deny`. Every finding is fixed. No `#[allow(clippy::...)]` without a comment explaining why the lint is wrong for this specific case.
3. **TDD from the spec.** Tests derive from acceptance criteria in `spec/User Stories.md`. Write test, watch it fail, implement until it passes. Every test traces to a story number.
4. **Use LSP.** rust-analyzer diagnostics are BLOCKING, not advisory. Fix diagnostics before moving on. Implementing agents must use the LSP tool.
5. **One story at a time.** Implement in dependency order per `spec/Implementation Plan.md`. Run the relevant E2E test after each story. Do NOT move to the next story until the current one passes E2E.
6. **No speculation.** Do not build for imagined futures. Do not add configurability. Do not add abstractions for one-time operations. The spec defines what exists. Build exactly that.
7. **`cargo sqlx prepare --workspace` after every phase** that adds or changes `sqlx::query!()` calls. Commit `.sqlx/` — it is NOT in .gitignore.

### What to Test Where

| Test with `cargo test` (unit) | Test with `cargo leptos end-to-end` (E2E) |
|------|------|
| Phase transition logic | Database operations |
| Phone number normalization | SMS delivery (dry-run) |
| OTP hashing/verification logic | Leptos component rendering |
| Assignment algorithm (cycle validity, scoring) | Full user flows (login, enroll, confirm) |
| Session token generation logic | Auth guards and redirects |

## E2E Tests

E2E test stubs exist in `end2end/tests/` and encode user stories as executable specifications. They are FAILING by design — make them pass story by story.

Test environment requires:
- `SAMETE_TEST_MODE=true` — fixed OTP code "000000"
- `SAMETE_SMS_DRY_RUN=true` — log SMS instead of sending
- Postgres running with migrated DB (`just db-reset`)

## Key Commands

| Command | Purpose |
|---------|---------|
| `just dev` | Start dev server (hot reload) |
| `just test` | Run unit tests |
| `just clippy` | Run clippy (SSR) |
| `just e2e` | Run Playwright E2E tests |
| `just check` | Full validation: fmt + clippy + test |
| `just db-reset` | Drop, create, migrate database |
| `just prepare` | Generate sqlx offline query data |
| `bacon` | Continuous clippy. Keys: `s` SSR, `h` hydrate, `t` tests |
