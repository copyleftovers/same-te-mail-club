# J — Product sense / value beyond declared scope

HEAD 1a3c8a5. Read-only. Evidence = file:line. Known exclusions (prompt §3) not re-listed.

Binding: stop-yapping (tables, no padding) · first-principles (value derived from what the club is FOR, not feature norms) · kiss (smallest change that removes the friction; no rewrites) · dry (no restating known items).

## Product model

| Axis | Fact | Source |
|---|---|---|
| Purpose | Blind self-expression → one stranger receives it → in-person meetup circle-share. App = logistics only; value happens offline. | spec/product/Introduction.md "HOW IT WORKS", "WHAT I'M BUILDING TOWARD" |
| Emotional core | (1) creating without an audience, (2) receiving one unique physical object, (3) **the meetup** where sender is named and hears how it landed. | Introduction.md; Product Spec.md §Meetup Format |
| Identity guard | "This is NOT: a pen-pal service … Secret Santa" | Product Spec.md:5 |
| Personas | Newcomer (fears: "stupid", "weird", phone shared), Returning (complacency), Inviter (social risk), Organizer (solo; **time is the binding constraint**). | spec/product/Personas.md |
| Numbers | 11–15/cohort, pool ~12 at start, ≤~50 near-term; mobile is the real device. | Product Spec.md §Cohort Model; guidance/component-evaluation-framework.md P4 |
| Lifecycle | Enrollment (~5d) → Preparation/creation (10–14d, confirm deadline) → Assignment (organizer generates/swaps) → Delivery (send ~2d; receive-confirm at ~day 5) → Complete → meetup (≤1 wk, offline). All transitions + all SMS are manual organizer clicks; no scheduler by design. | Product Spec.md:36-44; spec/technical/Architecture.md:37; src/admin/season.rs:106,144 |
| Constraints | Nova Poshta document tier (sender needs recipient name+phone+branch; ~100 UAH); TurboSMS ~$0.025/SMS, Cyrillic; physical artifacts unique → failure handled by human forwarding. | Product Spec.md §Failure Protocols; Architecture.md:82 |
| Owner priorities (last 2 sessions) | Trust via verifiable diligence; no nannying, own design calls; verify claims vs source. No product-feature direction recorded. | history/2026-07-15/session.md:9,70; history/2026-07-13/session.md (narrative) |

## Journeys — friction table

### Participant (newcomer / returning)

| # | Step / state | Friction | Evidence | Sev |
|---|---|---|---|---|
| P1 | Delivery: Assigned → confirms receipt | **Recipient name/phone/branch disappear the moment the participant confirms their OWN receipt.** Send and receive are independent; anyone who receives before shipping loses the address they need. Dead end. | src/pages/home.rs:241-244 (`ReceiptConfirmed` replaces `Assigned`, carries no recipient); render_receipt_confirmed home.rs:1094-1121 | MAJOR |
| P2 | Delivery: Assigned | Receipt form ("Отримав(ла) лист?") shown on day 0, alongside the recipient card — before anyone could have shipped. Spec: prompt at day 5. "Не отримав" is irreversible (UPDATE only where `no_response`), so an early/curious tap = permanent false forwarding signal + P1 address loss. | home.rs:1086 (form rendered with assignment); home.rs:592-597 (`receipt_status = 'no_response'` guard); Product Spec.md:43 | MAJOR |
| P3 | Assignment/Delivery phase, participant NOT in the cycle (not enrolled or unconfirmed) | Sees "Розподіл · В процесі · За кілька хвилин дізнаєшся, кому надіслати свій лист" for the whole delivery window — a promise to someone who will never get an assignment. Complete phase tells them "Дякуємо, що був(ла) з нами". (Distinct from the known preparation-phase confirm-ready leak.) | home.rs:336 (Assignment → Assigning for all); home.rs:222-225 (no outgoing → Assigning); home.rs:317,339; uk.json:29,43 | MAJOR |
| P4 | Enrollment after signup deadline, before organizer advances | Form + button still render; submit fails "Термін реєстрації вже минув". State never checks the deadline. Gap length = organizer latency. | home.rs:107-153 (no deadline check) vs home.rs:399 | MINOR |
| P5 | Enrolled / Preparing (creation period) | Theme and content guideline vanish after enrolling — exactly when creating. Spec: theme shown "during enrollment and creation period". | HomeState::Enrolled/Preparing carry no theme, home.rs:38-44; theme only in render_enrollment_open home.rs:799-804; Product Spec.md:155 | MINOR |
| P6 | Enrollment copy | Pen-pal framing ("знайдемо тобі пару для листування") contradicts identity; "Кому саме, дізнаєшся після реєстрації" is false (learn after assignment). | uk.json:11,12; Product Spec.md:5 | MINOR |
| P7 | Assigning copy | "За кілька хвилин" — assignment is organizer-triggered, can take days. | uk.json:29 | MINOR |
| P8 | Assigned | Shipping guidance is one line. No sending window (~2d), no "document tier", no cost hint — newcomer's practical fear unaddressed. | uk.json:147; Product Spec.md:42 | MINOR |
| P9 | Not received | "Записано / Повідомлено організатору" — no next step, no expectation (organizer will contact / forwarding may follow). | uk.json:40-41; home.rs:1100-1107 | MINOR |
| P10 | Complete / whole app | **Zero meetup information anywhere** — no date, place, or what happens there. The meetup is the payoff; the app ends with "До наступного разу". Story 2.1 promises "expected meetup window". | `grep -rni meetup src migrations` → 0 hits; seasons table has no meetup column (migrations/20260314000002_create_tables.sql:34-42); uk.json:43; User Stories.md:135 | MAJOR (value) |
| P11 | SMS | No link in any SMS ("Заходь в додаток") — it's a web app, monthly cadence, 90-day sessions; participants must remember the URL. | uk.json:98,99; src/admin/sms.rs:191,283 (body = bare key); no site-URL in src/config.rs | MAJOR |
| P12 | Newcomer between seasons | NoSeason is the first screen after onboarding: no "how it works", nothing about the ritual. Personas list fears the app never addresses. | uk.json:142 + home.rs:1132-1137; Personas.md "Fears" | MINOR |
| P13 | Preparing | Deadline is a date only; Story 2.2 asks for a countdown. | home.rs:945-948; User Stories.md:154 | MINOR |
| P14 | Cancelled | No SMS; a participant who already made mail learns only by opening the app (by spec). Body promises SMS for next season, fine. | User Stories.md (4.3 AC); uk.json:45 | OK (declared) |

### Organizer

| # | Step | Friction / workaround | Evidence | Sev |
|---|---|---|---|---|
| O1 | Throughout | **Counts only, no names.** Enrolled / confirmed / not-received / no-response are integers. To personally nudge or run the forwarding protocol the organizer must go to the DB. Participant table has no per-season status. | src/admin/state.rs:33-43; admin/page.rs:555-645; participants table cols admin/page.rs:2100-2103 | MAJOR |
| O2 | Delivery: forwarding protocol | Cycle viz is name→name with no receipt status; organizer must mentally trace "who failed → who should forward to whom". The spec's protocol (forward to the skipped block's downstream) is mechanical and computable. | AssignmentLink {sender_id, sender_name, recipient_name} src/admin/assignments.rs:25-29; Product Spec.md §Non-compliance, §Contiguous failures | MAJOR |
| O3 | Delivery: not-received | No alert to organizer; must poll /admin. Story 2.4: "triggers organizer notification". | User Stories.md:187; only `not-received-alert` count, admin/page.rs:637-645 | MAJOR |
| O4 | Launch / Advance → SMS | Two-step coupling: launch then separately "Надіслати відкриття"; advance to Delivery then separately "Надіслати призначення". Forgetting = participants never know. Story 4.2: launch *triggers* SMS. | admin/season.rs:106 (launch has no SMS); admin/sms.rs:155,238; User Stories.md 4.2 AC | MINOR |
| O5 | Confirm nudge timing | Spec: ~1h before confirm deadline. Manual button → organizer must be online at that hour. No hint on admin of time remaining. | User Stories.md 5.4 AC; Architecture.md:37; admin/page.rs:910-930 | MINOR |
| O6 | Season mistakes | No edit for dates/theme after create; extending a confirm deadline (common when people need time) requires cancel + recreate, which strands enrollments, or DB edit. | server fns: create/launch/advance/cancel only (src/admin/season.rs:20,106,144,184); only UPDATEs are launch/phase/cancel (season.rs:114,167,214) | MAJOR |
| O7 | Advance-blocked hint | "Спочатку опублікуй призначення на сторінці Розподілу" — no publish action, no separate page (single /admin; action is "Згенерувати"). | uk.json:76; admin/page.rs:537-538,714-720; app.rs:131-133 | MINOR |
| O8 | Social graph | Known groups only via raw SQL. Past pairings ARE used (good). | admin/assignments.rs:70-95; Product Spec.md §App Scope "organizer edits database directly" | MINOR (declared) |
| O9 | Repeat-failure judgment | Deactivation needs memory of who failed/no-showed across seasons; no per-participant season history, no deactivation reason. | participants.rs:35-50,76-93; Product Spec.md §Deferred "Season history" | MINOR |

## Candidates (beyond declared scope / declared-but-unbuilt)

| ID | Improvement | Who | Problem evidence | Value | Effort | Risk |
|---|---|---|---|---|---|---|
| C1 | Keep recipient card visible after own receipt confirmation (ReceiptConfirmed carries + renders recipient details) | Participant | P1 home.rs:241-244 | H | S | Low; E2E `receipt-thanks` test stays |
| C2 | Gate receipt form: render only after N days from delivery start (or behind an explicit "I got mail" reveal), and allow NotReceived → Received correction | Participant, Organizer (fewer false alarms) | P2 home.rs:1086, 592-597; Product Spec.md:43 | H | S–M (needs delivery-start timestamp; none exists — seasons has no phase timestamps) | Low–Med; E2E receipt flow touch |
| C3 | Non-participant states: in Assignment/Delivery/Complete show "цього сезону ти не береш участі — чекай на наступний" instead of Assigning/thanks | Non-confirmed participants | P3 home.rs:222-225,317,336,339 | H | S (enrollment/confirmed check exists pattern home.rs:114-127,162-176) | Low |
| C4 | Site URL in SMS bodies (config `SITE_URL`, appended) | All participants | P11 uk.json:98-99 | H | S | Low; +1 SMS segment cost possible (Cyrillic 70-char segments) — measure |
| C5 | Named lists in admin: per-phase "who" (unconfirmed, no-response, not-received) with tel: links | Organizer | O1 state.rs:33-43 | H | M | Low; read-only queries |
| C6 | Forwarding helper: receipt status on each cycle link + computed "ask X to forward to Y" for each contiguous failed block | Organizer | O2 assignments.rs:25-29; Product Spec.md contiguous-failure rule | H | M (pure fn, unit-testable) | Low |
| C7 | Meetup info: one nullable season text field (date/place), shown from Delivery through Complete; optional meetup SMS | Participant (payoff), Organizer (fewer side-channel messages) | P10 `grep meetup` = 0; User Stories.md:135 | H | M (migration + form + 1 display + optional SMS) | Low; text field only — NOT RSVP/scheduling (respects Product Spec exclusion) |
| C8 | Edit season dates/theme pre-terminal (deadline extension) with existing validation | Organizer | O6 season.rs (no update fn) | M–H | M | Med: must re-validate signup<confirm, future dates; phase-aware which fields editable |
| C9 | SMS organizer on "not received" (reuse sms::send_sms, organizer phone = admin user) | Organizer | O3 User Stories.md:187 | M | S | Low; organizer phone available from users.role='admin' |
| C10 | Copy fixes: pen-pal framing, "після реєстрації", "кілька хвилин", advance-blocked hint, richer shipping + not-received next-step | Participants, Organizer | P6, P7, P8, P9, O7 uk.json:11,12,29,41,76,147 | M | S (uk.json only) | Very low |
| C11 | Theme + one-line guideline persist in Enrolled/Preparing | Participant | P5 home.rs:38-44 | M | S | Low |
| C12 | Hide enroll form when signup deadline passed (state-level check, show closed message) | Participant | P4 home.rs:107-153 vs 399 | M | S | Low |
| C13 | Inline "send SMS now" prompt immediately after launch/advance (or auto-send on launch per Story 4.2) | Organizer | O4 | M | S–M | Med if auto (irreversible send); prompt variant low |
| C14 | "How it works" block for NoSeason / first visit (4 steps + meetup) | Newcomer | P12 | M | S | Low |
| C15 | Admin countdown to confirm deadline + nudge-window hint | Organizer | O5 | L–M | S | Low |
| C16 | Known-groups admin UI (create group, add/remove members, weight) | Organizer | O8 | M | M–L | Med; scope expansion |
| C17 | Per-participant season history (enrolled/confirmed/receipt/reported) on participants table | Organizer | O9 | M | M | Low; derived from existing tables |
| C18 | Countdown on participant deadline | Participant | P13 | L | S | Low (hydration care: SSR/WASM time mismatch) |

Rejected: scheduled SMS cron (contradicts deliberate no-scheduler decision Architecture.md:37; C13/C15 cover the pain at lower complexity). RSVP/meetup scheduling (explicit exclusion; C7 is display-only).

## Top 8 (value/effort)

| Rank | ID | Why it ranks | Value/Effort |
|---|---|---|---|
| 1 | C1 | Live dead end in the core send step; one state field + render. | H/S |
| 2 | C3 | Every non-confirmed pool member is told a false promise for ~a week each season. | H/S |
| 3 | C4 | Every SMS currently leads nowhere clickable; the whole notification channel's conversion depends on it. | H/S |
| 4 | C2 | Prevents irreversible false "not received" + compounds with C1; protects forwarding-protocol signal quality. | H/S–M |
| 5 | C10 | Copy-only; fixes identity drift (pen-pal) and three false/obsolete statements. | M/S |
| 6 | C5 | Removes the organizer's DB dependency for nudging and failure handling — the solo operator's main time sink. | H/M |
| 7 | C6 | Turns the spec's forwarding protocol into an on-screen instruction exactly when the organizer is under pressure. | H/M |
| 8 | C7 | Connects the app to the ritual's payoff; closes declared-but-unbuilt "meetup window". | H/M |

Next tier: C9 (S), C11 (S), C12 (S), C8 (M).
