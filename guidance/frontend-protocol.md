# Frontend Protocol

Binding implementation rules for all CSS, styling, and visual work. Companion to `guidance/design-system.md` (what the system IS) and `guidance/dev-protocol.md` (general development rules).

Read `design-system.md` first. This document tells you how to build with it.

---

## Toolchain

### Tailwind v4 via cargo-leptos

- **Activation**: `tailwind-input-file = "style/tailwind.css"` in `[package.metadata.leptos]`
- **No Node, no npm, no `tailwind.config.js`.** cargo-leptos downloads the v4.2.1 standalone binary automatically.
- **Do not run `tailwindcss --watch` separately.** cargo-leptos owns the Tailwind build lifecycle. `just dev` / `cargo leptos watch` handles everything.
- **Source detection**: `@source "../src";` is required in `style/tailwind.css`. v4 auto-detection does not reliably reach `src/` from `style/`.

### File Layout

```
style/
  tailwind.css     ← @import "tailwindcss" + @theme + @source + @layer
  main.css         ← DELETED or empty. All CSS moves into tailwind.css.
public/
  fonts/
    CyGroteskGrandDark.woff2
    Mont-Regular.woff2
    Mont-SemiBold.woff2
  favicon.svg
  logo.svg
  logo-white.svg
```

cargo-leptos concatenates `style-file` output before Tailwind output. Since all CSS now lives in `tailwind.css`, the `style-file` field can remain pointing at `style/main.css` (empty) or be removed. Either is fine — no CSS should live outside `tailwind.css`.

### CSS Output

`target/site/pkg/samete.css` — served via `<Stylesheet id="leptos" href="/pkg/samete.css"/>` in `app.rs`. Already configured.

---

## CSS Architecture

### Single File

All design tokens, base styles, component classes, and custom utilities live in `style/tailwind.css`. This is a ~15-screen app with one developer. Splitting into multiple CSS files adds indirection without reducing complexity. If the file exceeds ~500 lines, split by concern: `style/tokens.css`, `style/components.css`, imported via `@import` in `tailwind.css`.

### Layer Order

```
@import "tailwindcss";   ← provides: theme → base → components → utilities

Your CSS goes into:
  @layer base        — global resets, focus ring, font-face
  @layer components  — .btn, .badge, .field, .prose-page, .data-table, etc.
  @utility           — single-property variant-aware utilities (grain, scrollbar-none, text-balance)
```

**Never write unlayered CSS.** Unlayered CSS beats all layered CSS regardless of specificity. This breaks Tailwind utility overrides silently.

### Two-Tier Tokens

```css
@theme {
  /* Tier 1: raw tokens → generates utility classes */
  --color-brand-orange: oklch(0.63 0.22 31);
  /* ... all raw tokens from design-system.md */
}

:root {
  /* Tier 2: semantic aliases → CSS variables only, no utility generation */
  --color-surface: var(--color-brand-cream);
  /* ... all semantic aliases from design-system.md */
}
```

To use semantic aliases in markup: `bg-(--color-surface)` (Tailwind arbitrary-value syntax) or `var(--color-surface)` in component CSS.

---

## Component Styling

### The Leptos Component Is the Abstraction

Do not use `@apply` to build component classes from utilities. The Rust component is the single source of truth for markup + styling. If a button appears in 8 places, write one `<Button>` component. The class list lives in one place.

Use `@layer components` for CSS-level component classes only when:
1. The pattern has variant axes (button variants, badge statuses) that benefit from `data-*` + CSS custom property hooks
2. The pattern repeats across 5+ Leptos components where extracting a shared Rust component is awkward
3. The pattern is a layout primitive (content container, data table) not a Leptos component

### Variant Pattern

Variants use `data-*` attributes + CSS custom property hooks (Lea Verou pseudo-private pattern):

```css
@layer components {
  .btn {
    --_bg: var(--btn-bg, var(--color-accent));
    --_fg: var(--btn-fg, white);
    /* ... uses --_bg and --_fg ... */
  }

  .btn[data-variant="secondary"] {
    --btn-bg: transparent;
    --btn-fg: var(--color-text);
  }
}
```

In Leptos:
```rust
<button class="btn" data-variant="secondary" disabled=move || !hydrated.get()>
```

This is the CSS-equivalent of cva. No JS needed.

### When to Use Inline Utilities vs Component Classes

| Situation | Approach |
|-----------|----------|
| One-off layout container | Inline utilities in `view!` |
| Element with variant axes (button, badge) | `@layer components` class with `data-*` hooks |
| Repeated structural pattern (content column) | `@layer components` class |
| Element with >12 utilities appearing in multiple files | Extract a Leptos component |
| Dynamic value from Rust signal | `style=("property", signal)` |

---

## Banned Patterns

### CSS

- **`@apply` to build component classes.** The Leptos component is the abstraction. Use `@layer components` with native CSS properties, not `@apply` chains.
- **Unlayered CSS.** Everything inside `@layer` or `@utility`.
- **Hardcoded color values.** Every color references a token or semantic alias. No `#FB4417`, no `rgb(251, 68, 23)`. If you need an opacity variant, use `oklch(from var(--token) l c h / alpha)`.
- **Ad-hoc spacing values.** Use Tailwind's spacing scale. If a specific value is needed, add it to `@theme` with a name.
- **`z-index` without documentation.** The grain overlay is `z-index: 1`. Modals/toasts are `z-index: 50`. Dropdowns are `z-index: 40`. Document any new layer.
- **`overflow-x: hidden` on `body`.** Use `overflow-x: clip` on an inner wrapper if needed. `overflow: hidden` on `body` breaks `position: fixed` in Safari.
- **`!important`**. If you need it, the specificity is wrong. Fix the specificity.

### Tailwind in Rust

- **`format!()` to construct class names.** Tailwind's scanner cannot detect dynamically constructed strings. Use static lookup (`match` arms with full literal class names) or `@source inline()` in CSS.
- **`class:` directive with computed names.** `class:hidden=signal` is fine (literal). `class={format!("text-{}", color)}` is not.

### HTML

- **Steps/sequences as `<div>`.** Use `<ol><li>`. The landing page used divs; the app does not.
- **Missing `data-testid`.** Every interactive element and every assertable display element gets a `data-testid`. This is the E2E contract (see `end2end/README.md`). **Error/alert containers that tests assert on MUST have `data-testid="action-error"` or a page-specific testid.** CSS class selectors (`.error`, `.alert`) are banned in tests — they couple tests to styling and break on class renames.
- **Form inputs without ARIA linkage.** `aria-describedby` for error messages, `aria-invalid` for error state, `aria-live="assertive"` on error containers. See `design-system.md § Form Fields`.

---

## Class Ordering Convention

When writing inline Tailwind utilities, follow this order for scannability:

```
1. Layout mode       (flex, grid, block, hidden)
2. Position          (relative, absolute, sticky, z-*)
3. Sizing            (w-*, h-*, min-*, max-*)
4. Flex/grid props   (items-*, justify-*, gap-*, col-span-*)
5. Spacing           (p-*, m-*, space-*)
6. Typography        (font-*, text-*, leading-*, tracking-*)
7. Colors & borders  (bg-*, text-*, border-*, ring-*, rounded-*)
8. Effects           (shadow-*, opacity-*)
9. Transitions       (transition-*, duration-*)
10. Interactive      (cursor-*, pointer-events-*)
11. State variants   (hover:*, focus-visible:*, disabled:*)
12. Responsive       (sm:*, md:*, lg:*)
13. Dark mode        (dark:*)
```

Not linter-enforced. Consistency is the goal.

---

## Admin vs Participant

Both use the same tokens. Density differs via `[data-layout="admin"]` on the admin layout root element.

The admin layout root is the `<AdminGuard>` wrapper or a `<div data-layout="admin">` inside it. Participant pages have no `data-layout` attribute (default density applies).

Admin-specific component classes (`.data-table`, `.stat-card`) live in the same `@layer components` block. They reference `--density-space-*` tokens which adapt automatically under `[data-layout="admin"]`.

---

## @font-face

Declared in `@layer base` inside `style/tailwind.css`:

```css
@layer base {
  @font-face {
    font-family: 'CyGrotesk';
    src: url('/fonts/CyGroteskGrandDark.woff2') format('woff2');
    font-weight: 900;
    font-display: swap;
  }

  @font-face {
    font-family: 'Mont';
    src: url('/fonts/Mont-Regular.woff2') format('woff2');
    font-weight: 400;
    font-display: swap;
  }

  @font-face {
    font-family: 'Mont';
    src: url('/fonts/Mont-SemiBold.woff2') format('woff2');
    font-weight: 600;
    font-display: swap;
  }
}
```

Font URLs use `/fonts/` (root-relative) because cargo-leptos serves `public/` as the static root.

---

## Grain Overlay

Applied in `@layer base` as `body::after`:

```css
@layer base {
  body::after {
    content: '';
    position: fixed;
    inset: 0;
    z-index: 1;
    pointer-events: none;
    opacity: 0.04;
    background-image: url("data:image/svg+xml,..."); /* feTurbulence SVG, 283 bytes */
    background-size: 200px;
    mix-blend-mode: overlay;
  }

  @media (prefers-reduced-motion: reduce) {
    body::after { display: none; }
  }
}
```

Changes from landing page: `z-index: 1` (was 100), `opacity: 0.04` (was 0.09), `mix-blend-mode: overlay` (was multiply — multiply is near-invisible on both cream and black as confirmed by Chrome inspection).

---

## Verification Checklist (Before Declaring Styling Work Done)

- [ ] No unlayered CSS in `style/tailwind.css`
- [ ] No hardcoded color values — `grep -rn '#[0-9a-fA-F]' style/` returns nothing
- [ ] No `@apply` usage
- [ ] No `format!()` constructing class names in `src/`
- [ ] All interactive elements have `:focus-visible` ring (inherited from base or component-specific)
- [ ] All form fields have `aria-describedby`, `aria-invalid`, `aria-live` where applicable
- [ ] All actionable elements have `data-testid`
- [ ] All `<ol>` for sequential content, not `<div>`
- [ ] Grain overlay at `z-index: 1`
- [ ] `prefers-reduced-motion: reduce` hides grain and disables transitions
- [ ] `prefers-color-scheme: dark` reassigns semantic tokens (if dark mode is implemented)
- [ ] `cargo leptos build` produces working CSS output
- [ ] Visual spot-check in browser: fonts load, colors match tokens, buttons/inputs styled

---

## Sources

All decisions trace to `ai-driven-research/same-te-design-system/surveys/`:
- `brand-guidelines.md` — official PDF palette, typography, logo rules
- `font-system.md` — woff2 file selection, weight rationale, @font-face declarations
- `logo-system.md` — SVG variant mapping, app recommendations
- `tailwind-setup.md` — cargo-leptos integration, v4 config, source detection
- `css-patterns.md` — token architecture, component patterns, variant system, form styling
- `landing-page-analysis.md` — contrast failures, clipping, grain behavior, app-scale concerns
