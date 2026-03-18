# CSS Patterns Reference — Tailwind v4 for Leptos SSR

Authoritative guidance for implementing the design system. All patterns are Tailwind v4 CSS-first, tested against primary sources. This document is written for a solo developer maintaining a ~15-screen SSR app with no component library.

---

## Table of Contents

1. [Tailwind v4 Architecture](#1-tailwind-v4-architecture)
2. [Token Structure — @theme{}](#2-token-structure--theme)
3. [@layer, @utility, and @apply — When Each Applies](#3-layer-utility-and-apply--when-each-applies)
4. [Utility-First at Scale — The Maintainability Tension](#4-utility-first-at-scale--the-maintainability-tension)
5. [CSS Custom Properties + Tailwind Interplay](#5-css-custom-properties--tailwind-interplay)
6. [Component Variant Patterns Without a Framework](#6-component-variant-patterns-without-a-framework)
7. [Form Styling](#7-form-styling)
8. [Admin vs Participant UI — Same Tokens, Different Density](#8-admin-vs-participant-ui--same-tokens-different-density)
9. [Dark Mode / Theme Switching](#9-dark-mode--theme-switching)
10. [Leptos-Specific Considerations](#10-leptos-specific-considerations)
11. [Class Ordering Convention](#11-class-ordering-convention)

---

## 1. Tailwind v4 Architecture

### What changed from v3

Tailwind v4 is a ground-up rewrite. The key shifts that affect this codebase:

- **No `tailwind.config.js` needed.** All configuration lives in CSS via `@theme {}`.
- **`@import "tailwindcss"` replaces the three `@tailwind` directives.** Single line.
- **All theme values are native CSS custom properties.** `var(--color-orange-500)` works anywhere.
- **`@layer components` still exists but is no longer the recommended home for custom utilities.** Use `@utility` for single-element variant patterns, `@layer components` for multi-property layout primitives, and Leptos components for structural reuse.
- **`@apply` still exists but is more constrained.** Avoid it except for third-party override contexts.
- **Container queries are built-in.** No plugin needed.
- **Automatic content detection** scans all non-gitignored files. Rust `.rs` files are picked up automatically.

### The CSS layer order

Tailwind v4 declares layers in this order, lowest-to-highest precedence for normal styles:

```
theme → base → components → utilities → unlayered
```

**Critical implication:** Unlayered CSS (plain CSS outside any `@layer`) always beats layered CSS regardless of specificity. This means a plain `.foo { color: red }` will override a Tailwind utility class. Write your own CSS inside `@layer` blocks to avoid accidental overrides.

---

## 2. Token Structure — @theme{}

### The two tiers: raw tokens and semantic aliases

Define tokens in two layers. The `@theme {}` block holds raw tokens that generate utility classes. A `:root {}` block (outside `@theme`) holds semantic aliases that reference the raw tokens.

```css
@import "tailwindcss";

/* ─── Tier 1: Raw tokens → generates utility classes ─────────────────── */
@theme {
  /* Brand palette */
  --color-brand-orange:  oklch(0.72 0.18 42);
  --color-brand-pink:    oklch(0.74 0.14 355);
  --color-brand-gray:    oklch(0.45 0.01 250);
  --color-brand-blue:    oklch(0.55 0.16 250);
  --color-brand-black:   oklch(0.12 0.00 0);
  --color-brand-cream:   oklch(0.97 0.02 90);

  /* Typography */
  --font-display: "CyGrotesk", ui-sans-serif, sans-serif;
  --font-body:    "Mont",      ui-sans-serif, sans-serif;

  /* Spacing — keep the default Tailwind spacing scale, add named overrides if needed */
  /* --spacing-* is additive; the default 4px-base scale is retained unless reset */

  /* Radius */
  --radius-sm:  0.25rem;
  --radius-md:  0.5rem;
  --radius-lg:  0.75rem;
  --radius-pill: calc(infinity * 1px);
}

/* ─── Tier 2: Semantic aliases → CSS variables only, no utility generation ── */
:root {
  --color-surface:       var(--color-brand-cream);
  --color-surface-raised: var(--color-white, oklch(1 0 0));
  --color-text:          var(--color-brand-black);
  --color-text-muted:    var(--color-brand-gray);
  --color-accent:        var(--color-brand-orange);
  --color-accent-alt:    var(--color-brand-pink);
  --color-link:          var(--color-brand-blue);
  --color-error:         oklch(0.55 0.22 25);
  --color-success:       oklch(0.58 0.16 160);
}
```

### Why this split matters

`@theme {}` tokens generate utility classes (`bg-brand-orange`, `text-brand-cream`, etc.). Semantic aliases in `:root {}` are CSS variables only — they don't generate utilities, which is intentional. You reference semantic aliases via `var(--color-surface)` in component CSS or `bg-(--color-surface)` in Tailwind's arbitrary-value syntax.

The split means you can swap the entire visual theme by reassigning `:root` aliases without touching markup. The raw scale stays stable; the semantic layer changes per context (dark mode, admin vs participant, etc.).

### Naming conventions

- Raw palette: `--color-brand-{name}` — explicit brand prefix prevents collision with Tailwind's defaults
- Semantic: `--color-{role}` — role-based, not value-based (`--color-error`, not `--color-red`)
- Components: prefix with component name when scoped (`--btn-bg`, `--field-border`)
- Pseudo-private (Lea Verou pattern): prefix with `_` when a variable is internal to a component (`--_btn-bg`)

### Resetting the default palette

The default Tailwind colors are extensive and may generate unwanted utility classes. To use only brand colors:

```css
@theme {
  --color-*: initial;   /* removes all default color utilities */

  /* now define only your brand colors */
  --color-brand-orange: oklch(0.72 0.18 42);
  /* ... */
  --color-white: oklch(1 0 0);
  --color-black: oklch(0 0 0);
}
```

Be conservative here. Resetting `--color-*` removes `text-white`, `bg-transparent`, `ring-black`, etc. Prefer keeping the Tailwind defaults and just adding brand colors unless the output size matters.

---

## 3. @layer, @utility, and @apply — When Each Applies

### Decision tree

```
Do you need the style to respond to Tailwind variants (hover:, lg:, disabled:)?
  Yes → use @utility
  No  → use @layer components OR @layer base (element resets)

Is it a single-element, purely presentational pattern?
  Yes → @utility or inline utilities in markup
  No  → Leptos component (the Rust component IS the abstraction)

Is it a structural layout primitive shared across many screens?
  Yes → @layer components with CSS nesting
```

### @utility — for variant-aware custom utilities

Use `@utility` for single properties or small clusters that you want to use with modifiers:

```css
@utility text-balance {
  text-wrap: balance;
}

@utility scrollbar-none {
  scrollbar-width: none;
  &::-webkit-scrollbar { display: none; }
}

@utility grain {
  /* grain texture overlay — applied to a pseudo-element via @layer base or inline */
  background-image: url("/assets/grain.png");
  background-repeat: repeat;
  mix-blend-mode: overlay;
  opacity: 0.04;
  pointer-events: none;
}
```

These work with all variants: `lg:text-balance`, `dark:scrollbar-none`.

### @layer components — for multi-property named primitives

Reserve `@layer components` for patterns you'd otherwise repeat in 5+ Leptos components where extracting a Rust component is awkward (e.g., a prose content container, a card shell, a badge). Use CSS nesting:

```css
@layer components {
  /* Content container for participant-facing screens */
  .prose-page {
    max-width: 65ch;
    margin-inline: auto;
    padding-inline: var(--spacing-4);

    & h1 {
      font-family: var(--font-display);
      font-size: var(--text-3xl);
      color: var(--color-brand-black);
    }

    & p {
      font-family: var(--font-body);
      line-height: var(--leading-relaxed);
      color: var(--color-text-muted);
    }
  }

  /* Status badge shell — variant handled by data-* attributes */
  .badge {
    display: inline-flex;
    align-items: center;
    gap: var(--spacing-1);
    padding: var(--spacing-0\.5) var(--spacing-2);
    border-radius: var(--radius-pill);
    font-size: var(--text-xs);
    font-weight: var(--font-weight-semibold);
    font-family: var(--font-body);
  }
}
```

### @apply — narrow valid uses only

`@apply` is not gone in v4, but it is a trap in most situations. It reintroduces the indirection that utility-first CSS was designed to eliminate: you change CSS to affect HTML appearance, and you need to trace through two files to understand what an element looks like.

**Valid use cases:**

1. Overriding third-party component libraries where you cannot touch the HTML:
   ```css
   .select2-dropdown {
     @apply rounded-lg shadow-lg border border-gray-200;
   }
   ```

2. Applying Tailwind utilities inside a `<style>` block in a framework component (Leptos doesn't use this pattern — skip it).

**Do not use `@apply` to:**
- Build your own component class out of utilities (`.btn { @apply px-4 py-2 rounded ... }`)
- DRY up utility repetition — that's what Leptos components and CSS variables are for
- Create "semantic" CSS from utilities — this negates the utility-first approach entirely

The Tailwind team's position: "We highly recommend using template partials [i.e., components] so the styles and structure can be encapsulated in one place" — the component IS the abstraction.

---

## 4. Utility-First at Scale — The Maintainability Tension

### The real critique (not the strawman)

The honest critique of utility-first at scale is not "it looks ugly" or "it mixes concerns." It is:

1. **Long class lists are hard to scan.** A button with 15 utilities gives no immediate signal about which properties are load-bearing vs cosmetic.
2. **Variant explosion.** When a button has primary/secondary/destructive × normal/disabled/loading states, encoding all combinations as utility strings in markup becomes unwieldy without a framework like React to do the switching.
3. **No single source of truth for a visual pattern.** If `.btn-primary` appears in 8 Leptos components with the same 15 classes, a color change requires 8 edits.

### How to actually resolve it

**Resolution 1 — The Leptos component is the abstraction.** The Rust component is the single source of truth. If you need a button that appears in 8 places, write one `ButtonPrimary` component in Rust and put all the utility classes there. This is precisely equivalent to extracting a React component — the markup with all utilities lives in one place.

```rust
#[component]
fn PrimaryButton(children: Children, disabled: ReadSignal<bool>) -> impl IntoView {
    view! {
        <button
            class="inline-flex items-center justify-center gap-2 px-5 py-2.5 \
                   rounded-pill font-body font-semibold text-sm text-white \
                   bg-brand-orange hover:bg-orange-600 \
                   disabled:opacity-50 disabled:cursor-not-allowed \
                   focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-orange \
                   transition-colors"
            disabled=disabled
        >
            {children()}
        </button>
    }
}
```

**Resolution 2 — CSS custom properties for variant axes.** Instead of encoding every variant as separate utility strings, define a CSS component class with custom-property hooks. The variant sets the properties; the component class uses them.

```css
@layer components {
  .btn {
    /* --- variant hooks with defaults --- */
    --_btn-bg:      var(--btn-bg, var(--color-brand-orange));
    --_btn-color:   var(--btn-color, var(--color-brand-cream));
    --_btn-border:  var(--btn-border, transparent);

    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-2);
    padding: var(--spacing-2\.5) var(--spacing-5);
    border-radius: var(--radius-pill);
    border: 1px solid var(--_btn-border);
    background-color: var(--_btn-bg);
    color: var(--_btn-color);
    font-family: var(--font-body);
    font-weight: var(--font-weight-semibold);
    font-size: var(--text-sm);
    cursor: pointer;
    transition: background-color 150ms, color 150ms;

    &:focus-visible {
      outline: 2px solid var(--_btn-bg);
      outline-offset: 2px;
    }

    &:disabled,
    &[aria-disabled="true"] {
      opacity: 0.5;
      cursor: not-allowed;
    }

    /* Hover only when actually hoverable (touch devices: skip) */
    @media (hover: hover) {
      &:hover:not(:disabled):not([aria-disabled="true"]) {
        filter: brightness(0.92);
      }
    }
  }
}
```

Then variants are set via data attributes in Leptos:

```rust
// In Rust:
<button class="btn" data-variant="secondary">...</button>
<button class="btn" data-variant="destructive">...</button>
```

```css
/* Variant overrides via data attributes */
[data-variant="secondary"] {
  --btn-bg:     transparent;
  --btn-color:  var(--color-brand-black);
  --btn-border: var(--color-brand-gray);
}

[data-variant="destructive"] {
  --btn-bg:   var(--color-error);
  --btn-color: var(--color-brand-cream);
}
```

This is the CSS-equivalent of cva (class-variance-authority) — data-attribute selectors set variant-specific CSS custom properties, and the base class consumes them. No JS required.

**Resolution 3 — Accept inline repetition for non-reused elements.** For one-off layout containers, status displays, and page-specific elements, inline utilities are correct. The "problem" of duplication only exists if the same pattern appears in multiple files. Adam Wathan's original insight holds: **every new CSS class is an opportunity for new complexity; utilities constrain the design space**.

### On "div soup" (class list length)

Long class lists are a readability concern, not a correctness concern. Strategies:

1. **Group by concern with a comment on its own line** (in Rust string literals, use the `\` line continuation or just accept long strings — the compiler doesn't care).
2. **Name the groups mentally:** layout → sizing → spacing → color → typography → interactive states → responsive. Writing in that order makes scanning easier.
3. **Extract the Leptos component** when a class list exceeds ~12 utilities and appears in more than one place. That's the signal.

---

## 5. CSS Custom Properties + Tailwind Interplay

### Three uses of CSS variables in Tailwind v4

**Use 1: Theme tokens via `var()`** — reference raw or semantic tokens in custom CSS:
```css
.something { color: var(--color-brand-orange); }
```

**Use 2: Arbitrary value syntax** — reference CSS variables as Tailwind utility arguments:
```html
<div class="bg-(--color-surface) text-(--color-text)">
```
The `(--var-name)` syntax is v4's way to use a CSS variable as an arbitrary value without `[var(--x)]` bracket noise.

**Use 3: CSS variable as a component prop hook** — the Lea Verou pseudo-private pattern. The component class defines `--_internal: var(--public-api, fallback)`. External code sets `--public-api`; the component reads `--_internal`. This avoids cascade pollution:

```css
.field {
  --_field-border-color: var(--field-border-color, var(--color-brand-gray));

  border: 1px solid var(--_field-border-color);
}

/* Error state: set the public API variable on the container */
.field[data-error="true"] {
  --field-border-color: var(--color-error);
}
```

### When to use a CSS variable directly vs a Tailwind utility

| Situation | Approach |
|-----------|----------|
| Static brand color on a one-off element | Tailwind utility: `text-brand-orange` |
| Semantic role that may change per context | CSS variable: `color: var(--color-text)` or `text-(--color-text)` |
| Dynamic value from Rust (runtime state) | Inline `style` attribute: `style=("color", some_signal)` |
| Component variant control | CSS custom property hook + data attribute selector |
| Dark mode color swap | CSS variable redefined under `@media (prefers-color-scheme: dark)` or `.dark` class |

### Responsive semantic tokens (Josh Comeau pattern)

Instead of applying breakpoints per component, redefine a token at the breakpoint:

```css
:root {
  --page-gutter: var(--spacing-4);    /* 1rem on small screens */

  @media (width >= theme(--breakpoint-md)) {
    --page-gutter: var(--spacing-8);  /* 2rem on larger screens */
  }
}
```

Every component that uses `padding-inline: var(--page-gutter)` automatically gets the right value. No per-component breakpoints needed.

---

## 6. Component Variant Patterns Without a Framework

### The problem cva solves (and the CSS equivalent)

`cva` (class-variance-authority) is a JS utility that maps prop combinations to class lists. It exists because React components have props and JSX has no equivalent to data-attribute CSS selectors. In a Leptos/SSR context, you have:

1. **Rust enum → `data-*` attribute → CSS selector** — the idiomatic equivalent
2. **CSS custom property hooks** — reduce the number of distinct class combinations
3. **`@layer components` with CSS nesting** — encode all variants in one CSS block

### Button variants (full example)

```css
@layer components {
  .btn {
    /* Custom property API */
    --_bg:     var(--btn-bg,     var(--color-brand-orange));
    --_fg:     var(--btn-fg,     oklch(1 0 0));
    --_border: var(--btn-border, transparent);
    --_ring:   var(--btn-ring,   var(--color-brand-orange));

    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--spacing-2);
    padding-block: var(--spacing-2);
    padding-inline: var(--spacing-5);
    border: 1px solid var(--_border);
    border-radius: var(--radius-pill);
    background: var(--_bg);
    color: var(--_fg);
    font-family: var(--font-body);
    font-weight: 600;
    font-size: var(--text-sm);
    line-height: 1;
    cursor: pointer;
    transition: filter 120ms ease, opacity 120ms ease;
    text-decoration: none;
    white-space: nowrap;

    &:focus-visible {
      outline: 2px solid var(--_ring);
      outline-offset: 2px;
    }

    @media (hover: hover) {
      &:hover:not(:disabled):not([aria-disabled="true"]):not([data-loading]) {
        filter: brightness(0.9);
      }
    }

    /* Disabled — covers both HTML :disabled and Leptos hydration gate aria-disabled */
    &:disabled,
    &[aria-disabled="true"] {
      opacity: 0.45;
      cursor: not-allowed;
      pointer-events: none;
    }

    /* Loading state — spinner is a child element */
    &[data-loading] {
      opacity: 0.75;
      cursor: wait;
      pointer-events: none;
    }
  }

  /* Variant: secondary */
  .btn[data-variant="secondary"] {
    --btn-bg:     transparent;
    --btn-fg:     var(--color-brand-black);
    --btn-border: currentcolor;
    --btn-ring:   var(--color-brand-gray);
  }

  /* Variant: ghost */
  .btn[data-variant="ghost"] {
    --btn-bg:   transparent;
    --btn-fg:   var(--color-brand-blue);
    --btn-border: transparent;
  }

  /* Variant: destructive */
  .btn[data-variant="destructive"] {
    --btn-bg:   var(--color-error);
    --btn-fg:   oklch(1 0 0);
    --btn-ring: var(--color-error);
  }

  /* Size modifier */
  .btn[data-size="sm"] {
    padding-block: var(--spacing-1\.5);
    padding-inline: var(--spacing-3);
    font-size: var(--text-xs);
  }

  .btn[data-size="lg"] {
    padding-block: var(--spacing-3);
    padding-inline: var(--spacing-7);
    font-size: var(--text-base);
  }
}
```

In Leptos:

```rust
view! {
    <button class="btn" data-variant="secondary" data-size="sm" disabled=move || !hydrated.get()>
        "Cancel"
    </button>
    <button class="btn" data-variant="destructive">
        "Delete"
    </button>
}
```

### Status badge variants

```css
@layer components {
  .badge {
    --_bg:   var(--badge-bg,   var(--color-brand-gray));
    --_fg:   var(--badge-fg,   oklch(1 0 0));

    display: inline-flex;
    align-items: center;
    gap: var(--spacing-1);
    padding: 0.125rem var(--spacing-2);
    border-radius: var(--radius-pill);
    background: var(--_bg);
    color: var(--_fg);
    font-family: var(--font-body);
    font-size: var(--text-xs);
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }

  .badge[data-status="active"]    { --badge-bg: var(--color-success); }
  .badge[data-status="pending"]   { --badge-bg: var(--color-brand-orange); }
  .badge[data-status="error"]     { --badge-bg: var(--color-error); }
  .badge[data-status="inactive"]  { --badge-bg: var(--color-brand-gray); }
  .badge[data-status="confirmed"] { --badge-bg: var(--color-brand-blue); }
}
```

---

## 7. Form Styling

### The accessible field pattern

The gold standard field structure — label above, input, error below — with proper ARIA linkage:

```html
<!-- In Leptos (simplified) -->
<div class="field" data-error={has_error}>
  <label for="phone" class="field-label">Phone number</label>
  <input
    id="phone"
    name="phone"
    type="tel"
    class="field-input"
    aria-describedby="phone-error"
    aria-invalid={has_error}
  />
  <span id="phone-error" class="field-error" aria-live="assertive" role="alert">
    {error_message}
  </span>
</div>
```

```css
@layer components {
  .field {
    display: flex;
    flex-direction: column;
    gap: var(--spacing-1\.5);
  }

  .field-label {
    font-family: var(--font-body);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-text);
  }

  .field-input {
    width: 100%;
    padding: var(--spacing-2\.5) var(--spacing-3);
    border: 1.5px solid var(--color-brand-gray);
    border-radius: var(--radius-md);
    background: var(--color-surface-raised);
    color: var(--color-text);
    font-family: var(--font-body);
    font-size: var(--text-base);
    transition: border-color 120ms;

    &::placeholder {
      color: var(--color-text-muted);
      opacity: 0.6;
    }

    &:focus-visible {
      outline: none;
      border-color: var(--color-brand-blue);
      box-shadow: 0 0 0 3px oklch(from var(--color-brand-blue) l c h / 0.15);
    }

    /* Error state — applied via :has() on the parent container */
    .field[data-error="true"] & {
      border-color: var(--color-error);

      &:focus-visible {
        border-color: var(--color-error);
        box-shadow: 0 0 0 3px oklch(from var(--color-error) l c h / 0.15);
      }
    }

    &:disabled {
      opacity: 0.5;
      cursor: not-allowed;
      background: oklch(from var(--color-brand-gray) l c h / 0.08);
    }
  }

  .field-error {
    font-family: var(--font-body);
    font-size: var(--text-sm);
    color: var(--color-error);
    /* Hidden when empty — browser hides zero-height element */
    min-height: 0;

    /* Only visible when there is content */
    &:empty {
      display: none;
    }
  }
}
```

### focus-visible vs focus

**Use `:focus-visible` for custom focus rings, not `:focus`.**

- `:focus` fires on every focus event: keyboard Tab, mouse click, script
- `:focus-visible` fires only when the browser's heuristic determines the user needs the indicator (keyboard navigation, programmatic focus without recent pointer activity)

The practical result: clicking a button with a mouse does not show a focus ring; tabbing to it does. This is the expected browser behavior since 2022 (Baseline Widely Available) and the right default.

Reset the browser's default outline, then restore it with `:focus-visible`:

```css
/* In @layer base */
:focus {
  outline: none;           /* Remove default everywhere */
}

:focus-visible {
  outline: 2px solid var(--color-brand-blue);
  outline-offset: 2px;
}
```

Or use Tailwind utilities per-element:

```html
<button class="focus:outline-none focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-blue">
```

### Disabled state — the Leptos hydration gap problem

Leptos renders buttons with `disabled` on the server during SSR and the hydration gap. After WASM loads, the `disabled` attribute is removed by the `Effect`. This means `:disabled` is the correct CSS target — it matches the actual HTML attribute.

However, if you need to style a "logically disabled" element that does not have the HTML `disabled` attribute (e.g., a link styled as a button, or an `aria-disabled` element), use `aria-disabled`:

```css
/* HTML :disabled — covers <button disabled>, <input disabled> */
&:disabled { opacity: 0.45; cursor: not-allowed; }

/* ARIA disabled — for non-form elements or explicit ARIA */
&[aria-disabled="true"] { opacity: 0.45; cursor: not-allowed; pointer-events: none; }
```

Tailwind v4 provides both as variants: `disabled:opacity-45` and `aria-disabled:opacity-45`.

### OTP / phone input specifics

For the OTP and phone number inputs this app uses:

```css
.field-input[type="tel"],
.field-input[inputmode="numeric"] {
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.1em;
}
```

---

## 8. Admin vs Participant UI — Same Tokens, Different Density

### The core tension

Participant screens: spacious, emotional, brand-forward, single action per screen. Admin screens: dense, informational, multiple actions per view.

Both should use the same token set. Density is achieved by choosing different points on the existing spacing scale, not by defining a separate admin token set.

### Layout density context

Use a data attribute on the admin layout root to set a denser spacing baseline:

```css
/* Default (participant) density — already set by :root */
:root {
  --density-space-sm: var(--spacing-3);   /* 0.75rem */
  --density-space-md: var(--spacing-5);   /* 1.25rem */
  --density-space-lg: var(--spacing-8);   /* 2rem */
}

/* Admin density — tighter */
[data-layout="admin"] {
  --density-space-sm: var(--spacing-1\.5); /* 0.375rem */
  --density-space-md: var(--spacing-3);    /* 0.75rem */
  --density-space-lg: var(--spacing-5);    /* 1.25rem */
}
```

Components that use `--density-space-*` tokens automatically adapt. Static elements (h1, page title) ignore density tokens and use fixed spacing.

### Admin-specific layer

Add an admin-specific `@layer` (same layer as `components`, just in a separate file or block):

```css
@layer components {
  /* Admin data table */
  .data-table {
    width: 100%;
    border-collapse: collapse;
    font-family: var(--font-body);
    font-size: var(--text-sm);
  }

  .data-table th {
    padding: var(--density-space-sm) var(--density-space-md);
    text-align: left;
    font-weight: 600;
    color: var(--color-text-muted);
    border-bottom: 1px solid oklch(from var(--color-brand-gray) l c h / 0.2);
    white-space: nowrap;
  }

  .data-table td {
    padding: var(--density-space-sm) var(--density-space-md);
    border-bottom: 1px solid oklch(from var(--color-brand-gray) l c h / 0.08);
    vertical-align: middle;
  }

  .data-table tr:last-child td {
    border-bottom: none;
  }

  /* Stat/metric card for admin dashboard */
  .stat-card {
    padding: var(--density-space-md);
    background: var(--color-surface-raised);
    border-radius: var(--radius-lg);
    border: 1px solid oklch(from var(--color-brand-gray) l c h / 0.15);
  }

  .stat-card__label {
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
  }

  .stat-card__value {
    font-family: var(--font-display);
    font-size: var(--text-2xl);
    color: var(--color-text);
    line-height: 1.1;
    margin-top: var(--spacing-1);
  }
}
```

### Typography scale usage

Admin: `text-xs` and `text-sm` are the primary body sizes. `text-base` is for important values.
Participant: `text-base` and `text-lg` for body. `text-2xl`+ for headings and key status displays.

Both use the same display font for headings; only the sizing and spacing differ.

---

## 9. Dark Mode / Theme Switching

This app is Ukrainian, primarily web-based, and does not require a manual dark-mode toggle as a shipping requirement. Use `prefers-color-scheme` only (zero JS):

```css
@media (prefers-color-scheme: dark) {
  :root {
    --color-surface:        var(--color-brand-black);
    --color-surface-raised: oklch(0.18 0.01 250);
    --color-text:           var(--color-brand-cream);
    --color-text-muted:     oklch(0.65 0.01 250);
  }
}
```

If a manual toggle becomes a requirement later, add `@custom-variant dark (&:where([data-theme=dark], [data-theme=dark] *))` and set `data-theme` in Rust. The Leptos server must set the attribute before first paint to avoid flash (requires a small inline `<script>` reading `localStorage` — acceptable for SSR).

---

## 10. Leptos-Specific Considerations

### Class detection in Rust source files

Tailwind v4 scans `.rs` files as plain text. It detects complete, unbroken class name strings. This works:

```rust
class="px-4 py-2 rounded-lg bg-brand-orange text-white"
```

This does NOT work (Tailwind cannot detect the constructed name):

```rust
let color = "orange";
format!("bg-{color}-500")  // broken — not in source as a complete token
```

For dynamic class sets, use the `class:name=signal` pattern (Leptos applies the class conditionally) or write both branches as full strings:

```rust
class=("bg-brand-orange", is_primary)
class=("bg-transparent border border-brand-gray", !is_primary)
```

Or use `@source inline(...)` in CSS to safelist classes that cannot appear as complete tokens in source:

```css
@source inline("bg-brand-{orange,pink,blue,gray,black,cream}");
```

### Leptos class syntax

```rust
// Static classes — string literal, Tailwind scans this
class="btn"
class="btn px-5 py-2"

// Conditional class — Tailwind scans both strings
class:active=move || is_active.get()
class:disabled=move || !hydrated.get()

// Tuple syntax — reactive class addition
class=("ring-2 ring-brand-orange", move || is_focused.get())

// Dynamic (avoid when possible — Tailwind cannot detect)
class=move || format!("badge badge--{}", status.get())
// Instead: use data attributes + CSS selectors
```

### The hydration gate — CSS implications

The pattern `disabled=move || !hydrated.get()` means buttons are disabled during SSR. CSS must style `[disabled]` and `[aria-disabled="true"]` equivalently. Do not rely on `:disabled` alone for accessible styling — pair it with `[aria-disabled]` as shown in Section 7.

---

## 11. Class Ordering Convention

Write utility classes in this order. This is not enforced by a linter here, but consistent ordering makes class lists scannable at a glance:

```
1. Layout mode          flex, grid, block, hidden, contents
2. Position             relative, absolute, sticky, inset-*, z-*
3. Display sizing       w-*, h-*, min-*, max-*, aspect-*
4. Flexbox/grid props   flex-col, items-center, justify-between, gap-*, col-span-*
5. Spacing              p-*, px-*, py-*, m-*, mx-*, my-*, space-*
6. Typography           font-*, text-*, leading-*, tracking-*, uppercase
7. Colors & borders     bg-*, text-*, border, border-*, ring-*, rounded-*
8. Effects              shadow-*, opacity-*, blur-*
9. Transitions          transition, duration-*, ease-*
10. Interactive states  cursor-*, select-*, pointer-events-*
11. State variants      hover:*, focus-visible:*, active:*, disabled:*, aria-*:*
12. Responsive          sm:*, md:*, lg:*, xl:*
13. Dark mode           dark:*
```

Example:

```html
<button class="
  inline-flex
  items-center justify-center gap-2
  px-5 py-2.5
  font-body font-semibold text-sm
  bg-brand-orange text-white rounded-pill
  shadow-sm
  transition-[filter] duration-100
  cursor-pointer
  hover:brightness-90
  focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-orange
  disabled:opacity-45 disabled:cursor-not-allowed
">
```

Prettier (with the Tailwind prettier plugin) will enforce a consistent machine order. The above is the human order; both serve the goal of scannable class lists. Choose one and be consistent — the Prettier plugin's order is perfectly fine if you're using it.

---

## Sources

- Tailwind CSS v4 official docs: `tailwindcss.com/docs` — @theme, @layer, @utility, @apply, dark mode, hover/focus/state variants, responsive design, detecting classes
- Adam Wathan, "CSS Utility Classes and 'Separation of Concerns'" (adamwathan.me) — primary source for the philosophical argument
- Tailwind CSS v4.0 release blog — architecture changes, CSS-first configuration
- Josh Comeau on CSS custom properties (joshwcomeau.com) — semantic token aliasing, responsive tokens, media-query variable pattern
- Lea Verou, "Custom Properties with Defaults" — pseudo-private property pattern for component APIs
- CSS-Tricks, "A Complete Guide to Custom Properties" — two-tier raw/semantic token layering
- MDN: `:focus-visible`, `:has()`, Cascade Layers — authoritative browser behavior specs
- Every Layout (every-layout.dev) — intrinsic layout composition patterns (Stack, Box, Cluster)
- web.dev, Learn Forms: Accessibility — `aria-describedby` pattern, `aria-live` for errors
- Open Props (open-props.style) — design token naming conventions and scale patterns
- moderncss.dev — component CSS API patterns with CSS custom properties and data attributes
