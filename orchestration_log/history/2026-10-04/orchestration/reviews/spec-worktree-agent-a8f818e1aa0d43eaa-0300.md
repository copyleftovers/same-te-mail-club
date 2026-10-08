# Spec Review: worktree-agent-a8f818e1aa0d43eaa

Verdict: FAIL
Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-a8f818e1aa0d43eaa
Branch: worktree-agent-a8f818e1aa0d43eaa
HEAD SHA: 55c4f4ffffdfcfd2c47286080c562bbe73487f2f
Reviewed at: 2026-10-08T03:10Z
Diff: `git diff 384feac..55c4f4f` (merge-base with claude/loving-johnson-7l8hn5)
Files reviewed:
- src/types.rs
- src/pages/home.rs
- locales/uk.json
- end2end/tests/fixtures/mail_club_page.ts
- end2end/tests/mail_club.spec.ts
- end2end/tests/visual-audit.spec.ts
- .sqlx/ (3 files: 1 removed, 1 renamed, 1 added; these match the 2 changed confirm_receipt queries)
- style/components.css (rhythm rule, read to find the cause of the pixel defect)
- end2end/screenshots/{light,dark}-{desktop,mobile}/home-assignment-and-receipt-form.png, home-receipt-confirmed-thanks.png, home-receipt-not-received-reported.png (all 12 viewed). Baseline comparison: agent-ac57401dbc39b3668/.../light-desktop/home-assignment-and-receipt-form.png

## Findings

FAIL. Issues found:

Misinterpreted (render defect; blocks the pass):
- H2.2 "timing hint above the form". The code at src/pages/home.rs `render_own_receipt`, NoResponse arm, places a bare `<p data-testid="receipt-timing-hint">` as a direct `.prose-page` child between `<article class="card">` and the receipt `<section>`. This defeats the section-rhythm rule `.prose-page > :where(section, article, .card, .cycle-viz-container) + :where(...) { margin-top: var(--density-space-lg) }` (style/components.css ~L521), for two reasons. First, card+p does not match it, so the hint sits about 3px under the card border and reads as a caption of the recipient card. Second, p+section does not match it either, so the receipt section loses its `--density-space-lg` top margin. This shows in all 4 captures of `home-assignment-and-receipt-form` (light/dark × desktop/mobile). In the baseline the card-to-"Отримав(ла) лист?" gap is about 40px; here it has collapsed. The plan's verbatim markup is the origin, but the rendered result breaks the design-system vertical rhythm, so it fails. Fix: move the hint inside the receipt `<section>`. Either wrap the NoResponse arm in a `<section>` that holds hint + form, or render the hint inside `render_receipt_form`. Keep the hint above the form controls and keep testid `receipt-timing-hint` unchanged. Re-capture all 4 dirs.

Partial (forced by a plan contradiction; integrator action, not a cause of the FAIL):
- H2.3 / Base recipe "copy expectRosterStatus/expectRosterNote verbatim; test calls expectRosterStatus(NAMES.B,'received') + expectRosterNote(...)". Both the POM methods and the 2 calls are omitted (mail_club.spec.ts ~L866-879 carries a comment saying they join with P-R). This is justified: P-R is not on the base (`grep season-roster` = 0), so those calls would fail on this branch. The plan only anticipated `expectNoForwardingRequests` being vacuous. The integrator must add the 2 roster asserts once P-R is merged. Also, `expectNoForwardingRequests` is vacuous until P-F, as the plan acknowledges.

Minor:
- end2end/tests/visual-audit.spec.ts:704 has a stale comment that names the deleted `ReceiptConfirmed` state. The plan's grep gate covers home.rs only, so it was missed.
- Desktop column shift: on the assignment+form state the long hint widens the shrink-wrapped content column. Content starts at x=331 instead of x=375, which differs from the thanks/reported states and from the baseline. Check this after the fix above; it may persist because it comes from text length.

Verified compliant:
- H2.1: `ReceiptStatus::can_transition_to` is exact per spec, with an enum doc line and 3 tests. I ran all 3 and all are green.
- H2.2 `HomeState::Delivery { recipient, own_receipt }` replaces Assigned/ReceiptConfirmed. The struct rename RecipientCard→RecipientDetails is justified because `#[component] fn RecipientCard` already exists (home.rs ~L1053). `resolve_delivery_state` uses `fetch_one`. `confirm_receipt` has the read→filter→guarded UPDATE with COALESCE, as specified. `render_delivery`/`render_own_receipt` match the spec. `render_receipt_confirmed` is deleted and the toast comment is updated. The plan grep gate on home.rs gives zero matches.
- Keys: exactly the 2 P-H2 keys, appended last, with the exact strings.
- E2E edits for H2.3 match the spec, apart from the roster omission above. The H7c capture is added.
- Server-side probe on an isolated release server (sibling DB samete_ph2spec, free port, directly seeded delivery cycle, session cookies; torn down and dropped afterwards):
  - NoResponse→NotReceived(note): 200.
  - NotReceived→NotReceived: 500 "Вже підтверджено…", unchanged.
  - NotReceived→Received with no note: 200 and the note is kept (COALESCE).
  - Received→NotReceived: 500. Received→Received: 500.
  - SSR after Received still renders `recipient-name` (C1).
  - 12 concurrent NotReceived submits: 1×200 / 11×500.
  - 12 concurrent corrections: 1×200 / 11×500.
  - No panics.
- P-COPY cross-check: `home_reported_label` "…Якщо лист усе ж прийде — познач це нижче." is followed directly, in the same section, by the button "Лист усе ж прийшов". The wording and the placement read coherently. (This branch still shows the old value "Повідомлено організатору"; P-COPY is not merged yet.)
- Received/NotReceived states render cleanly in all 4 modes: card kept, h2 section heading, secondary pill. `cargo fmt --check` is clean.

## Reasoning

I took the requirements from PLAN-product.md §P-H2 (H2.1–H2.3 + gates), the Base recipe and the Key-ownership table. I compared the merge-base diff line by line against them and did not trust the implementer's report. The Rust logic matches the spec almost verbatim. The single rename is forced by a real name clash, which I confirmed in source.

I did not rely on code reading for the server-side transition guard. I probed it empirically against the implementer's release binary, which contains the new i18n string, on an isolated port and a sibling DB. Sequential illegal transitions are refused. The late-arrival correction keeps the original note. Concurrent double-submits yield exactly one winner in both races.

The FAIL rests on rendered pixels. I viewed all 12 PNGs at native resolution and compared them with a pre-change baseline capture. The new hint paragraph breaks the established section rhythm in every viewport and mode. I traced the cause to the depth-bounded sibling rule in components.css. The fix is small and local: wrap the hint into the receipt section. The roster omission is a plan inconsistency handled transparently and is left to the integrator. It is not counted against the unit.
