# Components — Deep Dive

## Summary

The Leptos component library ecosystem offers 7 main options ranging from full-featured design systems (Thaw, Leptodon) to headless, utility-focused primitives (leptix) and Material Design wrappers (leptos-material). Most support Leptos 0.8 and Tailwind CSS, though design approach varies significantly: some provide pre-styled components, others provide unstyled/headless components for maximum customization. For this project, which has a custom brand design system with very specific color tokens, typography, and component variants, the choice is narrower: either adopt a headless component library to build atop the custom design system, or hand-roll components (current approach).

## Per-Library Analysis

### Thaw

- **URL:** https://github.com/thaw-ui/thaw
- **Stars/Activity:** Active development; latest commit on main (0.5-beta branch for Leptos 0.8)
- **Leptos version:** 0.6, 0.7, **0.8 (main branch, beta)**
- **Tailwind compatible:** No
- **Components provided:** Fluent Design component library (full suite, docs at thawui.vercel.app)
- **What it does:** Provides pre-styled UI components following Microsoft Fluent Design principles. Comprehensive design system with color, typography, and component specifications baked in.
- **Relevance to this project:** **MEDIUM** — Thaw is a complete design system, but it imposes Fluent Design aesthetics. The project has a custom brand system (orange, cream, custom fonts) that does not align with Fluent. Adopting Thaw would mean either rethemeing it (unclear if possible) or abandoning the custom design system.
- **Adoption recommendation:** **SKIP** — The project's design system is binding and non-negotiable. Thaw's Fluent aesthetic is orthogonal. Effort to retheme would exceed effort to maintain hand-rolled components.

---

### leptos-struct-table

- **URL:** https://github.com/Synphonyte/leptos-struct-table
- **Stars/Activity:** Active development; regular commits visible
- **Leptos version:** 0.8 (latest releases 0.15+)
- **Tailwind compatible:** Yes — includes `TailwindClassesPreset` and dedicated Tailwind integration examples
- **Components provided:** Table/data grid with sorting, virtualization, pagination, multi-select, editable cells, async data loading
- **What it does:** Headless table component that generates data grids from Rust structs. Async-first, highly customizable renderers, virtualization for large datasets.
- **Relevance to this project:** **HIGH** — Tables are likely needed in admin views (participant lists, season data, assignment cycles). Headless design means it accepts custom class presets, so integration with the project's Tailwind setup is straightforward. Async-first design matches Leptos 0.8 patterns.
- **Adoption recommendation:** **ADOPT** — Use for admin data tables. The `TailwindClassesPreset` can be configured with the project's custom tokens. No design conflicts because the library is headless.

---

### leptix

- **URL:** https://github.com/leptix/leptix
- **Stars/Activity:** Active; latest release v0.2.3 (Nov 22, 2024). 187 commits on master.
- **Leptos version:** Not explicitly specified, but supports CSR/SSR via feature flags. Works with Actix, Axum (suggesting modern Leptos 0.7/0.8)
- **Tailwind compatible:** Yes — examples show Trunk + Tailwind, Actix + Tailwind, Axum + Tailwind
- **Components provided:** Accessible primitives — Accordion, Checkbox, Slider, Tabs, Toggle, ToggleGroup, RadioGroup, Label, Progress, Switch, ScrollArea, Separator, AspectRatio, Avatar, Collapsible, Toolbar. Dialogs and floating components TODO.
- **What it does:** Headless, unstyled accessibility-first components. Developers apply custom styling via class attributes. Early-stage development (components evolving).
- **Relevance to this project:** **HIGH** — Leptix is headless and Tailwind-compatible, meaning it can be integrated without conflicts. The library provides form primitives (Checkbox, RadioGroup, Slider) and layouts (Tabs, Accordion) that could reduce hand-rolled code. Accessibility-first aligns with project's WCAG AA commitment. Early-stage means fewer guarantees, but the library's philosophy (unstyled + accessible) is sound.
- **Adoption recommendation:** **EVALUATE** — Consider for form primitives and layout components (Tabs, Accordion). The headless design is compatible with the custom design system. Trade-off: early-stage means some components (Dialogs, floating) are incomplete. Start with lower-risk components (Checkbox, RadioGroup) and assess.

---

### leptos-material

- **URL:** https://github.com/jordi-star/leptos-material
- **Stars/Activity:** 19 commits, appears lower-activity. Last commit timestamp not visible in metadata.
- **Leptos version:** Not specified in README; claims stable Rust compatibility
- **Tailwind compatible:** No mention
- **Components provided:** Material Design wrappers — Checkbox, Textfield, Button, Icon, Card, Datepicker, Select, Chips, and others. Wraps Material Web Components.
- **What it does:** Thin Leptos wrapper around Material Web Components (native web components). Brings Material Design to Leptos without reimplementation.
- **Relevance to this project:** **LOW** — Material Design is orthogonal to the project's brand (orange + cream + custom fonts). Adopting Material would require either rethemeing Material (unclear if MWC allows brand-level customization) or abandoning the brand system. Low activity on the repo is also a concern.
- **Adoption recommendation:** **SKIP** — Material Design contradicts the project's custom brand system. Low repository activity suggests risk of abandonment.

---

### Rust Radix

- **URL:** https://github.com/rustforweb/radix
- **Stars/Activity:** 66 stars. **ARCHIVED and READ-ONLY as of Feb 2, 2026**. No longer maintained.
- **Leptos version:** Not specified; supports Dioxus, Leptos, Yew (unclear which versions)
- **Tailwind compatible:** Not mentioned
- **Components provided:** Components, icons, colors, templates ported from Radix (Radix UI primitives)
- **What it does:** Unstyled, accessible component library (Radix port) for multiple Rust web frameworks.
- **Relevance to this project:** **NONE** — The project is archived and unmaintained. Using archived, read-only code introduces risk: no bug fixes, no Leptos 0.9 / 0.10 support as Leptos evolves, no community.
- **Adoption recommendation:** **SKIP** — Dead project. Do not adopt.

---

### Rust shadcn/ui

- **URL:** https://github.com/rustforweb/shadcn-ui
- **Stars/Activity:** 222 stars. **ARCHIVED and READ-ONLY as of Feb 2, 2026**. No longer maintained.
- **Leptos version:** Not specified
- **Tailwind compatible:** Not mentioned in README
- **Components provided:** shadcn/ui port (copy-paste component library inspired by original shadcn/ui)
- **What it does:** Copy-paste, beautifully designed components ported to Rust for Leptos and Yew.
- **Relevance to this project:** **NONE** — The project is archived and unmaintained. Same risk as Rust Radix.
- **Adoption recommendation:** **SKIP** — Dead project. Do not adopt.

---

### Leptodon

- **URL:** https://github.com/openanalytics/leptodon
- **Stars/Activity:** Active; latest commit v1.0.1 (Mar 19, 2026). 198 commits on main. Docs at leptodon.dev. Starter template available.
- **Leptos version:** Not specified in README; check Cargo.toml for exact version
- **Tailwind compatible:** Yes — includes codegen to generate `.tailwind` files containing Leptodon source for Tailwind processing
- **Components provided:** Full component library (see leptodon.dev for details). Flowbite-inspired.
- **What it does:** Pre-styled, Tailwind-integrated component library inspired by Flowbite. Generates Tailwind-processable files via codegen.
- **Relevance to this project:** **MEDIUM** — Leptodon is Tailwind-first and actively maintained. However, it is "Flowbite-inspired," meaning it likely carries Flowbite aesthetics (neutral grays, typical SaaS defaults) rather than custom brand colors. Adopting Leptodon would mean either rethemeing it to match the brand (orange, cream, custom fonts) or accepting Flowbite's design language, losing the brand identity.
- **Adoption recommendation:** **EVALUATE with caution** — If Leptodon allows per-component color/token overrides via Tailwind's arbitrary-value syntax, it could work. However, the project's brand system (OKLCH color tokens, custom fonts, pill-shaped buttons with specific spacing) is very specific. Check: (1) Can Leptodon components be restyled via Tailwind overrides? (2) How much effort to retheme? If either is high-effort, skip.

---

## Category Verdict

### Do Not Adopt (Dead Projects)
- **Rust Radix, Rust shadcn/ui**: Both archived and read-only as of Feb 2, 2026. Do not depend on dead code.

### Skip (Design Conflicts)
- **Thaw**: Fluent Design system is incompatible with the project's custom brand (orange, cream, custom fonts).
- **leptos-material**: Material Design is incompatible with the project's brand. Low repository activity is also a risk.

### Evaluate (Headless/Compatible)
1. **leptos-struct-table** — **ADOPT for admin data tables**. Headless design + Tailwind support means zero conflicts with the custom design system. Use `TailwindClassesPreset` to integrate with project tokens. No new dependencies on specific aesthetics.

2. **leptix** — **EVALUATE for form primitives**. Headless, Tailwind-compatible, accessibility-first. Early-stage (some components TODO), but the philosophy aligns. Start with low-risk components (Checkbox, RadioGroup) and assess whether reducing hand-rolled code outweighs early-stage risk.

3. **Leptodon** — **EVALUATE only if restyling is low-effort**. Tailwind-integrated and actively maintained, but Flowbite-inspired means it carries SaaS design aesthetics. Only adopt if it allows deep Tailwind overrides to match the brand system (orange accents, cream surfaces, custom fonts, pill buttons). High retheme effort = skip.

### Recommendation for This Project

**Current approach (hand-rolled components) is optimal IF:**
- The project's component count is small (it is — buttons, form fields, badges, data tables)
- The brand system is binding and must be preserved (it is)
- Effort to maintain hand-rolled components < effort to retheme a pre-styled library (likely true for this project's scope)

**Consider adopting IF:**
- **leptos-struct-table** for admin data tables (headless, no design conflicts, high value-add for complex table logic)
- **leptix** for specific form primitives IF the early-stage risk is acceptable and the library proves stable over the next 1-2 releases

**Do not adopt:**
- Thaw, leptos-material, Radix, shadcn/ui (design conflicts, dead, or low activity)
- Leptodon (unless significant redesign effort is acceptable)

### Suggested Next Steps

1. **Assess admin table complexity:** If admin views require sorting, multi-select, virtualization, or complex filtering, adopt leptos-struct-table. The headless design ensures zero conflicts with the custom design system, and the async-first approach matches Leptos 0.8 patterns.

2. **Assess form primitive coverage:** Count how many form components (Checkbox, RadioGroup, Toggle, Tabs, Accordion) the project needs. If significant (>5 across the app), evaluate leptix for those specific components. Start with one (e.g., Checkbox) and assess stability before expanding.

3. **Document component decisions:** If any external component library is adopted, document the integration approach (e.g., "leptos-struct-table tables styled via custom TailwindClassesPreset") in guidance or implementation notes.

4. **Do not adopt Leptodon.** The effort to retheme Flowbite aesthetics to match the project's brand (orange, cream, custom fonts, pill shapes, specific spacing) is likely to exceed the effort to maintain hand-rolled components for a small-scope app.
