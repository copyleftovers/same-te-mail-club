
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
1. **Correct By Construction** — `manifestos/Manifesto, rust - "correct by construction".md`
2. **Simple Made Easy + Decomplect** — `manifestos/Manifesto, "simple made easy".md` and `manifestos/Manifesto, KISS - "decomplect".md`

Tension resolution: type richness warranted when it eliminates entanglement (enum replacing boolean flags). Not warranted when it introduces entanglement (generic trait hierarchies braiding concerns).

## Project Phase

Scaffold complete. Next: implementation planning per `spec/Architecture.md` Phase 1–6 order.

## Key Files

- `spec/Architecture.md` — authoritative technical architecture (supersedes Technical Research Brief)
- `spec/Product Spec.md` — product decisions, season structure, failure protocols
- `spec/User Stories.md` — acceptance criteria for all epics
- `README.md` — config decisions, setup instructions (shared with team)

## Reference Projects

- `../filmorator` — Leptos 0.8 workspace, most mature CI/CD, CLAUDE.md patterns
- `../monobank-sync` — sqlx offline mode, release-plz + cargo-dist, pre-commit with SQLx hook
- `../gallery-rs` — cargo-chef + distroless Docker pattern
