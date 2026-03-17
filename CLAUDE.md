
Upon starting, check this index of manifestos first: 'https://raw.githubusercontent.com/ryzhakar/LLM_MANIFESTOS/refs/heads/main/README.md'.
Use the 'manifest-oath' skill to swear to any of those manifestos when prompted.
This project being a rust project and striving to be idiomatic, use the 'correctness-by-construction' manifesto by default.
If continuing a session after context compaction - reswear to the active manifestos anew.
If swearing to more than 1 manifesto, figure out their interplay and interdependencies early: hierarchy, governance, conflict resolution, interference, amplification.
Upon figuring out the graph of manifesto interdependence and multiactivation, write it down in the most natural way accessible to you.

Delegate often and well.
Generally, you would want to use simpler models for any subagents, unless there's a good reason to do otherwise.
For any given delegation, you need to make an explicit decision whether to retain the conversation or not.
Rely on externalized context for delegation as a first-class citizen, prefering it to the handing-down the conversation history whenever possible.
Context, instructions and preferences are externalized as manifestos, plans, artifacts, operational notes, etc.

Plans must survive handoff to agents who lack your context. Use defensive-planning skill to do so.

If anything can be delegated and done in parallel, use multiple parallel agents.
One of the workflows where this pattern lends itself beautifully is objective fault analysis based on each of the active manifestos by separate agents.

---

# CLAUDE NOTES

## Active Manifestos

Bind to all three on session start (treat as unified framework):
1. **Correct By Construction** — fetch from LLM_MANIFESTOS repo: `Manifesto, rust - "correct by construction".md`
2. **Simple Made Easy** — fetch from LLM_MANIFESTOS repo: `Manifesto, "simple made easy".md`
3. **First Principles — Break the Mold** — fetch from LLM_MANIFESTOS repo: `Manifesto, first-principles - "break the mold".md`

Interplay: First Principles sits above as meta-cognitive arbiter — it questions whether CBC and SME are being applied from axioms or from habit. CBC and SME are peer constitutions. Type richness warranted when it eliminates entanglement (enum replacing boolean flags). Not warranted when it introduces entanglement (generic trait hierarchies braiding concerns). First Principles breaks ties: does this type encode a truth of the domain, or is it just convention?

## Project Status

All 6 implementation phases complete. E2E flakiness root-caused and fixed (2026-03-17). See `end2end/e2e-research.md` § Confirmed Root Causes.

## Guidance

- @guidance/dev-protocol.md — feedback loop, compiler rules, LSP, TDD, unit vs E2E boundary (**binding**)
- @guidance/leptos-idioms.md — Leptos 0.8 patterns (mandatory for all component work)
- @guidance/delegation-playbook.md — agent delegation framework and prompt template
- @guidance/debugging-policy.md — E2E failure delegation, long-running command rules
- @end2end/README.md — E2E testing guide, POM contract, wait strategies (**binding**)

## Product Design

→ `spec/product/` — vision, personas, product decisions
→ `spec/technical/` — architecture, data model, user stories

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
