# Implementation Plan: Wave 2 — Product Improvements (W2)

Binding read by author: first-principles, simple-made-easy, kiss, yagni, correct-by-construction, stop-yapping; defensive-planning skill.
Executors: sonnet implementers, one unit each, isolated worktree, spec-review → quality-review → orchestrator integrates. Guidance-doc unit (W2-DOC) = opus (conventions.md hard override).

## Preamble

Wave 1 (`orchestration_log/history/2026-10-04/plans/PLAN-W1.md`: U1 registration tickets, U2 test-mode confinement, U3 admin bootstrap, U4 swap) fixes launch blockers. Wave 2 removes product dead ends a participant or the solo organizer hits every season. Evidence verified against source at HEAD `1a3c8a5` before planning (file:line below). Line numbers drift after wave 1: locate code by SYMBOL, never by line.

| Problem (verified) | Evidence | Unit |
|---|---|---|
| Recipient card vanishes when participant confirms OWN receipt; send and receive are independent | `src/pages/home.rs` `resolve_delivery_state`: `ReceiptConfirmed { status }` replaces `Assigned`, carries no recipient | W2-H2 |
| "Не отримав(ла)" irreversible; tapped on day 0 = permanent false forwarding signal | `confirm_receipt`: `WHERE … receipt_status = 'no_response'`; form rendered with the card from day 0 (`render_assignment_details` → `render_receipt_form`) | W2-H2 |
| Non-enrolled participant sees live "Лист готовий" CTA in Preparation; click silently no-ops | `resolve_preparation_state` checks only `confirmed_ready_at`; `confirm_ready` UPDATE hits 0 rows → `Ok(())` | W2-H1 |
| Non-participants told "За кілька хвилин дізнаєшся…" for the whole Assignment+Delivery window, thanked at Complete | `get_home_state`: `Phase::Assignment => Ok(HomeState::Assigning)`; `resolve_delivery_state`: no outgoing → `Assigning`; Complete for everyone | W2-H1 |
| SMS bodies carry no link to the web app | `uk.json` `sms_*_body`; `src/admin/sms.rs` sends the bare key; no site URL in `src/config.rs` | W2-S |
| Organizer sees counts only — no names for unconfirmed / no-response / not-received; `receipt_note` has no reader | `src/admin/state.rs` `AdminSeason` (integers only); `grep -rn receipt_note src` → written in `confirm_receipt`, read nowhere | W2-R |
| Forwarding protocol (Product Spec §Non-compliance, §Contiguous failures) computed mentally | `AssignmentLink` has no receipt status | W2-F |
| Zero meetup info anywhere; Story 2.1 AC "expected meetup window" | `grep -rni meetup src migrations` = 0 | W2-M1, W2-M2 |
| Branch cannot be changed after onboarding (Stories 1.3 AC3, 2.1 AC2) | `render_enrollment_open` `Some(existing_address)` arm renders only hidden inputs; no settings route | W2-H3 |
| Copy: pen-pal framing, false "після реєстрації", "кілька хвилин", phantom "сторінка Розподілу", phantom "налаштування акаунту", thin shipping guidance | `uk.json` keys listed in W2-U0 | W2-U0 |
| Vacuous E2E assertions | `mail_club.spec.ts` titles listed in W2-V / W2-H3 / W2-M2 | W2-V, W2-H3, W2-M2 |

## Design decisions (final — do not revisit)

| # | Decision | Rationale |
|---|---|---|
| D1 | **Include C1** (recipient card persists). `HomeState::Assigned` + `ReceiptConfirmed` merge into one `Delivery { recipient: RecipientCard, own_receipt: ReceiptStatus }`. | One variant per real situation; the card can no longer be dropped by construction. |
| D2 | **C2 partial.** Make `NotReceived → Received` reversible (late arrival); add a timing hint above the form. **Drop time-gating** of the form. | Gating needs a delivery-start timestamp that does not exist (new column + phase bookkeeping). Reversibility removes the only irreversible harm. Spec timing is organizer-driven (receipt-nudge SMS). |
| D3 | Receipt transitions live in one rule: `ReceiptStatus::can_transition_to` (types.rs). `Received` is final. | Rules gathered, not scattered. A mistaken "received" causes no false alarm → no correction path needed (YAGNI). |
| D4 | **Include C3 + a-scope Gap 4.** New `HomeState::NotParticipating`; `confirm_ready` errors when not enrolled. | Kills a false promise shown to every non-participant every season. |
| D5 | **Include C4.** `SAMETE_SITE_URL` env, required (https) when SMS are live, defaulted in dry-run; appended to all 4 participant SMS. OTP SMS untouched. | Every SMS currently leads nowhere clickable. Cost: Cyrillic UCS-2 bodies + ~25-char URL ≈ 2 segments (~$0.05/SMS, ≈ $3/season at 15 people × 4 SMS) — accepted. |
| D6 | **Include C10** as one i18n-only unit (W2-U0) that also adds EVERY new wave-2 key. | Single writer of `locales/uk.json` → no cross-unit conflicts. Unused keys compile fine; orphan gate at the end. |
| D7 | **Include C5 + a-scope "notes + who"** as a season roster table in the admin season card (W2-R). | Removes the organizer's DB dependency for nudging and forwarding. No new CSS: reuses `.data-table`. |
| D8 | Roster status is an enum with notes inside the variants that can carry them. Badge mapping: Unconfirmed/AwaitingReceipt → `pending` (amber, open), Confirmed → `ready` (blue), Received → `confirmed` (green), NotReceived → `error` (red — a delivery failure; the amber `.alert` count banner stays as is). | Correct by construction; badge families per design-system §Badges. |
| D9 | **Include C6** as a pure function over the cycle chain (W2-F). Request = (first failed sender of each contiguous failed block) → (recipient of the block's last failed sender). No requests when nobody or everybody failed. | Product Spec rule is mechanical; pure fn is unit-testable exhaustively. |
| D10 | **Include C7** display-only: one nullable `seasons.meetup_details TEXT` (≤300 chars, CHECK), set/cleared by the organizer from the admin season card in any non-cancelled phase (incl. Complete); shown to participants of the current season (incl. enrollment screen). **No meetup SMS, no RSVP.** | Meetup is the ritual's payoff; Story 2.1 AC. RSVP is an explicit spec exclusion; SMS = YAGNI. |
| D11 | Meetup is NOT on the create-season form. | One place to set it; meetup usually fixed after creation. |
| D12 | **Branch edit (a-scope Gap 2)** = "Змінити адресу" button in the enrollment saved-address card (UI-mode signal) that swaps in the existing city/number fields; server contract (`use_existing_address`) unchanged. **No settings page.** | Enrollment is the spec'd per-season checkpoint; a settings page adds a route + guard for no extra spec value. Signal holds view mode only, never input values (leptos-idioms compliant). |
| D13 | Enrolled state shows the address the participant is enrolled with. | Confirms the change to the user; makes the E2E assertion an actual effect. |
| D14 | Story 2.1 timeline: enrollment screen shows signup deadline + creation (confirm) deadline + meetup when set. | Closes a-scope 2.1 PART. |
| D15 | Dropped (YAGNI / out of scope): C8 season edit, C9 organizer SMS on not-received, C11 theme persistence, C12 deadline-closed enroll form, C13–C18, countdown (Story 2.2), receipt-form time gate, settings page, meetup SMS. | Not in wave-2 scope list or cost > value now. |
| D16 | Swap E2E (T:649) is owned by wave-1 U4 — not touched here. | Avoid overlap. |
| D17 | Pixel evidence: new UI states that the existing visual-audit flow reaches get one `captureState` each (roster+forwarding A43, not-received-reported H7c, change-address H2b, meetup everywhere via audit meetup set at launch). `NotParticipating` is not captured: it reuses the captured `.empty-state` template verbatim. | Spec-reviewer judges rendered pixels (conventions §Visual pipeline) at minimum added capture cost. |

## Write-sets

| Unit | Files written |
|---|---|
| W2-U0 | `locales/uk.json` |
| W2-S | `src/config.rs`, `src/admin/sms.rs`, `README.md` |
| W2-V | `end2end/tests/mail_club.spec.ts`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/fixtures/capture-constants.ts` |
| W2-H1 | `src/pages/home.rs`, `.sqlx/*`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/fixtures/mail_club_page.ts` |
| W2-R | `src/admin/state.rs`, `src/admin/page.rs`, `.sqlx/*`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/visual-audit.spec.ts` |
| W2-F | `src/admin/assignments.rs`, `src/admin/page.rs`, `.sqlx/*`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/visual-audit.spec.ts` |
| W2-H2 | `src/types.rs`, `src/pages/home.rs`, `.sqlx/*`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/visual-audit.spec.ts` |
| W2-H3 | `src/pages/home.rs`, `.sqlx/*`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/visual-audit.spec.ts` |
| W2-M1 | `migrations/20261004000002_add_season_meetup_details.sql` (new), `src/admin/season.rs`, `src/admin/state.rs`, `src/admin/page.rs`, `.sqlx/*`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/fixtures/mail_club_page.ts`, `end2end/tests/visual-audit.spec.ts` |
| W2-M2 | `src/pages/home.rs`, `.sqlx/*`, `end2end/tests/mail_club.spec.ts`, `end2end/tests/fixtures/mail_club_page.ts` |
| W2-DOC (opus) | `guidance/design-system.md` |

## Dependency / parallelism graph

```
[wave 1 fully integrated + its Global Gates green on main]
        │
        ├── W2-U0 ─┐
        ├── W2-S  ─┤   (parallel: disjoint write-sets)
        └── W2-V  ─┤
                   ▼
                W2-H1 → W2-R → W2-F → W2-H2 → W2-H3 → W2-M1 → W2-M2
                                 │
                                 └── W2-DOC (any time after W2-R integrated; parallel)
```
- Every unit after the first row edits `mail_club.spec.ts` + POM; E2E tests are anchored to titles introduced by predecessors (the serial chain's DB order matters) → strictly sequential: each unit branches from main AFTER its predecessor is integrated.
- Rust lanes collapse onto the same chain: home.rs (H1, H2, H3, M2), page.rs (R, F, M1), state.rs (R, M1).
- W2-H1 needs W2-U0 (keys) and W2-V (title renames) integrated. W2-S needs nothing but wave 1.
- Integration order: U0, S, V (any order among them), H1, R, F, H2, H3, M1, M2, DOC.

## Pre-flight gate (EVERY unit, before any edit)

```bash
git log --oneline -1
grep -c "pub fn test_mode" src/config.rs
grep -c "pub recipient_id: String" src/admin/assignments.rs
ls migrations | grep -c registration_tickets
```
**REQUIRED:** `1`, `1`, `1`. Any `0` → STOP, report BLOCKED ("wave 1 not integrated").
Additionally, for every unit except U0/S/V: every predecessor in the graph shows in `git log --oneline` (orchestrator gives SHAs in the dispatch). Missing → BLOCKED.

## Shared environment (every unit)

Identical to `orchestration_log/history/2026-10-04/plans/PLAN-W1.md` §"Shared environment", restated:

Postgres:
```bash
pg_isready -h localhost -p 5432 || docker compose -f /home/user/same-te-mail-club/docker-compose.yml up -d
until pg_isready -h localhost -p 5432; do sleep 1; done
```
**REQUIRED:** `localhost:5432 - accepting connections`.

Per-unit sibling DB (`<unit>` = `w2u0|w2s|w2v|w2h1|w2r|w2f|w2h2|w2h3|w2m1|w2m2`):
```bash
export DATABASE_URL=postgres://samete:samete@localhost:5432/samete_<unit>
sqlx database drop -y; sqlx database create && sqlx migrate run
```

sqlx cache (after ANY change to a `query!`/`query_as!`/`query_scalar!` string, including whitespace):
```bash
cargo sqlx prepare --workspace -- --features ssr
git status --short .sqlx
```
**REQUIRED:** exit 0; added/modified `.sqlx/query-*.json` for every new/changed query; `ls .sqlx | wc -l` ≠ 0. Commit `.sqlx/`.

Baseline (on the branch point, BEFORE editing; paste in report):
```bash
cargo test 2>&1 | grep -E "^test result"
SQLX_OFFLINE=true cargo test --features ssr 2>&1 | grep -E "^test result"
```
Sum `passed` across lines → `BARE_BASE`, `SSR_BASE`.

Standard gates (after the unit's last change, in order):
```bash
cargo fmt --all -- --check
SQLX_OFFLINE=true cargo clippy --features ssr --no-default-features -- -D warnings
SQLX_OFFLINE=true cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings
cargo test 2>&1 | grep -E "^test result"
SQLX_OFFLINE=true cargo test --features ssr 2>&1 | grep -E "^test result"
```
**REQUIRED:** fmt prints nothing, exit 0. Both clippy runs end `Finished`, zero `warning:`/`error:` lines. Every `test result:` line `ok.` with `0 failed`. Summed `passed` = `BARE_BASE + <unit bare delta>` and `SSR_BASE + <unit ssr delta>` exactly (deltas given per unit).

E2E gate (isolated harness; never `just e2e*`, never :3000, never DB `samete`):
```bash
export PLAYWRIGHT_BROWSERS_PATH=/opt/pw-browsers
[ -d end2end/node_modules ] || (cd end2end && npm ci)
bash scripts/isolated-capture.sh e2e_<unit>_<run> full > /tmp/e2e_<unit>_<run>.log 2>&1; echo "exit=$?" >> /tmp/e2e_<unit>_<run>.log
grep -E "exit=|[0-9]+ (passed|failed|flaky|skipped)" /tmp/e2e_<unit>_<run>.log
```
Run with `run_in_background`; read the log with `grep`/`tail` on the FILE. **REQUIRED:** `exit=0`; one `N passed` line with N = baseline passed + unit E2E delta; no `failed`; no `flaky`; `2 skipped`. Every title named in the unit appears passed (`grep -F "<title>" /tmp/e2e_<unit>_<run>.log`). Runs `<run>` = 1,2,3 — all three must pass (W2-U0: one run). Record the E2E baseline passed count from wave-1's final run (orchestrator supplies it in the dispatch; if absent, run the gate once on the branch point before editing).

Pixel evidence (units with UI change): after run 3, list the PNGs named in the unit under `end2end/screenshots/{light-desktop,light-mobile,dark-desktop,dark-mobile}/` (`ls … | grep <name>`); the spec-reviewer reads them at native resolution, both viewports, both modes.

Commit: one-line conventional commit, no body, no `Co-Authored-By`, no AI attribution (CLAUDE.md). Commit in the worktree only; report SHA.

Read hygiene: chunked reads ≤400 lines; never read `node_modules`, `target`, `package-lock.json`, screenshots (except the spec-reviewer).

---

# W2-U0 — i18n foundation + copy fixes

## Why This Matters
Single writer of `locales/uk.json` for the whole wave; fixes identity-contradicting and false copy (j-feel P6–P9, O7; a-scope).

## What You Must Do
1. Replace the VALUES of these existing keys (key names unchanged), exactly:
```json
"home_enroll_invitation": "Новий сезон клубу відкрито. Долучайся: створи щось своє — і хтось із клубу отримає це поштою.",
"home_enroll_expectation": "Ти надішлеш один лист і отримаєш один — від когось із клубу. Кому надсилати, дізнаєшся після розподілу, коли твій лист уже буде готовий.",
"home_assigning_desc": "Організатор формує пари. Щойно все буде готово, надішлемо SMS — і тут з'явиться адреса твого отримувача.",
"home_send_instructions": "Відправ лист протягом 2 днів Новою Поштою (тариф «Документи», близько 100 грн — сплачуєш ти) на це відділення.",
"home_reported_label": "Організатор уже знає і зв'яжеться з тобою, якщо знадобиться пересилання. Якщо лист усе ж прийде — познач це нижче.",
"season_advance_blocked_hint": "Спочатку згенеруй розподіл у розділі «Розподіл» нижче.",
"home_error_no_delivery_address": "Спочатку вкажи адресу доставки.",
"sms_season_open_body": "Новий сезон «Саме Те» відкрито! Реєструйся тут:",
"sms_assignment_body": "Твій отримувач уже відомий — дивись, кому надсилати:",
"sms_receipt_nudge_body": "Отримав(ла) лист? Підтверди тут:"
```
2. Append these NEW keys, in this order, as the last entries of the object (preserve valid JSON: comma after the previous last entry):
```json
"home_not_participating_heading": "Цей сезон — без тебе",
"home_not_participating_body": "Ти не береш участі в поточному сезоні. Коли відкриється реєстрація на наступний — надішлемо SMS.",
"home_error_not_enrolled": "Ти не зареєстрований(а) в цьому сезоні.",
"home_receipt_timing_hint": "Відповідай, коли лист прийде. Якщо за 5 днів після SMS про отримувача нічого немає — натисни «Не отримав(ла)».",
"home_received_after_all_button": "Лист усе ж прийшов",
"home_change_address_button": "Змінити адресу",
"home_enrolled_address": "Доставка: відділення №{{ branch_number }}, {{ city }}.",
"home_enroll_confirm_deadline": "Лист має бути готовий до {{ deadline }}.",
"home_meetup_label": "Зустріч",
"admin_meetup_label": "Зустріч учасників",
"admin_meetup_placeholder": "Дата, час і місце — напр.: 15 листопада, 18:00, Арт-простір «Купол»",
"admin_meetup_hint": "Учасники сезону бачать це на головній сторінці. Порожнє поле прибирає оголошення.",
"admin_meetup_save_button": "Зберегти",
"admin_meetup_saving_loading": "Зберігаю...",
"admin_meetup_saved_toast": "Зустріч збережено!",
"season_error_meetup_too_long": "Опис зустрічі задовгий — максимум 300 символів.",
"season_error_no_season_for_meetup": "Немає сезону, для якого можна оголосити зустріч.",
"admin_roster_title": "Учасники сезону",
"admin_roster_note_column": "Примітка",
"admin_roster_status_unconfirmed": "Не підтвердив(ла)",
"admin_roster_status_confirmed": "Готовий(а)",
"admin_roster_status_awaiting_receipt": "Чекає на лист",
"admin_roster_status_received": "Отримав(ла)",
"admin_roster_status_not_received": "Не отримав(ла)",
"admin_forwarding_title": "Пересилання",
"admin_forwarding_description": "Хтось не надіслав свій лист. Попроси першого учасника кожного зламаного відрізка переслати отриманий лист далі:",
"admin_forwarding_request": "{{ from }} → {{ to }}"
```
3. No other key added, removed, renamed, or re-valued. No Rust/TS change.

## Verification Gates
```bash
python3 -c "import json;d=json.load(open('locales/uk.json'));print(len(d))"
```
**REQUIRED:** previous count + 27.
```bash
git diff --stat
```
**REQUIRED:** only `locales/uk.json`.
Standard gates (bare/ssr deltas 0). E2E gate ×1 (delta 0) — proves no copy-regex test broke.

Commit: `fix(i18n): correct participant copy and add wave-2 strings`

---

# W2-S — Site link in every participant SMS

## Why This Matters
SMS are the only push channel; a monthly web app without a link loses participants. (j-feel P11/C4.)

## What You Must Do
### S.1 `src/config.rs` (post-U2/U3 shape: `Config { database_url, sms: SmsMode, admin_bootstrap, … }`) — TDD
Step 1 — add to the existing `#[cfg(test)] mod tests` (red):
```rust
    #[test]
    fn site_url_defaults_in_dry_run() {
        assert_eq!(
            site_url_from_var(None, &SmsMode::DryRun { test_mode: false }).ok().as_deref(),
            Some(DRY_RUN_SITE_URL)
        );
    }

    #[test]
    fn site_url_required_for_live_sms() {
        let live = SmsMode::Live { token: "t".into(), sender: "s".into() };
        assert!(matches!(site_url_from_var(None, &live), Err(ConfigError::MissingSiteUrl)));
    }

    #[test]
    fn site_url_trims_whitespace_and_trailing_slash() {
        let live = SmsMode::Live { token: "t".into(), sender: "s".into() };
        assert_eq!(
            site_url_from_var(Some(" https://club.example.ua/ ".into()), &live).ok().as_deref(),
            Some("https://club.example.ua")
        );
    }

    #[test]
    fn site_url_rejects_plain_http() {
        let dry = SmsMode::DryRun { test_mode: true };
        assert!(matches!(site_url_from_var(Some("http://club.example.ua".into()), &dry), Err(ConfigError::InvalidSiteUrl(_))));
    }

    #[test]
    fn site_url_rejects_bare_scheme() {
        let dry = SmsMode::DryRun { test_mode: false };
        assert!(matches!(site_url_from_var(Some("https://".into()), &dry), Err(ConfigError::InvalidSiteUrl(_))));
    }
```
Add `site_url_from_var` and `DRY_RUN_SITE_URL` to the test module's `use super::{…}`. Update the U2 test helper `config_with` literal to include `site_url: String::new(),`.
Step 2 — `SQLX_OFFLINE=true cargo test --features ssr config::tests` → **REQUIRED:** compile error (red).
Step 3 — implement:
```rust
/// Public URL used in dry-run mode (local dev / E2E), where SMS are only logged.
const DRY_RUN_SITE_URL: &str = "http://127.0.0.1:3000";

/// Parse `SAMETE_SITE_URL`: required (https) when SMS are live; dry-run defaults
/// to [`DRY_RUN_SITE_URL`]. Trailing `/` and surrounding whitespace are trimmed.
fn site_url_from_var(raw: Option<String>, sms: &SmsMode) -> Result<String, ConfigError> {
    let Some(raw) = raw else {
        return match sms {
            SmsMode::DryRun { .. } => Ok(DRY_RUN_SITE_URL.to_owned()),
            SmsMode::Live { .. } => Err(ConfigError::MissingSiteUrl),
        };
    };
    let url = raw.trim().trim_end_matches('/');
    let has_host = url.strip_prefix("https://").is_some_and(|host| !host.is_empty());
    if has_host {
        Ok(url.to_owned())
    } else {
        Err(ConfigError::InvalidSiteUrl(url.to_owned()))
    }
}
```
`ConfigError` add:
```rust
    #[error("missing SAMETE_SITE_URL (required when SMS are sent)")]
    MissingSiteUrl,
    #[error("SAMETE_SITE_URL must be an https:// URL, got {0}")]
    InvalidSiteUrl(String),
```
`Config` add last field:
```rust
    /// Public https URL of the app, appended to every participant SMS.
    pub site_url: String,
```
`from_env`: after `sms` is computed: `let site_url = site_url_from_var(std::env::var("SAMETE_SITE_URL").ok(), &sms)?;` and add `site_url` to the `Self { … }` literal. Extend the `from_env` doc with one sentence on `SAMETE_SITE_URL`.
Step 4 — rerun step-2 command → **REQUIRED:** `ok`, the 5 new tests pass.

### S.2 `src/admin/sms.rs` — TDD
Step 1 — append (red):
```rust
#[cfg(test)]
mod tests {
    use super::with_site_link;

    #[test]
    fn site_link_follows_body_after_a_space() {
        assert_eq!(
            with_site_link("Підтверди тут:", "https://club.example.ua"),
            "Підтверди тут: https://club.example.ua"
        );
    }
}
```
Step 2 — `cargo test sms::tests` → red. Step 3 — add above `// ── Server functions`:
```rust
/// Append the app link: every participant SMS asks the reader to act in the app.
#[cfg(any(feature = "ssr", test))]
fn with_site_link(body: &str, site_url: &str) -> String {
    format!("{body} {site_url}")
}
```
In the four `send_*_sms` fns replace the message construction:
- `send_season_open_sms`: `let message = with_site_link(td_string!(Locale::uk, sms_season_open_body), &config.site_url);`
- `send_assignment_sms`: `let message = with_site_link(td_string!(Locale::uk, sms_assignment_body), &config.site_url);`
- `send_confirm_nudge_sms`: `let message = with_site_link(&format!("{prefix}{deadline_str}."), &config.site_url);`
- `send_receipt_nudge_sms`: `let message = with_site_link(td_string!(Locale::uk, sms_receipt_nudge_body), &config.site_url);`
Every `sms::send_sms(&config, &http_client, &target.phone, …)` in this file passes `&message`.
Step 4 — rerun → green.

### S.3 `README.md`
In the Environment Variables table, add directly after the `TURBOSMS_SENDER` row:
```
| `SAMETE_SITE_URL` | Yes, unless `SAMETE_SMS_DRY_RUN=true` | Public `https://` URL of the app; appended to every participant SMS. Dry-run default `http://127.0.0.1:3000` |
```

## Verification Gates
```bash
grep -c "let message = with_site_link" src/admin/sms.rs
grep -n "send_sms(" src/admin/sms.rs
grep -rn "SAMETE_SITE_URL" src/
```
**REQUIRED:** `4`; exactly 4 `send_sms(` lines, each ending `&message)`; `SAMETE_SITE_URL` only in `src/config.rs`.
Boot gate:
```bash
SQLX_OFFLINE=true cargo build --features ssr
env -u SAMETE_SMS_DRY_RUN -u SAMETE_TEST_MODE -u SAMETE_SITE_URL DATABASE_URL=postgres://x TURBOSMS_TOKEN=t TURBOSMS_SENDER=s LEPTOS_SITE_ADDR=127.0.0.1:3923 ./target/debug/samete 2>&1 | grep -c "MissingSiteUrl"
```
**REQUIRED:** `1`, process exits non-zero immediately.
Standard gates: bare delta +1, ssr delta +6. E2E gate ×3, delta 0 (proves dry-run default boots).

Commit: `feat(sms): append site link to participant SMS`

---

# W2-V — Replace vacuous E2E assertions (test-only)

## Why This Matters
These tests pass whether or not their AC holds (a-scope Gap 6). Test-only; zero `src/` change.

## What You Must Do
### V.1 `end2end/tests/fixtures/capture-constants.ts` — append:
```ts
const UK_MONTHS_GENITIVE = [
  "січня", "лютого", "березня", "квітня", "травня", "червня",
  "липня", "серпня", "вересня", "жовтня", "листопада", "грудня",
];

/**
 * Mirror of src/date_format.rs::format_date_uk for a futureDeadline() value
 * ("YYYY-MM-DDTHH:MM", UTC — create_season stores it as UTC verbatim).
 */
export function formatDateUk(isoMinute: string): string {
  const [date, time] = isoMinute.split("T");
  const [year, month, day] = date.split("-").map(Number);
  return `${day} ${UK_MONTHS_GENITIVE[month - 1]} ${year}, ${time}`;
}
```
### V.2 POM (`mail_club_page.ts`) — add in the Auth section:
```ts
  async expectPhoneStep() {
    await expect(this.page.getByTestId("phone-input")).toBeVisible();
  }

  async expectNoInviteCodeStep() {
    await expect(this.page.getByTestId("invite-code-step")).not.toBeVisible();
  }

  /** No session: the authenticated home redirects to /login. */
  async expectNoSession() {
    await this.page.goto("/");
    await expect(this.page).toHaveURL(/\/login/);
  }
```
and in the Home Screen section:
```ts
  async expectSeasonCancelled() {
    await expect(this.page.getByTestId("season-cancelled")).toBeVisible();
  }

  async expectSeasonDeadline(text: string) {
    await expect(this.page.getByTestId("season-deadline")).toContainText(text);
  }
```
### V.3 `mail_club.spec.ts`
- Import: `import { ADMIN_PHONE, futureDeadline, formatDateUk } from "./fixtures/capture-constants";`
- Test `"1.2 — unregistered phone is rejected"` → rename `"1.1 — unregistered phone is routed to the invite-code step"`, body:
```ts
      const app = new MailClubPage(page);
      await app.reachInviteCodeStep("+380679999999");
      await app.expectNoSession();
```
- Test `"2.2 — ready-confirm deadline visible with countdown"` → rename `"2.2 — confirm deadline shown is the season's confirm deadline"`, body:
```ts
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectSeasonDeadline(formatDateUk(CONFIRM_DEADLINE));
```
- Test `"home screen — no active season after cancel"` → rename `"4.3 — participant sees the distinct cancelled-season state"`, body:
```ts
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectSeasonCancelled();
```
- Test `"6.1 — deactivated participant cannot sign in"` body:
```ts
    const app = new MailClubPage(page);
    await app.attemptLogin(DEACTIVATE_PHONE);
    // Distinct from an unknown phone: no invite-code step is offered.
    await app.expectPhoneStep();
    await app.expectNoInviteCodeStep();
    await app.expectNoSession();
```

## Verification Gates
```bash
git diff --stat
grep -n "1.2 — unregistered phone is rejected\|ready-confirm deadline visible with countdown\|no active season after cancel" end2end/tests/mail_club.spec.ts
```
**REQUIRED:** only the 3 V files; zero matches.
Mutation check (proves non-vacuity; revert after): in `mail_club.spec.ts` temporarily change `formatDateUk(CONFIRM_DEADLINE)` to `formatDateUk(SIGNUP_DEADLINE)`, run E2E gate once → **REQUIRED:** that title FAILS. Revert; `git diff` shows no residue.
Standard gates (deltas 0). E2E gate ×3, delta 0; titles: the 3 renamed + `6.1 — deactivated participant cannot sign in`.

Commit: `test(e2e): assert actual effects for login, deadline, cancel and deactivation`

---

# W2-H1 — Participation-aware home states

## Why This Matters
Non-participants are told assignment is minutes away (C3) and non-enrolled users get a live confirm CTA that silently no-ops (a-scope Gap 4).

## What You Must Do
### H1.1 TDD — pure helpers in `src/pages/home.rs`
Step 1 — extend `mod tests` `use super::{…}` with `Participation, assignment_state, preparation_state, HomeState` and add (red):
```rust
    // ── participation ────────────────────────────────────────────────────────
    fn some_time() -> time::OffsetDateTime {
        time::OffsetDateTime::now_utc()
    }

    #[test]
    fn no_enrollment_row_is_not_enrolled() {
        assert_eq!(Participation::from_enrollment(None), Participation::NotEnrolled);
    }

    #[test]
    fn enrollment_without_confirmation_is_enrolled() {
        assert_eq!(Participation::from_enrollment(Some(None)), Participation::Enrolled);
    }

    #[test]
    fn enrollment_with_confirmation_is_confirmed() {
        assert_eq!(Participation::from_enrollment(Some(Some(some_time()))), Participation::Confirmed);
    }

    #[test]
    fn preparation_not_enrolled_is_not_participating() {
        assert!(matches!(preparation_state(Participation::NotEnrolled, String::new(), false), HomeState::NotParticipating));
    }

    #[test]
    fn preparation_enrolled_is_preparing() {
        assert!(matches!(preparation_state(Participation::Enrolled, "d".into(), true), HomeState::Preparing { deadline_passed: true, .. }));
    }

    #[test]
    fn preparation_confirmed_is_confirmed() {
        assert!(matches!(preparation_state(Participation::Confirmed, String::new(), false), HomeState::Confirmed));
    }

    #[test]
    fn assignment_confirmed_is_assigning() {
        assert!(matches!(assignment_state(Participation::Confirmed), HomeState::Assigning));
    }

    #[test]
    fn assignment_enrolled_unconfirmed_is_not_participating() {
        assert!(matches!(assignment_state(Participation::Enrolled), HomeState::NotParticipating));
    }

    #[test]
    fn assignment_not_enrolled_is_not_participating() {
        assert!(matches!(assignment_state(Participation::NotEnrolled), HomeState::NotParticipating));
    }
```
Step 2 — `cargo test home::tests` → red (compile error).
Step 3 — implement. `HomeState`: add after `Assigning`:
```rust
    /// Season is past enrollment (or complete) and this participant is not in
    /// its cycle: not enrolled, or enrolled but never confirmed ready.
    NotParticipating,
```
In the Pure helpers section:
```rust
/// A participant's standing in a season, from their enrollment row.
#[cfg(any(feature = "ssr", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Participation {
    NotEnrolled,
    Enrolled,
    Confirmed,
}

#[cfg(any(feature = "ssr", test))]
impl Participation {
    /// `None` = no enrollment row; `Some(confirmed_ready_at)` otherwise.
    fn from_enrollment(confirmed_ready_at: Option<Option<time::OffsetDateTime>>) -> Self {
        match confirmed_ready_at {
            None => Self::NotEnrolled,
            Some(None) => Self::Enrolled,
            Some(Some(_)) => Self::Confirmed,
        }
    }
}

/// Home state in the Preparation phase.
#[cfg(any(feature = "ssr", test))]
fn preparation_state(participation: Participation, confirm_deadline: String, deadline_passed: bool) -> HomeState {
    match participation {
        Participation::NotEnrolled => HomeState::NotParticipating,
        Participation::Enrolled => HomeState::Preparing { confirm_deadline, deadline_passed },
        Participation::Confirmed => HomeState::Confirmed,
    }
}

/// Home state in the Assignment phase: only confirmed participants await a recipient.
#[cfg(any(feature = "ssr", test))]
fn assignment_state(participation: Participation) -> HomeState {
    match participation {
        Participation::Confirmed => HomeState::Assigning,
        Participation::NotEnrolled | Participation::Enrolled => HomeState::NotParticipating,
    }
}
```
Step 4 — rerun → green (9 new).

### H1.2 SSR wiring (`src/pages/home.rs`)
- Add:
```rust
/// Load the participant's standing in `season_id`.
#[cfg(feature = "ssr")]
async fn fetch_participation(
    pool: &sqlx::PgPool,
    user_id: uuid::Uuid,
    season_id: uuid::Uuid,
) -> Result<Participation, ServerFnError> {
    let row = sqlx::query_scalar!(
        r#"
        SELECT confirmed_ready_at
        FROM enrollments
        WHERE user_id = $1 AND season_id = $2
        "#,
        user_id,
        season_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(db_err)?;
    Ok(Participation::from_enrollment(row))
}
```
- `resolve_preparation_state`: delete the `confirmed` EXISTS query and the `if confirmed` block; body becomes: `let participation = fetch_participation(pool, user_id, season.id).await?;` then the existing `test_mode`/`deadline_passed` computation (keep the U2 `test_mode()?` line as is), then `Ok(preparation_state(participation, confirm_str, deadline_passed))`.
- `get_home_state` match: `Phase::Assignment => Ok(assignment_state(fetch_participation(&pool, user.id, season.id).await?)),`
- `resolve_delivery_state`: `let Some(a) = outgoing else { return Ok(HomeState::NotParticipating); };` and update its comment to "No outgoing assignment in Delivery: not in this season's cycle."
- Complete for non-participants: the no-active-season branch query becomes
```rust
        let most_recent = sqlx::query!(
            r#"
            SELECT id, phase AS "phase: Phase"
            FROM seasons
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&pool)
        .await
        .map_err(db_err)?;
```
and the arm `Some(Phase::Complete)` becomes: if `took_part_in(&pool, user.id, row.id).await?` → `HomeState::Complete`, else `HomeState::NotParticipating`; other arms keep their mapping (match on `most_recent.map(|r| (r.id, r.phase))`). Add:
```rust
/// Whether the participant sent mail in `season_id` (was in its cycle).
#[cfg(feature = "ssr")]
async fn took_part_in(pool: &sqlx::PgPool, user_id: uuid::Uuid, season_id: uuid::Uuid) -> Result<bool, ServerFnError> {
    sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1 FROM assignments WHERE sender_id = $1 AND season_id = $2
        ) AS "exists!"
        "#,
        user_id,
        season_id,
    )
    .fetch_one(pool)
    .await
    .map_err(db_err)
}
```
- `confirm_ready`: replace the UPDATE with
```rust
    // Idempotent for an already-confirmed participant (COALESCE keeps the first
    // timestamp); zero rows means there is no enrollment to confirm.
    let affected = sqlx::query!(
        r#"
        UPDATE enrollments
        SET confirmed_ready_at = COALESCE(confirmed_ready_at, now())
        WHERE user_id = $1 AND season_id = $2
        "#,
        user.id,
        season.id,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?
    .rows_affected();

    if affected == 0 {
        return Err(ServerFnError::new(td_string!(Locale::uk, home_error_not_enrolled)));
    }
```
- `render_home_state`: add arm
```rust
        HomeState::NotParticipating => view! {
            <div class="empty-state" data-testid="not-participating">
                <h1 class="empty-state-headline">{t!(i18n, home_not_participating_heading)}</h1>
                <p class="empty-state-body">{t!(i18n, home_not_participating_body)}</p>
            </div>
        }
        .into_any(),
```
and add `data-testid="season-complete"` to the `HomeState::Complete` `<div class="empty-state">`.
- sqlx prepare.

### H1.3 E2E
Spec constants: `PHONES` add `D: "+380670000010",`; `NAMES` add `D: "Тестова Людина Г",`; `CODES` type + initializer add `D: string` / `D: ""`.
POM (Home Screen section):
```ts
  async expectNotParticipating() {
    await expect(this.page.getByTestId("not-participating")).toBeVisible();
    await expect(this.page.getByTestId("confirm-ready-button")).not.toBeVisible();
    await expect(this.page.getByTestId("recipient-name")).not.toBeVisible();
  }

  async expectAssigningInProgress() {
    await expect(this.page.getByTestId("assigning-in-progress-badge")).toBeVisible();
  }

  async expectSeasonComplete() {
    await expect(this.page.getByTestId("season-complete")).toBeVisible();
  }
```
Spec (each `test(...)` uses `const app = new MailClubPage(page);`):
1. Insert after `"setup — participant C enrolls"`:
```ts
    // Setup: D exists and is active but never enrolls (non-participant states below).
    test("setup — participant D registers but does not enroll", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      CODES.D = await app.generateInviteCode();
      await app.selfRegister(PHONES.D, CODES.D, NAMES.D);
      await app.expectRedirectedToOnboarding();
      await app.completeOnboarding(CITY_KYIV, "2");
      await app.expectRedirectedToHome();
    });
```
2. Insert after `"home screen — creation period message shown"`:
```ts
    // Story 2.2 Given "enrolled participant": a non-enrolled user gets no confirm action.
    test("2.2 — non-enrolled participant is told they are not in this season", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.D);
      await app.goHome();
      await app.expectNotParticipating();
    });
```
3. Replace test `"home screen — participant sees 'preparing' during assignment"` with title `"home screen — confirmed participant sees assignment in progress"`, body: login A, goHome, `await app.expectAssigningInProgress();`. Insert after it:
```ts
    test("home screen — non-participant sees not-participating during assignment", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.D);
      await app.goHome();
      await app.expectNotParticipating();
    });
```
4. Insert after `"2.3 — only one recipient visible"`: test `"2.3 — non-participant sees no recipient during delivery"` — login D, goHome, `expectNotParticipating()`.
5. Test `"home screen — season complete message shown"` body: login A, goHome, `await app.expectSeasonComplete();`. Insert after it: test `"home screen — non-participant sees not-participating after completion"` — login D, goHome, `expectNotParticipating()`.

## Verification Gates
```bash
grep -n "Ok(HomeState::Assigning)" src/pages/home.rs
grep -n "confirmed_ready_at IS NULL" src/pages/home.rs
```
**REQUIRED:** zero matches for both.
Standard gates: bare +9, ssr +9. E2E ×3: delta +5; titles: the 5 new + `home screen — confirmed participant sees assignment in progress` + `home screen — season complete message shown` + `5.3 — admin triggers season-open SMS` (count `3` still holds — D registers after it).

Commit: `feat(home): show non-participants a not-participating state and reject unenrolled confirm`

---

# W2-R — Season roster for the organizer (who + receipt notes)

## Why This Matters
Organizer sees integers only; nudging and forwarding require names, phones and the participant's note (j-feel O1/C5; a-scope Gap 3).

## What You Must Do
### R.1 TDD — `src/admin/state.rs`
Step 1 — append (red):
```rust
#[cfg(test)]
mod tests {
    use super::{RosterStatus, roster_status};
    use crate::types::ReceiptStatus;

    #[test]
    fn unassigned_unconfirmed_is_unconfirmed() {
        assert_eq!(roster_status(false, None, None), RosterStatus::Unconfirmed);
    }

    #[test]
    fn unassigned_confirmed_is_confirmed() {
        assert_eq!(roster_status(true, None, None), RosterStatus::Confirmed);
    }

    #[test]
    fn no_response_is_awaiting_receipt() {
        assert_eq!(roster_status(true, Some(ReceiptStatus::NoResponse), None), RosterStatus::AwaitingReceipt);
    }

    #[test]
    fn received_keeps_note() {
        assert_eq!(
            roster_status(true, Some(ReceiptStatus::Received), Some("пом'ятий".into())),
            RosterStatus::Received { note: Some("пом'ятий".into()) }
        );
    }

    #[test]
    fn not_received_keeps_note() {
        assert_eq!(
            roster_status(true, Some(ReceiptStatus::NotReceived), Some("нічого".into())),
            RosterStatus::NotReceived { note: Some("нічого".into()) }
        );
    }
}
```
Step 2 — `cargo test state::tests` → red. Step 3 — implement (shared types, no cfg):
```rust
/// One enrolled participant of the current season, for the organizer.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RosterEntry {
    pub user_id: uuid::Uuid,
    pub name: String,
    pub phone: String,
    pub status: RosterStatus,
}

/// Where an enrolled participant stands. Notes exist only once they answered.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RosterStatus {
    Unconfirmed,
    Confirmed,
    AwaitingReceipt,
    Received { note: Option<String> },
    NotReceived { note: Option<String> },
}

impl RosterStatus {
    /// Stable value for `data-roster-status` (E2E contract).
    pub fn key(&self) -> &'static str {
        match self {
            Self::Unconfirmed => "unconfirmed",
            Self::Confirmed => "confirmed",
            Self::AwaitingReceipt => "awaiting_receipt",
            Self::Received { .. } => "received",
            Self::NotReceived { .. } => "not_received",
        }
    }

    /// `.badge[data-status]` family (design-system §Badges).
    pub fn badge_status(&self) -> &'static str {
        match self {
            Self::Unconfirmed | Self::AwaitingReceipt => "pending",
            Self::Confirmed => "ready",
            Self::Received { .. } => "confirmed",
            Self::NotReceived { .. } => "error",
        }
    }

    /// The participant's receipt note, if they left one.
    pub fn note(&self) -> Option<&str> {
        match self {
            Self::Received { note } | Self::NotReceived { note } => note.as_deref(),
            Self::Unconfirmed | Self::Confirmed | Self::AwaitingReceipt => None,
        }
    }
}

/// Derive roster status: an incoming assignment row (if any) decides; else confirmation.
#[cfg(any(feature = "ssr", test))]
fn roster_status(confirmed: bool, receipt: Option<ReceiptStatus>, note: Option<String>) -> RosterStatus {
    match receipt {
        Some(ReceiptStatus::NoResponse) => RosterStatus::AwaitingReceipt,
        Some(ReceiptStatus::Received) => RosterStatus::Received { note },
        Some(ReceiptStatus::NotReceived) => RosterStatus::NotReceived { note },
        None if confirmed => RosterStatus::Confirmed,
        None => RosterStatus::Unconfirmed,
    }
}
```
Add at the top of the file `#[cfg(any(feature = "ssr", test))] use crate::types::ReceiptStatus;` (gated: only `roster_status` uses it; ungated it is an unused import in the hydrate build). Step 4 — green.

### R.2 Query + state
`AdminSeason` add last field `pub roster: Vec<RosterEntry>,` (doc: "Enrolled participants of this season, by name."). Add SSR row struct + helper:
```rust
#[cfg(feature = "ssr")]
struct RosterRow {
    user_id: uuid::Uuid,
    name: String,
    phone: String,
    confirmed: bool,
    receipt_status: Option<crate::types::ReceiptStatus>,
    receipt_note: Option<String>,
}

/// Enrolled participants of `season_id` with their own (incoming) receipt row.
#[cfg(feature = "ssr")]
async fn query_roster(pool: &sqlx::PgPool, season_id: uuid::Uuid) -> Result<Vec<RosterEntry>, ServerFnError> {
    let rows = sqlx::query_as!(
        RosterRow,
        r#"
        SELECT
            u.id AS user_id,
            u.name,
            u.phone,
            (e.confirmed_ready_at IS NOT NULL) AS "confirmed!: bool",
            a.receipt_status AS "receipt_status?: crate::types::ReceiptStatus",
            a.receipt_note AS "receipt_note?"
        FROM enrollments e
        JOIN users u ON u.id = e.user_id
        LEFT JOIN assignments a ON a.season_id = e.season_id AND a.recipient_id = e.user_id
        WHERE e.season_id = $1
        ORDER BY u.name
        "#,
        season_id,
    )
    .fetch_all(pool)
    .await
    .map_err(db_err)?;
    Ok(rows
        .into_iter()
        .map(|r| RosterEntry {
            user_id: r.user_id,
            name: r.name,
            phone: r.phone,
            status: roster_status(r.confirmed, r.receipt_status, r.receipt_note),
        })
        .collect())
}
```
In `get_admin_state`: `let roster = query_roster(&pool, row.id).await?;` and `roster,` in the `AdminSeason` literal. sqlx prepare.

### R.3 `src/admin/page.rs`
In `render_active_season`: `let roster = season.roster.clone();`. Directly after the not-received alert block (`not-received-alert`) and before `render_phase_sms(...)`, insert `{render_roster(roster, i18n)}`. Add:
```rust
/// Season roster: who is enrolled, their phone, status and receipt note.
/// Empty roster renders nothing.
fn render_roster(
    roster: Vec<RosterEntry>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    if roster.is_empty() {
        return ().into_any();
    }
    view! {
        <div data-testid="season-roster">
            <h3 class="overline-label">{t!(i18n, admin_roster_title)}</h3>
            <div class="data-table-wrapper">
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>{t!(i18n, participants_table_name)}</th>
                            <th>{t!(i18n, participants_table_phone)}</th>
                            <th>{t!(i18n, participants_table_status)}</th>
                            <th>{t!(i18n, admin_roster_note_column)}</th>
                        </tr>
                    </thead>
                    <tbody>
                        {roster
                            .into_iter()
                            .map(|entry| {
                                let note = entry.status.note().map(str::to_owned);
                                view! {
                                    <tr data-testid="season-roster-row">
                                        <td data-testid="season-roster-name">{entry.name.clone()}</td>
                                        <td>
                                            <a class="info-link" href=format!("tel:{}", entry.phone)>{entry.phone.clone()}</a>
                                        </td>
                                        <td>
                                            <span
                                                class="badge"
                                                data-status=entry.status.badge_status()
                                                data-roster-status=entry.status.key()
                                                data-testid="season-roster-status"
                                            >
                                                {roster_status_label(&entry.status, i18n)}
                                            </span>
                                        </td>
                                        <td data-testid="season-roster-note">{note}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
        </div>
    }
    .into_any()
}

fn roster_status_label(
    status: &RosterStatus,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    match status {
        RosterStatus::Unconfirmed => t!(i18n, admin_roster_status_unconfirmed).into_any(),
        RosterStatus::Confirmed => t!(i18n, admin_roster_status_confirmed).into_any(),
        RosterStatus::AwaitingReceipt => t!(i18n, admin_roster_status_awaiting_receipt).into_any(),
        RosterStatus::Received { .. } => t!(i18n, admin_roster_status_received).into_any(),
        RosterStatus::NotReceived { .. } => t!(i18n, admin_roster_status_not_received).into_any(),
    }
}
```
Import `RosterEntry, RosterStatus` from `crate::admin::state` (or `super::state`, matching the file's existing import style). `format!` here builds an `href`, not a class (allowed).

### R.4 E2E
POM (Admin section):
```ts
  private rosterRow(name: string): Locator {
    return this.page.getByTestId("season-roster-row").filter({
      has: this.page.getByTestId("season-roster-name").filter({ hasText: name }),
    });
  }

  async expectRosterStatus(name: string, status: string) {
    await expect(this.rosterRow(name).getByTestId("season-roster-status")).toHaveAttribute("data-roster-status", status);
  }

  async expectRosterNote(name: string, text: string) {
    await expect(this.rosterRow(name).getByTestId("season-roster-note")).toHaveText(text);
  }

  async expectRosterSize(count: number) {
    await expect(this.page.getByTestId("season-roster-row")).toHaveCount(count);
  }
```
Spec:
1. Insert after `"2.2 — confirmed participant cannot un-confirm"`:
```ts
    // Story 4.4 / O1: organizer sees who still has to confirm, by name.
    test("4.4 — admin sees which enrolled participants have not confirmed", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      await app.expectRosterSize(3); // D is not enrolled
      await app.expectRosterStatus(NAMES.A, "confirmed");
      await app.expectRosterStatus(NAMES.B, "unconfirmed");
      await app.expectRosterStatus(NAMES.C, "unconfirmed");
    });
```
2. Test `"2.4 — admin sees not-received alert"` body: login admin, goToDashboard, `await expect(page.getByTestId("not-received-alert")).toContainText("1");` → move into POM as `expectNotReceivedAlert(count: string)` and call it.
3. Insert after it:
```ts
    // Story 2.4 / Product Spec failure protocol: organizer sees who got nothing and their note.
    test("2.4 — admin sees who reported not received, with their note", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      await app.expectRosterStatus(NAMES.A, "received");
      await app.expectRosterStatus(NAMES.B, "not_received");
      await app.expectRosterNote(NAMES.B, "Пошта не надійшла");
      await app.expectRosterStatus(NAMES.C, "awaiting_receipt");
    });
```
`visual-audit.spec.ts`: insert after test `"setup — participant C confirms receipt"`:
```ts
  test("capture admin — delivery roster", async ({ page }) => {
    const app = new MailClubPage(page);
    await app.login(ADMIN_PHONE);
    await app.goToDashboard();
    await expect(page.getByTestId("season-roster")).toBeVisible();
    await captureState(page, "admin-delivery-roster", { stateId: "A43", route: "/admin" });
    await captureSection(page, "admin-delivery-roster", "season-roster");
  });
```

## Verification Gates
```bash
grep -rn "receipt_note" src/ | grep -v "^src/pages/home.rs"
```
**REQUIRED:** ≥1 match in `src/admin/state.rs` (note now has a reader).
Standard gates: bare +5, ssr +5. E2E ×3: delta +3 (2 mail_club + 1 visual-audit); titles: 2 new + `2.4 — admin sees not-received alert` + `capture admin — delivery roster`. Pixels: `admin-delivery-roster`, `admin-confirm-phase`, `admin-delivery-phase-sms-controls` (all 4 mode dirs) + `sections/admin-delivery-roster__season-roster*`.

Commit: `feat(admin): show season roster with receipt status and notes`

---

# W2-F — Forwarding requests (Product Spec failure protocol)

## Why This Matters
When a sender fails, the organizer must ask the right person to forward the right mail — mechanical, error-prone under pressure (j-feel O2/C6).

## What You Must Do
### F.1 TDD — `src/admin/assignments.rs` (shared code, runs under bare `cargo test`)
`AssignmentLink` add last field:
```rust
    /// Receipt the recipient reported for this link (`NoResponse` until they answer).
    pub recipient_receipt: crate::types::ReceiptStatus,
```
Add:
```rust
/// Ask `from_name` to forward the mail they received to `to_name`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ForwardingRequest {
    pub from_name: String,
    pub to_name: String,
}
```
Step 1 — append (red):
```rust
#[cfg(test)]
mod forwarding_tests {
    use super::{AssignmentLink, ForwardingRequest, forwarding_requests};
    use crate::types::ReceiptStatus::{self, NoResponse, NotReceived, Received};

    /// Ring names[0]→names[1]→…→names[0]; statuses[i] = what names[i+1] reported.
    fn ring(names: &[&str], statuses: &[ReceiptStatus]) -> Vec<AssignmentLink> {
        (0..names.len())
            .map(|i| {
                let r = names[(i + 1) % names.len()];
                AssignmentLink {
                    sender_id: names[i].into(),
                    sender_name: names[i].into(),
                    recipient_id: r.into(),
                    recipient_name: r.into(),
                    recipient_receipt: statuses[i],
                }
            })
            .collect()
    }

    fn req(from: &str, to: &str) -> ForwardingRequest {
        ForwardingRequest { from_name: from.into(), to_name: to.into() }
    }

    const ABCDE: [&str; 5] = ["A", "B", "C", "D", "E"];

    #[test]
    fn nobody_failed_needs_no_forwarding() {
        assert!(forwarding_requests(&ring(&ABCDE, &[Received; 5])).is_empty());
    }

    #[test]
    fn single_failure_forwards_to_skipped_recipient() {
        // B never sent to C: B forwards what B received (A's mail) to C.
        let chain = ring(&ABCDE, &[Received, NotReceived, Received, Received, Received]);
        assert_eq!(forwarding_requests(&chain), vec![req("B", "C")]);
    }

    #[test]
    fn contiguous_failures_need_one_request() {
        // Product Spec example: B and C fail → B forwards A's mail to D.
        let chain = ring(&ABCDE, &[Received, NotReceived, NotReceived, Received, Received]);
        assert_eq!(forwarding_requests(&chain), vec![req("B", "D")]);
    }

    #[test]
    fn separate_blocks_get_separate_requests() {
        let chain = ring(&ABCDE, &[NotReceived, Received, NotReceived, Received, Received]);
        assert_eq!(forwarding_requests(&chain), vec![req("C", "D"), req("A", "B")]);
    }

    #[test]
    fn block_wrapping_the_ring_end_is_one_request() {
        let chain = ring(&ABCDE, &[NotReceived, Received, Received, Received, NotReceived]);
        assert_eq!(forwarding_requests(&chain), vec![req("E", "B")]);
    }

    #[test]
    fn all_but_one_failed_returns_own_mail() {
        // Only A sent; B forwards A's mail back to A.
        let chain = ring(&ABCDE, &[Received, NotReceived, NotReceived, NotReceived, NotReceived]);
        assert_eq!(forwarding_requests(&chain), vec![req("B", "A")]);
    }

    #[test]
    fn everybody_failed_has_nobody_to_forward() {
        assert!(forwarding_requests(&ring(&ABCDE, &[NotReceived; 5])).is_empty());
    }

    #[test]
    fn no_response_is_not_a_failure() {
        assert!(forwarding_requests(&ring(&ABCDE, &[NoResponse; 5])).is_empty());
    }
}
```
Order note: iteration starts AFTER the first non-failed link, so `separate_blocks_get_separate_requests` (first non-failed = index 1) yields the index-2 block before the index-0 block — the expected vector above encodes that; do not change it.
Step 2 — `cargo test forwarding_tests` → red. Step 3 — implement:
```rust
/// Forwarding requests for one cohort chain (ordered: link i's recipient is
/// link i+1's sender). A link failed when its recipient reported `NotReceived`.
/// Each maximal run of failed links i..=j yields one request: the run's first
/// sender (who still holds the mail they received) forwards it to link j's
/// recipient. No requests when nobody — or everybody — failed.
#[must_use]
pub fn forwarding_requests(chain: &[AssignmentLink]) -> Vec<ForwardingRequest> {
    let n = chain.len();
    let failed = |i: usize| chain[i].recipient_receipt == crate::types::ReceiptStatus::NotReceived;
    let Some(start) = (0..n).find(|&i| !failed(i)) else {
        return Vec::new();
    };
    let mut requests = Vec::new();
    let mut run_start: Option<usize> = None;
    for step in 1..=n {
        let i = (start + step) % n;
        match (failed(i), run_start) {
            (true, None) => run_start = Some(i),
            (false, Some(first)) => {
                let last = (i + n - 1) % n;
                requests.push(ForwardingRequest {
                    from_name: chain[first].sender_name.clone(),
                    to_name: chain[last].recipient_name.clone(),
                });
                run_start = None;
            }
            (true, Some(_)) | (false, None) => {}
        }
    }
    requests
}
```
Step 4 — green (8).

### F.2 Data
- Every `AssignmentLink { … }` literal in `src/` (`grep -rn "AssignmentLink {" src`): links built from freshly generated assignments get `recipient_receipt: crate::types::ReceiptStatus::NoResponse`; links built in `get_assignment_preview` get `recipient_receipt: a.receipt_status`.
- `AssignmentRow` (SSR struct) add `receipt_status: crate::types::ReceiptStatus`; EVERY `query_as!(AssignmentRow, …)` select list gains `a.receipt_status AS "receipt_status: crate::types::ReceiptStatus"`. sqlx prepare.

### F.3 `src/admin/page.rs` — `render_assignment_section`
After computing `cohorts_for_viz`, add:
```rust
    // Forwarding is only meaningful once assignments are released.
    let forwarding: Vec<ForwardingRequest> = if is_assignment_phase {
        Vec::new()
    } else {
        p.cohorts.iter().flat_map(|c| forwarding_requests(&c.chain)).collect()
    };
```
Directly after the cycle-visualization block in the `view!`, insert `{render_forwarding_requests(forwarding, i18n)}`. Add:
```rust
/// Organizer instructions for the Product Spec forwarding protocol.
fn render_forwarding_requests(
    requests: Vec<ForwardingRequest>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    if requests.is_empty() {
        return ().into_any();
    }
    view! {
        <div data-testid="forwarding-requests">
            <h3 class="overline-label">{t!(i18n, admin_forwarding_title)}</h3>
            <p>{t!(i18n, admin_forwarding_description)}</p>
            <ul>
                {requests
                    .into_iter()
                    .map(|r| view! {
                        <li
                            data-testid="forwarding-request"
                            data-from-name=r.from_name.clone()
                            data-to-name=r.to_name.clone()
                        >
                            {t!(i18n, admin_forwarding_request, from = r.from_name.clone(), to = r.to_name.clone())}
                        </li>
                    })
                    .collect_view()}
            </ul>
        </div>
    }
    .into_any()
}
```
Import `forwarding_requests, ForwardingRequest` with the other `assignments` imports.

### F.4 E2E
POM (assignments section):
```ts
  /** Name of the participant who sends to `recipientName`, from the cycle link list. */
  async cycleSenderNameFor(recipientName: string): Promise<string> {
    const recipientId = await this.cycleSenderId(recipientName);
    const link = this.page.getByTestId("cycle-link").and(this.page.locator(`[data-recipient-id="${recipientId}"]`));
    await expect(link).toBeAttached();
    return (await link.getAttribute("data-sender-name")) as string;
  }

  async expectForwardingRequest(fromName: string, toName: string) {
    const requests = this.page.getByTestId("forwarding-request");
    await expect(requests).toHaveCount(1);
    await expect(requests.first()).toHaveAttribute("data-from-name", fromName);
    await expect(requests.first()).toHaveAttribute("data-to-name", toName);
  }
```
Spec: insert after `"2.4 — admin sees who reported not received, with their note"`:
```ts
    // Product Spec §Non-compliance: B got nothing → B's sender forwards what they received to B.
    test("2.4 — admin is told who should forward mail to the participant who got nothing", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      const senderOfB = await app.cycleSenderNameFor(NAMES.B);
      await app.expectForwardingRequest(senderOfB, NAMES.B);
    });
```
`visual-audit.spec.ts`, test `"capture admin — delivery roster"`: before `captureState`, add `await expect(page.getByTestId("forwarding-requests")).toBeVisible();` (audit B reported not received).

## Verification Gates
```bash
grep -rn "AssignmentLink {" src | wc -l
grep -rn "recipient_receipt" src | wc -l
```
**REQUIRED:** every non-declaration literal has a `recipient_receipt:` line (second count ≥ first count + 1).
Standard gates: bare +8, ssr +8. E2E ×3: delta +1; titles: new + `capture admin — delivery roster`. Pixels: `admin-delivery-roster` (4 dirs).

Commit: `feat(admin): show forwarding requests for failed deliveries`

---

# W2-H2 — Delivery card persists; not-received is correctable

## Why This Matters
C1 dead end (address disappears on own receipt) + C2 irreversible false alarm.

## What You Must Do
### H2.1 TDD — `src/types.rs`
Step 1 — add to `mod tests` (red):
```rust
    #[test]
    fn first_receipt_answer_is_allowed() {
        assert!(ReceiptStatus::NoResponse.can_transition_to(ReceiptStatus::Received));
        assert!(ReceiptStatus::NoResponse.can_transition_to(ReceiptStatus::NotReceived));
    }

    #[test]
    fn late_arrival_corrects_not_received() {
        assert!(ReceiptStatus::NotReceived.can_transition_to(ReceiptStatus::Received));
    }

    #[test]
    fn other_receipt_transitions_are_refused() {
        use ReceiptStatus::{NoResponse, NotReceived, Received};
        for (from, to) in [
            (Received, NotReceived),
            (Received, Received),
            (Received, NoResponse),
            (NotReceived, NotReceived),
            (NotReceived, NoResponse),
            (NoResponse, NoResponse),
        ] {
            assert!(!from.can_transition_to(to), "{from:?} -> {to:?}");
        }
    }
```
Step 3 — implement:
```rust
impl ReceiptStatus {
    /// Participant-driven receipt transitions: a first answer, or correcting a
    /// `NotReceived` report when the mail arrives late. `Received` is final.
    #[must_use]
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::NoResponse, Self::Received | Self::NotReceived) | (Self::NotReceived, Self::Received)
        )
    }
}
```
Update the enum doc to mention the transition rule. Green (3).

### H2.2 `src/pages/home.rs`
- Replace variants `Assigned { … }` and `ReceiptConfirmed { status }` with:
```rust
    /// Delivery phase, participant is in the cycle. The recipient card stays
    /// visible whatever they reported about their OWN incoming mail — sending
    /// and receiving are independent.
    Delivery {
        recipient: RecipientCard,
        own_receipt: ReceiptStatus,
    },
```
and add
```rust
/// Who the participant sends to.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecipientCard {
    pub name: String,
    pub phone: String,
    pub city: String,
    pub branch_number: i32,
}
```
- `resolve_delivery_state`: incoming query `.fetch_one(pool)` (every cycle member is also a recipient) returning `ReceiptStatus`; return `Ok(HomeState::Delivery { recipient: RecipientCard { name: a.recipient_name, phone: a.recipient_phone, city: a.nova_poshta_city, branch_number: a.nova_poshta_number }, own_receipt })`. Update the fn doc.
- `confirm_receipt`: doc "One-way latch" → "Transitions per `ReceiptStatus::can_transition_to`". Replace the assignment lookup + UPDATE with:
```rust
    let current = sqlx::query!(
        r#"
        SELECT id, receipt_status AS "receipt_status: ReceiptStatus"
        FROM assignments
        WHERE recipient_id = $1 AND season_id = $2
        "#,
        user.id,
        season_id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(db_err)?
    .filter(|row| row.receipt_status.can_transition_to(new_status))
    .ok_or_else(|| ServerFnError::new(td_string!(Locale::uk, home_error_already_confirmed)))?;

    // Guarded on the status just read: a concurrent answer cannot be overwritten.
    // A correction without a new note keeps the original note.
    let affected = sqlx::query!(
        r#"
        UPDATE assignments
        SET receipt_status = $1, receipt_note = COALESCE($2, receipt_note)
        WHERE id = $3 AND receipt_status = $4
        "#,
        new_status as ReceiptStatus,
        note,
        current.id,
        current.receipt_status as ReceiptStatus,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?
    .rows_affected();

    if affected == 0 {
        return Err(ServerFnError::new(td_string!(Locale::uk, home_error_already_confirmed)));
    }
```
Move the existing `new_status` and `note` computations ABOVE this block (they are needed by the `.filter`).
- Rendering: delete `render_receipt_confirmed`; rename `render_assignment_details` → `render_delivery(recipient: RecipientCard, own_receipt: ReceiptStatus, receipt_action, hydrated, i18n) -> AnyView`. Keep the h1, send instructions, and the card with testids `recipient-name`, `recipient-branch`, `recipient-phone` unchanged. Replace the trailing `{render_receipt_form(...)}` with `{render_own_receipt(own_receipt, receipt_action, receipt_pending, hydrated, i18n)}`:
```rust
fn render_own_receipt(
    own_receipt: ReceiptStatus,
    receipt_action: ServerAction<ConfirmReceipt>,
    receipt_pending: Memo<bool>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    match own_receipt {
        ReceiptStatus::NoResponse => view! {
            <p class="text-sm text-(--color-text-muted)" data-testid="receipt-timing-hint">
                {t!(i18n, home_receipt_timing_hint)}
            </p>
            {render_receipt_form(receipt_action, receipt_pending, hydrated, i18n)}
        }
        .into_any(),
        ReceiptStatus::Received => view! {
            <section>
                <h2>{t!(i18n, home_thanks_heading)}</h2>
                <p data-testid="receipt-thanks">{t!(i18n, home_received_confirmed_body)}</p>
            </section>
        }
        .into_any(),
        ReceiptStatus::NotReceived => view! {
            <section>
                <h2>{t!(i18n, home_not_received_heading)}</h2>
                <p data-testid="receipt-thanks">{t!(i18n, home_reported_label)}</p>
                <leptos::form::ActionForm action=receipt_action>
                    <button
                        class="btn"
                        data-variant="secondary"
                        type="submit"
                        name="received"
                        value="true"
                        data-testid="received-after-all-button"
                        disabled=move || receipt_pending.get() || !hydrated.get()
                    >
                        {move || if receipt_pending.get() {
                            t!(i18n, home_pending_sending).into_any()
                        } else {
                            t!(i18n, home_received_after_all_button).into_any()
                        }}
                    </button>
                </leptos::form::ActionForm>
            </section>
        }
        .into_any(),
    }
}
```
(The correction form has no `note` field → `note = None` → COALESCE keeps the original note.)
- `render_home_state`: replace the `Assigned` and `ReceiptConfirmed` arms with `HomeState::Delivery { recipient, own_receipt } => render_delivery(recipient, own_receipt, receipt_action, hydrated, i18n),`. Update the comment above `HomePage`'s receipt toast note (state name).
- sqlx prepare.

### H2.3 E2E
POM (Receipt section):
```ts
  async expectReceiptCorrectionAvailable() {
    await expect(this.page.getByTestId("received-after-all-button")).toBeVisible();
  }

  async correctReceiptToReceived() {
    await this.page.getByTestId("received-after-all-button").click();
    await expect(this.page.getByTestId("received-after-all-button")).not.toBeVisible();
    await expect(this.page.getByTestId("receipt-thanks")).toBeVisible();
  }

  async expectReceiptPrompt() {
    await expect(this.page.getByTestId("receipt-timing-hint")).toBeVisible();
    await expect(this.page.getByTestId("received-button")).toBeVisible();
  }

  async expectNoForwardingRequests() {
    await expect(this.page.getByTestId("forwarding-requests")).not.toBeVisible();
  }

  async expectNoNotReceivedAlert() {
    await expect(this.page.getByTestId("not-received-alert")).not.toBeVisible();
  }
```
Spec:
- `"home screen — delivery phase prompt shown"` body: login A, goHome, `await app.expectReceiptPrompt();`.
- `"2.4 — participant confirms receipt (received)"`: after `confirmReceipt(true)` add `await app.expectAssignmentVisible();` (C1 effect).
- `"2.4 — participant reports not received with note"`: after `confirmReceipt(false, …)` add `await app.expectAssignmentVisible();` and `await app.expectReceiptCorrectionAvailable();`.
- Insert after `"2.4 — admin is told who should forward mail to the participant who got nothing"`:
```ts
    // Late arrival: the not-received report is reversible; forwarding signal clears.
    test("2.4 — participant corrects a not-received report when mail arrives late", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.B);
      await app.goHome();
      await app.correctReceiptToReceived();
      await app.expectAssignmentVisible();
      await app.login(ADMIN_PHONE);
      await app.goToDashboard();
      await app.expectRosterStatus(NAMES.B, "received");
      await app.expectRosterNote(NAMES.B, "Пошта не надійшла");
      await app.expectNoForwardingRequests();
      await app.expectNoNotReceivedAlert();
    });
```
`visual-audit.spec.ts`, test `"capture home — not-received receipt (participant B)"`: after `await app.confirmReceipt(false);` add
```ts
    await expect(page.getByTestId("received-after-all-button")).toBeVisible();
    await captureState(page, "home-receipt-not-received-reported", { stateId: "H7c", route: "/" });
```

## Verification Gates
```bash
grep -n "ReceiptConfirmed\|Assigned {\|render_receipt_confirmed\|receipt_status = 'no_response'" src/pages/home.rs
```
**REQUIRED:** zero matches.
Standard gates: bare +3, ssr +3. E2E ×3: delta +1; titles: new + the 3 edited delivery titles + `capture home — not-received receipt (participant B)`. Pixels: `home-assignment-and-receipt-form`, `home-receipt-confirmed-thanks`, `home-receipt-not-received-reported` (4 dirs).

Commit: `feat(home): keep recipient card after receipt and allow late-arrival correction`

---

# W2-H3 — Branch editable during enrollment

## Why This Matters
Stories 1.3 AC3 / 2.1 AC2: a wrong branch today needs organizer SQL and mail goes to the wrong office.

## What You Must Do
### H3.1 `src/pages/home.rs`
- `HomeState::Enrolled` becomes `Enrolled { confirm_deadline: String, city: String, branch_number: i32 }` (doc: "Enrollment phase, participant IS enrolled; shows the address they enrolled with."). In `resolve_enrollment_state`, the `enrolled` branch loads the address with `fetch_one` (enrollment requires an address):
```rust
        let address = sqlx::query!(
            r#"
            SELECT nova_poshta_city, nova_poshta_number
            FROM delivery_addresses
            WHERE user_id = $1
            "#,
            user_id,
        )
        .fetch_one(pool)
        .await
        .map_err(db_err)?;
        Ok(HomeState::Enrolled {
            confirm_deadline: confirm_str,
            city: address.nova_poshta_city,
            branch_number: address.nova_poshta_number,
        })
```
(This SQL text is identical to the existing `existing_address` query → same `.sqlx` entry.)
- `Enrolled` render: after the `home_enrolled_milestone` paragraph add
```rust
                <p class="empty-state-body" data-testid="enrolled-address">
                    {t!(i18n, home_enrolled_address, branch_number = branch_number, city = city)}
                </p>
```
- `render_enrollment_open`: add `data-testid="season-guideline"` to the `<p>{t!(i18n, home_guideline)}</p>`. Extract the `None` arm's field markup into
```rust
/// City + branch inputs for a new delivery address (+ `use_existing_address=false`).
/// `current` (the saved address) only fills placeholders — never `value`.
fn render_new_address_fields(
    city_error: impl Fn() -> Option<String> + Copy + Send + Sync + 'static,
    np_number_error: impl Fn() -> Option<String> + Copy + Send + Sync + 'static,
    current: Option<(String, i32)>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView
```
It renders exactly today's `None`-arm markup (hidden `use_existing_address=false`, both `.field`s, all testids, aria attributes, error `<p>`s) with `placeholder` = `current` city / branch number when `Some`, else `"Київ"` / `"123"`.
Replace the `Some((city, branch_number))` arm with:
```rust
                Some((city, branch_number)) => {
                    // View mode only (never an input value): saved card ⇄ edit fields.
                    let (editing_address, set_editing_address) = signal(false);
                    view! {
                        {move || if editing_address.get() {
                            render_new_address_fields(city_error, np_number_error, Some((city.clone(), branch_number)), i18n)
                        } else {
                            render_saved_address(city.clone(), branch_number, set_editing_address, hydrated, i18n)
                        }}
                    }
                    .into_any()
                }
                None => render_new_address_fields(city_error, np_number_error, None, i18n),
```
`render_saved_address(city: String, branch_number: i32, set_editing_address: WriteSignal<bool>, hydrated: ReadSignal<bool>, i18n) -> AnyView` = today's `Some`-arm markup (card `existing-address`, three hidden inputs) plus, inside the `<article>` after the note `<p>`:
```rust
                <button
                    type="button"
                    class="btn"
                    data-variant="link"
                    data-testid="change-address-button"
                    disabled=move || !hydrated.get()
                    on:click=move |_| set_editing_address.set(true)
                >
                    {t!(i18n, home_change_address_button)}
                </button>
```
Server fn `enroll_in_season` unchanged.
- sqlx prepare (expected: no new entry; run anyway).

### H3.2 E2E
POM (enrollment section):
```ts
  async enrollWithChangedAddress(city: string, branchNumber: string) {
    await this.page.getByTestId("change-address-button").click();
    const cityInput = this.page.getByTestId("np-city-input");
    await expect(cityInput).toBeEditable();
    await cityInput.fill(city);
    await this.page.getByTestId("np-number-input").fill(branchNumber);
    await expect(cityInput).toHaveValue(city);
    await this.page.getByTestId("enroll-button").click();
    await expect(this.page.getByTestId("enroll-button")).not.toBeVisible();
  }

  async expectEnrolledAddress(city: string, branchNumber: string) {
    const address = this.page.getByTestId("enrolled-address");
    await expect(address).toContainText(city);
    await expect(address).toContainText(`№${branchNumber}`);
  }

  async expectContentGuidelines() {
    await expect(this.page.getByTestId("season-guideline")).toBeVisible();
    await expect(this.page.getByTestId("season-guideline")).not.toBeEmpty();
  }
```
Spec:
- `"2.1 — content guidelines visible during enrollment"` body: login A, goHome, `await app.expectContentGuidelines();`.
- Replace test `"2.1 — participant can set branch during enrollment"` with title `"2.1 — participant updates branch during enrollment"`, body:
```ts
      const app = new MailClubPage(page);
      await app.login(PHONES.B); // onboarded with Київ / 1
      await app.goHome();
      await app.enrollWithChangedAddress(CITY_LVIV, BRANCH_10);
      await app.expectEnrolledAddress(CITY_LVIV, BRANCH_10);
```
`visual-audit.spec.ts`, test `"capture home — enrollment open with existing address"`: after its `captureState`, add
```ts
    await page.getByTestId("change-address-button").click();
    await expect(page.getByTestId("np-city-input")).toBeVisible();
    await captureState(page, "home-enrollment-change-address", { stateId: "H2b", route: "/" });
```

## Verification Gates
Mutation check: temporarily make `change-address-button`'s `on:click` a no-op, run E2E once → **REQUIRED:** `2.1 — participant updates branch during enrollment` FAILS. Revert; no residue.
Standard gates (deltas 0). E2E ×3: delta 0; titles: the 2 edited + `capture home — enrollment open with existing address` + `capture home — enrolled (after enroll)`. Pixels: `home-enrollment-existing-address`, `home-enrollment-change-address`, `home-enrolled` (match by `ls | grep enroll`), 4 dirs.

Commit: `feat(home): let participants change their branch during enrollment`

---

# W2-M1 — Meetup announcement: storage + organizer form

## Why This Matters
The meetup is the ritual's payoff; Story 2.1 promises the meetup window; organizer otherwise messages everyone.

## What You Must Do
### M1.1 Migration (exact)
`migrations/20261004000002_add_season_meetup_details.sql`:
```sql
-- Organizer's free-text meetup announcement (date, time, place).
-- NULL = not announced yet.
ALTER TABLE seasons
    ADD COLUMN meetup_details TEXT
    CHECK (meetup_details IS NULL OR char_length(meetup_details) BETWEEN 1 AND 300);
```
Gate: `ls migrations | sort | tail -1` → **REQUIRED:** this file. If any existing migration sorts after it → BLOCKED.

### M1.2 TDD — `src/admin/season.rs`
Step 1 — append (red):
```rust
#[cfg(test)]
mod tests {
    use super::{MEETUP_DETAILS_MAX_CHARS, MeetupDetailsError, normalize_meetup_details};

    #[test]
    fn empty_clears_meetup() {
        assert_eq!(normalize_meetup_details(""), Ok(None));
    }

    #[test]
    fn whitespace_clears_meetup() {
        assert_eq!(normalize_meetup_details("  \n "), Ok(None));
    }

    #[test]
    fn meetup_is_trimmed() {
        assert_eq!(normalize_meetup_details("  15 листопада, 18:00 \n"), Ok(Some("15 листопада, 18:00".into())));
    }

    #[test]
    fn max_length_counts_characters_not_bytes() {
        let cyrillic = "ї".repeat(MEETUP_DETAILS_MAX_CHARS);
        assert_eq!(normalize_meetup_details(&cyrillic), Ok(Some(cyrillic.clone())));
    }

    #[test]
    fn over_max_length_is_refused() {
        let long = "a".repeat(MEETUP_DETAILS_MAX_CHARS + 1);
        assert_eq!(normalize_meetup_details(&long), Err(MeetupDetailsError::TooLong));
    }
}
```
Step 3 — implement:
```rust
/// Maximum meetup announcement length in characters; mirrored by the DB CHECK
/// (migration 20261004000002) and the admin textarea `maxlength`.
pub const MEETUP_DETAILS_MAX_CHARS: usize = 300;

#[cfg(any(feature = "ssr", test))]
#[derive(Debug, PartialEq, Eq)]
enum MeetupDetailsError {
    TooLong,
}

/// Trim; empty clears the announcement.
#[cfg(any(feature = "ssr", test))]
fn normalize_meetup_details(raw: &str) -> Result<Option<String>, MeetupDetailsError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().count() > MEETUP_DETAILS_MAX_CHARS {
        return Err(MeetupDetailsError::TooLong);
    }
    Ok(Some(trimmed.to_owned()))
}

/// Set or clear the meetup announcement of the most recent season (admin only).
///
/// # Errors
///
/// Returns `Err` if caller is not admin, the text exceeds
/// [`MEETUP_DETAILS_MAX_CHARS`], or there is no season / it is cancelled.
#[server]
pub async fn set_meetup_details(details: String) -> Result<(), ServerFnError> {
    use crate::auth;
    use crate::i18n::i18n::{Locale, td_string};

    let (pool, _user) = auth::require_admin().await?;

    let details = normalize_meetup_details(&details).map_err(|MeetupDetailsError::TooLong| {
        ServerFnError::new(td_string!(Locale::uk, season_error_meetup_too_long))
    })?;

    let affected = sqlx::query!(
        r#"
        UPDATE seasons
        SET meetup_details = $1
        WHERE id = (SELECT id FROM seasons ORDER BY created_at DESC LIMIT 1)
          AND phase <> 'cancelled'
        "#,
        details,
    )
    .execute(&pool)
    .await
    .map_err(db_err)?
    .rows_affected();

    if affected == 0 {
        return Err(ServerFnError::new(td_string!(Locale::uk, season_error_no_season_for_meetup)));
    }
    Ok(())
}
```

### M1.3 `src/admin/state.rs`
`AdminSeason` + `AdminSeasonRow`: add `pub meetup_details: Option<String>` / `meetup_details: Option<String>` after `theme`. Season query select list: add `s.meetup_details,` after `s.theme,`. Map into the literal. sqlx prepare.

### M1.4 `src/admin/page.rs`
- `AdminPage`: `let set_meetup_action = ServerAction::<SetMeetupDetails>::new();` (import `SetMeetupDetails` + `MEETUP_DETAILS_MAX_CHARS` from `super::season` / `crate::admin::season`, matching the file's import style). Add `set_meetup_action.version().get(),` to the `admin_state` source tuple; add `.or_else(|| set_meetup_action.value().get().and_then(Result::err))` to `action_error`; add toast Effect:
```rust
    Effect::new(move |_| {
        if let Some(Ok(())) = set_meetup_action.value().get() {
            set_toast.set(Some(t_string!(i18n, admin_meetup_saved_toast).into()));
        }
    });
```
- Thread `set_meetup_action: ServerAction<SetMeetupDetails>` as a new last-but-two parameter (before `hydrated`) through `render_season_section` → `render_active_season` (update both call sites).
- In `render_active_season`: `let meetup_details = season.meetup_details.clone();`; after the roster block (W2-R) insert:
```rust
            {if phase == crate::types::Phase::Cancelled {
                ().into_any()
            } else {
                render_meetup_form(meetup_details, set_meetup_action, hydrated, i18n)
            }}
```
Add:
```rust
/// Organizer form for the season's meetup announcement (empty = clear).
fn render_meetup_form(
    current: Option<String>,
    set_meetup_action: ServerAction<SetMeetupDetails>,
    hydrated: ReadSignal<bool>,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> AnyView {
    let pending = set_meetup_action.pending();
    view! {
        <leptos::form::ActionForm action=set_meetup_action>
            <div class="field">
                <label class="field-label" for="meetup-details">{t!(i18n, admin_meetup_label)}</label>
                <textarea
                    class="field-input"
                    id="meetup-details"
                    name="details"
                    rows="3"
                    maxlength=MEETUP_DETAILS_MAX_CHARS.to_string()
                    placeholder=move || t_string!(i18n, admin_meetup_placeholder)
                    data-testid="meetup-details-input"
                    aria-describedby="meetup-details-hint action-error"
                    aria-invalid=move || set_meetup_action.value().get().and_then(Result::err).map(|_| "true")
                >
                    {current.unwrap_or_default()}
                </textarea>
                <p id="meetup-details-hint" class="text-sm text-(--color-text-muted)">
                    {t!(i18n, admin_meetup_hint)}
                </p>
            </div>
            <button
                class="btn"
                data-variant="secondary"
                type="submit"
                data-testid="save-meetup-button"
                disabled=move || pending.get() || !hydrated.get()
                aria-busy=move || pending.get().then_some("true")
            >
                {move || if pending.get() {
                    t!(i18n, admin_meetup_saving_loading).into_any()
                } else {
                    t!(i18n, admin_meetup_save_button).into_any()
                }}
            </button>
        </leptos::form::ActionForm>
    }
    .into_any()
}
```

### M1.5 E2E
Spec constant (top, after `SEASON_THEME`): `const MEETUP_DETAILS = "15 листопада, 18:00 — Арт-простір «Купол», вул. Січових Стрільців, 1";`
POM (season section):
```ts
  async setMeetupDetails(text: string) {
    await this.page.goto("/admin");
    await expect(this.page.getByTestId("save-meetup-button")).toBeEnabled();
    await this.page.getByTestId("meetup-details-input").fill(text);
    await this.clickAndWaitForResponse(this.page.getByTestId("save-meetup-button"), "set_meetup_details");
  }

  /** Fresh SSR load proves persistence, not the typed DOM value. */
  async expectMeetupDetailsSaved(text: string) {
    await this.page.goto("/admin");
    await expect(this.page.getByTestId("meetup-details-input")).toHaveValue(text);
  }
```
Spec: insert after `"4.2 — participants can see season after launch"`:
```ts
    // Story 2.1 timeline AC (meetup window): organizer announces the meetup.
    test("2.1 — admin announces the meetup for the season", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(ADMIN_PHONE);
      await app.setMeetupDetails(MEETUP_DETAILS);
      await app.expectMeetupDetailsSaved(MEETUP_DETAILS);
    });
```
`visual-audit.spec.ts`: constant `const AUDIT_MEETUP = "22 листопада, 19:00 — Простір «Довга-довга назва для перевірки переносу рядків», вул. Велика Васильківська, 100, Київ";` next to `SEASON_THEME`. In test `"capture admin — signup phase (after launch)"`, directly after `await app.launchSeason();` add `await app.setMeetupDetails(AUDIT_MEETUP);` (if the test then needs `/admin` loaded, the existing `goToDashboard`/`goto` that follows handles it; if the test captures right after launch without navigating, add `await page.goto("/admin");` before its `captureState`).

## Verification Gates
```bash
psql "$DATABASE_URL" -c "UPDATE seasons SET meetup_details = repeat('x', 301)" 2>&1 | grep -c "violates check constraint"
```
(run on the unit DB after creating one season row via the app or `INSERT INTO seasons (signup_deadline, confirm_deadline) VALUES (now()+interval '1 day', now()+interval '2 days')`) → **REQUIRED:** `1`.
Standard gates: bare +5, ssr +5. E2E ×3: delta +1; titles: new + `capture admin — signup phase (after launch)`. Pixels: `admin-signup-phase`, `admin-season-complete`, `admin-delivery-roster` (4 dirs).

Commit: `feat(admin): let the organizer announce the season meetup`

---

# W2-M2 — Meetup + full timeline on participant home

## Why This Matters
Participants learn the meetup in-app; Story 2.1 timeline (signup deadline, creation deadline, meetup).

## What You Must Do
### M2.1 TDD — `src/pages/home.rs`
Step 1 — tests (red):
```rust
    #[test]
    fn participating_states_show_meetup() {
        for state in [
            HomeState::Confirmed,
            HomeState::Assigning,
            HomeState::Complete,
            HomeState::Enrolled { confirm_deadline: String::new(), city: String::new(), branch_number: 1 },
        ] {
            assert!(shows_meetup(&state), "{state:?}");
        }
    }

    #[test]
    fn non_participating_states_hide_meetup() {
        for state in [
            HomeState::NoSeason,
            HomeState::EnrollmentNotOpen,
            HomeState::NotParticipating,
            HomeState::Cancelled,
        ] {
            assert!(!shows_meetup(&state), "{state:?}");
        }
    }
```
Step 3 — implement (exhaustive, NO wildcard arm):
```rust
/// Whether the meetup announcement belongs on this page: the current season's
/// (would-be) participants only.
#[cfg(any(feature = "ssr", test))]
fn shows_meetup(state: &HomeState) -> bool {
    match state {
        HomeState::EnrollmentOpen { .. }
        | HomeState::Enrolled { .. }
        | HomeState::Preparing { .. }
        | HomeState::Confirmed
        | HomeState::Assigning
        | HomeState::Delivery { .. }
        | HomeState::Complete => true,
        HomeState::NoSeason
        | HomeState::EnrollmentNotOpen
        | HomeState::NotParticipating
        | HomeState::Cancelled => false,
    }
}
```

### M2.2 Server
- Add:
```rust
/// Home page payload: the state plus the season's meetup announcement when this
/// participant should see it.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HomeView {
    pub state: HomeState,
    pub meetup_details: Option<String>,
}
```
- `get_home_state` returns `Result<HomeView, ServerFnError>`. Move its current body into `#[cfg(feature = "ssr")] async fn resolve_home(pool: &sqlx::PgPool, user_id: uuid::Uuid) -> Result<(HomeState, Option<String>), ServerFnError>` returning `(state, season meetup_details)`; `get_home_state` = `require_auth` → `resolve_home` → `Ok(HomeView { meetup_details: meetup.filter(|_| shows_meetup(&state)), state })`.
- `SeasonInfoRow` + its query: add `meetup_details` (`… theme, meetup_details`). The most-recent-season query (W2-H1 shape) adds `meetup_details` to its select; its meetup is returned for the Complete/NotParticipating paths (filtered by `shows_meetup`).
- `HomeState::EnrollmentOpen` add `confirm_deadline: String`; `resolve_enrollment_state` passes `confirm_str.clone()` / `confirm_str` accordingly.
- sqlx prepare.

### M2.3 Render
- `HomePage`: Resource type follows; in the `Ok(view)` arm:
```rust
                                Ok(HomeView { state, meetup_details }) => view! {
                                    {render_home_state(state, enroll_action, confirm_action, receipt_action, hydrated, i18n)}
                                    {meetup_details.map(|details| render_meetup(details, i18n))}
                                }
                                .into_any(),
```
- Add:
```rust
fn render_meetup(
    details: String,
    i18n: leptos_i18n::I18nContext<crate::i18n::i18n::Locale>,
) -> impl IntoView {
    view! {
        <section class="card mt-(--density-space-md)" data-testid="season-meetup">
            <dl class="info-list">
                <div class="info-item">
                    <dt class="info-label">{t!(i18n, home_meetup_label)}</dt>
                    <dd class="info-value whitespace-pre-line" data-testid="season-meetup-details">{details}</dd>
                </div>
            </dl>
        </section>
    }
}
```
- `render_enrollment_open(deadline, confirm_deadline, …)`: give the signup paragraph `data-testid="enroll-signup-deadline"` and add directly after it:
```rust
        <p class="deadline" data-testid="enroll-confirm-deadline">
            {t!(i18n, home_enroll_confirm_deadline, deadline = confirm_deadline.to_string())}
        </p>
```

### M2.4 E2E
Spec import `formatDateUk` (W2-V). POM:
```ts
  async expectEnrollmentTimeline(signup: string, confirm: string) {
    await expect(this.page.getByTestId("enroll-signup-deadline")).toContainText(signup);
    await expect(this.page.getByTestId("enroll-confirm-deadline")).toContainText(confirm);
  }

  async expectMeetup(text: string) {
    await expect(this.page.getByTestId("season-meetup-details")).toHaveText(text);
  }

  async expectNoMeetup() {
    await expect(this.page.getByTestId("season-meetup")).not.toBeVisible();
  }
```
Spec:
- Replace test `"2.1 — season timeline and theme visible"` with title `"2.1 — season timeline visible: deadlines and meetup"`, body:
```ts
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectHomeContent(SEASON_THEME);
      await app.expectEnrollmentTimeline(formatDateUk(SIGNUP_DEADLINE), formatDateUk(CONFIRM_DEADLINE));
      await app.expectMeetup(MEETUP_DETAILS);
```
- Insert after `"2.3 — non-participant sees no recipient during delivery"`:
```ts
    test("2.3 — participant sees the meetup announcement during delivery", async ({ page }) => {
      const app = new MailClubPage(page);
      await app.login(PHONES.A);
      await app.goHome();
      await app.expectMeetup(MEETUP_DETAILS);
    });
```
  and add `await app.expectNoMeetup();` at the end of `"2.3 — non-participant sees no recipient during delivery"`.
- `"home screen — season complete message shown"`: append `await app.expectMeetup(MEETUP_DETAILS);`. `"home screen — non-participant sees not-participating after completion"`: append `await app.expectNoMeetup();`.

## Verification Gates
Mutation check: temporarily make `shows_meetup` return `true` for `NotParticipating` → `cargo test home::tests` **REQUIRED:** `non_participating_states_hide_meetup` fails. Revert.
Standard gates: bare +2, ssr +2. E2E ×3: delta +1; titles: new + the 4 edited. Pixels: `home-enrollment-available`, `home-enrolled`, `home-assignment-and-receipt-form`, `home-season-complete` (4 dirs; audit meetup from W2-M1).

Commit: `feat(home): show season timeline and meetup announcement`

---

# W2-DOC — Badge doc sync (opus)

## What You Must Do
`guidance/design-system.md` §Badges:
- Paragraph **"M2 — `pending` remains forward-prep."**: replace with one paragraph stating `pending` is emitted by the admin season roster for unconfirmed and awaiting-receipt participants (`.badge[data-status="pending"]`, `src/admin/page.rs` `render_roster`); keep the stepper-connector disambiguation sentence.
- Table row `error`: Meaning → `Failure (e.g. roster: participant reported mail not received)`.
No other change. Gate: `git diff --stat` = 1 file; `grep -n "forward-prep" guidance/design-system.md` → no line about `pending` being unemitted.

Commit: `docs(design-system): record roster badge usage`

---

## Testing Decisions
- Pure logic unit-tested through its function interface; DB-bound server fns covered by E2E (project rule).
- Modules: `home.rs` — `Participation::from_enrollment`, `preparation_state`, `assignment_state`, `shows_meetup` (bare + ssr). `types.rs` — `ReceiptStatus::can_transition_to` (bare + ssr). `state.rs` — `roster_status` (bare + ssr). `assignments.rs` — `forwarding_requests` incl. wraparound/all-fail/all-but-one (bare + ssr). `season.rs` — `normalize_meetup_details` (bare + ssr). `config.rs` — `site_url_from_var` (ssr only; module is ssr-gated). `admin/sms.rs` — `with_site_link`.
- Not unit-tested (justified): SQL wiring and rendering → E2E effect assertions with testids only.
- Every E2E change asserts an effect visible to the user or organizer; two mutation checks (W2-V, W2-H3) + one unit mutation check (W2-M2) prove non-vacuity.
- Prior art: `src/assignment.rs` tests, `src/types.rs` tests, `end2end/README.md` POM contract.

## Forbidden Patterns

### BANNED: signal-driven form values
```rust
// BANNED — input values in signals for ActionForm submission
<input on:input=move |ev| set_city.set(event_target_value(&ev)) />
```
The W2-H3 signal toggles VIEW MODE only.

### BANNED: `value=` attributes on ActionForm text inputs/textareas used for prefill
Prefill via placeholder (W2-H3) or textarea children (W2-M1) only.

### BANNED: `#[cfg]` tokens or bare `>` comparisons inside `view!`; `attr:` prefix on new native-element attributes
Use bare `aria-invalid`, `aria-busy` (existing `attr:aria-busy` lines are untouched).

### BANNED: wildcard arms on `HomeState` / `RosterStatus` / `ReceiptStatus` in new matches
New matches are exhaustive so a future variant forces a decision.

### BANNED: reading `receipt_status`/notes on the client to derive organizer status
Status derivation lives in `roster_status` (server) only.

### BANNED: format!() class names; testid selectors in CSS; new CSS files or classes
No `style/` change in wave 2. Existing classes + inline Tailwind utilities only.

### BANNED: E2E shortcuts
`waitForTimeout`, `networkidle`, `waitForLoadState`, `force: true`, `page.evaluate`, `getByText`, `getByRole` with name, CSS-class selectors, raw selectors in spec files for NEW assertions (add POM methods), imports from `@playwright/test` in specs.

### BANNED: environment-touching commands
`just e2e*`, `just db-reset`, `just _kill-stale`, port 3000, DB `samete`, bare `cargo sqlx prepare` without `-- --features ssr`, `git push`, `git merge`/commits to main, commits outside the worktree.

### BANNED: lint escapes
No new `#[allow(...)]` without a one-line WHY. Every new `pub fn` returning `Result` has `# Errors`.

### BANNED: scope creep
No: season edit, settings page, meetup SMS, RSVP, countdown, theme persistence, receipt-form time gate, organizer not-received SMS, changes to `uk.json` outside W2-U0, changes to wave-1 code beyond the lines named here. Out-of-scope findings → DONE_WITH_CONCERNS.

### BANNED: editing `locales/uk.json` in any unit except W2-U0
If a unit needs a string that W2-U0 did not add → BLOCKED.

## Definition of Done (per unit; binary)
1. Pre-flight gate passed (output pasted).
2. Every "What You Must Do" step applied; none skipped or substituted.
3. Unit-specific gates show REQUIRED OUTPUT (command + output pasted).
4. Standard gates: fmt clean; both clippy zero warnings; both test runs `ok`, `0 failed`, passed = baseline + stated delta.
5. `.sqlx/` regenerated with `-- --features ssr` when a query changed; `SQLX_OFFLINE=true` clippy passes.
6. E2E gate green 3 consecutive runs (W2-U0: 1), passed = baseline + stated delta, named titles passed.
7. Pixel PNG list provided (UI units).
8. `git diff --stat main...HEAD` = exactly the unit's write-set; `git status --short` clean.
9. One-line conventional commit (message given per unit); SHA reported.
10. Status DONE / DONE_WITH_CONCERNS / BLOCKED. A gate failing twice → STOP, report BLOCKED (task, output, attempts).

## Global Gates (orchestrator, on main after W2-M2 + W2-DOC)
```bash
SQLX_OFFLINE=true cargo clippy --no-default-features --features ssr -- -D warnings
SQLX_OFFLINE=true cargo clippy --target wasm32-unknown-unknown --features hydrate --no-default-features -- -D warnings
cargo test && SQLX_OFFLINE=true cargo test --features ssr
python3 - <<'EOF'
import json, re, pathlib
keys = json.load(open('locales/uk.json'))
src = "".join(p.read_text() for p in pathlib.Path('src').rglob('*.rs'))
print([k for k in keys if not re.search(r'\b' + re.escape(k) + r'\b', src)])
EOF
bash scripts/isolated-capture.sh e2e_w2_final full > /tmp/e2e_w2_final.log 2>&1; echo "exit=$?"
```
**REQUIRED:** clippy clean ×2; both test runs `ok`, `0 failed`, bare passed = wave-1 final + **33** (H1 9, R 5, F 8, H2 3, M1 5, M2 2, S 1), ssr passed = wave-1 final + **38** (bare set + S config 5); orphan list `[]`; `exit=0`, 0 failed, 2 skipped, passed = wave-1 final + **12** (H1 5, R 3, F 1, H2 1, M1 1, M2 1).

## Plan self-review
- Coverage: j-feel top 8 → C1 H2, C2 H2 (partial, D2), C3 H1, C4 S, C10 U0, C5 R, C6 F, C7 M1+M2. a-scope majors → branch edit H3, notes+who R, confirm-ready to non-enrolled H1. Vacuous tests → T:184/503/833/885 V; T:390/417 H3; T:398 M2; T:649 wave-1 U4 (D16); additionally T:586, T:763, T:780 (regex-only) converted in H1/R.
- Placeholders: none ("TBD"/"TODO"/"similar to" absent). Every new type/fn named in a later unit is defined in an earlier one: `Participation` (H1) → used H1 only; `HomeState::NotParticipating` (H1) → M2 `shows_meetup`; `HomeState::Delivery` (H2) → M2; `Enrolled { city, branch_number }` (H3) → M2 test literal; `RosterEntry/RosterStatus` (R) → page.rs R; `expectRosterStatus/expectRosterNote` (R) → H2 E2E; `forwarding-requests` testid (F) → H2 E2E; `formatDateUk` (V) → M2; `MEETUP_DETAILS` + `setMeetupDetails` (M1) → M2 + visual-audit; `cycleSenderId` (wave-1 U4) → F.
- Order sensitivity: H1 registers D after the 5.3 count test; R's roster size 3 relies on D unenrolled; H2's correction runs after F's forwarding assertion; M1's meetup is set before M2's timeline test.
