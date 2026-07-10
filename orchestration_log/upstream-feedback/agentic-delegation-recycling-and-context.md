# Upstream feedback — `agentic-delegation` skill: cost model, recycling & context budgeting

**For:** the skill maintainer (you), to apply upstream. Not applied here — this project mirrors the full doctrine locally in `orchestration_log/reference/conventions.md` → "Orchestration Doctrine — Delegation, Recycling & Context Budgeting".

**Target file:** repo `ryzhakar/claude-skills`, path `orchestration/4.0.0/skills/agentic-delegation/SKILL.md` (installed @ `9bef092`; the plugin cache is ephemeral — `autoUpdate:true` — edit the repo, not the cache).

---

## What the skill has today (scattered; no dedicated section)
- **L23–25** (`<you_are_the_orchestrator>`): the cost-asymmetry doctrine — fresh-launch costs 3–5k init tokens; a SendMessage continuation costs only the delta; "continue the existing agent" for same-domain follow-up.
- **L447–452, 480–510** (`<correct_mid_flight>`): two correction paths — SendMessage (scoped, keeps context) vs TaskStop + fresh launch (fundamental failure).
- **L311** (`<ensure_liveness>`): continuations also get a cron backstop.

## The gap — two layers, both bit us in production

**Layer 1 — the COST MODEL is incomplete (sharpest correction; targets L23–25).**
The skill frames continuation as strictly cheaper, which quietly implies "route work through the agent that holds context." But **authoring content costs the same tokens in the orchestrator's context whether the orchestrator writes it directly or dictates it to an agent — it lands in the dispatch prompt either way.** So a scribe only ADDS overhead (framing) and SUBTRACTS fidelity (transcription/interpretation). The true asymmetry is not fresh-vs-continue; it is **READ vs WRITE**:
- Delegation's only real win is making an agent ABSORB token cost the orchestrator would otherwise bear — READING/processing large inputs that never enter the orchestrator's context.
- Delegation NEVER wins for emitting content the orchestrator already holds.
Missing consequence: the direct-write path exists wherever the harness permits writes (e.g. a checkpoint phase for memory work); under a file-op prohibition, delegation is FORCED and its overhead is a cost of the prohibition, not a benefit. And if guidance must be authored via delegation, the agent must be opus.

**Layer 2 — recycling is WINDOW-BOUNDED (targets L23–25 + the correction paths).**
L23–25 omits that recycling spends a depleting, model-bounded context WINDOW. Read literally ("continue the existing agent"), it invites drive-to-exhaustion. It did: a **sonnet** debug agent recycled three times (heavy log-reads + serve/curl) until its resident context overflowed with **"Prompt is too long"** — unrecoverable, zero salvage. The skill has no window budget, no model-window awareness, no "when to STOP recycling," and no distinction between context-exhaustion and rate-limit failures.

## What to add (crystallized doctrine)
1. **Cost model:** authoring-cost is identical direct-vs-dictated; delegate to ABSORB READS, never to emit held content; write directly where the harness permits (checkpoint memory work), else delegation is forced overhead; guidance-if-delegated MUST be opus.
2. **Model-dependent windows** as the budget denominator: Opus ≈ 1M · Sonnet ≈ 250k · Haiku ≈ 200k; budget against the agent's own. **Model selection has two drivers: reasoning tier AND context budget** — a context-heavy task may warrant opus purely for its window.
3. **Estimation:** `resident ≈ init(~6k) + prompt + Σ(text_bytes ÷ 3–4) + Σ(images × ~1.5k) + output`; content-type matters (prose 4 B/tok, code/JSON/logs 3 B/tok, images ~1.5k tok each regardless of byte-bulk); runtime probe = `stat -f %z` on the agent transcript jsonl, % vs the agent's window. (Empirical: sonnet overflowed at 735 KB dense text ≈ 245k tok; opus survived 4158 KB image-heavy on 1M.)
4. **Recycle-vs-fresh decision:** recycle when the follow-up is a tight delta leveraging held context AND load < ~65% of the window; else fresh-launch SEEDED from the on-disk artifact (window near ceiling / new domain / reaped / tier change). Own until the DELTA STOPS BEING CHEAP — hand off BEFORE the wall.
5. **Discipline:** externalize state to files first (so a successor is cheap and recycling is a fast-lane over a file-backed spine); plan-time budget → SPLIT a task whose inputs alone approach the window into single-question agents; failure-mode distinction — "Prompt is too long" = context exhausted (NOT resumable, decompose) vs "session/rate limit" = resumable via SendMessage after reset.

## Suggested placement
- **L23–25:** replace the fresh-vs-continue framing with the READ-vs-WRITE cost model + a one-line note that continuation spends a depleting, model-bounded window.
- **New `<recycle_agents>` subsection** in `<launch_and_monitor>`, after `<correct_mid_flight>` (~L515), carrying items 2–5.

The full ready-to-adapt prose is codified in this project at `orchestration_log/reference/conventions.md` → "Orchestration Doctrine — Delegation, Recycling & Context Budgeting (2026-07-10)".
