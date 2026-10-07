# Code Quality Review: worktree-agent-ad7158d9abfc86219

Worktree: /home/user/same-te-mail-club/.claude/worktrees/agent-ad7158d9abfc86219
Branch: worktree-agent-ad7158d9abfc86219
Diff range: bbb9a13^..bbb9a13 (unit P-COPY; locales/uk.json only, 10+/10-)
Reviewed at: 2026-10-07T22:00Z
Spec verdict (A3): PASS (spec-worktree-agent-ad7158d9abfc86219-2130.md)
Gates: no build, fmt, clippy or test run. The diff is JSON-only and the dispatch said to create no target/. Key count (225 to 225) and exact values were already machine-checked by the spec reviewer and were not repeated here.

## Code Quality Review

### Summary
Ten existing values in `locales/uk.json` were replaced with the planned strings. Each claim in the new copy was checked against current source and the product spec:
- `home_send_instructions`: about 2 days and about 100 UAH, document tier, sender pays. These match Product Spec §5 and Personas.
- `season_advance_blocked_hint`: generating the assignments is what clears `advance_blocked`. `assignments_released` = `COUNT(DISTINCT a.sender_id) > 0`, state.rs:150. The «Розподіл» section renders after the season section (page.rs:264-275 vs h2 page.rs:1084), so "нижче" is true.

The copy itself is clear, consistent in tone and an improvement. Two strings point at UI or behaviour that this unit does not provide; other units add it. That makes integration order a hard constraint.

### Strengths
- Participant vocabulary converges on «отримувач». `sms_assignment_body` (uk.json:100) and `home_assigning_desc` (uk.json:29) now match `home_assigned_heading` "Твій отримувач" (uk.json:31). The participant-facing «призначення» and «додаток» (no app exists, it is a web page) are gone.
- The gender form is now uniform. `sms_receipt_nudge_body` (uk.json:102) moves from "Отримав/ла" to "Отримав(ла)", matching uk.json:34/36/37/43.
- False promises are removed. The old "Кому саме, дізнаєшся після реєстрації" contradicted the flow, because the recipient is revealed after assignment. "За кілька хвилин" was untrue during an organizer-driven phase. "сторінці Розподілу" named a page that does not exist; it is a section. "налаштуваннях акаунту" also named something that does not exist.
- `home_send_instructions` (uk.json:147) turns the thin "send it" into an actionable instruction: deadline, carrier, tariff, cost and who pays. All of it is traceable to Product Spec.md:42.
- `home_error_no_delivery_address` (uk.json:130) is short and sends the user to no phantom location.

### Critical Issues (Must Fix)
None.

### Important Issues (Should Fix)
1. **SMS bodies end in a bare colon until P-S lands (integration-order constraint)**
   - File: locales/uk.json:99, 100, 102, consumed at src/admin/sms.rs:191, 283, 465 (`let message = td_string!(...)`, sent verbatim)
   - Issue: "Реєструйся тут:", "дивись, кому надсилати:" and "Підтверди тут:" are only complete sentences once P-S's `with_site_link` appends `" {site_url}"`. Under P-COPY alone, a live send tells the participant to act "тут:" and gives no "тут". The coupling is implicit: no type or test ties these 3 keys to the link appender. This is a braid between two units with separate write-sets.
   - Impact: if P-COPY reaches a deployed build without P-S, every season-open, assignment and receipt-nudge SMS goes out broken. SMS is the only push channel.
   - Fix (integration rule; nothing changes inside P-COPY's write-set):
     - (a) Integrate P-S before P-COPY, or in the same push. Never push main with P-COPY and without P-S.
     - (b) After both land, verify with `grep -c "let message = with_site_link" src/admin/sms.rs` == 4. That is P-S's own gate, and it closes this coupling.
     - (c) If P-S is dropped or deferred, revert these 3 values before push.
     - Optional hardening for P-S, not P-COPY: a unit test that each of the 3 body keys ends with ":", asserted together with `with_site_link`. This turns the implicit contract into a checked one.
2. **`home_reported_label` instructs "познач це нижче" with no control below until P-H2 lands (second coupling, not in the spec review)**
   - File: locales/uk.json:41, rendered at src/pages/home.rs:1118-1125 (`render_receipt_confirmed`, `NotReceived` arm)
   - Issue: at this HEAD the `NotReceived` arm renders only the heading and this paragraph. There is no form and no button below it, and `confirm_receipt` still only updates rows WHERE `receipt_status = 'no_response'`, so the state is irreversible. The "Лист усе ж прийшов" control arrives with P-H2 (key `home_received_after_all_button`, plan D2/D3).
   - Impact: if P-COPY integrates without P-H2, a participant who reported "not received" is told to mark a late arrival. The page offers no way to do it, and the server would reject it anyway. This is a new false instruction of the very kind P-COPY exists to remove.
   - Fix (integration rule): treat P-H2 like P-S. Integrate P-H2 before P-COPY or in the same push. If P-H2 is deferred, temporarily drop the second sentence ("Якщо лист усе ж прийде — познач це нижче.") before push.

### Minor Issues (Nice to Have)
1. **Dead regex alternatives at mail_club.spec.ts:660**
   - File: end2end/tests/mail_club.spec.ts:660 `expectHomeContent(/розподіл|хвилин|надіслати/i)`
   - Issue: "хвилин" and "надіслати" matched the removed `home_assigning_desc`. Neither appears in the new Assigning state copy. The assertion now passes only through the h1 "Розподіл" (home.rs:1214). Dead alternatives mislead readers into thinking the description is asserted. A `<main>` text regex also violates the README's testid-only selector contract in spirit.
   - Impact: test-clarity only. The test still passes and still discriminates the Assigning state by its heading.
   - Fix: not P-COPY's write-set, which excludes TS. Route it to P-H1, which rewrites the Assigning and non-participant states and touches this test. Replace it with a testid assertion on the assigning state, or at minimum narrow it to `/розподіл/i`. Do not add the new copy's words, because copy-coupled regexes are what made these branches dead.
2. **Near-duplicate phrasing on the enrollment screen**
   - File: locales/uk.json:11 ("створи щось своє") next to uk.json:15 `home_guideline` ("Створи щось від себе"), rendered consecutively at home.rs:809/813
   - Issue: two adjacent paragraphs open with nearly the same imperative.
   - Impact: slight redundancy in the copy; nothing is wrong.
   - Fix: optional later copy polish. Out of the plan's fixed-string contract; do not reopen P-COPY for it.
3. **Pre-existing participant-facing «призначення» outside the diff**
   - File: locales/uk.json:135 `home_error_already_confirmed` ("…або призначення не знайдено.")
   - Issue: this is the last participant-facing use of the term this diff otherwise retires. Admin uses («Надіслати призначення», toast, released note) are an organizer register and acceptable.
   - Impact: minor term drift. Out of diff scope; noted for a future copy sweep only.
4. **Hard-coded tariff price will drift**
   - File: locales/uk.json:147 ("близько 100 грн")
   - Issue: the Nova Poshta price is an external fact. "близько" softens it, and it matches Product Spec, but it will age.
   - Impact: negligible now. Revisit when Product Spec's figure changes; the two should move together.

### Assessment
**Ready to merge:** Yes, integration-gated. P-COPY integrates only together with or after P-S (Important #1) and P-H2 (Important #2). If either is deferred, revert the affected values before push.
**Reasoning:** The 10 values are correct to plan, factually grounded in Product Spec and current source, and improve consistency (one term for "recipient", one gender-suffix form). The only risk is that 4 strings are completed by sibling units, so integration order must close that gap. Nothing in P-COPY's own write-set needs to change.
