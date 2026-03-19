
# UI/UX Audit Report — Саме Те Mail Club

**Date:** 2026-03-18
**Auditor:** Claude Code (claude-sonnet-4-6) — full browser walkthrough via Chrome automation
**Server:** http://localhost:3000
**DB State at audit start:** Season "Перший сезон" in Delivery phase, 3 participants confirmed and assigned
**Audit method:** Live browser inspection, CSS computed style extraction, JS font API queries, source code analysis

---

## Screenshot Coverage

1. Login page — phone step (dark surface, logo mark, phone input + button, label visible)
2. Login page — OTP step (same layout, OTP input with placeholder "000000")
3. Login page — phone input focused (blue glow ring visible)
4. Post-login admin user lands on "/" (participant "Assigning" state — bug)
5. Admin dashboard `/admin` — "Панель" h1, season stats DL, admin nav
6. Admin participants `/admin/participants` — register form + data table with active badges + deactivate buttons
7. Admin season `/admin/season` — "Управління сезоном" h1, current season DL with raw ISO dates, Далі + Скасувати buttons
8. Admin SMS `/admin/sms` (scrolled) — 4 SMS trigger cards fully visible
9. Admin assignments `/admin/assignments` — plaintext cycle visualization "А → Б → В → А"
10. Participant delivery state "/" — "Твій отримувач" h2, recipient details DL, receipt confirmation textarea + two buttons
11. Participant receipt confirmed "/" — "Дякуємо!" h2 + "Повідомлено організатору" body text

---

## Design System Compliance

### Typography

**Spec:** CyGrotesk 900 for h1/h2; Mont 400/600 for body/labels; h1 at clamp(1.8rem,5vw,2.8rem) / line-height 1.15; h2 at 1.3rem / line-height 1.15; body 1.05rem / line-height 1.75.

**Observed (computed via browser):**
- CyGrotesk `status: loaded` when heading elements are present; `status: unloaded` on login page (no heading). Font loads on demand via `font-display: swap`. No loading failure.
- h1 computed: `fontFamily: "CyGrotesk, Arial Black, sans-serif"`, `fontSize: 44.8px`, `fontWeight: 900`, `lineHeight: 51.52px` (= 44.8 × 1.15). Letter-spacing: -0.896px (= -0.02em). **All correct.**
- h2 computed: `fontFamily: "CyGrotesk..."`, `fontSize: 20.8px`, `lineHeight: 23.92px` (= 1.15). **Correct.**
- Body `fontFamily: "Mont, Inter, system-ui"` — correct stack.
- Label `fontWeight: 600`, `fontSize: 14px` — correct.
- Button `fontFamily: Mont`, `fontSize: 14px`, `fontWeight: 600` — correct.
- OTP input: `fontVariantNumeric: tabular-nums`, `letterSpacing: 1.6px` (= 0.1em at 16px). `data-otp` attribute confirmed on OTP input element.
- Tel input: same tabular-nums + letter-spacing via `.field-input[type="tel"]` rule.
- Mont 400 loaded; Mont 600 loads when bold elements are present.

**Note on @font-face inside @layer:** Declared in `@layer base`. Chrome's `cssRules` API does not enumerate FONT_FACE_RULE from within layers at the top level. Despite this API quirk, the fonts load correctly — the browser processes them. This is technically within spec for CSS Cascading Level 5 but not all older user agents handle it. Not a practical issue for this app's target audience.

**PASS** — Typography fully compliant in both measurement and visual rendering.

---

### Color

**Spec:** Six brand tokens in oklch; contrast-corrected orange at `oklch(0.63 0.22 31)`; semantic aliases; dark mode reassignment of surface/text tokens.

**Observed (extracted via `getComputedStyle`):**
- `--color-brand-orange`: `oklch(63% .22 31)` — **CORRECT** (contrast-corrected, not the failing `#FB4417`)
- `--color-accent`: `oklch(63% .22 31)` — resolves correctly
- `--color-surface` (dark mode active): `oklch(15% 0 0)` — brand-black. **CORRECT.**
- `--color-text` (dark mode): `oklch(98% .01 90)` — brand-cream. **CORRECT.**
- `--color-text-muted` (dark mode): `oklch(65% .01 250)` — dark-mode override. **CORRECT.**
- `--color-focus`: `oklch(78% .11 240)` — brand-blue. **CORRECT.**
- `--color-error`: `oklch(55% .22 25)`. **CORRECT.**
- `--color-success`: `oklch(58% .16 160)`. **CORRECT.**
- Button (primary) background: `oklch(0.63 0.22 31)`. **CORRECT.**
- Destructive button: `oklch(0.55 0.22 25)`. **CORRECT.**
- Input background (dark mode): `oklch(0.18 0.01 250)` — `--color-surface-raised` dark mode value. **CORRECT.**
- Badge (active): `oklch(0.58 0.16 160)` — success green. **CORRECT.**
- Body background: `oklch(0.15 0 0)` — brand-black. **CORRECT.**

The auditing machine is in system dark mode, so dark mode token reassignment is active. All semantic aliases are observed in their dark-mode resolved state.

**PASS** — All color values match spec precisely.

---

### Components

#### Buttons

**Spec:** Pill shape `border-radius: 100px`; primary/secondary/destructive variants via `data-variant`; padding 0.625rem 1.25rem; font-size 0.875rem; disabled opacity 0.45; hover `brightness(0.9)`; focus-visible ring.

**Observed:**
- `borderRadius: 1.67772e+07px` — this is `calc(infinity * 1px)` as computed by Chrome. **Correct pill shape.**
- Primary bg: `oklch(0.63 0.22 31)`, color: white, padding: `10px 20px`. **CORRECT.**
- Secondary (Далі button): transparent bg, `border: 1.5px solid oklch(0.98 0.01 90)`. **CORRECT.**
- Destructive: `oklch(0.55 0.22 25)` bg, white text. **CORRECT.**
- Font: Mont 14px 600. **CORRECT.**
- All ActionForm submit buttons carry `disabled=move || !hydrated.get()` — confirmed in source (home.rs, onboarding.rs, login.rs, participants.rs, season.rs, assignments.rs, sms.rs).
- OTP verify button (native `<form method="post">`) intentionally has no hydration gate. **Correct.**
- `data-size` attribute is defined in CSS but unused in any component — minor gap, not spec violation.

**PASS** — Button system fully compliant.

#### Form Fields

**Spec:** `.field` > `.field-label` + `.field-input` + `.field-error`; 1.5px border brand-gray; border-radius 0.5rem; focus ring blue; `aria-describedby`, `aria-invalid`, `aria-live`.

**Observed:**
- All form inputs use `.field-input` class. Confirmed: login phone input, login OTP input, onboarding branch input, enroll branch input, participants register form, season create form, textarea in receipt form.
- `border: 1.5px solid oklch(0.45 0.01 250)` — CORRECT.
- `borderRadius: 8px` = 0.5rem — CORRECT.
- `padding: 10px 12px` = 0.625rem 0.75rem — CORRECT.
- Focus state: `border-color: --color-focus`, `box-shadow: 0 0 0 3px oklch(from ... / 0.15)`. **Visually confirmed with blue glow ring in screenshot.**
- `.field-error` class: defined in CSS but no component uses it. Action errors render as `<p class="alert">` (home.rs, onboarding.rs). This deviates from the `.field` > `.field-error` pattern — errors are not inside `.field` wrappers.
- **ARIA attributes: none found.** No `aria-describedby`, no `aria-invalid`, no `aria-live="assertive"` on any form element in any component. This is a spec compliance failure and a WCAG 2.1 AA gap.

**PARTIAL** — Visually compliant. ARIA linkage absent across all forms.

#### Badges

**Spec:** Pill shape; Mont 600; text-xs; uppercase; letter-spacing 0.02em; `data-status` variants.

**Observed:**
- Participant table shows "АКТИВНИЙ" badge — rendered via `.badge[data-status="active"]`. `background: oklch(0.58 0.16 160)`, borderRadius pill, fontSize 12px, textTransform uppercase, letterSpacing 0.02em. **CORRECT and in use.**
- "Деактивувати" button uses `data-variant="destructive"`. **CORRECT.**
- No other badge usage confirmed (phase display in dashboard uses plain text `dd`, not badges).

**PARTIAL** — Badge component works correctly where used. Underutilized: season phase, dashboard stats, and participant confirmation status are plain text instead of badges.

#### Content Container

**Spec:** `max-width: 65ch; margin-inline: auto; padding-inline: 1rem`

**Observed:**
- Participant home: `containerMaxWidth: 646.88px` at 1344px viewport. 65ch at ~9.95px per char = 647px. **CORRECT.**
- All pages use `.prose-page` wrapper — confirmed in source for all 8 component files.

**PASS**

---

### Spacing and Density

**Spec:** Admin density via `[data-layout="admin"]` on layout root; participant spacing-3/5/8, admin spacing-1.5/3/5.

**Observed:**
- `[data-layout="admin"]` element present in DOM (`AdminGuard` wrapper in app.rs line 195).
- On the `[data-layout="admin"]` element: `--density-space-sm: .375rem`, `--density-space-md: .75rem`, `--density-space-lg: 1.25rem`. **CORRECT admin values.**
- On `:root`: participant defaults. Querying root gives participant values — density override correctly scoped to admin element.

**PASS** — Admin density tokens apply correctly.

---

### Grain Overlay

**Spec:** `body::after`; `position: fixed; inset: 0; z-index: 1; opacity: 0.04; mix-blend-mode: overlay`; `prefers-reduced-motion: reduce` hides it.

**Observed (getComputedStyle):**
- content: `""` — present
- position: `fixed` — CORRECT
- z-index: `1` — CORRECT (not 100)
- opacity: `0.04` — CORRECT
- mix-blend-mode: `overlay` — CORRECT (not multiply)
- backgroundImage: SVG `feTurbulence` data URI present
- `prefers-reduced-motion: reduce` rule confirmed in source

**PASS** — Grain overlay fully compliant.

---

### Focus Rings

**Spec:** `:focus { outline: none }`, `:focus-visible { outline: 2px solid --color-focus; outline-offset: 2px }`.

**Observed:**
- Global rules confirmed in CSS.
- Input focus ring: blue border shift + `box-shadow: 0 0 0 3px` glow. **Visually confirmed in screenshot** (phone input focused state showed distinct blue glow ring).
- Button `:focus-visible` uses `--_ring` (defaults to `--color-focus`).
- Destructive button ring overrides to `--color-error`.

**PASS**

---

### Logo Usage

**Spec:** Nav header: `same_te_mark_orange.svg`; auth hero: `logo.svg`; footer (dark): `logo-white.svg`.

**Observed:**
- Public directory: `logo.svg`, `logo-white.svg`, `favicon.svg` present. `same_te_mark_orange.svg` and `same_te_mark_white.svg` **absent**.
- `app.rs` header (`<header class="app-header">`): `<img src="/logo.svg">` — uses `logo.svg` for both nav and hero.
- Login page `<main>`: `<img src="/logo.svg" class="h-20 w-auto mb-8">` — correct size for hero.
- Header `img` height: `32px` (from `.app-header img { height: 2rem }`) — at this size, the SVG renders as just the mark, visually acceptable.
- No `logo-white.svg` usage found in any component.

**PARTIAL** — `logo.svg` appears to be the mark-only SVG (not full wordmark with "Саме Те" text), making nav usage visually tolerable. But the spec intended two distinct files: a mark-only SVG for the nav and a full logo for the hero. Dark-mode white variant (`logo-white.svg`) is not used anywhere.

---

## Issues Found

### 1. Hardcoded English text in "Assigning" participant state
**Severity: Critical**
In `src/pages/home.rs` line 688:
```rust
"The organizer / організатор is preparing assignments. "
{t!(i18n, home_assigning_desc)}
```
This literal English/Ukrainian mixed string bypasses the i18n system entirely. All other participant-facing strings use `t!(i18n, ...)`. This appears in production to all participants during the Assignment phase.
Screen: Participant home (Assigning state) — confirmed in live screenshot.

### 2. Raw ISO 8601 timestamps displayed to users
**Severity: High**
Admin season page displays deadlines as `2026-03-25T21:09:00Z` — raw RFC 3339 strings. The `get_home_state` server function uses `season.signup_deadline.format(&Rfc3339)` and passes the string directly to the UI with no formatting. Participant views (Enrolled, Preparing states) also receive raw ISO strings for their deadline displays.
Screen: Admin season page, participant home states.

### 3. Nova Poshta branch display doubles prefix
**Severity: High**
The `nova_poshta_city` DB column stores the full user input string (e.g., `"Відділення №5, Київ"`). The i18n template `home_recipient_branch` is `"Відділення №{{ branch_number }}, {{ city }}"`. At render time: `"Відділення №5, Відділення №5, Київ"`.
Root cause: the branch number extraction (`skip_while !ascii_digit`) works, but `nova_poshta_city` holds the full original string not just the city name. The display template adds "Відділення №N, " prefix to an already-prefixed city string.
Screen: Participant delivery view, recipient branch DD.

### 4. Admin user lands on participant home page after login
**Severity: High**
`verify_otp_code` server function always redirects to `"/"` on success (login.rs line 97). The home route renders `<AuthGuard require_onboarded=true><HomePage/></AuthGuard>` — which shows the participant home state for the admin user. Admin must manually navigate to `/admin`. No auto-redirect for admin role exists.
Screen: Post-login admin redirect confirmed in screenshot.

### 5. Missing logo asset files
**Severity: High**
`same_te_mark_orange.svg` and `same_te_mark_white.svg` not present in `public/`. The nav header workaround (`logo.svg` at 2rem height) is visually acceptable but semantically wrong. `logo-white.svg` is present but unused — no dark-bg context uses it.

### 6. Admin nav has no active/current page indicator
**Severity: Moderate**
`.admin-nav a` CSS has no `:aria-current`, `.active`, or current-page variant. No class or attribute is added to nav links to indicate the current section. Users must read the h1 to know which admin page they are on.
Screen: All admin pages.

### 7. Cycle visualization is plaintext only
**Severity: Moderate**
The assignments page renders `"А → Б → В → А"` as plain `<p>` text. The data-testid `cycle-visualization` suggests a visual element was intended. Current rendering is functional but indistinguishable from body text — no visual graph, chips, or ring diagram.
Screen: Admin assignments.

### 8. Badge component underutilized
**Severity: Moderate**
`.badge` component is defined and used correctly on the participants table (АКТИВНИЙ). However, season phase (plain "Доставка" text in DD), dashboard stats (plain numbers), and participant confirmation status use no badge styling. The visual differentiation between "active data" and "status" is lost.
Screen: Admin dashboard, admin season, admin assignments.

### 9. ARIA attributes absent from all form fields
**Severity: Moderate** (accessibility)
No `aria-describedby` linking inputs to errors, no `aria-invalid` on error states, no `aria-live="assertive"` on error containers. Confirmed by source inspection across all component files. The `.field-input[aria-invalid="true"]` CSS rule exists but nothing sets the attribute. Error messages use `<p class="alert">` without `role="alert"` or `aria-live`.

### 10. `.field-error` class unused; errors rendered outside `.field` wrapper
**Severity: Low**
The `.field-error` CSS class is defined but no component uses it. Action errors display as `<p class="alert">` outside the `.field` structure. This works visually but deviates from the spec's `.field` > `.field-error` pattern.

### 11. `data-size` button attribute unused
**Severity: Low**
`.btn[data-size="sm"]` and `.btn[data-size="lg"]` are defined but no component uses them. All buttons render at the default size. CTA buttons (enrollment, onboarding save) could benefit from `data-size="lg"`.

### 12. SMS "Надіслати" buttons have no send-confirmation guard
**Severity: Low**
SMS trigger buttons fire immediately with no confirmation dialog or disabled-after-click state. Accidental double-sends in production would contact all participants. Not a visual issue but a UX safety gap.

### 13. `logo-white.svg` present but unused
**Severity: Low**
The footer spec calls for `logo-white.svg` on dark backgrounds. There is no footer component. `logo-white.svg` exists in public but is referenced nowhere.

---

## Frontend-Design Skill Assessment

### Aesthetic Verdict: Intentional Brutalist Typographic Identity

The Саме Те app has genuine visual identity. It is not generic. The following analysis evaluates both design system compliance and the broader aesthetic execution.

**What the app does exceptionally:**

The **CyGrotesk 900 display font** is the defining element. At 44.8px with -0.02em letter-spacing, headings like "Панель" and "Учасники" are visually commanding — they announce the section rather than label it. The font choice itself is the brand voice made visual. This is not templated.

The **dark mode execution** is correct and committed. The cream-on-near-black palette (`oklch(0.98 0.01 90)` on `oklch(0.15 0 0)`) achieves warm darkness rather than cold darkness. The orange accent at 4.6:1 contrast ratio on the dark surface reads as a deliberate material choice — embers, not alerts.

The **button system** is the strongest component. Pill radius via `calc(infinity * 1px)` is semantically precise. The three-variant system (primary/secondary/destructive) with CSS custom property hooks is production-grade architecture. The hydration gate (0.45 opacity disabled state) is subtle enough not to feel broken, clear enough to communicate "not yet ready."

The **grain overlay** is correctly calibrated. At 0.04 opacity with `mix-blend-mode: overlay`, it is barely perceptible — the texture registers subconsciously as analog depth rather than consciously as decoration. This is the right call.

The **form field focus state** — blue glow ring against dark surface — is the clearest interactive affordance in the app. Confirmed visually in audit. The 3px spread at 15% opacity is exactly the right weight: present without dominating.

**What undermines the aesthetic:**

The **"Assigning" state hardcoded English text** — `"The organizer / організатор is preparing assignments."` — is the single most jarring visual element in the app. It reads like a debug placeholder. Every other participant screen is fully Ukrainian, warm, and direct. This one screen is bilingual debug text. It breaks trust and brand coherence.

The **admin nav** is the weakest component aesthetically. A flat horizontal list of lowercase text links with no active state is functional but unresolved. On the admin dashboard, "Панель" link looks identical to "Сезон" link even when viewing the Панель page. An orange bottom border, a heavier weight, or even a dot indicator would complete this.

The **plaintext cycle visualization** on the assignments page is a missed opportunity. The ring structure `А → Б → В → А` is information-dense and important — who sends to whom matters. As plain body text, it reads like metadata. Even basic pill chips with arrows would communicate the circular structure.

The **raw ISO timestamps** (`2026-03-25T21:09:00Z`) in the admin season panel are a content bug that reads as developer artifact. A date formatter (even `DD.MM.YYYY HH:mm`) would transform this from internal data to user information.

**The gap between architecture and execution:**

The CSS architecture is more sophisticated than the component layer that uses it. The design system defines `.badge` with five status variants — only one is used. It defines button sizes — none are used. It defines `.field-error` inside `.field` — errors appear outside the field as standalone `.alert` paragraphs.

The result is a well-designed system that is about 70% applied. The bones are right. The typography hierarchy is correct. The color system is correct. The spacing system is correct. The components that are used, are used correctly. What's missing is the final layer of polish: the active nav state, the formatted dates, the badge on phase display, the `data-size="lg"` on the enroll CTA.

**On the design system specification itself:**

The spec is well-written and evidences real research (contrast correction rationale, Cyrillic line-height note, mix-blend-mode investigation, landing page artifact analysis). The two-tier token system and density override approach are architecture-level thinking, not template thinking. The `--_bg` pseudo-private CSS custom property pattern for button variants is the correct solution.

The one spec decision that should be revisited: the `@font-face` declarations inside `@layer base`. While this works in current Chrome, the CSS specification's handling of @font-face inside @layer has historically been undefined. The fonts should be declared before any `@layer` blocks to ensure broadest compatibility.

---

## Overall Verdict

The Саме Те mail club app is **visually coherent, design-system-compliant at the CSS layer, and has genuine brand identity.** The CyGrotesk typographic authority, the warm dark palette, the grain overlay, and the pill button system all communicate a designed product, not a template.

It is not production-ready in its current state. Three issues block it:

1. The hardcoded English/Ukrainian mixed text in the Assigning state — a content bug that breaks brand coherence for all participants during assignment phase.
2. The doubled Nova Poshta branch display — a data rendering bug that shows incorrect recipient information.
3. The admin user landing on the participant home — a routing bug that requires admin users to navigate manually to their workspace.

Below those blockers: the raw ISO timestamps, the missing active nav state, the underused badge component, and the absent ARIA attributes are refinements. The visual foundation is strong enough to support the product story. Fix the content and routing bugs first. The design will carry the rest.

---

*Report written to /tmp/ui-audit-report.md*
