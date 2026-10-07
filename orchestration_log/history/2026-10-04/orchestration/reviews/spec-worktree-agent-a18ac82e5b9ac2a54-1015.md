# Spec Review: worktree-agent-a18ac82e5b9ac2a54 (T0, round 2)

Verdict: PASS
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a18ac82e5b9ac2a54
Branch: worktree-agent-a18ac82e5b9ac2a54
HEAD SHA: d09cbe7ac3b455fc43cea25fa8817af937bf2d96
Reviewed at: 2026-10-07 (UTC)
Diff range: 57f0109..d09cbe7 (57f0109 verified as an ancestor)
Files reviewed:
- src/pages/login.rs (full diff)
- src/admin/page.rs (full diff, plus whitespace-insensitive diff)
- src/pages/home.rs (full diff; outside the declared write-set, see Finding M1)
- scripts/assert-playwright-ran.sh, scripts/isolated-capture.sh, justfile, .github/workflows/ci.yml
- Spec: orchestration_log/history/2026-10-04/plans/PLAN-blockers.md § T0, Lanes, Shared env, DoD; owner override (root fix, no recursion_limit, seams at component and `.into_any()`)
- Round-1 verdict: reviews/spec-worktree-agent-a18ac82e5b9ac2a54-2300.md

Binding: yagni.md and first-principles.md were loaded from /tmp/claude-manifesto-repo/LLM_MANIFESTOS/manifestos/.

## Gates I ran myself

Env: CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=3 SQLX_OFFLINE=true. No RUSTFLAGS or RUSTC_BOOTSTRAP was set. rustc 1.99.0.

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0, no output |
| Headroom: `#![recursion_limit = "96"]` added as line 1 of main.rs and lib.rs; SSR bin build | exit=0, `Compiling samete` present, 0 overflow lines |
| Headroom: wasm hydrate lib build | exit=0, `Compiling samete` present, 0 overflow lines |
| Revert of the headroom attribute | `roots-untouched` (git diff --quiet on main.rs and lib.rs) |
| Sabotage (a): `git checkout 57f0109 -- src/pages/login.rs src/admin/page.rs` (staged diff showed exactly those 2 files) | bin exit=101, wasm exit=101; "overflow the depth limit" count 1 and 1 |
| Revert and prove clean | `git checkout HEAD -- …`; `git status --short` was empty; rebuild bin exit=0, wasm exit=0 at the default limit |
| `cargo clippy --features ssr --no-default-features -- -D warnings` | exit 0. Its only `warning:` line is the dependency future-incompat note (proc-macro-error2), not crate code |
| `cargo test` (bare) | ok, 68 passed, 0 failed |
| `cargo test --features ssr` | ok, 75 passed, 0 failed |
| `grep recursion_limit` in src/, .github, justfile, scripts | 0 hits |
| ci.yml codegen/E2E grep | exactly 3 matches (:49, :52, :133); `yaml-ok` |
| justfile counts | assert = 4, touch marker = 4, `cargo leptos end-to-end` = 5 (see Note N1) |

Extra probe, to judge the home.rs write-set excursion: I reverted only home.rs to its 57f0109 state, set the limit to 96, and built the SSR bin. It failed with exit=101 and `queries overflow the depth limit!`, query depth +98. The type named in the trace is `render_assignment_details`'s view: H1, P, `Article.card` > `Dl.info-list`, then the `render_receipt_form` `Section`. The wasm lib passed in that configuration. The home.rs change is therefore forced by the plan's mandatory headroom-96 gate. Afterwards I restored home.rs from HEAD; status was clean.

I did not run wasm clippy (accepted as ENV scope, as noted) or E2E (optional per the dispatch). The worktree's target/ existed before my run; I created none.

## Round-1 items re-checked

1. LoginPage `.into_any()`: FIXED (login.rs:679).
2. InviteCodeStep / NameStep extraction: FIXED. Both are private `#[component]`s with `.into_any()`. Their bodies match the base step-3/4 `<div>` blocks token-for-token (testids, `style:display` closures, comments). The router `view!` is now `<PhoneStep/> <OtpStep/> <InviteCodeStep/> <NameStep/>` + `.into_any()`.
3. ci.yml comment: FIXED. It now carries the spec wording verbatim ("clippy/test never run codegen, so layout-depth overflows in Leptos views only surface in a real build."). No recursion_limit reference remains.
4. InviteCodeStatusCell / InviteCodeRedeemer: FIXED. They are folded back into InviteCodeCard as inline `{view!{…}.into_any()}` erasures. The markup inside is unchanged.
5. Dangling "see conventions: view-type depth" reference: FIXED (0 grep hits). The WHY comment on PhoneStep is self-contained.
6. Sabotage evidence: (a) verified RED here. (b) and (c) I take from the report only.

## Findings

PASS. Spec compliant on every T0 requirement. Minor deviations remain; none of them blocks.

Minor (non-blocking):
- M1. home.rs:1036-1101. This file is outside the T0 write-set, but the headroom-96 gate forced the change (reproduced above). The extension rule covers it ("apply the same move … at that location"). Form deviation: the `<article class="card">` was extracted into a new `#[component] RecipientCard`. The rule reserves component extraction for `<For>` children and match/if branches, and says a card gets an in-place `.into_any()`. The type effect is the same, since the component body ends in `.into_any()`, and by reading the markup is DOM-identical: same elements, attrs, testids, and the `tel:` href built the same way. `render_receipt_form` gaining `.into_any()` is a section-level erasure and is permitted. Optional fix, for consistency with round-1 item 4: inline `{view!{<article…>…</article>}.into_any()}` in place of the component. Note also that the HTML-identity gate covers /login and /admin only, so the assigned-home markup is protected by E2E and code reading, not by that gate.
- M2 (carried from round 1). `InviteCodeCard` takes `revoke_pending: Memo<bool>` as a 4th prop (page.rs:2003-2008) rather than recomputing it. It is the same Memo, so behaviour is identical.
- M3 (carried from round 1). `OtpStep` takes the callback as a generic `on_resend: F` with `F: Fn() + 'static`, not `resend: impl Fn() + Send + Sync + 'static`. The cfg-dual `resend` bindings correctly stay outside every `view!`.
- M4. Extra section-level erasures added under the extension rule: render_sms_report_inline :1045, render_cycle_visualization :1164, SwapFormSection :1656, ParticipantListSection :2274, home `render_receipt_form`, plus `ParticipantRow` (a `<For>` child, so component extraction is allowed). All are permitted forms. I did not test each one's individual necessity. The orchestrator should confirm the implementer's report lists each as `file:line → erasure` (DoD requirement); the report body was not available to me.
- M5. 6 commits rather than the single named commit. DoD 7 allows "commit(s)". All are one-line conventional messages with no body or attribution.

Note:
- N1. The `grep -c "cargo leptos end-to-end" justfile` gate expects 4 and reads 5. The 5th hit is the plan-mandated comment line at justfile:24 ("… cargo leptos end-to-end exits 0 on build failure."). The 4 recipe lines (:30, :38, :45, :60) are each unchanged and guarded before and after. This is a plan-internal inconsistency, not an implementation defect.

Verified compliant (unchanged since round 1): assert-playwright-ran.sh is byte-identical to T0.3 and executable; isolated-capture.sh has rm/build/artifact-check exactly as in T0.6; ci.yml has no second guard copy; `attr:aria-busy` sites are untouched (A1 owns them).

## Reasoning

I read T0, Lanes, Shared environment and the DoD from the canonical plan and applied the owner override. Then I diffed 57f0109..d09cbe7, a confirmed ancestor, file by file, and checked every item round 1 failed against the new code. I did not trust the implementer's pasted evidence for the load-bearing claims. I re-ran headroom-96 on both crate roots, with a recompile confirmed in the logs. I re-ran sabotage (a), which goes red with real overflows on both crates and then reverts clean and rebuilds green. I also ran fmt, SSR clippy and both test suites.

The one judgment call is the home.rs excursion. I tested whether it was forced instead of accepting the pasted trace. With home.rs at base, the bin overflows at 96 inside `render_assignment_details`, so the touch is mandated by the plan's own headroom gate under its extension rule. The remaining question is whether the seam takes the form of a component or an inline erasure. That is a shape deviation with no behavioural or type-depth consequence, so I record it as Minor and do not block on it. Demanding a rewrite with zero functional effect would be ceremony. It is flagged so the orchestrator can choose consistency with round-1 item 4.

VERDICT: PASS
