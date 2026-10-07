# A-SCOPE — declared scope vs implementation (HEAD 1a3c8a5, read-only, no builds/tests run)

Bound manifestos read: stop-yapping, first-principles, kiss, dry. Commitments: terse fragments only; derive from code not docs; one table, no repeats; cross-reference not duplicate.

Legend. Status: OK+T = IMPLEMENTED+TESTED, OK-U = IMPLEMENTED-UNTESTED, PART = PARTIAL, MISS = MISSING, CONTR = CONTRADICTED.
`T:n` = end2end/tests/mail_club.spec.ts:n. `U:` = Rust unit test. "vacuous" = test passes whether or not the AC holds. Findings marked *(derived)* come from code reading, not execution.
Spec sources: spec/technical/User Stories.md (US), Architecture.md (ARCH), spec/product/Product Spec.md (PS).

## Trace table

| Story | AC | Status | Code | Test | Note |
|---|---|---|---|---|---|
| 1.1 | Login = universal entry (new + returning) | OK+T | src/pages/login.rs:39,126 | T:126,177 | |
| 1.1 | OTP-verified, no account -> prompt invite code | OK+T | login.rs:207-222 (pending_phone cookie, 300s) | T:134 | |
| 1.1 | Verified phone = account phone, no edit | OK+T | login.rs:391-399,430 | T:126,177 | phone read from cookie, not form |
| 1.1 | Valid code -> name -> account -> onboarding | OK+T | login.rs:382-474 | T:126,217,226 | |
| 1.1 | Invalid/used/revoked -> error, unlimited retry, no rate limit | OK+T | login.rs:313-360 | T:134,149,162 | retry loop untested; `register_with_code` failure paths redirect with NO error (login.rs:424-439) |
| 1.1 | Single-use code | OK+T | login.rs:413-447 (FOR UPDATE); CHECKs migrations/..03:11-13 | T:162 | no concurrency test |
| 1.1 | Records distributor (referral) | OK-U | migrations/..03:5 NOT NULL; admin/invite_codes.rs:83,141 | none asserts distributor | |
| 1.1 | Legal name required | OK-U | login.rs:401-405 (trim non-empty only); login.rs:1043 | no negative test | "matches gov ID" unverifiable in code |
| 1.1 | Deactivated phone cannot re-register | OK-U | users.phone UNIQUE (migrations/..02:3); login.rs:199-206 | none | T:885 covers sign-in only |
| 1.1 | Deactivated phones silently redirected pre-invite step | OK-U | login.rs:199-206 | T:885 asserts only `/login` URL = identical to unknown-phone outcome (vacuous) | ARCH:161 says "rejected at OTP request stage"; code sends OTP (login.rs:68-72,90). Spec-vs-spec conflict; code follows US |
| 1.1 | Codes = 2 Ukrainian words, dashes | OK+T | invite_codes.rs:252,262 | U: invite_codes.rs:315-409 | |
| 1.1 | Codes never expire | OK-U | no expiry column | none | |
| 1.2 | Phone sole auth; no password/email/social | OK+T | login.rs:39,126; auth.rs:76-228 | T:177 | |
| 1.2 | Long-lived sessions; re-auth only on expiry/new device | OK-U | auth.rs:246 (90d), login.rs:180 Max-Age 7776000, auth.rs:262-285 | none (no expiry test) | |
| 1.2 (ARCH) | OTP: 6-digit, hashed, 10-min expiry, 3 attempts/code, 60s + 5/h per phone | OK-U | auth.rs:76-158, login.rs:235-291 | T uses fixed "000000"; rate limit bypassed in test mode (auth.rs:77,121) | zero unit tests; E2E cannot reach expiry/attempt/rate-limit paths |
| 1.2 (ARCH) | Lock after 10 failed verifies/phone/hour; global SMS cap ~50/h | MISS | grep "global/circuit/lock" in src = 0 | none | ARCH:169,171 |
| 1.3 | Onboarding on first sign-in, before app | OK+T | app.rs:262-293 AuthGuard | T:194 | |
| 1.3 | Collects branch (required) | OK+T | onboarding.rs:19-77 | T:201 | |
| 1.3 | "Participant can update their branch later (during season enrollment or in settings)" | MISS | enrollment form hides inputs when address exists (home.rs:820-849); no settings route (app.rs:100-145); onboarding guard redirects onboarded users away (app.rs:283) | T:417 vacuous | see Gap 2 |
| 1.3 | Onboarding once; returning skip | OK+T | app.rs:283 | T:210 | |
| 1.4 | Logout on every authenticated page | OK+T | app.rs:195 (nav hidden only on /login,/onboarding) | T:901,914 (home, admin) | not shown on /onboarding |
| 1.4 | Server deletes session row | OK-U | login.rs:532, auth.rs:295 | T checks redirect only; no replay of old cookie | |
| 1.4 | Redirect to login; always succeeds | OK+T | login.rs:531-547 (errors ignored) | T:901,914 | corrupt-session path untested |
| 1.5 | Generator in admin participants area; pick participant or self | OK+T | admin/page.rs:1713; invite_codes.rs:83,266 | T:80 | only self (sole user) exercised |
| 1.5 | Every code linked to distributor | OK+T | schema NOT NULL | T:80 | |
| 1.5 | Code shown once for copy | OK+T | page.rs:1766-1775 | T:80 reads value | |
| 1.5 | List: code, distributor, status, redeemer | OK-U | invite_codes.rs:141-186 | T:98 status only | |
| 1.6 | List shows redeemer name + date | OK-U | invite_codes.rs:150-186 | none | |
| 1.6 | Revoke unused; revoked stays listed | OK+T | invite_codes.rs:193-262 | T:110,149 | |
| 1.6 | Used codes cannot be revoked | OK-U | invite_codes.rs:225 | none | |
| 1.6 | Filterable/scannable (by code, distributor) | OK-U | page.rs:2044-2067,1792-1852 | none in mail_club.spec.ts | |
| 2.1 | Per-season, no auto-enroll | OK+T | home.rs:473-484; enrollments PK | T:407 | |
| 2.1 | "Participant can update their Nova Poshta branch during enrollment" | MISS | home.rs:820-849 | T:417 vacuous | = row 1.3/AC3 |
| 2.1 | Enrollment closes at sign-up deadline | OK-U | home.rs:399 (server only) | U: home.rs:1244-1263 helper; E2E bypassed (home.rs:398) | UI keeps showing enroll button after deadline (EnrollmentOpen has no passed flag, home.rs:30-35) |
| 2.1 | "Participant sees the season timeline (creation deadline, expected meetup window)" | PART | home.rs:815 shows signup deadline only; meetup window absent (grep meetup/зустріч in src+locales = 0) | T:398 asserts theme text only | |
| 2.1 | Content guidelines displayed before opting in | PART | home.rs:813 + uk.json:15 (a creative brief) | T:390 labelled "guidelines" but asserts `season-theme` | PS "Content Guidelines"/"Ownership & Consent" prohibitions (illegal/threatening/hurtful, no photographing) not shown |
| 2.1 | No explicit withdrawal | OK-U | no withdraw fn | n/a | |
| 2.2 | Deliberate confirm action | OK+T | home.rs:954-967 | T:511 | |
| 2.2 | Given "enrolled participant" | PART | home.rs:162-186 shows confirm CTA to ANY non-confirmed user; home.rs:530-542 UPDATE hits 0 rows, returns Ok | none | Gap 4 |
| 2.2 | Button gone after deadline | OK-U | home.rs:181,932 | U: is_past_deadline only; E2E bypassed | |
| 2.2 | Unconfirmed excluded from graph | OK-U | admin/assignments.rs:275 | all 3 users confirm in T; exclusion unexercised | |
| 2.2 | Irreversible | OK+T | home.rs:535 latch | T:521 | UI only |
| 2.2 | Deadline visible "with countdown" | PART | home.rs:949-952 static formatted date | T:503 visibility only | no countdown anywhere (grep countdown = login OTP only) |
| 2.3 | Recipient real name, phone, branch in-app | OK+T | home.rs:196-250,1035-1081 | T:708 | asserts truthy/+380; never asserts recipient != self or correct pair |
| 2.3 | SMS nudge when available | OK+T | admin/sms.rs:238 (manual, per ARCH:37) | T:691 | server allows send in 'assignment' phase (sms.rs:256) pre-release; UI hides it |
| 2.3 | Only own recipient; nothing about sender | OK+T | home.rs:213 (sender_id=user) | T:720 | |
| 2.4 | Prompt "5 days after assignment" | PART | home.rs:1035-1083 form appears on assignment; no 5-day logic | T:745 | organizer-driven by design; AC timing not enforced |
| 2.4 | In-app prompt; SMS nudge if no response | OK+T | sms.rs:425 | T:729 | targeting (non-responders only) untested: fired before any response |
| 2.4 | Binary + optional note | OK+T | home.rs:557-633,972-1030 | T:745,754 | note text never asserted |
| 2.4 | "not received" triggers organizer notification | PART | state.rs:149, page.rs:637-641 (count only) | T:763 | note NEVER displayed: `receipt_note` written home.rs:621, read nowhere else (grep). Admin sees count, not who. Gap 3 |
| 3.1 | Organizer sees confirmed count first | OK+T | page.rs:599 | T:595 | |
| 3.1 | Every participant in exactly one cohort; each a single cycle | OK+T | assignment.rs:299,316 | U: assignment.rs:440-568 | |
| 3.1 | Target 11-15, any N>=3; 25 -> 13+12 | OK+T | assignment.rs:88-170 | U: assignment.rs:366-430 (25 at :403) | |
| 3.1 | Generation guard N>=3 | OK-U | assignments.rs:279 | none | DELETE of old rows (assignments.rs:259) precedes guard, no tx: failed regenerate wipes assignments |
| 3.2 | Known groups weighted, soft, lowest-score pick | OK+T | assignment.rs:47,178,197; assignments.rs:66-107 | U: assignment.rs:583-640 | no DB-integrated test; past pairings also weighted (PS Data Policy) |
| 3.3 | "Swaps must preserve the single-loop topology (the app validates this)" | CONTR | assignments.rs:318-416: UPDATEs applied (369-388) BEFORE validation (391-413), no tx/rollback | T:649 asserts only `cycle-visualization` visible (vacuous) | Gap 1 |
| 3.3 | Organizer sees full graph per cohort | OK+T | page.rs:1147,1400 | T:616 | |
| 3.3 | Assignments visible only on advance to Delivery | OK+T | home.rs:337 | T:628,708 | |
| 4.1 | Admin-only create; required deadlines; optional theme | OK+T | season.rs:20-96 | T:251,285 | negative validation (past/order) untested |
| 4.1 | Theme shown "during enrollment and creation period" | PART | enrollment: home.rs:799; Preparing state carries no theme (home.rs:40-44,946-952) | T:390,398 (enrollment only) | PS "Season theme display" |
| 4.1 | Not enrollable until launched | OK+T | home.rs:389; season.rs:77 | T:263 | |
| 4.1 | One active season | OK+T | migrations/..02 one_active_season | T:274 | |
| 4.2 | "Launch triggers SMS notification to all registered participants (Story 5.3)" | PART | season.rs:106-133 sends nothing; separate manual button sms.rs:155 | T:362 (separate step) | ARCH:37 decision "organizer triggers all SMS"; US wording stale |
| 4.2 | Enrollment opens on launch; no un-launch | OK+T | season.rs:112 | T:291,376 | |
| 4.3 | Admin only; any non-terminal phase; terminal not re-cancellable | OK+T | season.rs:191-211; types.rs:48 | U: types.rs:184-213; T:821 (from enrollment only) | admin-only server check untested |
| 4.3 | Confirmation step | OK+T | page.rs:733-790 | T:810,821 | |
| 4.3 | "Distinct 'season cancelled' state, not the generic 'no active season' message" | OK-U | home.rs:318,1229-1235 (`season-cancelled`) | T:833 regex `/no season\|немає сезону\|SMS/` matches generic AND cancelled copy (uk.json:45 contains "SMS"); testid never asserted | vacuous |
| 4.3 | "Cancel is only available in the UI after launch" | CONTR | page.rs:733 branch = `!is_terminal`, ignores `launched` (comment "any time before terminal") | none | MINOR |
| 4.3 | No SMS on cancel; destructive styling | OK-U | season.rs:184-222; page.rs:745 | none | |
| 4.4 | Season-open count | OK+T | sms.rs:48; page.rs:876 | T:369 | |
| 4.4 | Assignment "N senders (M not yet notified)" | OK+T | sms.rs:68; page.rs:966 | T:696-704 | shows M only |
| 4.4 | Confirm-nudge count | OK+T | sms.rs:83; page.rs:917 | T:491 | |
| 4.4 | Receipt-nudge count | OK-U | sms.rs:104; page.rs:1003 | none | |
| 4.4 | Counts update after send | OK+T | page.rs:72-79 (version refetch) | T:704 (assignment only) | |
| 4.5 | Single /admin page; season + participants sections | OK+T | app.rs:136; page.rs:289 | T:302 | |
| 4.5 | Phase-aware morph; no-season -> create form; terminal -> summary + "create new" | OK+T | page.rs:314-318,796-812 | T:251,801,821 | deferred_items.md last Open bullet ("create-form unreachable once any season exists") is STALE: AfterTerminalSeason form exists (page.rs:803) and T:801 uses it |
| 4.6 | Only phase-relevant SMS, hidden not disabled | OK+T | page.rs:824-1020 | T:332,459,570,672 | |
| 4.7 | Assignment->Delivery advance disabled until generated | OK+T | page.rs:537 | T:605,616 | UI-only: season.rs:144-176 + types.rs:28 allow advance with 0 assignments |
| 4.7 | Other transitions enabled | OK+T | types.rs:28 | T:348,450 | ARCH:518 "advance disabled until confirm deadline passes" not implemented |
| 4.8 | Swap via two name `<select>`s, UUID value, only after generate | OK+T | page.rs:1572,1112 | T:638,649 | |
| 5.1 | SMS has no recipient details; sent once | OK-U | uk.json:99; sms.rs:238-312 (notified_at IS NULL) | T:691 triggers once | body + once-only not asserted |
| 5.2 | Only non-responders; single reminder | OK-U | sms.rs:425-496 | T:729 (before any response) | |
| 5.3 | All active participants; informational | OK+T | sms.rs:155-235 | T:362 (count 3) | deactivated exclusion untested |
| 5.4 | Enrolled+unconfirmed only; single; none to confirmed | OK-U | sms.rs:327-420 | T:495 (before any confirm) | filter untested |
| 5.4 | "~1 hour before deadline" | PART | manual button; deadline only used in message text (sms.rs:351-378); no scheduler (grep spawn = pool logger only, main.rs:68) | T:495 | ARCH:37 by design; US wording implies automatic |
| 6.1 | Admin-only deactivate | OK+T | participants.rs:76-101 | T:877 | no guard against deactivating self/admin |
| 6.1 | Cannot sign in or enroll | OK-U | auth.rs:334; participants.rs:92 (sessions deleted) | T:885 vacuous (equals unknown-phone result); enroll block untested | |
| 6.1 | Excluded from season-open SMS | OK-U | sms.rs:182 `status='active'` | none | |
| 6.1 | Re-entry = new invite + different phone | OK-U | phone UNIQUE | none | |
| PS Entry | "No self-registration... organizer manually signs them up" | CONTR (doc) | login.rs:382 self-registers via invite code | T:126 | PS stale vs US 1.1/1.5; code follows US. PS "App Scope: Registration (organizer creates account)" same |
| PS Failure protocol | Organizer learns who got nothing -> forwarding | PART | count only (page.rs:637) | T:763 | = Gap 3 |
| PS Privacy | Sender sees recipient name/phone/branch only; recipient learns nothing | OK+T | home.rs:196-250 | T:708,720 | |
| PS Cohorts | Parallel cohorts when pool > 15 | OK+T | assignment.rs:88 | U: :390-430,554-568 | no E2E with >15 |
| PS Success | One cohort completes a season end-to-end | OK+T | full chain | T:72-846 (3 participants) | no >=11 cohort E2E; cohort cycle-viz only in capture spec |
| PS Platform | No Telegram | OK-U | grep telegram src = 0 | n/a | |

## Gaps (severity-ranked)

1. **BLOCKER — 3.3 swap corrupts assignments.** `swap_assignment` (src/admin/assignments.rs:369-388) commits both UPDATEs on the pool, then validates (391-413). Failure returns Err but the broken mapping stays. *(derived)* Exchanging the recipients of two senders in one cycle always splits it (two loops, or a self-loop when B is A's recipient), so every in-cohort swap fails validation after persisting. A cross-cohort swap merges two cohorts into one over-size loop and passes validation (validator treats all rows as one cohort, assignments.rs:187-213). `advance_season` (season.rs:144) does not re-validate, so a corrupted graph can be released. Test T:649 passes regardless (asserts only that the viz renders), T:708/720 assert truthiness only. Recommend: run a real swap test on 3+ users asserting graph validity + no self-assignment, and make swap transactional with a position-exchange semantic.
2. **MAJOR — Branch cannot be updated in-app (1.3 AC3, 2.1 AC2).** Every user is onboarded (forced), so the enrollment form always takes the "saved address" branch with no inputs (home.rs:820-849); no settings route (app.rs:100-145). A wrong branch means mail goes to the wrong Nova Poshta branch; fix needs organizer SQL. T:417 is vacuous: POM skips the fill when inputs are absent (end2end/tests/fixtures/mail_club_page.ts:161).
3. **MAJOR — Receipt note and reporter identity invisible to organizer (2.4, PS Failure Protocols).** Admin sees only `Не отримано: N` (page.rs:637-641, uk.json:84). `receipt_note` has no reader outside its INSERT (home.rs:621). The forwarding protocol needs the identity of the non-receiver; ARCH:520 promised per-participant receipt status.
4. **MAJOR — Non-enrolled users see a live "confirm ready" CTA in Preparation** (home.rs:162-186; no enrollment check) and the click silently no-ops (home.rs:530-542; doc comment at :496 says it errors). Users who never enrolled believe they are in the graph. Also Delivery/Assignment show "Assigning" forever to non-participants (home.rs:223-225).
5. **MAJOR (unverifiable) — Auth hardening untested and partly missing.** No unit/E2E for OTP expiry, attempt cap, rate limits, session expiry (E2E runs `SAMETE_TEST_MODE`, which fixes the OTP to 000000 and disables rate limits and deadline gates: auth.rs:77,121; home.rs:180,398,521). No guard stops that env var in production. ARCH-declared per-phone hourly lockout and global SMS cap are MISSING.
6. **MAJOR (test quality) — vacuous/mislabelled assertions** hide spec gaps: T:417 (branch update), T:390 (guidelines vs theme), T:398 (timeline), T:503 (countdown), T:649 (swap), T:833 (cancelled state), T:184 "unregistered phone rejected" (US 1.1 routes it to invite step; only asserts URL stays /login), T:885 (deactivated sign-in).
7. **MINOR — 2.1 timeline incomplete:** no creation deadline on the enroll screen, no meetup window anywhere; guideline copy omits prohibitions and consent rules; theme absent in Preparation; no countdown.
8. **MINOR — 4.3:** cancel button available pre-launch (CONTR vs AC). Generation not transactional (assignments.rs:259 DELETE before N>=3 guard at :279). Assignment SMS server-allowed pre-release. Advance Assignment->Delivery server-side unguarded. Deactivated users still counted in confirm/receipt nudge targets and the cohort pool (no `status` filter in assignments.rs:273-276, sms.rs nudge queries). Deactivate has no self/admin guard.
9. **DOC DRIFT:** PS says no self-registration (stale); ARCH:161 deactivated-rejected-at-OTP vs code/US silent redirect; US 4.2/5.2/5.4 imply automation vs ARCH:37 manual; deferred_items.md "create-form unreachable" bullet stale (see row 4.5).

## Scope creep (implemented, no spec backing)

- OTP resend cooldown UI (login.rs:727-889).
- Stepper / compact mobile stepper, toast system, skeletons, dark mode, grain overlay (src/components/*, style/).
- `EnrollmentNotOpen` home state (home.rs:27) and pre-launch participant count (page.rs:603).
- Cycle-viz SVG geometry scaling for cohort size (page.rs:1167-1400, 13 unit tests).
- Free-text invite-code filter with result count (page.rs:1792-1852; US only says "filterable").
- Idempotency tables `season_open_notifications` + nudge timestamp columns (migrations/20260624000001) — supports "sent once", spec'd implicitly.
- Capture/tooling: visual-audit specs, cohort seed, isolated-capture harness.

## Coverage counts (95 trace rows incl. 1.2-ARCH and PS rows)

| Status | Rows |
|---|---|
| IMPLEMENTED+TESTED | 52 |
| IMPLEMENTED-UNTESTED | 27 |
| PARTIAL | 10 |
| MISSING | 3 (1.2 ARCH lockout/global cap; 1.3 branch update; 2.1 branch update) |
| CONTRADICTED | 2 code (3.3 swap; 4.3 cancel-after-launch) + 1 doc (PS self-registration) |

"IMPLEMENTED+TESTED" counts a row when any test touches the behavior; ~9 of those 52 rest on weak/vacuous assertions (Gap 6), so effective verified coverage is lower. Counts via awk over the table.
