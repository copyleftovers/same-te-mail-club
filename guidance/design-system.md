# Design System — Саме Те Mail Club

Binding specification for all visual implementation. Derived from brand guidelines, landing page analysis, CSS pattern research, and font/logo system investigation (see `research/`).

This document defines **what** the visual system is. For **how** to implement it, see `guidance/frontend-protocol.md`.

---

## Palette

Six brand tokens. All values in oklch for perceptual uniformity; hex provided for reference only.

### Raw Tokens

| Token | oklch | Hex | Origin |
|-------|-------|-----|--------|
| `--color-brand-orange` | `oklch(0.63 0.22 31)` | `#D93A12` | Brand orange, **contrast-corrected** from landing page `#FB4417` (which fails WCAG AA on white at 3.53:1). This value yields ~4.6:1 on white. |
| `--color-brand-pink` | `oklch(0.88 0.04 15)` | `#EED3D0` | Warm accent for dark surfaces |
| `--color-brand-gray` | `oklch(0.45 0.01 250)` | `#565656` | Body text on light surfaces |
| `--color-brand-blue` | `oklch(0.78 0.11 240)` | `#8DC1FF` | Tags, focus rings, secondary accent on dark |
| `--color-brand-black` | `oklch(0.15 0.00 0)` | `#161616` | Dark surfaces, heavy text |
| `--color-brand-cream` | `oklch(0.98 0.01 90)` | `#FAF9F6` | Page background, light surfaces |

**Contrast-correction rationale:** The original brand orange (`#FB4417`) fails WCAG AA when used as a background with white text (3.53:1 at 17.6px bold — needs 4.5:1). The corrected value darkens ~15% to pass AA at all text sizes. The brand guidelines PDF defines orange as a primary color but does not specify a contrast commitment — the visual identity is preserved; the luminance shifts slightly.

### Semantic Aliases

These are CSS custom properties on `:root`, not `@theme` tokens. They do not generate Tailwind utility classes. They are consumed via `var()` in component CSS or `bg-(--color-surface)` in Tailwind arbitrary-value syntax.

```
--color-surface           → --color-brand-cream
--color-surface-raised    → white
--color-surface-dark      → --color-brand-black
--color-text              → --color-brand-black
--color-text-muted        → --color-brand-gray
--color-text-on-dark      → --color-brand-cream
--color-text-on-dark-muted→ --color-brand-pink
--color-accent            → --color-brand-orange
--color-accent-alt        → --color-brand-pink
--color-focus             → --color-brand-blue
--color-error             → oklch(0.55 0.22 25)
--color-success           → oklch(0.58 0.16 160)
```

### Dark Mode

`prefers-color-scheme: dark` only. No manual toggle. Semantic aliases are reassigned:

```
--color-surface       → --color-brand-black
--color-surface-raised→ oklch(0.18 0.01 250)
--color-text          → --color-brand-cream
--color-text-muted    → oklch(0.65 0.01 250)
```

Raw tokens do not change. Semantic tokens do.

---

## Typography

### Font Files

Three woff2 files, copied from `same-te-landing/fonts/` into `public/fonts/`:

| File | Family | Weight | Role |
|------|--------|--------|------|
| `CyGroteskGrandDark.woff2` | CyGrotesk | 900 | Display headings |
| `Mont-Regular.woff2` | Mont | 400 | Body text, form inputs |
| `Mont-SemiBold.woff2` | Mont | 600 | Labels, buttons, emphasis |

All served as woff2-only with `font-display: swap`.

### Font Stacks

```
--font-display: 'CyGrotesk', 'Arial Black', sans-serif
--font-body:    'Mont', 'Inter', system-ui, sans-serif
```

### Hierarchy

| Level | Font | Weight | Size | Line-height | Letter-spacing | Use |
|-------|------|--------|------|-------------|----------------|-----|
| Page title | CyGrotesk | 900 | `clamp(1.8rem, 5vw, 2.8rem)` | 1.15 | -0.02em | Page-level headings (`<h1>`) |
| Section heading | CyGrotesk | 900 | 1.3rem | **1.15** | -0.02em | Section titles, step headings. Landing page used `line-height: 1` which clips Cyrillic ascenders (confirmed: 2px overflow on all step headings). Use 1.15. |
| Overline label | Mont | 600 | 0.8rem | 1.4 | 0.1em | Uppercase labels above sections |
| Body | Mont | 400 | 1.05rem (16.8px) | 1.75 | normal | Paragraph text, descriptions |
| Body emphasis | Mont | 600 | 1.05rem | 1.75 | normal | Highlighted sentences, punchlines |
| UI label | Mont | 600 | text-sm (0.875rem) | 1.4 | normal | Form labels, button text, badges |
| Body secondary | Mont | 400 | text-sm (0.875rem) | 1.5 | normal | Step descriptions, captions |
| Small | Mont | 400 | text-xs (0.75rem) | 1.5 | normal | Footer text, metadata |

### OTP/Phone Input

```css
font-variant-numeric: tabular-nums;
letter-spacing: 0.1em;
```

---

## Logo

### Files to Include

Copy from `same-te-landing/Logo/SVG/` and root:

| Use | File | Source |
|-----|------|--------|
| Favicon | `favicon.svg` | Mark/orange, tight crop |
| Nav header (light bg) | `same_te_mark_orange.svg` | Mark/orange |
| Nav header (dark bg) | `same_te_mark_white.svg` | Mark/white |
| Auth page hero | `logo.svg` | Main/orange, tight crop |
| Footer (dark bg) | `logo-white.svg` | Main/white, tight crop |

### Logo Rules (from brand guidelines)

- Clear space: one "C" height on all sides
- Never rotate, stroke, stretch, or use secondary + emblem simultaneously
- On light: orange or black. On dark: white or pink.

---

## Components

### Buttons

Three variants via `data-variant` attribute. Two sizes via `data-size`.

| Variant | Background | Text | Border | Use |
|---------|-----------|------|--------|-----|
| primary (default) | `--color-accent` | white | none | Main CTA, form submit |
| secondary | transparent | `--color-text` | `currentcolor` | Cancel, secondary action |
| destructive | `--color-error` | white | none | Delete, deactivate |

| Size | Padding | Font size |
|------|---------|-----------|
| default | `0.625rem 1.25rem` | text-sm |
| sm | `0.375rem 0.75rem` | text-xs |
| lg | `0.75rem 1.75rem` | text-base |

Shape: `border-radius: 100px` (pill). All buttons.

States:
- **Disabled / hydration gate**: `opacity: 0.45; cursor: not-allowed; pointer-events: none`
- **Loading**: `opacity: 0.75; cursor: wait; pointer-events: none`
- **Hover** (pointer devices only): `filter: brightness(0.9)`
- **Focus-visible**: `outline: 2px solid var(--_ring); outline-offset: 2px`

### Form Fields

Structure: `.field` > `.field-label` + `.field-input` + `.field-error`

- Label: Mont 600, text-sm, `--color-text`
- Input: Mont 400, text-base, 1.5px border `--color-brand-gray`, `border-radius: --radius-md` (0.5rem), padding `0.625rem 0.75rem`
- Error: Mont 400, text-sm, `--color-error`, `aria-live="assertive"`
- Focus: border shifts to `--color-focus`, 3px glow ring at 15% opacity
- Error state: border shifts to `--color-error`, glow ring matches
- Disabled: `opacity: 0.5; cursor: not-allowed`

### Badges

Pill shape. Variants via `data-status`:

| Status | Background |
|--------|-----------|
| active | `--color-success` |
| pending | `--color-accent` |
| error | `--color-error` |
| inactive | `--color-brand-gray` |
| confirmed | `--color-brand-blue` |

Mont 600, text-xs, uppercase, letter-spacing 0.02em.

### Content Container

`max-width: 65ch; margin-inline: auto; padding-inline: var(--spacing-4)`

Participant-facing pages: spacious. Admin pages: same container but tighter density (see below).

---

## Density

Participant and admin screens share the same tokens. Density differs via `[data-layout="admin"]` on the layout root.

| Token | Participant | Admin |
|-------|-----------|-------|
| `--density-space-sm` | spacing-3 (0.75rem) | spacing-1.5 (0.375rem) |
| `--density-space-md` | spacing-5 (1.25rem) | spacing-3 (0.75rem) |
| `--density-space-lg` | spacing-8 (2rem) | spacing-5 (1.25rem) |

Admin typography: text-xs and text-sm are primary. Participant typography: text-base and text-lg are primary.

---

## Grain Overlay

SVG `feTurbulence` noise, 283 bytes inline. Applied as `body::after` with `position: fixed; pointer-events: none`.

**Changes from landing page:**
- `z-index: 1` (not 100 — app needs stacking room for modals, toasts, dropdowns)
- `opacity: 0.04` with `mix-blend-mode: overlay` (not multiply — multiply is near-invisible on both cream and black surfaces as confirmed by Chrome inspection; overlay produces visible texture at both extremes)
- `@media (prefers-reduced-motion: reduce)`: hide entirely

---

## Focus

Global default in `@layer base`:

```css
:focus { outline: none; }
:focus-visible {
  outline: 2px solid var(--color-focus);
  outline-offset: 2px;
}
```

Components may override ring color (e.g., destructive button uses `--color-error` for ring).

---

## Spacing Scale

Use Tailwind's default 4px-based scale. Do not define a custom scale unless a specific value is missing. The landing page's 14 ad-hoc spacing values are not carried forward — they were appropriate for a single-file landing page but would produce visual drift at app scale.

Named semantic spacing tokens (`--density-space-*`) use points on the Tailwind scale, not custom values.

---

## Radius Scale

| Token | Value | Use |
|-------|-------|-----|
| `--radius-sm` | 0.25rem | Small elements, tags |
| `--radius-md` | 0.5rem | Form inputs, cards |
| `--radius-lg` | 0.75rem | Larger cards, panels |
| `--radius-pill` | `calc(infinity * 1px)` | Buttons, badges |

---

## Accessibility Commitments

- All text meets WCAG 2.1 AA contrast (4.5:1 normal, 3:1 large text ≥18.67px bold)
- The contrast-corrected orange (`--color-brand-orange`) passes AA on white and cream
- Every interactive element has `:focus-visible` ring
- `:disabled` and `[aria-disabled="true"]` styled equivalently
- `prefers-reduced-motion: reduce` disables animations
- Steps/sequences use `<ol>` not `<div>`
- Form fields use `aria-describedby` for error messages, `aria-invalid` for error state, `aria-live="assertive"` on error containers

---

## What Is NOT In This System

- No animation library. Transitions are `120ms ease` on interactive elements only.
- No dark mode toggle. System preference only.
- No icon system beyond the logo mark. If icons are needed later, add them then.
- No responsive breakpoints beyond Tailwind defaults. The app is single-column at all sizes currently.
- No print styles. The app is not printed.
