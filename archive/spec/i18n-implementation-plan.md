# Implementation Plan: i18n Migration — Externalize Ukrainian Strings

## Preamble

103+ Ukrainian string literals are hardcoded across 8 source files. This migration externalizes every one of them into `leptos_i18n` v0.6 — a compile-time i18n library that bakes strings into the binary and makes invalid keys compiler errors. The result is a single `locales/uk.json` file as the authoritative source of all user-facing Ukrainian text.

E2E tests are already decoupled from string content (all selectors use `data-testid`). The tests will not change.

---

## Phase 1: Infrastructure — Dependencies and Config Files

### Why This Matters

`leptos_i18n` reads `i18n.json` and `locales/uk.json` at compile time via `load_locales!()`. Nothing compiles until these exist.

### What You Must Do

**Step 1.1 — Create `i18n.json`** at the crate root (`/Users/ryzhakar/pp/same-te-mail-club/i18n.json`):

```json
{
  "default": "uk",
  "locales": ["uk"]
}
```

**Step 1.2 — Create `locales/` directory** and `locales/uk.json`:

```json
{
  "common_loading": "Завантаження...",
  "common_unknown": "невідомо",
  "common_send_button": "Надіслати / Send",

  "login_otp_sms_body": "Ваш код: { code }",

  "onboarding_page_title": "Налаштування акаунту",
  "onboarding_description": "Вкажіть ваше відділення Nova Poshta для отримання посилок.",
  "onboarding_branch_placeholder": "Відділення №1, Київ",
  "onboarding_error": "Помилка: { error }",

  "home_enroll_open_heading": "Відкрита реєстрація / Enrollment Open",
  "home_theme_label": "Тема / Theme: ",
  "home_signup_deadline_label": "Реєстрація до / Deadline: ",
  "home_guideline": "Надішліть щось, що відображає вас.",
  "home_enroll_branch_placeholder": "Відділення №1, Київ",
  "home_enroll_button": "Зареєструватись / Enroll",
  "home_enrolled_heading": "Ви зареєстровані / You are enrolled",
  "home_enrolled_desc": "Реєстрація підтверджена. Створіть свій лист.",
  "home_confirm_deadline_label": "Дедлайн підтвердження / Confirm deadline: ",
  "home_preparing_heading": "Підготовка / Preparation",
  "home_deadline_label": "Дедлайн / Deadline: ",
  "home_confirm_ready_button": "Підтвердити готовність / Confirm Ready",
  "home_ready_confirmed_heading": "Готовність підтверджена / Ready Confirmed",
  "home_waiting_assignment": "Очікуйте на розподіл.",
  "home_assigning_heading": "Розподіл / Assignment",
  "home_assigning_desc": "Зачекайте — скоро отримаєте повідомлення.",
  "home_assigned_heading": "Ваш отримувач / Your recipient",
  "home_name_label": "Ім'я / Name",
  "home_phone_label": "Телефон / Phone",
  "home_recipient_branch": "Відділення №{ branch_number }, { city }",
  "home_confirm_receipt_heading": "Підтвердити отримання / Confirm receipt",
  "home_receipt_note_placeholder": "Пошкоджена упаковка, неправильний пакет, тощо...",
  "home_received_button": "Отримав(ла) / Received",
  "home_not_received_button": "Не отримав(ла) / Not received",
  "home_thanks_heading": "Дякуємо! / Thanks!",
  "home_reported_label": "Повідомлено / Reported",
  "home_complete_heading": "Сезон завершено / Season Complete",
  "home_thanks_participation": "Дякуємо за участь!",

  "participants_page_title": "Учасники",
  "participants_register_section_title": "Зареєструвати учасника",
  "participants_list_title": "Список учасників",
  "participants_name_placeholder": "Іваненко Іван Іванович",
  "participants_register_button": "Зареєструвати",
  "participants_table_name": "Ім'я",
  "participants_table_phone": "Телефон",
  "participants_table_status": "Статус",
  "participants_table_actions": "Дії",
  "participants_status_active": "Активний",
  "participants_status_deactivated": "Деактивований",
  "participants_deactivate_button": "Деактивувати",
  "participants_deactivated_label": "Деактивовано",

  "season_page_title": "Управління сезоном",
  "season_create_form_title": "Створити новий сезон",
  "season_signup_deadline_label": "Дедлайн реєстрації (Signup deadline)",
  "season_confirm_deadline_label": "Дедлайн підтвердження (Confirm deadline)",
  "season_theme_label": "Тема сезону / Theme (необов'язково)",
  "season_theme_placeholder": "Наприклад: Перший сезон",
  "season_create_button": "Створити сезон",
  "season_current_section_title": "Поточний сезон",
  "season_phase_label": "Фаза / Phase",
  "season_theme_display_label": "Тема",
  "season_signup_deadline_display": "Реєстрація до",
  "season_confirm_deadline_display": "Підтвердження до",
  "season_enrolled_label": "Зареєстровано / Enrolled",
  "season_confirmed_label": "Підтверджено / Confirmed",
  "season_launch_button": "Запустити сезон / Launch",
  "season_advance_button": "Перейти до наступної фази / Advance",
  "season_cancel_button": "Скасувати сезон / Cancel",

  "dashboard_no_season": "Немає активного сезону. / No active season.",
  "dashboard_create_season_button": "Створити сезон / Create season",
  "dashboard_phase_label": "Фаза / Phase",
  "dashboard_theme_label": "Тема",
  "dashboard_enrolled_label": "Зареєстровано / Enrolled",
  "dashboard_confirmed_label": "Підтверджено / Confirmed",
  "dashboard_not_received_label": "Не отримано / Not received: ",

  "assignments_page_title": "Розподіл / Assignments",
  "assignments_released_advance_note": "Опубліковано / Released — assignments confirmed. Advance season to Delivery to make them visible.",
  "assignments_confirmed_label": "Підтверджено / Confirmed: ",
  "assignments_no_season": "Немає активного сезону / No active season",
  "assignments_regenerate_button": "Перегенерувати / Regenerate",
  "assignments_generate_button": "Згенерувати / Generate",
  "assignments_released_note": "Опубліковано / Released — assignments are visible to participants.",
  "assignments_release_button": "Опублікувати / Release",
  "assignments_cohort_label": "Когорта / Cohort ",
  "assignments_cycles_label": "Цикли / Cycles",
  "assignments_apply_button": "Застосувати / Apply",

  "sms_season_open_body": "Новий сезон Mail Club відкрито! Зайди в додаток для реєстрації.",
  "sms_assignment_body": "Твоє призначення готове! Зайди в додаток щоб побачити адресата.",
  "sms_confirm_nudge_body": "Нагадування: підтверди готовність листа до { deadline }.",
  "sms_receipt_nudge_body": "Ти отримав/ла лист? Підтверди в додатку.",

  "sms_sent_label": "Надіслано / Sent: ",
  "sms_failed_label": "Невдало / Failed: ",
  "sms_season_open_section_title": "Відкриття сезону / Season Open",
  "sms_season_open_target": "Усі активні учасники / All active users",
  "sms_assignment_section_title": "Призначення / Assignment",
  "sms_assignment_target": "Відправники без повідомлення / Senders not yet notified",
  "sms_confirm_nudge_section_title": "Нагадування підтвердження / Confirm Nudge",
  "sms_confirm_nudge_target": "Зареєстровані без підтвердження / Enrolled without confirmation",
  "sms_receipt_nudge_section_title": "Нагадування отримання / Receipt Nudge",
  "sms_receipt_nudge_target": "Отримувачі без відповіді / Recipients without confirmation"
}
```

**Step 1.3 — Modify `Cargo.toml`**:

In `[dependencies]` add:
```toml
leptos_i18n = { version = "0.6", features = [] }
```

In `[features]`, append to the `hydrate` and `ssr` lists:
```toml
hydrate = [
    # ...existing entries...
    "leptos_i18n/hydrate",
]
ssr = [
    # ...existing entries...
    "leptos_i18n/ssr",
]
```

**Step 1.4 — Create `src/i18n.rs`**:

```rust
leptos_i18n::load_locales!();
```

**Step 1.5 — Modify `src/lib.rs`**:

Add `pub mod i18n;` immediately after the `pub mod app;` line. Do not put it inside a `#[cfg(feature)]` gate — it must be compiled for both `ssr` and `hydrate`.

**Step 1.6 — Modify `src/app.rs`**:

Add the import at the top:
```rust
use crate::i18n::provide_i18n_context;
```

In the `App` component body, after `provide_meta_context();`, add:
```rust
provide_i18n_context();
```

### Verification Gate 1

```bash
cargo check --features ssr 2>&1 | grep -E "^error" | head -20
```

**REQUIRED OUTPUT:** Zero lines starting with `error`. If `load_locales!()` or the import fails, fix it before proceeding.

---

## Phase 2: Component String Migration (8 Files)

### Why This Matters

Every `view!` macro that renders a Ukrainian string literal must call `t!()` instead. Every `placeholder` attribute that holds a Ukrainian string must call `t_string!()`. The compiler and verification grep will catch any missed string.

### Usage Reference (memorize this — no deviations)

**Text content in `view!`:**
```rust
// At component function top (ONCE per component):
let i18n = use_i18n();

// Inside view!:
<h1>{t!(i18n, my_key)}</h1>
<p>{t!(i18n, my_key)}</p>
<button>{t!(i18n, my_key)}</button>
```

**HTML attributes (placeholder, etc.):**
```rust
<input placeholder=move || t_string!(i18n, my_key) />
<textarea placeholder=move || t_string!(i18n, my_key)></textarea>
```

**Variable interpolation in `view!`:**
```rust
// locale file: "my_key": "Prefix { var_name } suffix"
{t!(i18n, my_key, var_name = rust_value)}
```

**Variable interpolation producing a String (for signals, Effects):**
```rust
t_string!(i18n, my_key, var_name = rust_value)
```

**Imports for components:**
```rust
use crate::i18n::{use_i18n, t, t_string};
// OR — if the macros are re-exported at crate root:
use leptos_i18n::{t, t_string};
use crate::i18n::use_i18n;
```

Determine the correct import path by checking what `load_locales!()` generates via `cargo doc` or LSP. Do not guess.

### What You Must Do — File by File

#### `src/admin/participants.rs`

Add `let i18n = use_i18n();` at the top of each component function body that contains Ukrainian strings.

Affected functions: `RegisterForm`, `ParticipantList`, `ParticipantsPage`.

| Location | Old | New |
|---|---|---|
| RegisterForm, placeholder | `placeholder="Іваненко Іван Іванович"` | `placeholder=move \|\| t_string!(i18n, participants_name_placeholder)` |
| RegisterForm, button | `"Зареєструвати"` | `{t!(i18n, participants_register_button)}` |
| ParticipantList, Suspense fallback | `"Завантаження..."` | `{t!(i18n, common_loading)}` |
| ParticipantList, th × 4 | `"Ім'я"`, `"Телефон"`, `"Статус"`, `"Дії"` | `{t!(i18n, participants_table_name)}` etc. |
| ParticipantList, status active | `"Активний"` | `t!(i18n, participants_status_active)` |
| ParticipantList, status deactivated | `"Деактивований"` | `t!(i18n, participants_status_deactivated)` |
| ParticipantList, deactivate button | `"Деактивувати"` | `{t!(i18n, participants_deactivate_button)}` |
| ParticipantList, inactive span | `"Деактивовано"` | `{t!(i18n, participants_deactivated_label)}` |
| ParticipantsPage, h1 | `"Учасники"` | `{t!(i18n, participants_page_title)}` |
| ParticipantsPage, h2 register | `"Зареєструвати учасника"` | `{t!(i18n, participants_register_section_title)}` |
| ParticipantsPage, h2 list | `"Список учасників"` | `{t!(i18n, participants_list_title)}` |

Note on `"Активний"` / `"Деактивований"` in if/else: These are currently `&str` in an `{if ...}` branch. They become `t!()` calls which return `impl IntoView`. The branch types will differ unless you wrap:
```rust
{move || if active {
    view! { {t!(i18n, participants_status_active)} }.into_any()
} else {
    view! { {t!(i18n, participants_status_deactivated)} }.into_any()
}}
```
Use LSP to confirm what type `t!()` returns; adjust if both branches return the same type and `into_any()` is not needed.

#### `src/admin/season.rs`

Add `let i18n = use_i18n();` to each component function with Ukrainian strings. Read the file to identify the component boundaries.

| Key | Value to replace |
|---|---|
| `season_create_form_title` | `"Створити новий сезон"` |
| `season_signup_deadline_label` | `"Дедлайн реєстрації (Signup deadline)"` |
| `season_confirm_deadline_label` | `"Дедлайн підтвердження (Confirm deadline)"` |
| `season_theme_label` | `"Тема сезону / Theme (необов'язково)"` |
| `season_theme_placeholder` | `placeholder="Наприклад: Перший сезон"` → `placeholder=move \|\| t_string!(i18n, season_theme_placeholder)` |
| `season_create_button` | `"Створити сезон"` |
| `season_current_section_title` | `"Поточний сезон"` |
| `season_phase_label` | `"Фаза / Phase"` |
| `season_theme_display_label` | `"Тема"` |
| `season_signup_deadline_display` | `"Реєстрація до"` |
| `season_confirm_deadline_display` | `"Підтвердження до"` |
| `season_enrolled_label` | `"Зареєстровано / Enrolled"` |
| `season_confirmed_label` | `"Підтверджено / Confirmed"` |
| `season_launch_button` | `"Запустити сезон / Launch"` |
| `season_advance_button` | `"Перейти до наступної фази / Advance"` |
| `season_cancel_button` | `"Скасувати сезон / Cancel"` |
| `season_page_title` | `"Управління сезоном"` |
| `common_loading` | `"Завантаження..."` |

#### `src/admin/dashboard.rs`

| Key | Value to replace |
|---|---|
| `common_loading` | `"Завантаження..."` |
| `dashboard_no_season` | `"Немає активного сезону. / No active season."` |
| `dashboard_create_season_button` | `"Створити сезон / Create season"` (appears twice) |
| `dashboard_phase_label` | `"Фаза / Phase"` |
| `dashboard_theme_label` | `"Тема"` |
| `dashboard_enrolled_label` | `"Зареєстровано / Enrolled"` |
| `dashboard_confirmed_label` | `"Підтверджено / Confirmed"` |
| `dashboard_not_received_label` | `"Не отримано / Not received: "` |

#### `src/admin/assignments.rs`

| Key | Value to replace |
|---|---|
| `assignments_page_title` | `"Розподіл / Assignments"` |
| `assignments_released_advance_note` | full string at line 659 |
| `common_loading` | `"Завантаження..."` |
| `assignments_confirmed_label` | `"Підтверджено / Confirmed: "` |
| `assignments_no_season` | `"Немає активного сезону / No active season"` |
| `assignments_regenerate_button` | `"Перегенерувати / Regenerate"` |
| `assignments_generate_button` | `"Згенерувати / Generate"` |
| `assignments_released_note` | full string at line 724 |
| `assignments_release_button` | `"Опублікувати / Release"` |
| `assignments_cohort_label` | `"Когорта / Cohort "` |
| `assignments_cycles_label` | `"Цикли / Cycles"` |
| `assignments_apply_button` | `"Застосувати / Apply"` |

#### `src/pages/onboarding.rs`

Affected functions: `OnboardingPage` (and the Effect closure inside it).

| Key | What changes |
|---|---|
| `onboarding_page_title` | `"Налаштування акаунту"` in h1 |
| `onboarding_description` | `"Вкажіть ваше відділення Nova Poshta для отримання посилок."` in p |
| `onboarding_branch_placeholder` | `placeholder="Відділення №1, Київ"` → `placeholder=move \|\| t_string!(i18n, onboarding_branch_placeholder)` |
| `onboarding_error` | format string in `Effect` |

The `Effect` case requires special handling. Replace:
```rust
set_error_msg.set(Some(format!("Помилка: {e}")));
```
With:
```rust
set_error_msg.set(Some(t_string!(i18n, onboarding_error, error = e)));
```
The `i18n` value is captured from the component scope (declared once at the top of `OnboardingPage`, before the `Effect`).

#### `src/pages/home.rs`

This is the largest file. Add `let i18n = use_i18n();` at the top of `render_home_state` function (since that's where most strings live) AND at the top of `HomePage` component (for the `<Suspense fallback=...>`).

Actually: read the file to determine whether `render_home_state` is a function or closure and whether it has access to `i18n`. If it's a standalone function, pass `i18n` as a parameter. Do not restructure the code beyond the minimum needed.

| Key | Location / old string |
|---|---|
| `common_loading` | Suspense fallback `"Завантаження..."` |
| `home_enroll_open_heading` | `"Відкрита реєстрація / Enrollment Open"` |
| `home_theme_label` | `"Тема / Theme: "` |
| `home_signup_deadline_label` | `"Реєстрація до / Deadline: "` |
| `home_guideline` | `"Надішліть щось, що відображає вас."` |
| `home_enroll_branch_placeholder` | `placeholder="Відділення №1, Київ"` → `t_string!` |
| `home_enroll_button` | `"Зареєструватись / Enroll"` |
| `home_enrolled_heading` | `"Ви зареєстровані / You are enrolled"` |
| `home_enrolled_desc` | `"Реєстрація підтверджена. Створіть свій лист."` |
| `home_confirm_deadline_label` | `"Дедлайн підтвердження / Confirm deadline: "` |
| `home_preparing_heading` | `"Підготовка / Preparation"` |
| `home_deadline_label` | `"Дедлайн / Deadline: "` |
| `home_confirm_ready_button` | `"Підтвердити готовність / Confirm Ready"` |
| `home_ready_confirmed_heading` | `"Готовність підтверджена / Ready Confirmed"` |
| `home_waiting_assignment` | `"Очікуйте на розподіл."` |
| `home_assigning_heading` | `"Розподіл / Assignment"` |
| `home_assigning_desc` | `"Зачекайте — скоро отримаєте повідомлення."` |
| `home_assigned_heading` | `"Ваш отримувач / Your recipient"` |
| `home_name_label` | `"Ім'я / Name"` |
| `home_phone_label` | `"Телефон / Phone"` |
| `home_recipient_branch` | `format!("Відділення №{recipient_branch_number}, {recipient_city}")` → `t!(i18n, home_recipient_branch, branch_number = recipient_branch_number, city = recipient_city)` |
| `home_confirm_receipt_heading` | `"Підтвердити отримання / Confirm receipt"` |
| `home_receipt_note_placeholder` | `placeholder="Пошкоджена упаковка..."` → `t_string!` |
| `home_received_button` | `"Отримав(ла) / Received"` |
| `home_not_received_button` | `"Не отримав(ла) / Not received"` |
| `home_thanks_heading` | `"Дякуємо! / Thanks!"` |
| `home_reported_label` | `"Повідомлено / Reported"` |
| `home_complete_heading` | `"Сезон завершено / Season Complete"` |
| `home_thanks_participation` | `"Дякуємо за участь!"` |

#### `src/admin/sms.rs` — UI strings only (server fn strings in Phase 3)

Add `let i18n = use_i18n();` to `SmsPage` component.

| Key | Old string |
|---|---|
| `sms_sent_label` | `"Надіслано / Sent: "` |
| `sms_failed_label` | `"Невдало / Failed: "` |
| `sms_season_open_section_title` | `"Відкриття сезону / Season Open"` |
| `sms_season_open_target` | `"Усі активні учасники / All active users"` |
| `common_send_button` | `"Надіслати / Send"` (4 occurrences) |
| `sms_assignment_section_title` | `"Призначення / Assignment"` |
| `sms_assignment_target` | `"Відправники без повідомлення / Senders not yet notified"` |
| `sms_confirm_nudge_section_title` | `"Нагадування підтвердження / Confirm Nudge"` |
| `sms_confirm_nudge_target` | `"Зареєстровані без підтвердження / Enrolled without confirmation"` |
| `sms_receipt_nudge_section_title` | `"Нагадування отримання / Receipt Nudge"` |
| `sms_receipt_nudge_target` | `"Отримувачі без відповіді / Recipients without confirmation"` |

### Verification Gate 2

```bash
cargo clippy --features ssr -- -D warnings 2>&1 | grep -c "^error"
```

**REQUIRED OUTPUT:** `0`

---

## Phase 3: Server Function String Migration

### Why This Matters

Server functions (`#[server]`) are Axum handlers. They do not run inside the Leptos component tree, so `use_i18n()` is not available. This is the only complex case in the migration. There are 5 strings across 2 files.

### What You Must Do

**Step 3.1 — Determine the correct API for context-free locale access.**

After `load_locales!()` compiles, run:
```bash
cargo doc --features ssr --open 2>/dev/null; grep -r "pub fn\|pub struct\|impl " target/doc/samete/i18n/ 2>/dev/null | head -40
```

You are looking for one of these patterns in the generated `i18n` module:
- `I18n::default()` — if `I18n` implements `Default` and defaults to `uk`
- `Locale::Uk.get_keys()` — if the locale exposes string keys directly
- `t_string!(Locale::Uk, key)` — if `t_string!` accepts a `Locale` value directly
- Some `I18n::from_locale(Locale::Uk)` constructor

Use LSP diagnostics (not guessing) to determine which pattern compiles.

**Step 3.2 — Add a server-only helper to `src/i18n.rs`** if the API requires it:

```rust
#[cfg(feature = "ssr")]
pub fn server_i18n() -> I18n {
    // use whichever API compiles:
    I18n::default()
}
```

This function must not be gated behind `not(feature = "ssr")`. It is SSR-only because server functions are SSR-only.

**Step 3.3 — Replace strings in `src/admin/sms.rs` server functions:**

| Function | Old | New |
|---|---|---|
| `send_season_open_sms` | `"Новий сезон Mail Club відкрито!..."` | `t_string!(server_i18n(), sms_season_open_body)` or equivalent |
| `send_assignment_sms` | `"Твоє призначення готове!..."` | `t_string!(server_i18n(), sms_assignment_body)` |
| `send_confirm_nudge_sms` | `format!("Нагадування:...{deadline_str}...")` | `t_string!(server_i18n(), sms_confirm_nudge_body, deadline = deadline_str)` |
| `send_confirm_nudge_sms` | `.unwrap_or_else(\|_\| String::from("невідомо"))` | `.unwrap_or_else(\|_\| t_string!(server_i18n(), common_unknown))` |
| `send_receipt_nudge_sms` | `"Ти отримав/ла лист?..."` | `t_string!(server_i18n(), sms_receipt_nudge_body)` |

**Step 3.4 — Replace string in `src/pages/login.rs` server function:**

| Function | Old | New |
|---|---|---|
| `request_otp` | `format!("Ваш код: {code}")` | `t_string!(server_i18n(), login_otp_sms_body, code = code)` |

`server_i18n()` is imported from `crate::i18n::server_i18n`. If `server_i18n` is added to `src/i18n.rs`, the import is `use crate::i18n::server_i18n;`.

### Verification Gate 3

```bash
cargo clippy --features ssr -- -D warnings 2>&1 | grep -c "^error"
```

**REQUIRED OUTPUT:** `0`

```bash
grep -r '"[А-Яа-яІіЇїЄєҐґ]' src/ 2>/dev/null
```

**REQUIRED OUTPUT:** Zero lines. If any Cyrillic string literals remain, they have not been migrated.

---

## Phase 4: Final Verification

### What You Must Do

Run each gate in sequence. Do not proceed to the next gate if the previous fails.

**Gate 4.1 — Clippy (SSR feature):**
```bash
cargo clippy --features ssr -- -D warnings 2>&1 | tail -5
```
**REQUIRED:** No lines containing `error[`.

**Gate 4.2 — Unit tests:**
```bash
cargo test 2>&1 | tail -5
```
**REQUIRED:** Output contains `test result: ok`.

**Gate 4.3 — No Ukrainian raw literals remain:**
```bash
grep -rn '"[А-Яа-яІіЇїЄєҐґ]' src/
```
**REQUIRED:** Zero output (no matches, no file names printed).

**Gate 4.4 — Locale file completeness check:**
```bash
# Every key used in src/ must exist in locales/uk.json.
# Extract all t!() and t_string!() key arguments:
grep -roh 't!\|t_string!' src/ --include="*.rs" | wc -l
```
This is informational. If any key is misspelled, Gate 4.1 will have already caught it (compile-time key validation).

**Gate 4.5 — E2E (run last, once only):**
```bash
just e2e > /tmp/e2e-i18n.log 2>&1; tail -10 /tmp/e2e-i18n.log
```
**REQUIRED:** Log ends with `56 passed` (or equivalent passing count). If E2E fails, do not re-run. Read the log, identify the specific test and error, fix it, re-run once.

---

## Forbidden Patterns

### BANNED: Skipping keys by leaving Ukrainian strings in place
```rust
// BANNED — this string will be caught by Gate 4.3
<h1>"Учасники"</h1>
```

### BANNED: Wrapping Ukrainian strings in helper functions to dodge the grep
```rust
// BANNED — this still has a Cyrillic literal in src/
fn title() -> &'static str { "Учасники" }
```

### BANNED: `#[allow(clippy::...)]` without a comment
```rust
// BANNED
#[allow(clippy::too_many_lines)]
```
(Note: `#[allow(clippy::too_many_lines)]` already exists in the codebase with comments. New ones require justification.)

### BANNED: Adding keys to `locales/uk.json` that are not in this plan
The complete key list is final. Do not add, rename, or remove keys. Any mismatch between the JSON and the `t!()` call is a compile error — which is the point.

### BANNED: Modifying `data-testid` attributes
E2E tests depend on them. Touch nothing.

### BANNED: Restructuring component logic
Do not refactor `render_home_state`, split components, or move code. Minimum viable change only.

### BANNED: Running `just e2e` more than once at the end
It's a 3-minute build. Use `cargo clippy --features ssr -- -D warnings` for iteration.

---

## Definition of Done

All must be true:

1. `i18n.json` exists at crate root with `{"default": "uk", "locales": ["uk"]}`
2. `locales/uk.json` exists at crate root with all keys from this plan
3. `src/i18n.rs` exists with `leptos_i18n::load_locales!();`
4. `src/lib.rs` has `pub mod i18n;` (not feature-gated)
5. `src/app.rs` calls `provide_i18n_context()` in `App` component
6. `cargo clippy --features ssr -- -D warnings` produces zero errors
7. `cargo test` produces `test result: ok`
8. `grep -rn '"[А-Яа-яІіЇїЄєҐґ]' src/` produces zero output
9. `just e2e` reports all tests passing
