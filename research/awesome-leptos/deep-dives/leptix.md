# leptix — Deep Technical Verification

**Report Date:** 2026-03-19
**Repo:** https://github.com/nishujangra/leptix
**Crate:** https://crates.io/crates/leptix

---

## Crate Metadata

- **Published on crates.io:** YES
- **Latest version:** 0.1.2 (published 2026-03-03)
- **Total downloads:** 40
- **Leptos dependency:** `^0.8.15` (requires `rustc >= 1.88`)
- **Last activity:** 2026-03-03 (16 days old)
- **Created:** 2026-02-15 (3 weeks old)
- **Commits:** 8 total
- **Contributors:** 1 (nishujangra — solo author)
- **Open issues:** 0
- **Stars/watchers:** 0

---

## Leptos 0.8 Compatibility

**VERIFIED ✓**

- Requires `leptos >= 0.8.15` (not version-locked; accepts minor/patch updates)
- Project uses `leptos = "0.8"` (compatible range)
- No trait conflicts detected
- README explicitly states: "Your app should also use leptos = 0.8.15 to avoid mixed-version trait errors"
- Only dependency: `gloo-timers ^0.3` with "futures" feature

---

## Component Inventory

| Component | Stability | Notes |
|-----------|-----------|-------|
| `Button` | **ALPHA** | Complete. Variant enum (Primary/Secondary/Danger/Outline). Includes disabled state, click callbacks, class composition. |
| `Input` | **ALPHA** | Complete. Text-first but supports arbitrary `input[type]`. Two-way signal binding (model prop). Placeholder support. |
| `Toast` | **ALPHA** | Complete. Variant enum (Success/Error/Warning/Info). Position enum (TopLeft–BottomRight). Auto-dismiss via duration. Close callback. |

**Status:** All three components are feature-complete but this crate is 2 weeks old, self-described as "Somehow alive. May die at any moment. No guarantees."

---

## Headless Verification

**NOT HEADLESS — Brings CSS Opinions**

Leptix is **NOT a headless component library**. It includes and injects hardcoded CSS for all components:

- `ButtonCSS` (button.css) — 68 lines of styled button classes
- `InputCSS` (input.css) — form field styling
- `ToastCSS` (toast.css) — toast notification layout and positioning

**CSS Architecture:**
- Injected into DOM via `inject_style_once("leptix-button-style", BUTTON_CSS)` at component mount
- Uses CSS custom properties (e.g., `var(--leptix-primary, #2563eb)`) for theme override
- Default palette: blues/grays/reds (generic, not aligned to any brand)
- Default button padding: `0.5rem 1rem`, border-radius: `0.5rem`
- Uses a generic color scheme that conflicts with Mail Club's oklch-based design system

**Style Injection Mechanism:**
```rust
fn inject_style_once(id: &str, css: &str) {
    // Appends <style id="id"> to document.head at first component mount
}
```

**CSS Variables Exposed for Theming:**
- `--leptix-primary` / `--leptix-primary-hover`
- `--leptix-secondary` / `--leptix-secondary-hover`
- `--leptix-danger` / `--leptix-danger-hover`
- `--leptix-outline-text` / `--leptix-outline-border` / `--leptix-outline-hover`

**Theming would require:**
1. Overriding all 8 CSS variables before component mounts (timing is fragile)
2. Completely replacing button CSS in CSS cascade (high specificity, brittle)
3. Disabling Toast auto-position classes and rewriting layout

---

## Relationship to Rust Radix

**INDEPENDENT — No relation to Rust Radix**

- Leptix is a standalone, newly created library (February 2026)
- Not a port, fork, or successor to Rust Radix
- Rust Radix targets a different level of abstraction (headless, unstyled, focus on accessibility)
- Leptix prioritizes out-of-the-box styling over headlessness
- Different philosophy entirely

---

## Current Project Component Gap

### What Mail Club Hand-Rolls

**Buttons:** 34+ `<button>` elements across 18+ components
- Styled with `class="btn"` from Tailwind + Tailwind arbitrary values
- Support variants via `data-variant` attribute + CSS custom property hooks
- Hydration gates: `disabled=move || !hydrated.get()`
- No shared Rust component (inline in every page)

**Inputs:** Multiple patterns:
- ActionForm inputs with plain `<input class="field-input">`
- Validation state via `data-testid` and `aria-invalid`
- Error messages via `aria-live="assertive"` and `aria-describedby`
- Phone input with special handling (OTP field styling)
- No shared component abstraction

**Design System Compliance:**
- Buttons styled with Mail Club brand orange (`oklch(0.63 0.22 31)`)
- Inputs use Mail Club typography (Mont, CyGrotesk)
- Spacing uses Mail Club density tokens (`--density-space-*`)
- Dark mode support via semantic aliases
- Tailwind v4 with cargo-leptos

### Could leptix Replace Any of This?

**Button:** PARTIAL fit
- Could replace styling if project accepts leptix's generic look
- Would REQUIRE overriding 8 CSS variables + complete CSS replacement
- Loses Mail Club's design system cohesion (oklch tokens, Tailwind integration)
- Hydration gate pattern is identical (`disabled` prop); could work

**Input:** POOR fit
- Basic text input works, but no field wrapper (`<div class="field">` + label + error)
- No error state management or ARIA integration out of the box
- Toast auto-dismiss is useful but rarely needed in this app
- Two-way signal binding doesn't align with ActionForm's DOM-read pattern

**Toast:** UNUSED in this project
- Toast notifications not implemented; Leptos ErrorBoundary + redirect patterns preferred
- Would need custom styling to match Mail Club's brand

---

## Integration Effort

If Mail Club decided to adopt leptix:

### Per-Component Adoption Cost

**Button:**
- Extract 18+ inline button definitions into calls to `leptix::Button`
- Define 8 CSS variables on `:root` to match brand palette
- OR: completely replace leptix button CSS with Mail Club styles (defeats purpose)
- Override `disabled` styling to match Mail Club (opacity: 0.45 vs 0.5)
- **Effort:** 4–6 hours (mechanical refactoring + testing)
- **Risk:** Medium (style injection timing, CSS cascade specificity)

**Input:**
- Would need a wrapper component to add `<label>` + error handling
- Breaks ActionForm's FormData reading (needs name attributes)
- **Effort:** 10–12 hours (significant new abstraction)
- **Risk:** HIGH (defeats Leptos idiom, adds complexity)

**Toast:**
- Not used; skip

### Total Integration Effort
**16–18 hours** if adopting Button + Input
**-3–4 hours** if only Button

---

## Risks

### Maturity
- **2 weeks old**, 8 commits, solo maintainer
- Self-described: "May die at any moment. No guarantees"
- No track record in production
- Risk of abandonment or breaking changes

### Design System Lock-In
- CSS opinions hard-coded; theming via CSS variables is fragile
- Doesn't understand Mail Club's oklch palette or density tokens
- Diverges from established Tailwind v4 + cargo-leptos architecture

### Leptos Version Lock
- Pinned to `0.8.15`; if project upgrades to future Leptos, leptix may not
- Leptos 0.9 or 1.0 would require leptix maintenance (solo author responsibility)

### No Accessibility Beyond Aria
- Button component handles disabled state
- Input component has no aria-invalid, aria-describedby, aria-live patterns
- Toast has no focus trap, keyboard dismiss, or ARIA live region semantics

### API Mismatch
- Input's two-way signal binding (`model` prop) conflicts with Mail Club's ActionForm idiom (DOM-read pattern)
- Button's `on_click` callback works, but project prefers native `<button type="submit">` in ActionForms

### CSS Injection Side Effects
- `inject_style_once` appends to `<head>` at first mount; order is unpredictable
- Could conflict with Tailwind's style output if injected in wrong order
- Hard to debug if styles load after Tailwind rules

---

## Verdict

### **SKIP**

Leptix does not fit Mail Club's architecture or maturity bar:

1. **Not headless.** Brings CSS that conflicts with Mail Club's design system. Theming via CSS variables is fragile and incomplete.

2. **Too immature.** 2 weeks old, 1 maintainer, "no guarantees" status. Project cannot depend on this for production.

3. **Design system misalignment.** Leptix's generic blue/gray palette doesn't use oklch tokens. Mail Club has a complete, battle-tested design system in Tailwind v4.

4. **Idiomatic mismatch.** Leptix's Input model (two-way signal binding) conflicts with Mail Club's ActionForm pattern (DOM FormData read). Button variant system is redundant — Mail Club already uses `data-variant` + CSS custom properties.

5. **Integration cost vs. benefit.** 16–18 hours to adopt Button + Input for marginal benefit (buttons are already hand-rolled consistently; inputs are context-specific).

### If You Did Want a Component Library

Consider these alternatives instead:

- **[shadcn/ui for Leptos](https://github.com/martinproject/shadcn-leptos)** — copy-paste headless components, full design control
- **[Leptos Melt UI](https://docs.melt-ui.com/)** — headless, Svelte-focused but has Leptos bindings in progress
- **Hand-roll Button/Input as reusable Rust components** — what Mail Club already does implicitly, formalize with a `components/` module

### For This Project: Continue Current Approach

- Button: Already extracted consistently as `class="btn" data-variant=...`
- Input: Already wrapped with error handling, ARIA, and label
- Toast: Not needed; ErrorBoundary + redirect patterns work well

The project's design system (Tailwind v4 + cargo-leptos + semantic tokens) is **already the right abstraction level**. Adding a component library would add complexity without reducing it.

---

## Sources

- **leptix GitHub:** https://github.com/nishujangra/leptix
- **leptix crates.io:** https://crates.io/crates/leptix
- **leptix README:** https://raw.githubusercontent.com/nishujangra/leptix/master/README.md
- **Button source:** https://raw.githubusercontent.com/nishujangra/leptix/master/src/components/button.rs
- **Input source:** https://raw.githubusercontent.com/nishujangra/leptix/master/src/components/input.rs
- **Toast source:** https://raw.githubusercontent.com/nishujangra/leptix/master/src/components/toast.rs
- **Button CSS:** https://raw.githubusercontent.com/nishujangra/leptix/master/src/styles/button.css
- **Mail Club design system:** `guidance/design-system.md` (this codebase)
- **Mail Club frontend protocol:** `guidance/frontend-protocol.md` (this codebase)
