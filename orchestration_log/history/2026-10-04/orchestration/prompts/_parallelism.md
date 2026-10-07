# Parallelism directive (owner, binding, overrides any ordering in any plan)

Wall time MUST equal the duration of the single longest unit. Nothing waits on anything without proof.

1. Default: every unit is its own concurrent lane, started NOW, from claude/loving-johnson-7l8hn5.
2. "Depends on unit X" is REJECTED unless you state: the exact symbol (fn/type/field/table/testid) unit X introduces, its file, AND why the dependent unit cannot be written concurrently against X's planned signature (as specified in X's plan text). A planned signature is a contract: write against it now; the integrator merges. If the planned signature is fully specified in the plan, the dependency is NOT a reason to wait.
3. Shared files are NEVER a reason to sequence. Each lane has its own worktree. Merge conflicts are resolved at integration (union JSON keys, regenerate .sqlx, re-run gates).
4. "Gate: X must be integrated first" lines are void. Replace with: base = working branch (+ merge any EXISTING branch that already contains the needed symbols; otherwise implement against the planned signature).
5. Batching several units into one agent is allowed ONLY to save tokens on tiny units that edit the same few lines — never to encode an order. When in doubt: separate concurrent agents.
6. Never use the words wave, phase, stage, round, after X lands, once X is integrated. Plans describe units and contracts, not timelines.
7. Integration and review run per unit as each finishes; no unit waits for another's review.
