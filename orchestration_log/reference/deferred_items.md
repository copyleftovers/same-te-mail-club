# Deferred Items

Last updated: 2026-04-29

## Completed Since Last Update

**Invite Codes (Stories 1.1, 1.5, 1.6) — completed 2026-04-29.**
- Story 1.5: Admin can generate invite codes linked to a distributor (`src/admin/invite_codes.rs`, `src/invite_codes.rs`)
- Story 1.6: Admin can list and revoke unused codes
- Story 1.1: New participants self-register using a valid invite code after OTP verification (replaces manual admin registration form)
- Manual registration form in `src/admin/participants.rs` removed; replaced by invite code flow
- 17 new E2E tests added (suite grew from 58 to 75); Account Management block grew from 3 to 5 tests

---

## Admin UI Redesign

Stories 4.5 (single-page merge) and 4.6 (phase-gated SMS visibility) are written but not implemented. Stories 4.4 (SMS counts), 4.7 (advance gating), and 4.8 (swap dropdowns) are implemented and E2E-tested. The full single-page merge requires user approval of the proposal at `orchestration_log/recon/admin-redesign-proposal.md`.
