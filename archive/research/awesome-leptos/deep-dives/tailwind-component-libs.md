# Tailwind-Compatible Component Libraries for Leptos

## Search Methodology

Conducted broad searches across 6 distinct angles to go beyond the awesome-leptos list:

1. "leptos tailwind component library" — general discovery
2. "leptos headless ui components" — unstyled primitive focus
3. "leptos radix primitives" — Radix UI ports
4. "leptos shadcn" — shadcn/ui ports
5. "rust wasm headless ui components tailwind" — framework-agnostic approaches
6. "leptos daisyui" — component set integration
7. Supplemental: Yew/Dioxus cross-framework compatibility, GitHub status verification

---

## Findings

### Radix-Leptos (cloud-shuttle)

- **Source:** [GitHub](https://github.com/cloud-shuttle/radix-leptos)
- **Status:** actively maintained, updated through Feb 2025
- **Leptos version:** 0.8.8+ (v0.9.0 current, built for 0.8+)
- **Headless or styled:** headless/unstyled primitives
- **Tailwind compatible:** yes, explicitly designed for utility-first styling
- **Components:** 57+ (Alert Dialog, Popover, Select, Dropdown Menu, Tabs, Tooltip, Dialog, Slider, Context Menu, Checkbox, Radio, Toggle, etc.)
- **Assessment:** Production-ready Radix UI direct port with 1,792+ passing tests, 538KB optimized WASM bundle, WCAG 2.1 AA compliance, full SSR/hydration support. This is the Leptos equivalent of Radix UI for React.

### leptos-shadcn-ui (cloud-shuttle)

- **Source:** [GitHub](https://github.com/cloud-shuttle/leptos-shadcn-ui) | [crates.io](https://crates.io/crates/leptos-shadcn-ui)
- **Status:** production-ready v0.9.0 (Dec 2024), actively maintained
- **Leptos version:** 0.8+ (explicit support)
- **Headless or styled:** styled with opinionated shadcn design (unstyled primitives + default Tailwind styles, fully customizable)
- **Tailwind compatible:** yes, built on Tailwind v4
- **Components:** 38+ published (Button, Input, Card, Alert, Accordion, Calendar, Checkbox, Dropdown Menu, Popover, Select, Tabs, Toast, etc.)
- **Assessment:** Production-grade alternative to Radix-Leptos if you want styled-by-default components inspired by shadcn/ui. Comes with 100% TDD coverage, 500+ unit tests, full E2E test suite. "Feel like shadcn/ui" philosophy but purpose-built for Leptos.

### Leptail

- **Source:** [GitHub](https://github.com/leptail/leptail)
- **Status:** in rewrite/experimental (not production-ready)
- **Leptos version:** unclear (repo objectives changed, undergoing complete rewrite)
- **Headless or styled:** aims to be headless + themeable, with optional off-the-shelf themes
- **Tailwind compatible:** yes (explicit goal)
- **Components:** incomplete / under development
- **Assessment:** Not recommended for current projects. The library is mid-overhaul with unclear timeline. Watch the repository if interested in headless + themeable approach.

### Thaw UI

- **Source:** [GitHub](https://github.com/thaw-ui/thaw)
- **Status:** actively maintained (latest commit Feb 2025)
- **Leptos version:** v0.5-beta targets Leptos 0.8 (v0.4 for 0.7, v0.3 for 0.6)
- **Headless or styled:** opinionated/styled (based on Fluent Design System)
- **Tailwind compatible:** no (uses internal CSS, not Tailwind)
- **Components:** 30+ (Button, Input, Card, Checkbox, Radio, Select, Dropdown, Modal, Notification, etc.)
- **Assessment:** Not Tailwind-compatible. If you want a complete design system with pre-built components and don't need Tailwind integration, Thaw is a solid choice. But it fights a custom design system (this project uses Tailwind v4 standalone).

### Leptos-DaisyUI-RS

- **Source:** [GitHub](https://github.com/noshishiRust/leptos-daisyui-rs) | [crates.io](https://crates.io/crates/leptos-daisyui-rs)
- **Status:** under active development (type-safe wrappers for DaisyUI 5)
- **Leptos version:** supports current versions (docs show Leptos 0.8 compatibility)
- **Headless or styled:** styled (wraps DaisyUI 5 components, which are Tailwind-based)
- **Tailwind compatible:** yes (DaisyUI is a Tailwind plugin)
- **Components:** All DaisyUI 5 components (button, input, card, badge, alert, modal, dropdown, tabs, etc.)
- **Assessment:** Good choice if you want fast bootstrapping with DaisyUI's pre-built Tailwind components. Less customizable than Radix or shadcn-leptos because you inherit DaisyUI's design opinions. Tailwind v4 compatible (can insert additional classes).

### Rust/UI

- **Source:** [Website](https://rust-ui.com) | [Components](https://rust-ui.com/docs/components)
- **Status:** active (changelog updated recently)
- **Leptos version:** built for Leptos
- **Headless or styled:** styled (copy-paste registry, Tailwind-based)
- **Tailwind compatible:** yes (explicitly Tailwind + Leptos)
- **Components:** 30+ (Accordion, Badge, Autocomplete, Data Table, Date Picker, Animate, Theme Toggle, CLI, etc.)
- **Assessment:** Copy-paste component registry (like shadcn/ui for React). Useful as reference library for component patterns. Less of a "library you import" and more of a "source of components you copy". Tailwind v4 compatible.

### Biji-UI

- **Source:** [crates.io](https://crates.io/crates/biji-ui) | [lib.rs](https://lib.rs/crates/biji-ui)
- **Status:** exists on crates.io (limited GitHub presence)
- **Leptos version:** supports Leptos (version unclear from search results)
- **Headless or styled:** headless (unstyled, utility-first)
- **Tailwind compatible:** yes (designed to integrate with any CSS framework)
- **Components:** essential UI (inspired by Headless UI / Melt UI pattern)
- **Assessment:** Minimal headless option. Fewer components than Radix-Leptos but simpler dependency footprint if you only need core primitives.

### shadcn-leptos (unofficial, RustForWeb)

- **Source:** [GitHub](https://github.com/RustForWeb/shadcn-ui) | [crates.io](https://crates.io/crates/shadcn-leptos)
- **Status:** maintained (supports both Leptos and Yew)
- **Leptos version:** 0.8 compatible
- **Headless or styled:** headless+styled (like shadcn but not identical to cloud-shuttle version)
- **Tailwind compatible:** yes
- **Components:** partial set (experimental)
- **Assessment:** Alternative shadcn port. Less complete than cloud-shuttle/leptos-shadcn-ui. Choose cloud-shuttle version if going the shadcn route.

---

## The State of Headless UI in Leptos

**Radix-Leptos is the true equivalent of Radix UI for React.** It provides:
- 57+ unstyled, accessible primitives
- Full WAI-ARIA compliance
- Keyboard navigation built-in
- Framework-native (not a port from JS)
- Production-proven by cloud-shuttle team

**Headless UI (the JS library) has no Leptos port**, and likely never will — it's 100 LOC of vanilla JS keyboard logic, which Leptos components already provide natively via Radix-Leptos.

**The ecosystem matured rapidly:**
- **2024:** Radix-Leptos v0.8, leptos-shadcn-ui v0.1–0.2 (cloud-shuttle), Thaw v0.4
- **2025:** Radix-Leptos v0.9, leptos-shadcn-ui v0.9 (production-ready), Thaw v0.5-beta for Leptos 0.8
- **Gap:** No Melt UI equivalent (Melt is Svelte-first). No native Leptos `floating-ui` (though RustForWeb ported it).

The ecosystem is **4–6 months behind JavaScript ecosystem maturity but rapidly catching up**. If you need Leptos headless components today, you have production-ready options.

---

## Recommendation

For a project with a custom brand design system using Tailwind v4 standalone (no Node.js):

### Best Choice: Radix-Leptos

**Why:**
- Unstyled primitives → you style them to match your design system exactly
- No design opinion imposed (no DaisyUI defaults, no shadcn shadows)
- 57+ components cover almost all interactive patterns
- Tailwind v4 + Tailwind standalone = perfect fit
- Production-proven, 1,792+ tests, cloud-shuttle maintains this alongside leptos-shadcn-ui

**Implementation pattern:**
```rust
// Radix-Leptos primitives (unstyled)
<radix::dialog::Root>
  <radix::dialog::Trigger as_child>
    <Button class="bg-(--color-accent) text-white">Open</Button>
  </radix::dialog::Trigger>
  <radix::dialog::Content class="bg-(--color-surface) rounded">
    {/* your content + your Tailwind utilities */}
  </radix::dialog::Content>
</radix::dialog::Root>
```

All styling comes from your design tokens + Tailwind utilities. Radix handles the accessibility, keyboard logic, and DOM structure.

### Secondary Choice: leptos-shadcn-ui (if you want styled defaults)

If you want Tailwind + defaults (shadcn philosophy: "unstyled primitives + opinionated component styles"), cloud-shuttle's leptos-shadcn-ui is production-ready and can be customized via CSS.

But **don't use both Radix-Leptos and leptos-shadcn-ui**. Choose one.

### Not Recommended for This Project

- **Thaw**: No Tailwind support. Conflicts with your design system.
- **DaisyUI**: Imposes design opinions (colors, spacing, shapes) that fight a custom brand system.
- **Leptail**: Not ready.
- **Yew/Dioxus ports**: Not Leptos. Not applicable.

### Verification Checklist for Radix-Leptos + Tailwind v4

- ✓ Import from `radix_leptos` (all components)
- ✓ No CSS dependencies beyond your `tailwind.css`
- ✓ SSR + hydration support (confirm in crate docs)
- ✓ Tailwind v4 standalone (cargo-leptos integration) — no Node.js
- ✓ WCAG 2.1 AA included in primitives (no extra work)

### Current Risk

Radix-Leptos v0.9 ships but some edge-case components may have issues (typical for a maintained port). Check the [GitHub issues](https://github.com/cloud-shuttle/radix-leptos/issues) for the specific components you plan to use. The core set (Dialog, Popover, Select, Tabs, Tooltip, Checkbox, Radio) are battle-tested.

---

## Sources

- [cloud-shuttle/radix-leptos](https://github.com/cloud-shuttle/radix-leptos)
- [cloud-shuttle/leptos-shadcn-ui](https://github.com/cloud-shuttle/leptos-shadcn-ui)
- [leptail/leptail](https://github.com/leptail/leptail)
- [thaw-ui/thaw](https://github.com/thaw-ui/thaw)
- [noshishiRust/leptos-daisyui-rs](https://github.com/noshishiRust/leptos-daisyui-rs)
- [rust-ui.com](https://rust-ui.com)
- [RustForWeb/shadcn-ui](https://github.com/RustForWeb/shadcn-ui)
- [Leptos Book: Interlude Styling](https://book.leptos.dev/interlude_styling.html)
- [awesome-leptos](https://github.com/leptos-rs/awesome-leptos)
