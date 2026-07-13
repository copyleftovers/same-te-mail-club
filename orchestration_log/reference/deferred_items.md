# Deferred Items

Currently-open items only. Resolved items are deleted, not stamped — history lives in `codebase_state.md` Changes sections and `history/*/session.md`. Last swept: 2026-07-13.

## Open

- Leptos SSR reactive-disposal panic (intermittent `tower_http` 500s) — no fix commit exists; needs a Leptos-lifecycle investigation of the reactive-scope disposal race (first flagged: 2026-06-25).
- Manifesto SubagentStart oath hook injects 0 constitution elements — plugin-side, outside this repo's control; mitigated per-dispatch by carrying full manifesto paths in every prompt, but the hook itself is unfixed and taxes every subagent dispatch (first flagged: 2026-07-03).
- IP-based OTP rate-limiting absent — only phone-keyed limits exist (`check_otp_rate_limit`); needs middleware or an external store to add an IP dimension (first flagged: 2026-06-22).
- Mechanical/geometric visual assertions unbuilt — zero clip/overflow/overprint/`scrollWidth<=innerWidth` checks in `visual-audit.spec.ts`; agent eyes remain the only geometric gate (first flagged: 2026-06-25).
- `FIELD_DISCRIMINANT_SEPARATOR` duplicated in `src/admin/season.rs:12` and `src/pages/onboarding.rs:14`, error-field-routing idiom undocumented — consolidate to one definition (e.g. `error.rs`, which already holds `strip_server_error_prefix`) and document on next touch (first flagged: 2026-07-11).
- `#[cfg(any(feature = "ssr", test))]` gate formula (10 sites across `phone.rs`, `invite_codes.rs`, `assignment.rs`, `home.rs`) has no orienting WHY comment for future module authors — cosmetic, add on next touch (first flagged: 2026-07-12).
- Orphan Postgres DB `samete_ssr_debug2` still exists — `psql -c 'DROP DATABASE samete_ssr_debug2;'` at convenience (first flagged: 2026-07-10).
- `cohort-seed.sql` uses bare `ON CONFLICT DO NOTHING` (×5, no conflict target) — works, documented as deliberate, but untightened; add explicit conflict targets on next touch (first flagged: 2026-07-11).
- `admin-season-cancelled` / terminal-season create-form capture: the create-form UI is structurally unreachable once any season row exists (`get_admin_state()` has no phase filter; cancelling never clears the row) — if a create-form-after-cancel state is ever wanted, it requires a product decision + code change, not just a capture (first flagged: 2026-06-25).

## Decisions pending (not code work)

- `orchestration_log/reference/implementation_plan.md` (22KB, dated April, unreferenced anywhere) — needs a user keep/archive/delete call (first flagged: 2026-07-12).
