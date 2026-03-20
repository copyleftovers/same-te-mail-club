# Tier 1: Data Display & Tables Landscape

## Search Methodology

1. **awesome-leptos index** — Components and Libraries sections scanned for table/data/list entries
2. **crates.io API search** — queries: `leptos+table`, `leptos+grid`, `leptos+list`, `leptos+virtual`, `leptos+datatable`, `leptos+pagination`, `leptos+row`, `leptos+display` (20 results per query)
3. **WebSearch** — "leptos table component library 2025 2026", "leptos-struct-table tailwind v4 compatibility SSR", "leptos data grid virtual scroll table 2025", "leptos virtual list OR infinite scroll OR windowed list crate rust 2025", "leptodon table component data display leptos 2025 2026", "thaw leptos table component data grid SSR"
4. **Direct GitHub source** — Cargo.toml and key source files fetched verbatim for all candidates to verify claimed versions and styling approaches
5. **GitHub API** — last commit dates and contributor counts verified programmatically

---

## Candidates Found

### leptos-struct-table

- **crates.io:** https://crates.io/crates/leptos-struct-table
- **GitHub:** https://github.com/Synphonyte/leptos-struct-table
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8" }`
- **Styling approach:** Headless — no default styles. The `TableClassesProvider` trait is the extension point: consumers implement it or use a built-in preset. `TailwindClassesPreset` ships in the crate and produces hardcoded v3-named class strings (e.g., `"text-xs text-gray-700 uppercase bg-gray-200 dark:bg-gray-700 dark:text-gray-300"`). Full source verified from `src/class_providers/tailwind.rs`. Per-column overrides via `#[table(cell_class = "...", head_class = "...")]` field attributes append to whatever the provider produces. You can implement the trait yourself and emit any class strings you choose — the preset is opt-in.
- **Features:** Sorting (single and multi-column), virtualization (default), infinite scroll, pagination, async data loading, row selection (off/single/multi), editable cells via custom renderers, column drag-to-reorder, loading skeleton states
- **SSR support:** Yes. Requires `leptos-use` dependency with `"leptos-use/ssr"` in the ssr feature list. Working SSR example (`serverfn_sqlx`) ships in the repo.
- **Tailwind v4 notes:** `TailwindClassesPreset` emits hardcoded class name strings like `"bg-gray-200"`, `"dark:bg-gray-700"`, `"animate-pulse"`, `"w-[calc(85%-2.5rem)]"`. These are standard Tailwind utility names. In Tailwind v4 standalone mode, these classes still work as long as the source scanner can detect them in your Rust files. The preset file lives inside the crate's compiled code, not in your `src/` directory — so `@source "../src"` alone will NOT make Tailwind v4 scan the preset's class strings. Any class from the preset that is not independently referenced somewhere in your `src/` will be absent from the generated CSS. Workaround: use `@source inline()` in `style/tailwind.css` to declare the preset's class strings explicitly, or implement your own `TableClassesProvider` with classes already used elsewhere in your project.
- **Last commit:** 2026-02-03
- **Contributors:** 20
- **Notes:** Most mature Leptos table library. 70,453 total downloads. Uses `leptos-use 0.18` as a transitive dependency — already present in this project via `leptos_i18n`. Also depends on `send_wrapper`, `web-sys`, `wasm-bindgen`. The `leptos-windowing` and `leptos-pagination` crates (same author) are extracted from or designed for use with this library.

---

### Thaw (table component)

- **crates.io:** https://crates.io/crates/thaw
- **GitHub:** https://github.com/thaw-ui/thaw
- **Leptos dep in Cargo.toml:** `leptos = "0.8.5"` (workspace; crate latest is `0.5.0-beta`)
- **Styling approach:** Scoped CSS via a custom design token system (Fluent Design language). Styles live in per-component `.css` files (e.g., `thaw/src/table/table.css`) using CSS custom properties for colors and spacing. No Tailwind. The `Table` component mounts styles via `mount_style()` at render time and applies the `"thaw-table"` class to the `<table>` element. SSR requires wrapping the app root in `SSRMountStyleProvider` to inject CSS into `<head>` during server rendering.
- **Features:** Table, TableHeader, TableBody, TableRow, TableCell, TableHeaderCell (with drag-to-resize), TableCellLayout (with text truncation). No built-in sorting, filtering, pagination, or virtualization — it's a layout/styling component, not a data management component.
- **SSR support:** Yes. Feature flags: `csr`, `ssr`, `hydrate`. `SSRMountStyleProvider` required for CSS injection during SSR.
- **Tailwind v4 notes:** No Tailwind dependency or usage. Not applicable.
- **Last commit:** 2026-02-28
- **Contributors:** 23
- **Notes:** 117,379 total downloads. Active, well-maintained library. The table is a presentational structure component — it styles rows and columns but delegates data concerns entirely to the consumer. Integrating with this project would mean adopting Thaw's design token system alongside the existing oklch palette, which creates two parallel styling systems.

---

### Leptodon (table component)

- **crates.io:** https://crates.io/crates/leptodon
- **GitHub:** https://github.com/openanalytics/leptodon
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8.17", optional = true }`
- **Styling approach:** Leptodon uses `leptos-struct-table 0.18` internally and wraps it with its own `TailwindClassesPreset` implementation. The preset is identical in structure to the upstream one (same hardcoded Tailwind v3 class strings). The library generates `.tailwind` files containing all Leptodon source code for Tailwind to generate CSS against — this is a Tailwind v3 workflow assumption.
- **Features:** Full leptos-struct-table feature set (sorting, virtualization, pagination, etc.) plus a grouping module (`src/table/grouping.rs`). Also includes a `StyledHeadDragHandler` for visual column drag feedback.
- **SSR support:** Both demo and test apps are `leptos-ssr` applications. SSR is supported.
- **Tailwind v4 notes:** Explicitly uses Tailwind v3 workflows (`.tailwind` file generation). The hardcoded class strings carry the same v4 scanner problem as leptos-struct-table's preset. Additionally, the `.tailwind` file generation mechanism is a v3 content-path pattern incompatible with v4 standalone binary mode.
- **Last commit:** 2026-03-18
- **Contributors:** Not checked (Open Analytics NV project)
- **Notes:** Only 63 total downloads (very new — first published March 4, 2026). Essentially a thin wrapper around `leptos-struct-table` with Leptodon's design language applied. Adds the full Leptodon component suite dependency.

---

### leptos-shadcn-table

- **crates.io:** https://crates.io/crates/leptos-shadcn-table
- **GitHub:** https://github.com/cloud-shuttle/leptos-shadcn-ui (sub-crate at `packages/leptos/table`)
- **Leptos dep in Cargo.toml:** `leptos.workspace = true` (workspace sets `leptos = "0.8.9"`)
- **Styling approach:** Uses `tailwind_fuse` (variant-aware class merging utility) for conditional class composition. Classes are applied via props — the component accepts class strings from the consumer. Unstyled by default in the shadcn/ui sense: structure is provided, styling is consumer-supplied. No preset class strings baked in.
- **Features:** Structural table components only: `Table`, `TableHeader`, `TableBody`, `TableFooter`, `TableHead`, `TableRow`, `TableCell`, `TableCaption`. No sorting, filtering, pagination, or virtualization — pure presentation layer. Part of the larger `leptos-shadcn-ui` suite (25+ components).
- **SSR support:** Unknown from available sources. No SSR feature flags or documentation found. The library's description emphasizes TDD and WASM testing.
- **Tailwind v4 notes:** Uses `tailwind_fuse` for class merging, not preset strings. Consumer provides class names — compatible with any Tailwind version as long as the consumer writes valid classes.
- **Last commit:** 2026-01-10 (cloud-shuttle/leptos-shadcn-ui repo)
- **Contributors:** Not checked
- **Notes:** 3,731 total downloads. Thin presentational layer matching the shadcn/ui table API. No data management features.

---

### leptab

- **crates.io:** https://crates.io/crates/leptab
- **GitHub:** https://github.com/kodecraft-mark/leptab
- **Leptos dep in Cargo.toml:** `leptos = "0.6"`
- **Styling approach:** Hardcoded Tailwind CSS classes baked into the component. Not customizable via trait or props. The description says "styled by tailwindcss" but no mechanism for overriding classes is documented.
- **Features:** Sorting (ascending/descending), pagination, keyword search/filtering, CSV export, configurable rows-per-page
- **SSR support:** Not mentioned. CSR-only based on architecture (uses `on:input` signals, no SSR feature flags).
- **Tailwind v4 notes:** Requires Tailwind v3 (hardcoded class names). Targets Leptos 0.6, incompatible with this project.
- **Last commit:** 2024-06-03
- **Contributors:** 1
- **Notes:** 19,482 total downloads but only 90 recent — activity has flatlined. Targets Leptos 0.6; requires upgrade work to reach 0.8. Not actively maintained.

---

### leptos_datatable

- **crates.io:** https://crates.io/crates/leptos_datatable
- **GitHub:** https://github.com/mk0218/leptos-datatable
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.6.9", features = ["csr", "nightly"] }`
- **Styling approach:** Not documented. The crate accepts columns and data as props and renders an HTML table. No information on how CSS classes are applied.
- **Features:** Table rendering with data validation. No documented sorting, filtering, pagination, or virtualization.
- **SSR support:** No. `csr` feature only in Cargo.toml. No SSR feature flags.
- **Tailwind v4 notes:** Not applicable — targets Leptos 0.6, CSR only.
- **Last commit:** 2024-03-27
- **Contributors:** 1
- **Notes:** 3,900 total downloads. Abandoned alpha (version 0.1.1-alpha since March 2024). Targets Leptos 0.6 with nightly features.

---

### table-rs

- **crates.io:** https://crates.io/crates/table-rs
- **GitHub:** https://github.com/opensass/table-rs
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.7.7", optional = true }`
- **Styling approach:** Consumer-supplied classes via props. Described as "highly customizable."
- **Features:** Column sorting with `aria-sort`, real-time filtering with URL query parameter sync, debounced inputs, built-in pagination. README notes "🌱 Leptos Usage (TODO)" — the Leptos integration guide (`LEPTOS.md`) does not yet exist.
- **SSR support:** Unknown — no documentation found. The `lep` feature flag enables Leptos support; no ssr feature seen.
- **Tailwind v4 notes:** Classes passed in as props by the consumer, so framework version is irrelevant to the library itself.
- **Last commit:** 2026-01-17
- **Contributors:** Not checked (opensass org project)
- **Notes:** Only 1,813 total downloads. Targets Leptos 0.7; Leptos integration is explicitly marked as TODO in the README. Multi-framework (Yew, Dioxus, Leptos). Very early stage.

---

### leptos_virtual_scroller

- **crates.io:** https://crates.io/crates/leptos_virtual_scroller
- **GitHub:** https://github.com/Ovenoboyo/leptos_virtual_scroller
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8.11", features = ["csr"] }`
- **Styling approach:** Unstyled. Manages DOM rendering; styling is entirely consumer's responsibility.
- **Features:** `VirtualScroller` (list) and `VirtualGridScroller` (grid). Renders only viewport-visible items. Configurable item heights.
- **SSR support:** No. `csr` feature only; no SSR flags. Based on browser viewport measurement.
- **Tailwind v4 notes:** Not applicable — unstyled, no class generation.
- **Last commit:** 2025-12-25
- **Contributors:** 1
- **Notes:** Only 200 total downloads. Single maintainer, early stage (v0.1.1). CSR-only hard constraint because virtual scrolling requires measuring viewport dimensions in the browser.

---

### leptos-windowing + leptos-pagination

- **crates.io:** https://crates.io/crates/leptos-windowing / https://crates.io/crates/leptos-pagination
- **GitHub:** https://github.com/Synphonyte/leptos-windowing (same repo, two crates)
- **Leptos dep in Cargo.toml:** `leptos = "0.8"` (both crates)
- **Styling approach:** Unstyled primitives. Provides windowing/pagination state logic; rendering and styling are consumer's responsibility.
- **Features:** `leptos-windowing` — cached loading window for virtualization/pagination. `leptos-pagination` — pagination UI component built on top of `leptos-windowing`. Both have SSR feature flags.
- **SSR support:** Yes. Both crates have `ssr` feature that enables `leptos/ssr`, `leptos-use/ssr`, and `leptos-windowing/ssr`.
- **Tailwind v4 notes:** Unstyled — not applicable.
- **Last commit:** 2026-01-13
- **Contributors:** 1 (same author as leptos-struct-table)
- **Notes:** Very low downloads (563 and 398 respectively). These are internal building blocks used by `leptos-struct-table` and potentially useful standalone. `leptos-virtualization` is listed as TBD in the repo.

---

### leptos-infinity

- **crates.io:** https://crates.io/crates/leptos-infinity
- **GitHub:** https://github.com/Deepp0925/leptos_infinity
- **Leptos dep in Cargo.toml:** `leptos = {version = "0.4.8", features = ["wasm-bindgen", "csr"]}`
- **Styling approach:** Not documented.
- **Features:** "Optimization library for large list of DOM elements" — description only, no documentation found.
- **SSR support:** No. CSR-only.
- **Tailwind v4 notes:** Not applicable — targets Leptos 0.4.
- **Last commit:** 2023-08-11 (never updated past 0.0.0)
- **Contributors:** 1
- **Notes:** Published at version 0.0.0. Zero functional releases. Abandoned.

---

### ankurah-virtual-scroll

- **crates.io:** https://crates.io/crates/ankurah-virtual-scroll
- **GitHub:** https://github.com/ankurah/virtual-scroll
- **Leptos dep in Cargo.toml:** No Leptos dependency at workspace level. Framework-agnostic Rust/WASM state machine.
- **Styling approach:** Not applicable — pure state machine, no rendering.
- **Features:** Platform-agnostic virtual scroll state machine with bidirectional pagination, scroll position stability via intersection anchoring, three operational modes (Live/Backward/Forward). Documentation mentions Leptos/Dioxus can use `ScrollManager<V>` directly.
- **SSR support:** Unknown — designed for client-side infinite scroll behavior.
- **Tailwind v4 notes:** Not applicable.
- **Last commit:** 2026-01-23
- **Contributors:** Not checked (ankurah org)
- **Notes:** Only 280 total downloads. Low maturity. Framework-agnostic — requires building the rendering layer yourself in Leptos. Not a ready-to-use component.

---

## Landscape Summary

**Total candidates found:** 11 (1 from awesome-leptos index, 10 discovered via crates.io search and web search)

**Version compatibility with Leptos 0.8:**

| Library | Leptos version | Status |
|---------|----------------|--------|
| leptos-struct-table | `"0.8"` | Compatible |
| Thaw (table) | `"0.8.5"` | Compatible |
| Leptodon (wraps leptos-struct-table) | `"0.8.17"` | Compatible |
| leptos-shadcn-table | `"0.8.9"` (workspace) | Compatible |
| leptos-windowing / leptos-pagination | `"0.8"` | Compatible |
| leptos_virtual_scroller | `"0.8.11"` (CSR only) | CSR-only |
| table-rs | `"0.7.7"` | Version mismatch |
| leptab | `"0.6"` | Version mismatch |
| leptos_datatable | `"0.6.9"` | Version mismatch |
| leptos-infinity | `"0.4.8"` | Version mismatch |
| ankurah-virtual-scroll | none | Not Leptos-specific |

**SSR support among compatible libraries:**

- `leptos-struct-table`: Yes (requires `leptos-use/ssr` feature)
- `Thaw` (table): Yes (requires `SSRMountStyleProvider` wrapper)
- `Leptodon`: Yes
- `leptos-shadcn-table`: Unknown
- `leptos-windowing` / `leptos-pagination`: Yes
- `leptos_virtual_scroller`: No (CSR-only by design)

**Tailwind v4 scanner problem (critical for this project):**

The Tailwind v4 standalone binary scans source files for class strings. Libraries that embed hardcoded Tailwind class strings inside compiled crate code (not in the project's `src/` directory) are invisible to the scanner. This affects:

- `leptos-struct-table` `TailwindClassesPreset` — classes are in crate source, not project source
- `Leptodon` `TailwindClassesPreset` — same problem

Mitigation: implement a custom `TableClassesProvider` using only classes already present in the project's own source, or use `@source inline()` in `style/tailwind.css` to declare missing classes explicitly.

Libraries that take classes as consumer-supplied props (`leptos-shadcn-table` via `tailwind_fuse`) or are unstyled (`leptos-windowing`, `leptos_virtual_scroller`) have no scanner problem.

**Data management vs. presentation only:**

- Full data management (sort, filter, paginate, virtualize): `leptos-struct-table`, `leptab` (abandoned, Leptos 0.6)
- Presentation structure only: `Thaw` table, `leptos-shadcn-table`
- Pagination primitive: `leptos-pagination`
- Virtual scrolling primitive: `leptos_virtual_scroller` (CSR), `ankurah-virtual-scroll` (framework-agnostic)
- Reporting/static HTML: `report-leptos` (tangential — generates static reports, not interactive tables)

**Actively maintained (commit in last 3 months as of 2026-03-19):**

- `leptos-struct-table` (2026-02-03)
- `Thaw` (2026-02-28)
- `Leptodon` (2026-03-18)
- `table-rs` (2026-01-17)
- `leptos-windowing` / `leptos-pagination` (2026-01-13)
