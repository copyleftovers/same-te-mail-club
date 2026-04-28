# UI Fix Plan — Tracer-Bullet Passes

Source: `spec/ui-audit.md` + `spec/ui-audit-sequential.md` (cross-validated)

Three passes. Each pass is independently committable, does not regress E2E, and leaves the app in a visibly better state than before.

---

## Pass 1: Broken → Functional

Everything in this pass fixes something that makes the app look broken to a user. After this pass, no element is clipped, unrecognizable, or overlapping.

### 1.1 Hamburger icon renders as horizontal dash

The three `<span>` lines inside `.menu-toggle` lay out in a row because the flex container defaults to `flex-direction: row`. The icon is unrecognizable on every mobile page.

- **Scope:** CSS-only (`style/tailwind.css`)
- **Touch:** `.menu-toggle` rule
- **Verify:** Visual — three stacked horizontal lines at 414px viewport

### 1.2 Data table actions column clipped on mobile

The participant table extends past the viewport edge. The "ДІЯ" header and deactivate buttons are inaccessible — admin cannot deactivate participants on mobile.

- **Scope:** Rust markup (`src/admin/participants.rs`)
- **Touch:** Wrap `<table>` in a `.data-table-wrapper` div
- **Verify:** Horizontal scroll on the table at 414px; all columns reachable

### 1.3 Form field groups have no vertical spacing

Multiple `.field` divs stack as direct siblings without gap. Labels of the next field visually merge with the input above. Affects every multi-field form: register participant, create season, onboarding, enrollment.

- **Scope:** CSS (`style/tailwind.css`) — systemic fix preferred over per-form Rust changes
- **Touch:** `.field` component rule or adjacent-sibling combinator
- **Dependency:** Decide whether the gap lives in CSS (`.field + .field`) or in Rust markup (wrapper div with gap). CSS is broader; Rust is more explicit. Either works — pick one approach and apply it everywhere.
- **Verify:** Visible gap between consecutive field groups on all forms

### 1.4 Logo size conflict

`app.rs` uses `h-10` (40px). `.app-header img` CSS says `height: 2rem` (32px). Tailwind utility wins. One source of truth needed.

- **Scope:** One-line fix in either `src/app.rs` or `style/tailwind.css`
- **Touch:** Resolve which size is correct per design-system.md (2rem), then remove the conflicting declaration
- **Verify:** Logo height matches design-system.md across all pages

---

## Pass 2: Functional → Consistent

After this pass, the same pattern looks the same everywhere. Empty states use the empty-state component. The header behaves predictably. Spacing follows the density system.

### 2.1 Stepper responsive degradation

The 5-step horizontal stepper clips labels at both edges on 414px viewports. "РЕЄСТРАЦІЯ" and "ЗАВЕРШЕНО" are partially cut off.

- **Scope:** CSS and possibly Rust markup (`src/components/stepper.rs`, `style/tailwind.css`)
- **Touch:** The stepper needs a responsive strategy for narrow viewports. Options include: adding padding to the scroll container so edge labels aren't clipped; abbreviating labels on small screens; reducing marker/connector size; switching to a vertical layout below a breakpoint. The choice is a design decision.
- **Constraint:** The stepper must remain readable and all labels fully visible at 414px
- **Verify:** All 5 labels fully visible on iPhone XR; no clipping at edges

### 2.2 NoSeason state uses wrong pattern

Renders as a bare `<p>` tag. Every other empty state in the app uses the `.empty-state` component (centered, headline + body, min-height). The participant home page looks barren and broken in this state.

- **Scope:** Rust markup (`src/pages/home.rs`)
- **Touch:** The `HomeState::NoSeason` match arm
- **Verify:** Centered empty-state layout with headline and body text

### 2.3 Header layout: redundant logout on mobile

On participant mobile pages, both a "Вийти" button AND the hamburger icon are visible in the header. The hamburger opens a menu that also contains "Вийти". The header is cluttered; the two controls compete for space.

- **Scope:** Rust markup/CSS (`src/app.rs`, possibly `style/tailwind.css`)
- **Touch:** `HeaderNav` component — the `.header-nav` div wrapping the logout button
- **Decision needed:** Hide the header logout on mobile and rely on the hamburger menu? Or hide the hamburger on participant pages entirely (it only shows 2 items)? The hamburger is useless on login/onboarding too — consider hiding it when there's nothing meaningful to show.
- **Verify:** Clean 2-element header (logo + one control) on mobile participant pages

### 2.4 Stat card grid for terminal seasons

When a season is cancelled/complete, only one stat card (phase) is shown. It sits in a 2-col (mobile) or 3-col (desktop) grid, looking orphaned with empty cells.

- **Scope:** Rust markup (`src/admin/dashboard.rs`)
- **Touch:** The stat grid layout for terminal season states
- **Verify:** Single stat card spans full width or uses a non-grid layout in terminal state

### 2.5 Login form button width

Login buttons are narrower than the input fields above them, creating misaligned right edges.

- **Scope:** Rust markup (`src/pages/login.rs`)
- **Touch:** Button elements — add full-width on mobile
- **Verify:** Button edges align with input edges on mobile

### 2.6 Admin density boundary

The `[data-layout="admin"]` wrapper lives inside `<main>`, but the header and mobile menu live outside it. Admin nav chrome uses participant-density spacing while page content uses tighter admin density. The visual result is inconsistent spacing between header nav and page content.

- **Scope:** Architectural — `src/app.rs`, `style/tailwind.css`
- **Touch:** Either move the density attribute to a higher wrapper that includes the header, or explicitly set admin density on the header/mobile-menu when rendering admin pages.
- **Constraint:** Must not break E2E selectors or existing page layouts
- **Verify:** Consistent spacing between admin nav links and admin page content

---

## Pass 3: Consistent → Polished

After this pass, the app feels intentional. Minor spacing, contrast, and interaction issues are resolved.

### 3.1 Mobile menu refinements

- No close button / X affordance on the slide-in panel (only overlay tap or Escape)
- Logout button text clips at panel right edge
- For participants, the menu shows only 2 items (Головна + Вийти) — the full-height drawer feels empty/broken

**Scope:** Rust markup + CSS (`src/app.rs`, `style/tailwind.css`)

### 3.2 Stepper connector colors

Connectors between completed steps remain gray instead of turning green. The CSS selector `.step[data-status="completed"] + .step-connector` may not match the actual DOM order produced by the Rust iterator (step, connector, step vs. connector, step, connector, step).

**Scope:** CSS or Rust markup (`src/components/stepper.rs`, `style/tailwind.css`)

### 3.3 Header border visibility

The `oklch(... / 0.2)` border-bottom on `.app-header` is nearly invisible on cream. The header-to-content boundary is unclear.

**Scope:** CSS-only (`style/tailwind.css`)

### 3.4 Login vertical positioning

The `min-h-[80svh] justify-center` pushes the form below the natural reading position on mobile. When the keyboard opens, the input may be obscured.

**Scope:** Rust markup (`src/pages/login.rs`)

### 3.5 Form border visual weight

Input borders at `1.5px solid var(--color-brand-gray)` appear heavier than typical form inputs on the cream background. May benefit from reduced opacity for inactive state.

**Scope:** CSS-only (`style/tailwind.css`)

### 3.6 Remaining minor items

- Active mobile menu link highlight appears stronger than the specified 8% opacity
- Desktop admin nav: no visual separation between nav links and logout action
- No back-to-phone-step affordance on OTP screen
- `text-wrap: balance` on page titles to avoid orphaned short lines
- Hamburger lines could be slightly thicker for visibility (after flex-direction is fixed)

**Scope:** Mixed CSS + Rust, all independent

---

## Parallelism Map

```
Pass 1 (all independent, parallelizable):
  1.1 CSS-only ─────────────┐
  1.2 Rust (participants) ──┤── can run in parallel
  1.3 CSS-only ─────────────┤
  1.4 One-liner ────────────┘

Pass 2 (mostly independent):
  2.1 Stepper ──────────────┐
  2.2 Home page ────────────┤
  2.3 Header ───────────────┤── can run in parallel
  2.4 Dashboard ────────────┤   (2.3 and 2.6 touch app.rs
  2.5 Login ────────────────┤    — coordinate or sequence)
  2.6 Density boundary ─────┘

Pass 3 (mostly independent):
  All items independent except 3.1 and 3.2
  which both touch app.rs / stepper.rs respectively
```

## Verification Protocol

Each pass: `just check` (clippy + tests) then `just e2e` (full E2E suite). Visual spot-check at 414px (mobile) and ~1350px (desktop) for every affected page. Screenshot before/after for comparison.
