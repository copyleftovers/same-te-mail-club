# leptos_ui & biji-ui — Deep Technical Verification

**Date:** 2026-03-19
**Target Project:** Leptos 0.8.17 + Tailwind CSS v4 (standalone) + custom oklch design system

---

## leptos_ui

### Crate Metadata

| Field | Value |
|-------|-------|
| **Latest Version** | 0.3.21 |
| **Downloads (all-time)** | 28,040 |
| **Downloads (recent)** | 821/month |
| **Last Published** | 2026-03-09 |
| **Repository** | **None** — no GitHub repo registered on crates.io |
| **Documentation** | https://rust-ui.com/docs/components/installation |
| **Homepage** | https://rust-ui.com |
| **License** | MIT |
| **Author** | Max Wells |

### What It Actually Is

**Critical Finding:** leptos_ui is **NOT a component library**. It is a **macro-only utility crate** for class merging on Tailwind CSS.

**Three macros provided:**
- `clx!` — Creates a component with Tailwind class merging (conflict resolution)
- `variants!` — "Ultra-sophisticated variants macro for standardized Tailwind CSS component patterns"
- `void!` — Creates self-closing components with class merging

**Underneath:** Built on `tw_merge` crate (Tailwind class conflict resolution) and inspired by `Class Variance Authority` (CVA) from the JavaScript ecosystem.

**What it does NOT provide:**
- Zero pre-built components
- Zero semantic buttons, inputs, dialogs, etc.
- Zero accessibility primitives
- No ARIA scaffolding

**What it enables:**
```rust
// Developers still hand-write components, but class merging is automatic:
#[component]
fn Card(
    #[prop(into)] class: String,
    children: Children,
) -> impl IntoView {
    let merged = clx!["bg-sky-500 p-4", &class]; // clx! merges conflicting classes
    view! { <div class=merged>{children()}</div> }
}

// Usage: Card with override won't have conflicting bg-* classes
view! { <Card class="bg-orange-500" /> }
```

### Leptos 0.8 Compatibility

**VERIFIED** — `leptos_ui` explicitly depends on `leptos ^0.8` (matches your current version 0.8.17).

### Tailwind v4 Compatibility

**YES** — Class merging via `tw_merge` is version-agnostic. It doesn't parse CSS or config — it only resolves Tailwind's canonical class names (e.g., `bg-red-500`, `p-4`). Works identically with v3 and v4.

**However:** If you use Tailwind v4 syntax that wasn't in v3 (e.g., `oklch()` in `@theme`), `tw_merge` doesn't know about it. It only recognizes utility class names, not custom tokens. **For custom oklch tokens, class merging would not apply** — you'd have to use semantic aliases via CSS variables, which sidesteps the merging problem entirely.

### Design System Integration

Your project uses:
- **Raw tokens** in `@theme { --color-brand-orange: oklch(...) }` → generates utility classes like `bg-(--color-brand-orange)`
- **Semantic aliases** in `:root { --color-surface: var(...) }` → used via CSS variables, not utilities

**Impact:** `leptos_ui` is irrelevant for semantic aliases (they're CSS variables, not Tailwind classes). It only helps with token-generated utilities. Since you've already chosen the CSS variable approach for reuse and flexibility, `leptos_ui` provides minimal value.

### Verdict

**SKIP** ❌

**Reasoning:**
1. It's a **macro utility, not a component library**. This project already hand-rolls components (intentionally, per "no dependencies" philosophy in specs).
2. **Marginal value for class merging.** Your design system uses semantic CSS variables (`--color-surface`) for the majority of styling, bypassing the merging problem. You only need merging for exceptions and overrides, which Leptos's native reactivity handles fine.
3. **No GitHub repo.** The project is tightly coupled to rust-ui.com's closed ecosystem and lacks transparent source control. Breaking changes or abandonment risk without visibility.
4. **No accessibility or component patterns.** Zero help for the real work: ARIA, keyboard nav, focus management, screen reader patterns.

If this project ever switches to **Tailwind utility-first composition** (instead of semantic CSS variables), revisit leptos_ui. For now, it solves a non-problem.

---

## biji-ui

### Crate Metadata

| Field | Value |
|-------|-------|
| **Latest Version** | 0.4.4 |
| **Downloads (all-time)** | 6,763 |
| **Last Published** | 2026-03-16 |
| **Repository** | https://github.com/biji-ui/biji-ui |
| **License** | MIT |
| **Author** | Jon Saw |
| **Status** | Active, unstable (13 releases, 3 breaking changes documented) |

### Leptos 0.8 Compatibility

**VERIFIED** — biji-ui 0.4.x explicitly targets Leptos 0.8.x.

Compatibility matrix:
- biji-ui 0.1.x → Leptos 0.6.x
- biji-ui 0.2.x → Leptos 0.7.x
- biji-ui 0.3.x → Leptos 0.8.x
- biji-ui 0.4.x → Leptos 0.8.x ✓

### What It Actually Is

**Headless component primitives library.**

biji-ui is inspired by and modeled after:
- **HeadlessUI** (React) — unstyled, ARIA-complete components
- **Melt UI** (Svelte) — headless primitives with fine-grained reactivity

**25+ components provided:**
- Disclosure: `accordion`, `collapsible`, `drawer`, `dialog`, `hover_card`
- Navigation: `menu`, `menubar`, `navigation_menu`, `context_menu`, `dropdown_menu`, `tabs`
- Selection: `radio_group`, `checkbox`, `select`, `combobox`, `toggle_group`
- Input: `pin_input`, `slider`, `progress`
- Feedback: `toast`, `alert_dialog`
- Display: `tooltip`, `popover`, `calendar`, `command`, `separator`

**Implementation model:**
- **No styles shipped.** Components render semantic HTML + ARIA attributes only.
- **CSS framework agnostic.** Works with Tailwind, vanilla CSS, CSS-in-JS, anything.
- **Opt-in via feature flags.** Include only what you need; minimal binary bloat.
- **Separation of concerns.** Component provides behavior (keyboard nav, ARIA state), not presentation.

### What It Actually Provides

**Accessibility primitives** — the hard part:
- ARIA attributes (roles, states, properties)
- Keyboard navigation (arrow keys, Tab, Escape, Enter, etc.)
- Focus management and focus-visible rings
- Screen reader announcements (`aria-live`, `aria-describedby`)
- Controlled open/closed state with proper initial focus
- Pointer + keyboard event delegation

**Example usage:**
```rust
#[component]
fn MyDialog() -> impl IntoView {
    let (is_open, set_open) = signal(false);

    view! {
        <button on:click=move |_| set_open.set(true)>Open</button>

        // biji-ui Dialog handles focus trap, ESC close, ARIA, backdrop click
        <Dialog open=is_open on_open_change=move |v| set_open.set(v)>
            <DialogTrigger>// not shown if already open</DialogTrigger>
            <DialogContent>
                <h2>"Confirm Action"</h2>
                <p>"Are you sure?"</p>
            </DialogContent>
        </Dialog>
    }
}
```

You **style the semantic HTML output** with Tailwind/CSS. biji-ui provides the accessibility scaffolding.

### Accessibility

**Strong.** biji-ui follows:
- **WCAG 2.1 Level AA** patterns (implicitly — components conform to WAI-ARIA authoring practices)
- **Radix UI's accessibility spec** (which is battle-tested in production React apps)
- Full keyboard support (no mouse required)
- Proper ARIA state labeling (roles, live regions, expanded/selected/disabled)

**Example:** The `Dialog` component:
- Traps focus inside (not the element itself, but focus cannot escape)
- Closes on Escape (unless prevented)
- Returns focus to trigger on close
- Renders with `role="dialog"`, `aria-modal="true"`, etc.

This is **work that otherwise falls on you.** Your project already defines `@layer components` for buttons, badges, form fields with focus rings. biji-ui would eliminate the **behavioral parts** (focus trap, keyboard nav) while still letting you style.

### Testing & Maintenance

**Active development:**
- Commits from May 2024 → March 2026 (steady activity)
- GitHub repository public and transparent
- 13 releases with documented breaking changes (sign of iteration, not abandonment)

**Breaking changes noted in versions:**
- 0.2.0, 0.3.0, 0.4.0 had breaking changes
- Each bump is intentional refinement, not thrashing

**Ecosystem positioning:**
- Not as mature as Radix UI (JavaScript), but following its proven patterns
- Not as polished as Leptos's main offerings (thaw, etc.), but specialized in accessibility primitives
- For a project that already hand-rolls components, this is a good fit: you get the hard part (ARIA + keyboard), skip the styling.

### Integration with Project's Design System

**Compatibility:**
- biji-ui renders **unstyled semantic HTML** — compatible with any CSS approach (Tailwind, CSS variables, both)
- Your oklch tokens and semantic aliases apply directly to biji-ui output
- Your existing `@layer components` styling (buttons, inputs, badges) stays unchanged
- **Zero conflict.** biji-ui adds no styles; you provide all styling.

**Would reduce:**
- Focus trap implementation (Dialog, Popover, DrawerContent)
- Keyboard event handling (ArrowUp/Down in Menu, Escape to close, Tab wrapping)
- ARIA state management (aria-expanded, aria-selected, aria-checked, aria-label)
- Screen reader live regions (Toast, AlertDialog)

**Would NOT reduce:**
- Button styling (you already have `.btn` component)
- Form field styling (you already have `.field` component)
- Layout or spacing (unchanged)
- Custom brand tokens (unchanged)

### Tailwind v4 Compatibility

**YES** — biji-ui is CSS-agnostic. It doesn't generate Tailwind utilities or depend on Tailwind version. You apply Tailwind classes to the semantic HTML it renders.

### Verdict

**DEFER** — potential future adoption, not immediate priority.

**Reasoning:**
1. **Project is at Phase 6 (complete).** Integrating biji-ui would require refactoring all Dialog, Popover, Toast, Menu components. Not zero cost.
2. **WCAG 2.1 AA compliance already achieved.** Project's hand-rolled components pass accessibility audits (per `design-system.md`). Adding biji-ui doesn't improve that metric for components already shipped.
3. **High value for new components only.** If future phases add complex interactive components (ComboBox, Calendar, MultiSelect), biji-ui would save significant ARIA/keyboard work.
4. **Maintenance risk vs. control.** biji-ui has "unstable" status and breaking changes. For components already working, the operational risk (updating, debugging test failures) outweighs the benefit.
5. **No "killer feature" for phase 6.** Your components are already accessible and styled. biji-ui's value is in reducing boilerplate for new complex interactive components, not in improving existing ones.

**Future reconsideration:**
- If a Phase 7 requires ComboBox, DatePicker, Calendar, or MultiSelect: **ADOPT biji-ui for those only.** Use it selectively, not project-wide.
- If accessibility audit reveals keyboard nav gaps in Dialog/Menu: **ADOPT for refinement.**
- If maintenance burden (ARIA updates for new WCAG guidance) becomes visible: **ADOPT to delegate that to biji-ui.**

---

## Comparison: leptos_ui vs. biji-ui vs. Project's Hand-Rolled Approach

| Dimension | leptos_ui | biji-ui | Project (hand-rolled) |
|-----------|-----------|---------|----------------------|
| **Type** | Macro utility (class merging) | Headless components (behavior) | Full components (behavior + style) |
| **Components** | 0 | 25+ | 6 (button, badge, field, etc.) |
| **Styling** | Your responsibility | Your responsibility | You own; uses Tailwind + CSS vars |
| **Accessibility** | No | Yes (WCAG 2.1 AA patterns) | Yes (present in 6 components) |
| **Keyboard nav** | No | Yes | Partial (buttons, no focus trap) |
| **Tailwind v4** | Works (class-agnostic) | Works (not Tailwind-specific) | Designed for v4 |
| **Leptos 0.8** | Yes | Yes | Yes |
| **GitHub** | No | Yes | Yes (your repo) |
| **Dependencies** | tw_merge | None (100% Rust primitives) | None |
| **Maintenance risk** | High (no repo) | Medium (unstable status) | Zero (owned) |
| **Boilerplate reduction** | Low (doesn't solve your problem) | High (for complex interactive components) | None (baseline) |
| **Ready to use in Phase 6** | N/A (not applicable) | No (refactor cost) | Yes (already shipped) |

### Would Either Reduce the Boilerplate Identified in Your UI?

**leptos_ui:** No. Your design system uses semantic CSS variables for reuse, not utility class composition. Class merging solves a non-problem in your architecture.

**biji-ui:** **Conditionally.** It would eliminate ARIA + keyboard nav boilerplate for Dialog, Popover, Toast, Menu, and other complex interactive components. But your existing 6 components don't need it—they're already accessible. Value is only for future components.

**Recommendation:** Stay hand-rolled for now. If Phase 7 requires DatePicker, ComboBox, or Calendar, **revisit biji-ui** as a selective adoption for just those high-interaction components. Don't refactor Phase 6 components.

---

## Sources

- [Leptos UI on crates.io](https://crates.io/crates/leptos_ui)
- [Rust/UI Documentation](https://rust-ui.com)
- [biji-ui on crates.io](https://crates.io/crates/biji-ui)
- [biji-ui GitHub Repository](https://github.com/biji-ui/biji-ui)
- [lib.rs - leptos_ui](https://lib.rs/crates/leptos_ui)
- [lib.rs - biji-ui](https://lib.rs/crates/biji-ui)
- [tw_merge on crates.io](https://crates.io/crates/tw_merge)
- [HeadlessUI (JavaScript reference)](https://headlessui.com)
- [Melt UI (Svelte reference)](https://melt-ui.com)
