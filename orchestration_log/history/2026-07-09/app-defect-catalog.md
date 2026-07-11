# App Defect Catalog — 2026-07-10 Rendered-Verify Synthesis

SUITE defects (capture wrong/missing/mislabeled) are excluded per intake instructions:
- `admin-cycle-viz-12-nodes` no INDEX row (all dirs) — suite
- `home-receipt-not-received` wrong state — suite
- `home-enrollment-not-open` = no-season — suite (suite-fix outcome determines whether H4a/H1 are visually distinct)
- `login-otp-step` / `login-otp-step-resend-cooldown` L5/L7 identical — suite
- `admin-no-season-create-form-available` DB state leak — suite

---

## Summary Table

| ID | Severity | Affected screens | Modes/viewports | Summary |
|----|----------|-----------------|-----------------|---------|
| A1 | MAJOR | admin-cycle-viz-12-nodes | all 4 | 12-node ring: label collisions at 3 arc zones; garbled merged text; one artifact name |
| A2 | MAJOR | global-404-fallback | light+dark / desktop | 404 renders unstyled bare text, no layout container |
| A3 | BLOCKER | admin-invite-codes-mixed-statuses | light-desktop / desktop; dark-mobile / mobile | Sticky toast cuts mid-page across admin section card content |
| A4 | MAJOR | admin-create-season-form__error | dark / mobile | Error borders applied to all form fields including theme (no validation error) |
| A5 | MAJOR | home-assignment-and-receipt-form | dark / mobile | Phone number rendered in `--color-accent` orange — misleading interactive affordance |
| A6 | MAJOR | admin-participants-mixed-statuses | dark / mobile | Em-dash replacing deactivate button: ~2:1 contrast on dark raised surface |
| A7 | NOT-A-DEFECT | admin-assignment-phase-pre-generate | light / desktop+mobile; dark / mobile | "Далі" correctly disabled (advance_blocked gate) before assignments released; visual residual: disabled primary vs live destructive hierarchy unclear |
| A8 | NOT-A-DEFECT | onboard-branch-selection | dark / mobile | Onboarding uses prose-page (post-auth flow); auth-card+logo is login-only by design |
| A9 | MINOR | admin-season-complete | light / desktop; dark / mobile | `.alert` panel interior fill imperceptible in dark; left accent stripe not visible |
| A10 | NOTE | home-* (all participant states) | all | `get_home_state` filters `launched_at IS NOT NULL` — announced-but-unlaunched season invisible to participants; product question whether a "coming soon" state is needed |

---

## Entries

### A1 — Cycle-viz label collisions (12-node ring)

**Severity:** MAJOR
**Screens:** `admin-cycle-viz-12-nodes` (all four mode/viewport combos + sections crop)
**Modes/viewports:** light-desktop, light-mobile, dark-desktop, dark-mobile (identical in all)
**Source reports:** light-desktop, light-mobile, dark-desktop, dark-mobile, cohort-sections

At 12 participants with long double-barrel Ukrainian names, the SVG ring label layout breaks in three zones. Top-right arc: "Владислав-Ростислав Козаченко-Бабенко", "Катерина-Людмила…" and a third label stack with less than one line-height gap, overprinting each other into illegibility. Bottom-center: "Микола-Сергій Тка[ченко-Мороз]" surname is truncated mid-word; the label runs past its zone and merges with "Валентмарфина Сидоренко-Гриценко" producing a garbled single line. Right-middle cluster (nodes 3–5): "Ярослава-Оксана Василенко-Хоменко", "Христофор-Данило Петренко-Іваненко", and "Кило Патренко-Іваненко" (a rendering artifact — not a real name) share a 4px vertical band; the last entry is two colliding labels merged. Ring fits within viewport width at all sizes (no horizontal scroll), but the text is unreadable at the collision zones.

**First-principles plausibility:** This is a genuine geometric overflow. The label placement code uses fixed LABEL_OFFSET_Y + node angle to position text; at 12 nodes (~30° arc each) with long names the bounding boxes intersect. Not a correct-behavior candidate — no user can read the assignment ring in this state. Confirmed identical across all 4 mode/viewport combinations — the fix must be algorithmic (radial offset scaling, truncate+title, smaller font) not viewport-specific.

---

### A2 — 404 page unstyled

**Severity:** MAJOR
**Screens:** `global-404-fallback`
**Modes/viewports:** light-desktop (MAJOR), dark-desktop (MINOR — same defect, not dark-token failure)
**Source reports:** light-desktop, dark-desktop

The 404 route renders bare text "Сторінку не знайдено." flush to the top-left corner with no layout container, no `.empty-state` treatment, no centering, no header. The `.empty-state` + `.page-frame` primitives exist and are used on all other zero-content participant states. The 404 handler simply emits a raw text node. Light-mobile reviewer rated it 0 defects (caught in desktop only); dark-desktop rated MINOR (consistent defect, not a dark-mode-specific token failure).

**First-principles plausibility:** Clearly unintentional — every other empty state in the app uses the established component. No plausibility concern.

---

### A3 — Toast obscures admin section card content

**Severity:** BLOCKER
**Screens:** `admin-invite-codes-mixed-statuses`, `admin-participants-mixed-statuses`
**Modes/viewports:** light-desktop (BLOCKER for invite-codes); dark-mobile (MAJOR for both)
**Source reports:** light-desktop, dark-mobile

The sticky toast banner renders across the middle of the page after a revoke/deactivate action, cutting through an admin section card. On light-desktop: the "Код відкликано!" toast bisects the card mid-screen — the "Учасники" heading and the Запрошення sub-section are severed; the invite-code list begins below the toast visually attached to nothing. On dark-mobile: "Код відкликано!" overlaps the "СТВОРИТИ КОД" section heading, cutting off the distributor label below it. This is a real-use defect, not just a capture artifact — any user who revokes a code or deactivates a participant will see the toast overlap mid-page content before it dismisses. The sticky toast at `top: 0` inside `<main>` pushes content in flow but at the moment of appearance can visually overlap content that has already scrolled.

**First-principles plausibility:** Genuine defect. The toast is sticky-in-flow, but at the moment of injection the page may be scrolled past the toast anchor, so the toast appears to float over rather than push. This is a layout timing / scroll-position problem. Not correct behavior.

---

### A4 — Error borders on all create-season fields including valid ones

**Severity:** MAJOR
**Screens:** `admin-create-season-form__error`
**Modes/viewports:** dark / mobile
**Source reports:** dark-mobile

In the create-season error state (dark-mobile), all three form inputs — both date pickers and the theme text field — show red error borders, even though only the date fields are invalid. The theme input has no entered value and no validation error but is outlined in red. This is over-application of the `aria-invalid` / error-border styling: the error state appears to be applied at the form level rather than field-by-field.

**First-principles plausibility:** Plausible app defect. The light-desktop reviewer saw this screen without flagging it (rated 0 defects on that file); the dark-desktop reviewer also passed it. The dark-mobile reviewer flagged it as MAJOR. Possible that the error styling is contrast-hidden on light but visible on dark, making the underlying shared bug only apparent in dark. Warrants source inspection of `admin/season.rs` error propagation and how `aria-invalid` is set per-field vs globally.

---

### A5 — Phone number styled accent-orange

**Severity:** MAJOR
**Screens:** `home-assignment-and-receipt-form`
**Modes/viewports:** dark / mobile
**Source reports:** dark-mobile

The recipient phone number "+380680000004" in the assignment card renders in `--color-accent` orange — the same orange used for primary action buttons. On the dark surface this reads as a tappable CTA rather than static informational text. No other reviewer flagged this on light variants; the orange may be sufficiently contextually distinguished on cream but becomes misleading on the dark background.

**First-principles plausibility:** Genuine affordance mismatch. Phone numbers are informational text; orange is the primary interactive signal throughout the design system. If the phone renders via an `<a href="tel:...">` link styled with the accent color, that would be the source. If it is plain text inheriting an unintended color, it is a CSS specificity leak.

---

### A6 — Em-dash low contrast on deactivated row (dark)

**Severity:** MAJOR
**Screens:** `admin-participants-mixed-statuses`
**Modes/viewports:** dark / mobile
**Source reports:** dark-mobile

On the deactivated participant row, the deactivate-button column shows an em-dash "—" as a placeholder (replacing the action button for a terminal-state row). In dark mode on mobile the em-dash renders in gray text on the dark raised card surface at approximately 2:1 contrast — well below the WCAG 3:1 non-text / 4.5:1 text minimum. The ДЕАКТИВОВАНИЙ badge (gray fill, white text) in the same row passes; the em-dash text does not. This was not flagged in light-mode reports, indicating the em-dash color token works in light but the dark surface lift makes it fail.

**First-principles plausibility:** Genuine contrast failure. The `--color-text-muted` or `--color-brand-gray` token on the raised dark surface (`--color-surface-raised` at oklch 0.22) drops below AA. Source: `admin/participants.rs` em-dash rendering + CSS token assignment for that cell.

---

### A7 — "Далі" primary CTA muted at pre-generate state

**Severity:** MAJOR
**Screens:** `admin-assignment-phase-pre-generate`
**Modes/viewports:** light-desktop (MAJOR), light-mobile (MINOR), dark-mobile (MINOR)
**Source reports:** light-desktop, light-mobile, dark-mobile

The "Далі" (advance) button in the assignment pre-generate state renders muted or outline-style rather than solid primary orange. The "Скасувати" (cancel/destructive) button visually dominates, inverting the intended action hierarchy. Light-desktop reviewer rated this MAJOR; light-mobile and dark-mobile rated it MINOR; dark-desktop reviewer reported 0 defects on this file.

**First-principles plausibility:** Requires source inspection of `admin/page.rs` advance gating. If "Далі" in this state is intentionally gated (e.g. requires all participants confirmed before advancing to generate), the muted styling may be correct disabled/blocked-state rendering. The light-mobile reviewer noted "A30 state has all 3 confirmed, Далі should be active primary" — if the seed data has all 3 confirmed, the button should be enabled and orange. The discrepancy across reviewers (MAJOR / MINOR / 0) suggests either rendering inconsistency across modes or a borderline styling that reads differently under different contrast contexts. **Marked plausible-correct-behavior candidate pending source check.**

**Resolved-by-source: NOT-A-DEFECT.** `page.rs:421–422`: `advance_blocked = season.phase == Phase::Assignment && !season.assignments_released`. In the pre-generate state assignments are not yet released, so `advance_blocked = true` and the button is `disabled` by explicit design. The button carries no `data-variant` (line 565: `class="btn"` only), so it IS the primary orange variant — it just renders at `opacity: 0.45` per the disabled rule. The visual "muted" appearance reviewers saw is the correct disabled affordance, not an incorrect variant. Sole residual: the disabled primary button alongside an active destructive "Скасувати" inverts the visual action hierarchy during this blocked state — whether to accept this or dim the cancel too is a product decision, not a render defect.

---

### A8 — Onboarding missing logo/auth-card container (dark mobile)

**Severity:** MAJOR
**Screens:** `onboard-branch-selection`
**Modes/viewports:** dark / mobile
**Source reports:** dark-mobile

In dark-mobile, the onboarding branch-selection page starts directly with the CyGrotesk heading "Налаштування акаунту" with no logo mark and no `.auth-card` wrapper visible above it. All login steps show the logo mark above the card. No other reviewer (light-desktop, light-mobile, dark-desktop) flagged this — they all passed `onboard-branch-selection` without defect. Either the onboarding page deliberately omits the auth-card container (a design difference between login and onboarding) or the dark-mobile capture revealed a structural omission invisible on lighter surfaces.

**First-principles plausibility:** Plausible-correct-behavior candidate. The login flow uses `.auth-card`; onboarding may intentionally use `.prose-page` without the auth-card shell since it is a post-auth step, not an auth step. Requires source inspection of `pages/onboarding.rs` component structure vs `pages/login.rs`. If intentional, this is not a defect but a design inconsistency to evaluate as a product decision.

**Resolved-by-source: NOT-A-DEFECT.** `login.rs:650–668`: `LoginPage` wraps content in `<div class="page-frame"><div class="auth-card"><img src="/logo.svg" ...>`. `onboarding.rs:160`: `OnboardingPage` wraps in `<div class="prose-page flex flex-col pt-[10svh]">` — no auth-card, no logo. The comment at line 159 ("pt-[10svh]: viewport-relative top padding matches login layout for visual continuity") confirms onboarding intentionally mirrors the login vertical rhythm without the auth-card shell. Onboarding is a post-authentication step; the user is already identified; the auth-card and logo are an identity-establishment container for the anonymous entry flow only. The distinction is deliberate.

---

### A9 — Alert panel fill imperceptible in dark (season-complete)

**Severity:** MINOR
**Screens:** `admin-season-complete`
**Modes/viewports:** light-desktop (MINOR), dark-mobile (MINOR)
**Source reports:** light-desktop, dark-mobile

The "Не отримано: 1" alert panel in admin-season-complete renders with a red left-border accent (correct per spec) but the panel interior fill is nearly indistinguishable from the surrounding surface in dark mode. The spec requires `.alert` in dark to use solid `--color-panel-dark` fill to create a visible container. The fill token appears assigned but renders at insufficient contrast from the page base. In light-desktop the same panel was rated MINOR for the accent stripe not being visible at scale.

**First-principles plausibility:** Genuine minor defect. The `--color-panel-dark` token exists (oklch 0.22 base, +0.07L step), but if the admin section card itself is also oklch 0.22 raised surface, the alert panel on that card produces 0 elevation. The panel needs its fill to contrast against the card, not the page base.

---

---

### A10 — Announced-but-unlaunched season invisible to participants (design question)

**Severity:** NOTE
**Screens:** all participant home states
**Modes/viewports:** all
**Source:** coordinator append; surfaced by suite-fix D3 (H4a renders identical to H1)

`get_home_state` in `src/pages/home.rs:271-277` filters seasons with `launched_at IS NOT NULL`, so a season that has been created but not yet launched returns `NoSeason` to participants. The participant home renders the "no active season" empty state — indistinguishable from the case where no season exists. This is the app-level cause of the H4a=H1 capture identity (the suite defect). Whether participants should see a distinct "season announced, enrollment opens soon" state is a product decision, not a render defect.

**First-principles plausibility:** The current behavior is internally consistent code — the filter is intentional. A "coming soon" state would require a new `HomeState` variant, a new server-fn branch, a new UI state, and product copy. Needs explicit product decision before any fix is scoped.

---

## Notification Summary

10 entries cataloged: 1 BLOCKER (A3), 5 MAJOR (A1, A2, A4, A5, A6), 1 MINOR (A9), 3 design/plausibility candidates (A7, A8, A10). A7 and A8 require source inspection before treating as defects; A10 is a product decision (should participants see an announced-but-unlaunched season) with no render fix until decided.
