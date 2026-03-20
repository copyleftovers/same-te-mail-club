# Tier 1: Headless UI Primitives Landscape

## Search Methodology

Searches performed (2026-03-19):

1. **awesome-leptos index** — reviewed all entries in the Components and Libraries categories
2. **crates.io API** — queried the following terms (50 results each):
   - `leptos headless`
   - `leptos ui component`
   - `leptos radix`
   - `leptos primitives`
   - `leptos accessible`
   - `leptos aria`
   - `leptos unstyled`
   - `leptos accordion`
   - `leptos dialog`
   - `leptos modal`
   - `leptos tabs`
   - `leptos dropdown`
   - `leptos select`
   - `leptos tooltip`
   - `leptos popover`
   - `leptos combobox`
   - `leptos collapsible`
   - `leptos context menu`
   - `leptos checkbox radio`
   - `leptos switch toggle`
   - `leptos floating ui`
   - `floating-ui-leptos`
3. **GitHub search** — "leptos headless component", "leptos primitives accessible"
4. **Direct repository inspection** — fetched `Cargo.toml` files for all candidate projects to verify Leptos version claims
5. **Source code inspection** — fetched component implementation files to verify headless status and code quality

---

## Candidates Found

### leptix_primitives

- **crates.io:** https://crates.io/crates/leptix_primitives
- **GitHub:** https://github.com/leptix/leptix
- **Leptos dep in Cargo.toml:** `leptos = "0.6"`
- **Headless:** yes — no embedded CSS in source tree; components render primitive HTML elements with zero styling; consumers apply all classes
- **Components:** accordion, aspect_ratio, avatar, checkbox, collapsible, label, progress, radio_group, scroll_area, separator, slider, switch, tabs, toggle, toggle_group, toolbar (16 components total; dialogs and floating components explicitly excluded)
- **Styling API:** `class` prop accepted on each component root; data attributes for state (`data-state="active"`, `data-disabled`, etc.) for CSS targeting; follows Radix UI's `data-[state]` convention
- **Last commit:** v0.2.3 published 2024-11-22; repository shows 187 commits on master branch; last confirmed active release Nov 2024
- **SSR + hydration:** feature flags `csr`, `ssr`, `hydrate`, `nightly` — all map to corresponding Leptos features
- **Notes:** This is a genuine port of Radix UI primitives to Leptos. Source inspection of `tabs.rs` confirms proper ARIA attributes (`role="tablist"`, `role="tab"`, `role="tabpanel"`, `aria-selected`, `aria-labelledby`, `aria-controls`, `aria-orientation`), roving focus group, and full keyboard navigation. Targets **Leptos 0.6**, not 0.8. No upgrade path announced. 116 stars, 10 forks. The `leptix` crate name on crates.io (v0.1.2, github.com/nishujangra/leptix) is a separate unrelated project by a different author.

---

### biji-ui

- **crates.io:** https://crates.io/crates/biji-ui
- **GitHub:** https://github.com/biji-ui/biji-ui
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8" }`
- **Headless:** yes — source inspection of `dialog.rs` and `tooltip.rs` confirms no embedded CSS; components accept optional `class` props composed via `cn!` utility; tooltip uses inline `position: fixed` for positioning coordinates only
- **Components:** accordion, alert_dialog, calendar (chrono optional dep), checkbox, collapsible, combobox, command, context_menu, dialog, drawer, hover_card, menu, menubar, navigation_menu, pin_input, popover, progress, radio_group, select, separator, slider, switch, tabs, toast, toggle_group, tooltip (26 components, all feature-flagged)
- **Styling API:** optional `class` prop on each component element; `cn!` macro for class merging; state exposed via data attributes (`data-state`, `data-checked`, etc.)
- **Last commit:** 2026-03-19 (merge of PR #24 fixing Toast progress); multiple PRs merged in the week of 2026-03-12 to 2026-03-19
- **SSR + hydration:** leptos-use 0.16 is the only runtime dep; no explicit SSR/hydrate feature flags in Cargo.toml; depends on web-sys features (Element, HtmlElement, Window, etc.) suggesting CSR-focused; needs verification for SSR usage
- **Notes:** Actively maintained as of the research date. Broadest component selection among Leptos 0.8 headless options. Dialog implementation includes real focus trapping, keyboard (Escape, Tab/Shift+Tab), and `role="dialog"` + `aria-modal="true"`. Tooltip implementation includes pointer enter/leave, focus/blur, polygon-based hover gap detection, collision avoidance, and `role="tooltip"` + `aria-describedby`. 0 GitHub stars (very new or low visibility); 317 commits. `leptos-use = "0.16"` is compatible with leptos-use 0.18 in the project? Needs checking — project currently uses leptos-use 0.18.3. The `0.16` requirement would conflict.

---

### RustForWeb/radix (archived)

- **crates.io:** not published to crates.io as individual crates
- **GitHub:** https://github.com/RustForWeb/radix (archived 2026-02-02, read-only)
- **Leptos dep in Cargo.toml:** `leptos = "0.8.0"` (workspace dependency in root Cargo.toml)
- **Headless:** yes — source inspection of `switch.rs` and `progress.rs` confirms no CSS; renders primitive HTML with ARIA attributes and the Radix `as_child`/`Primitive` composition pattern
- **Components:** 28 packages in `packages/primitives/leptos/`: accessible-icon, arrow, aspect-ratio, avatar, checkbox, collection, compose-refs, direction, dismissable-layer, focus-guards, focus-scope, id, label, menu, popper, portal, presence, primitive, progress, roving-focus, separator, switch, toggle, use-controllable-state, use-escape-keydown, use-previous, use-size, visually-hidden
- **Styling API:** `as_child` prop enables rendering into consumer elements; `node_ref` for DOM access; state via data attributes (`data-state`, `data-disabled`, `data-orientation`); ARIA handled internally
- **Last commit:** project archived 2026-02-02; last commit prior to archive; 614 commits total
- **SSR + hydration:** Leptos 0.8.0 supports SSR; no explicit feature flags inspected in individual packages (uses workspace dep)
- **Notes:** This is a genuine, high-quality port of Radix UI primitives to Leptos, developed by the RustForWeb organization (same org that maintains floating-ui for Rust). Code quality is significantly above cloud-shuttle/radix-leptos (see below). However, **the project was archived on 2026-02-02 and is unmaintained**. Never published to crates.io. Can be consumed via git dependency. The 28 packages are primitives + utility hooks — not high-level components like tabs or dialog (those would be built atop primitives like menu, focus-scope, presence, etc.). 66 stars.

---

### cloud-shuttle/radix-leptos (WARNING: misleading name)

- **crates.io:** https://crates.io/crates/radix-leptos (v0.9.0)
- **GitHub:** https://github.com/cloud-shuttle/radix-leptos
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8.8", features = ["ssr", "csr"] }` (workspace)
- **Headless:** claimed yes; source inspection reveals minimal CSS class names embedded (`"radix-checkbox"`, `"radix-checkbox-label"`, `"radix-tabs"`, `"radix-tabs-list"`, etc.) suggesting styling hooks but no framework CSS
- **Components:** 77 items listed including accordion, aspect_ratio, separator, split_pane, resizable, button, checkbox, radio_group, toggle, switch, slider, range_slider, select, multi_select, combobox, date_picker, time_picker, calendar, badge, avatar, progress, alert, alert_dialog, toast, skeleton, dialog, popover, dropdown_menu, context_menu, hover_card, sheet, tooltip, tabs, menubar, navigation_menu, toolbar, label, form, search, otp_field, password_toggle_field, command_palette, code_editor, rich_text_editor, color_picker, file_upload, image_viewer, infinite_scroll, virtual_list, drag_drop, and more
- **Styling API:** CSS class names (`"radix-*"`) and `data-variant`/`data-size` attributes
- **Last commit:** 2025-09-25 ("feat: Complete remediation plan - fix all compilation errors")
- **SSR + hydration:** features `ssr`, `hydrate` in Cargo.toml — but a filed bug (issue #2, opened 2025-11-15, status: open) reports that `radix*` crates unconditionally enable the `ssr` feature during hydrate builds, breaking SSR/hydrate split compilation
- **Notes:** **This is not a genuine Radix UI port.** Source code inspection of `tabs.rs` reveals placeholder code: `aria-selected="false"` hardcoded and never updated, `handle_keydown` stubs that `prevent_default` but perform no navigation, comments like "In a real implementation, this would...". The `dialog.rs` always renders children regardless of `open` state with no focus trapping. The checkbox has `"radix-checkbox"` class names but no actual state management logic matching Radix primitives. Despite claims of "57+ components", "1,792+ passing tests", and "1,200+ tests", the implementations are scaffolding. The SSR feature flag bug is unfixed as of 2025-09-25. Last commit was a mass "remediation" of compilation errors. 16 stars.

---

### floating-ui-leptos

- **crates.io:** https://crates.io/crates/floating-ui-leptos (v0.6.0)
- **GitHub:** https://github.com/RustForWeb/floating-ui
- **Leptos dep in Cargo.toml:** `leptos = "0.8.0"` (workspace; individual package uses `leptos.workspace = true`)
- **Headless:** yes — this is a positioning utility library, not a UI component library; provides `use_floating` hook returning reactive x/y coordinate signals; no HTML emitted beyond what the consumer controls
- **Components:** not a component library; provides: `use_floating` hook, `FloatingOptions` (placement/side/alignment/offset config), middleware system (Arrow, AutoPlacement, Flip, Hide, Inline, Offset, Shift, Size), and collision detection utilities
- **Styling API:** n/a — the consumer receives coordinate values and applies them as `style` props; no class API
- **Last commit:** 2026-03-17 (active; multiple commits per week; also automated Renovate dependency updates)
- **SSR + hydration:** Leptos 0.8 SSR supported; 20,722 total downloads; actively maintained by RustForWeb organization (same org as the archived radix project)
- **Notes:** This is a positioning primitive, not a headless component library. It provides the building block needed to implement tooltips, popovers, and dropdowns with correct viewport-aware positioning. Separate from RustForWeb/radix (which is archived); floating-ui is actively maintained. v0.6.0 published 2025-11-04. 85 stars.

---

### leptos_context_menu

- **crates.io:** https://crates.io/crates/leptos_context_menu (v0.1.0)
- **GitHub:** https://github.com/Ovenoboyo/leptos_context_menu
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8.11", features = ["csr"] }`
- **Headless:** yes — explicitly documented: "This library does not provide any CSS. The menu will render as raw HTML elements. You must provide your own CSS to style the menu, position it correctly, and handle visibility states."
- **Components:** context menu (right-click menu) + bottom sheet variant
- **Styling API:** raw HTML elements; consumer provides all CSS including positioning
- **Last commit:** published 2025-12-25; 33 commits on main branch; 0 stars
- **Notes:** Single-purpose: context menus only. CSR-only (`features = ["csr"]` — no SSR support). Does not handle positioning — consumer must implement positioning via CSS. Very early stage (0 stars, no SSR).

---

### leptail (archived)

- **crates.io:** not checked (never published as a standalone crate)
- **GitHub:** https://github.com/leptail/leptail (archived 2024-11-23)
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.7.0-alpha", features = ["nightly"] }`
- **Headless:** partially — described as "headless, with a default Design System"; provides unstyled primitives but also ships a default theme
- **Components:** leptail_core, leptail_component, leptail_system sub-packages; specific component list not enumerated in README
- **Styling API:** not fully determined before archive
- **Last commit:** 2024-02-15; archived 2024-11-23 with note "still in experimentation level, objectives have changed, requires complete rewrite"; 50 total commits; 6 stars
- **Notes:** Archived. Targets Leptos 0.7-alpha. Not usable.

---

### accordion-rs (opensass)

- **crates.io:** https://crates.io/crates/accordion-rs (v0.2.6)
- **GitHub:** https://github.com/opensass/accordion-rs
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.7.7", optional = true }`
- **Headless:** yes — source inspection confirms no embedded CSS; accepts `class` prop; ARIA attributes are optional parameters
- **Components:** accordion only (single component)
- **Styling API:** `class` prop on all elements; also accepts inline `style` via param
- **Last commit:** v0.2.6 released 2025-04-17; 20 total commits
- **SSR + hydration:** not examined; Leptos 0.7.7 dependency (optional, activated by `lep` feature flag)
- **Notes:** Targets Leptos 0.7, not 0.8. Single component (accordion only). Part of the opensass ecosystem which builds similar single-component crates for Yew, Dioxus, and Leptos. The opensass pattern is: one crate per component, targets 0.7, accepts class props.

---

### scroll-rs (opensass)

- **crates.io:** https://crates.io/crates/scroll-rs (v0.2.3)
- **GitHub:** https://github.com/opensass/scroll-rs
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.7.7", optional = true }`
- **Headless:** yes — described as "headless, customizable scroll-to-target component"; no CSS in source
- **Components:** scroll-to-target button (single utility component)
- **Styling API:** `class` prop; custom icons accepted
- **Last commit:** v0.2.3 released 2025-04-17; 14 total commits
- **SSR + hydration:** Leptos 0.7 optional dep via `lep` feature
- **Notes:** Targets Leptos 0.7. Single narrow-use utility component (scroll to anchor). Part of opensass ecosystem.

---

### input-rs (opensass)

- **crates.io:** https://crates.io/crates/input-rs (v0.2.5)
- **GitHub:** https://github.com/opensass/input-rs
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.7.7", optional = true }`
- **Headless:** partially — accepts `class` props but purpose unclear without deeper inspection
- **Components:** text input, password input, textarea, telephone/phone number input with country code
- **Styling API:** `class` prop on input elements
- **Last commit:** v0.2.5 released 2025-04-17
- **SSR + hydration:** Leptos 0.7 optional dep
- **Notes:** Targets Leptos 0.7. Part of opensass ecosystem. Phone input with country code is an interesting niche component not otherwise available. 46 total commits.

---

### guillotine-ui

- **crates.io:** https://crates.io/crates/guillotine-ui (v0.0.1)
- **GitHub:** https://github.com/elephant-ladder/guillotine-ui
- **Leptos dep in Cargo.toml:** not accessible (GitHub 404 on repository — repo may be private or renamed)
- **Headless:** described as "inspired by BitsUI" (a headless Svelte library) and "minimal component library"
- **Components:** unknown (pre-alpha, API under active design)
- **Styling API:** unknown
- **Last commit:** published to crates.io 2025-11-17; repository currently inaccessible
- **Notes:** Pre-alpha (v0.0.1), inaccessible repository. Not evaluable.

---

### melt-ui

- **crates.io:** https://crates.io/crates/melt-ui (v0.0.6)
- **GitHub:** https://github.com/luoxiaozero/melt-ui
- **Leptos dep in Cargo.toml:** workspace Leptos version is `0.8.5` (workspace Cargo.toml inspected — it is the same repo as thaw-ui/thaw)
- **Headless:** no — the luoxiaozero/melt-ui repository redirects to thaw-ui/thaw which is a styled component library based on Fluent Design
- **Components:** Thaw UI styled components (Button, Input, Select, etc. with Fluent Design styling)
- **Styling API:** styled (Fluent Design system)
- **Last commit:** crates.io shows last published 2023-11-05; repository is Thaw UI
- **Notes:** melt-ui was an early project that evolved into / was absorbed by Thaw UI. The crate on crates.io is stale (2023). Not headless.

---

### leptos_aria

- **crates.io:** https://crates.io/crates/leptos_aria (v0.0.0), also `leptos_aria_button` (v0.0.0), `leptos_aria_interactions` (v0.0.0), `leptos_aria_utils` (v0.0.0)
- **GitHub:** https://github.com/ifiokjr/leptos_aria
- **Leptos dep in Cargo.toml:** `leptos = { git = "https://github.com/leptos-rs/leptos", rev = "b9f05f9" }` (pinned to an early pre-release git commit)
- **Headless:** intent is yes — "a port of the react-aria ecosystem for the leptos framework"
- **Components:** none implemented; project is in foundational/aspirational stage
- **Styling API:** n/a (not implemented)
- **Last commit:** 19 commits total; no releases published; created 2023-01-24; 3 stars
- **Notes:** Abandoned or dormant. Pinned to a pre-0.1 git commit of Leptos. Version 0.0.0 published to crates.io (unpublishable status). No functional code delivered.

---

### ankarhem/leptos-components

- **crates.io:** not published
- **GitHub:** https://github.com/ankarhem/leptos-components
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.4", features = ["nightly"] }`
- **Headless:** described as "headless ui component library for leptos"
- **Components:** unknown (5 commits total; repository structure shows `components` and `leptos-ui` directories)
- **Styling API:** unknown
- **Last commit:** 2023-07-25; 5 total commits; 0 stars
- **Notes:** Abandoned. Targets Leptos 0.4 (pre-stable). Not usable.

---

### thaw_snowpack

- **crates.io:** not published
- **GitHub:** https://github.com/TheToddmeister/thaw_snowpack
- **Leptos dep in Cargo.toml:** v0.4 (main branch) targets Leptos 0.7
- **Headless:** yes (aims to be headless)
- **Components:** unspecified (2 commits, project inception only)
- **Styling API:** unknown
- **Last commit:** 2024-10-23; 2 total commits; 0 stars
- **Notes:** Effectively non-existent. No components implemented. Never published.

---

### leptos-modal

- **crates.io:** https://crates.io/crates/leptos-modal (v0.3.1), also `leptos-modal-macros` (v0.3.1)
- **GitHub:** not provided in crates.io metadata
- **Leptos dep in Cargo.toml:** not inspectable (no repository URL)
- **Headless:** described as "Modal composable for Leptos" — unclear
- **Components:** modal/dialog only
- **Styling API:** unknown
- **Last commit:** published 2026-01-13; 129 total downloads across all versions
- **Notes:** Repository URL absent from crates.io metadata. Very low download count. Cannot evaluate quality or headless status without source access.

---

### leptos-floating (aonyx-ai)

- **crates.io:** https://crates.io/crates/leptos-floating (v0.1.0)
- **GitHub:** https://github.com/aonyx-ai/leptos-floating
- **Leptos dep in Cargo.toml:** `leptos = ">=0.8.3"`
- **Headless:** yes — positioning primitive only; provides `use_floating` hook with reactive coordinate signals
- **Components:** positioning utility only (`use_floating` hook, `FloatingOptions`)
- **Styling API:** n/a (coordinates returned as signals; consumer applies)
- **Last commit:** created 2026-02-16; 5 total commits; 0 stars
- **Notes:** Functionally overlaps with `floating-ui-leptos` (the established RustForWeb crate). Much simpler API (no middleware system). Very new (Feb 2026), 0 stars. Does not have the depth of `floating-ui-leptos`.

---

## Landscape Summary

**Total candidates found:** 17 libraries/projects

**Breakdown by Leptos version compatibility with 0.8:**

| Library | Leptos target | Status |
|---------|--------------|--------|
| biji-ui | 0.8 | Active (2026-03-19) |
| RustForWeb/radix | 0.8.0 | Archived 2026-02-02; git-only |
| cloud-shuttle/radix-leptos | 0.8.8 | Active but placeholder code + SSR bug |
| floating-ui-leptos | 0.8.0 | Active, maintained |
| leptos-floating (aonyx) | >=0.8.3 | Active but minimal |
| leptos_context_menu | 0.8.11 | Active, CSR-only, narrow scope |
| leptos-modal | unknown | Published 2026-01-13; no repo |
| leptix_primitives | 0.6 | Inactive since Nov 2024 |
| accordion-rs | 0.7.7 | Inactive since Apr 2025 |
| scroll-rs | 0.7.7 | Inactive since Apr 2025 |
| input-rs | 0.7.7 | Inactive since Apr 2025 |
| melt-ui | absorbed into Thaw | Stale (2023) |
| leptail | 0.7-alpha | Archived 2024-11-23 |
| leptos_aria | pre-0.1 git pin | Dormant (2023) |
| guillotine-ui | unknown (0.0.1 pre-alpha) | Repo inaccessible |
| ankarhem/leptos-components | 0.4 | Abandoned 2023 |
| thaw_snowpack | 0.7 | 2 commits, non-existent |

**Truly headless and targeting Leptos 0.8:**
- biji-ui — 26 components, genuinely headless, actively maintained
- RustForWeb/radix — 28 genuine primitive packages, genuinely headless, archived (git-only)
- floating-ui-leptos — positioning utility, not a component library
- leptos_context_menu — context menu only, CSR-only

**Notable disqualifications:**
- `cloud-shuttle/radix-leptos`: Despite the Radix name and claims of "57+ components", source inspection reveals placeholder/skeleton code with hardcoded ARIA states, non-functional event handlers, and an unfixed SSR feature flag bug that breaks standard Leptos split compilation
- `leptix_primitives`: Genuine quality implementation (tabs, switch confirmed functional with proper ARIA) but targets Leptos **0.6**, incompatible with 0.8 projects
- All opensass crates (accordion-rs, scroll-rs, input-rs): target Leptos 0.7

**The Leptos headless ecosystem for 0.8 is sparse.** The highest-quality options are biji-ui (active, 26 components, 0.8) and RustForWeb/radix (archived, 28 primitives, genuine Radix port, 0.8 but git-only).
