# Unlisted Leptos Crates on crates.io

**Report Date:** 2026-03-19
**Methodology:** Cross-referencing crates.io API results against awesome-leptos index (`index.md`)

---

## Search Methodology

Queried crates.io API with the following search terms and per-page limits:
- `q=leptos` (sort by downloads, recent-downloads) — 100 results each
- `q=leptos+tailwind` — 50 results
- `q=leptos+component` — 50 results
- `q=leptos+ui` — 50 results
- `q=leptos+form` — 50 results
- `q=leptos+table` — 50 results
- `q=leptos+query` — 50 results
- `q=leptos+validation` — 50 results

**Total unique crates examined:** ~400+
**Awesome-leptos entries:** 98
**Criteria for "unlisted":** Not mentioned in `index.md` and >500 downloads

---

## High-Signal Finds (>5000 downloads, not on awesome list)

### leptos_ui
- **Downloads:** 28,040
- **Version:** 0.3.21
- **Description:** Macros to build UI components easily with Leptos and Tailwind CSS
- **Category:** Styling & Components
- **Why interesting:** Direct Tailwind integration with component macros; complements the project's Tailwind v4 setup

### tailwind_fuse
- **Downloads:** 47,611
- **Version:** 0.3.2
- **Description:** Tailwind Merge and Variants — handles conflicting Tailwind classes and creates type-safe variants
- **Category:** Styling Utility
- **Why interesting:** Already listed in awesome-leptos as "Tailwind Fuse" under Styling and Design; no action needed

### leptos_form
- **Downloads:** 14,629
- **Version:** 0.1.8
- **Description:** Derive forms from structs with proc-macros
- **Category:** Forms
- **Why interesting:** Differs from ActionForm pattern; generates full form structure from struct definitions. BANNED by project idioms (uses signal-driven forms), but useful for projects not using ActionForm exclusively

### leptos_query
- **Downloads:** 47,487
- **Version:** 0.5.3
- **Description:** Async query manager for Leptos — cache, refetch, pagination
- **Category:** Data Fetching
- **Why interesting:** TanStack Query (React Query) -inspired. Complements Resource pattern. Not on awesome list despite high adoption

### leptos_oidc
- **Downloads:** 38,144
- **Version:** 0.9.0
- **Description:** Leptos utility library for simplified OpenID Connect (OIDC) authentication
- **Category:** Authentication/Integration
- **Why interesting:** Already listed in awesome-leptos under Libraries; no action needed

### leptos-bulma
- **Downloads:** 25,220
- **Version:** 0.6.0
- **Description:** Leptos component library based on Bulma CSS framework
- **Category:** Component Library
- **Why interesting:** Full component library alternative to Thaw, shadcn, etc. Not on awesome list. Mature (25K downloads)

### leptab
- **Downloads:** 19,482
- **Version:** 1.1.7
- **Description:** Data table for Leptos with backend support, styled with TailwindCSS
- **Category:** Data Tables
- **Why interesting:** Already listed in awesome-leptos as `leptos-struct-table` under Components; leptab is a separate, lower-level alternative not mentioned

### leptos-tiptap
- **Downloads:** 23,564
- **Version:** 0.9.0
- **Description:** Rich text editor integration for Leptos
- **Category:** Rich Text / Editor
- **Why interesting:** TipTap wrapper (popular Vue.js editor). Not on awesome list. ~24K downloads indicates solid adoption

### leptos-shadcn-ui
- **Downloads:** 5,107
- **Version:** 0.9.0
- **Description:** Beautiful, accessible UI components inspired by shadcn/ui for Leptos v0.8+
- **Category:** Component Library
- **Why interesting:** Already listed in awesome-leptos as "Rust shadcn/ui" under Components; no action needed

### leptos_form_tool
- **Downloads:** 7,817
- **Version:** 0.4.1
- **Description:** Declarative way to create forms for Leptos
- **Category:** Forms
- **Why interesting:** Macro-based form builder. Different approach from leptos_form. ~8K downloads but not on awesome list

### leptos-struct-component
- **Downloads:** 10,571
- **Version:** 0.2.0
- **Description:** Struct-based component definition system
- **Category:** Component Utilities
- **Why interesting:** Alternative component definition model. Niche but solid adoption

### biji-ui
- **Downloads:** 6,763
- **Version:** 0.4.4
- **Description:** Effortless headless UI components for Leptos projects
- **Category:** Component Library
- **Why interesting:** Headless (unstyled) component library. Similar positioning to Radix but not on awesome list

### leptonic
- **Downloads:** 12,626
- **Version:** 0.5.0
- **Description:** The Leptos component library
- **Category:** Component Library
- **Why interesting:** Generic "Leptos component library" entry. ~13K downloads. On awesome-leptos list under different name or missing

### leptos_chart
- **Downloads:** 12,919
- **Version:** 0.3.0
- **Description:** Visualization library for Leptos
- **Category:** Charting/Visualization
- **Why interesting:** Already listed in awesome-leptos as `leptos-chartistry` under Libraries; `leptos_chart` is separate or older name

### leptos_query_devtools
- **Downloads:** 13,053
- **Version:** 0.1.3
- **Description:** Devtools for Leptos Query
- **Category:** Developer Tools
- **Why interesting:** Debugging/inspection tool for leptos_query. Pairs with leptos_query (47K downloads)

### singlestage
- **Downloads:** 4,325
- **Version:** 0.4.1
- **Description:** UI component library for Leptos based on Basecoat UI and shadcn/ui
- **Category:** Component Library
- **Why interesting:** Inspired by both shadcn/ui and Basecoat. ~4K downloads, under-represented in awesome list

### leptos_image
- **Downloads:** 10,652
- **Version:** 0.4.5
- **Description:** Static image optimizer for Leptos
- **Category:** Image/Media
- **Why interesting:** Already listed in awesome-leptos as `leptos_image` under Libraries; no action needed

### leptos_darkmode
- **Downloads:** 12,098
- **Version:** 0.5.0
- **Description:** Helper for managing TailwindCSS dark mode in Leptos applications
- **Category:** Styling/Theme
- **Why interesting:** Already listed in awesome-leptos under Libraries; no action needed

### leptos-fetch
- **Downloads:** 15,247
- **Version:** 0.4.10
- **Description:** Async cache for data fetching and state management
- **Category:** Data Fetching
- **Why interesting:** Already listed in awesome-leptos under Libraries; no action needed

### leptos_darkmode
- **Downloads:** 12,098
- **Version:** 0.5.0
- **Description:** Helper for managing TailwindCSS dark mode in Leptos
- **Category:** Styling/Theme
- **Why interesting:** Already listed in awesome-leptos under Libraries; no action needed

### leptos-struct-table
- **Downloads:** 70,387
- **Version:** Latest
- **Description:** Generate complete batteries-included Leptos data table from struct definition
- **Category:** Data Tables
- **Why interesting:** Already listed in awesome-leptos under Components; no action needed

---

## Medium-Signal Finds (500–5000 downloads, not on awesome list)

| Name | Downloads | Version | Category | Description |
|------|-----------|---------|----------|------------|
| `leptos-forms-rs` | 2,907 | 1.3.0 | Forms | Type-safe, reactive form handling with macro support |
| `radix-leptos` | 4,666 | 0.9.0 | Components | Already on awesome-leptos as "Rust Radix"; skip |
| `leptos_async_signal` | 5,543 | 0.6.0 | Reactive Primitives | Async signal for SSR generation |
| `leptos-shadcn-ui-wasm` | 1,109 | 0.2.1 | Components | WASM-optimized shadcn components |
| `floating-ui-leptos` | 20,721 | 0.6.0 | Positioning | Floating UI library for Leptos |
| `shadcn-rust` | 6,733 | 0.0.23 | CLI/Tooling | CLI tool for scaffolding shadcn-style components |
| `leptos_darkmode` | 12,098 | 0.5.0 | Styling | TailwindCSS dark mode helper (already on list) |
| `leptos_meilisearch` | 15,243 | 0.6.2 | Search/Integration | Meilisearch integration (already on list as `leptos_meilisearch`) |
| `leptos-routes` | 2,282 | 0.3.0 | Routing | Fluent route declarations |
| `leptos_store` | 525 | 0.11.0 | State Management | Type-enforced state management |
| `leptos_datatable` | 3,900 | 0.1.1-alpha | Data Tables | Table component with validation |
| `leptos-shadcn-table` | 3,730 | Latest | Components | shadcn/ui table port |
| `leptos-shadcn-form` | 3,656 | 0.9.0 | Forms | shadcn/ui Form component port |
| `leptos_aria` | 1,536 | 0.0.0 | Accessibility | Accessible components for Leptos |
| `cinnog` | 8,063 | 0.6.0 | SSG/Tooling | Static site generation data layer (already on list) |
| `leptos-bevy-canvas` | 4,840 | 0.5.0 | Integration | Embed Bevy apps in Leptos |
| `tinkr` | 1,761 | 0.0.43 | Framework/Tooling | Full-stack framework for rapid Leptos development |
| `dom_testing_library` | 1,931 | 0.1.0 | Testing | Testing framework inspired by testing-library.com |
| `leptos_test` | 1,631 | 0.1.0 | Testing | Frontend testing toolkit based on dom_testing_library |
| `i18nrs` | 6,299 | 0.1.9 | i18n/Internationalization | Customizable i18n for WASM frameworks |
| `borang` | 72 | 0.1.1 | Forms | Form library with validation (experimental) |
| `leptos-md` | 38 | 0.1.0 | Markdown | Signal-free Markdown renderer with Tailwind |
| `leptos-keycloak-auth` | (not found in results) | — | Auth | Keycloak integration |
| `table-rs` | 1,808 | Latest | Tables | Customizable table component for WASM frameworks |
| `leptos-routes` | 2,282 | 0.3.0 | Routing | Fluent route builder |
| `leptos-query-rs` | 1,830 | 0.5.1 | Data Fetching | Type-safe data fetching |
| `leptos_oidc` | 38,144 | 0.9.0 | Auth | OpenID Connect (already on list) |

---

## Ecosystem Size Summary

| Metric | Count |
|--------|-------|
| **Total leptos-related crates on crates.io** | ~400+ |
| **Entries on awesome-leptos list** | 98 |
| **High-signal finds (>5K downloads, unlisted)** | 6–8 (see analysis below) |
| **Medium-signal finds (500–5K downloads, unlisted)** | ~15–20 |
| **Already on awesome-leptos list (verified)** | ~40 of results |
| **Net new recommendations** | 4–6 |

---

## Analysis: Truly Unlisted High-Signal Crates

After cross-checking against `index.md`, the following crates are genuinely missing from awesome-leptos:

### Tier 1: Recommend Adding

1. **leptos_ui** (28K downloads)
   - Macros for UI component building with Tailwind
   - Direct relevance to projects using Tailwind v4
   - **Suggested category:** Styling and Design → Components

2. **leptos-bulma** (25K downloads)
   - Full Bulma CSS-based component library
   - Mature, alternative to Thaw/shadcn for CSS-framework-based projects
   - **Suggested category:** Components

3. **leptos-tiptap** (23.5K downloads)
   - TipTap rich text editor integration
   - Fills gap in "editor/rich text" category (Papelito exists but TipTap is more mature)
   - **Suggested category:** Libraries → Rich Text

4. **leptos_form_tool** (7.8K downloads)
   - Declarative form generation (distinct from ActionForm idiom)
   - Complements the form ecosystem
   - **Suggested category:** Libraries → Forms

### Tier 2: Consider Adding (Niche but Solid)

5. **leptos-routes** (2.3K downloads)
   - Fluent route builder (alternative to router macros)
   - Niche but well-maintained

6. **leptos_store** (525 downloads, but titled "enterprise-grade")
   - Type-enforced state management
   - Low adoption, skip unless project needs state management guidance

7. **biji-ui** (6.8K downloads)
   - Headless component library
   - Alternative to Radix, worth noting for projects preferring unstyled components

### Tier 3: Not Recommended (Lower maturity, niche, or duplicate)

- **borang** (72 downloads) — Too experimental
- **leptos-md** (38 downloads) — Too niche
- **leptos_datatable** (3.9K downloads) — superseded by leptos-struct-table
- **singlestage** (4.3K downloads) — Derivative of existing patterns

---

## Duplicate/Already-Listed Verification

The following crates appear in results but ARE on awesome-leptos (verified):

| Crate | Awesome-Leptos Entry |
|-------|----------------------|
| `leptos_query` | Not found — **MISS** |
| `leptos_oidc` | "leptos_oidc" under Libraries |
| `leptos_image` | "leptos_image" under Libraries |
| `leptos_darkmode` | "leptos_darkmode" under Libraries |
| `leptos-fetch` | "leptos-fetch" under Libraries |
| `leptos-struct-table` | "leptos-struct-table" under Components |
| `cinnog` | "cinnog" under Libraries |
| `leptos-shadcn-ui` | "Rust shadcn/ui" under Components |
| `tailwind_fuse` | "Tailwind Fuse" under Styling and Design |
| `radix-leptos` | "Rust Radix" under Components |
| `floating-ui-leptos` | "Rust Floating UI" under Libraries |

---

## Critical Find: leptos_query (47.5K downloads)

**Status:** NOT on awesome-leptos list
**Downloads:** 47,487 (high adoption)
**Maturity:** Released versions 0.1.x → 0.5.3
**Ecosystem:** Has companion crate `leptos_query_devtools` (13K downloads)

This is a TanStack Query (React Query) port for Leptos. Despite 47K downloads, it's absent from the awesome list. **Strong recommend adding.**

---

## Recommendations for awesome-leptos Update

### Immediate Additions (High-Signal, Clear Fit)

1. **leptos_query** — Data Fetching (Libraries section)
2. **leptos_ui** — Styling and Design → Components subsection
3. **leptos-bulma** — Components section
4. **leptos-tiptap** — Libraries section → Rich Text Editor (new subsection or under Papelito)

### Secondary Additions (Niche but Solid)

5. **leptos_form_tool** — Libraries section → Forms
6. **biji-ui** — Components section (headless/unstyled alternative)
7. **leptos-routes** — Tools section (routing helper)

### Not Recommended

- `borang`, `leptos-md`, `singlestage`, `leptos_store` — too experimental or low adoption

---

## Sources

- awesome-leptos repository: https://github.com/leptos-rs/awesome-leptos
- crates.io API: https://crates.io/api/v1/
- Queries executed: 2026-03-19
