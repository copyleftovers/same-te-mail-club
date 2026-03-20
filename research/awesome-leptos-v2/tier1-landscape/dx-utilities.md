# Tier 1: DX Utilities & Hooks Landscape

## Search Methodology

Sources queried:
- awesome-leptos index (all Tools, Quality of Life, Libraries, Alternate Macros categories)
- crates.io API searches: `leptos-use`, `leptos hooks`, `leptos form`, `leptos tracing wasm`, `leptos utility`, `leptos server fn`, `leptos middleware`, `leptos context`, `leptos derive`, `leptos macro`, `leptos signal`, `leptos validation`, `wasm tracing console`, `leptos store state`, `leptos async`, `leptos router type`, `leptos hot reload`, `leptos seo meta`, `leptos auth`, `leptos toast notification`, `leptos dark mode`
- Direct GitHub Cargo.toml reads for every candidate to verify Leptos version
- docs.rs and official documentation for hook surface area
- Cross-verification: version history checked to identify abandoned vs. active crates

**Leptos version check protocol:** All `Cargo.toml` extracted directly — README claims are not trusted.

---

## Candidates Found

### leptos-use

- **crates.io:** https://crates.io/crates/leptos-use
- **GitHub:** https://github.com/Synphonyte/leptos-use
- **Leptos dep in Cargo.toml:** `leptos = "0.8"`
- **Category:** hooks / reactive utilities
- **What it provides:** 89 composable hooks inspired by VueUse/React-Use. Key groups:
  - *Signal timing:* `signal_debounced`, `signal_throttled`, `watch_debounced`, `watch_throttled`, `watch_pausable`, `watch_with_options`, `whenever`
  - *Timing functions:* `use_debounce_fn`, `use_throttle_fn`, `use_timeout_fn`, `use_interval`, `use_interval_fn`, `use_raf_fn`, `use_timestamp`
  - *DOM/element:* `use_element_bounding`, `use_element_size`, `use_element_hover`, `use_element_visibility`, `use_resize_observer`, `use_mutation_observer`, `use_intersection_observer`
  - *Browser:* `use_media_query`, `use_breakpoints`, `use_window_size`, `use_window_scroll`, `use_window_focus`, `use_document`, `use_window`, `use_document_visibility`
  - *Preferences:* `use_preferred_dark`, `use_preferred_contrast`, `use_prefers_reduced_motion`, `use_color_mode`
  - *Events:* `use_event_listener`, `on_click_outside`
  - *Storage/comms:* `use_cookie`, `use_clipboard`, `use_broadcast_channel`, `use_websocket`
  - *Locale:* `use_locale`, `use_locales`, `use_intl_number_format`
  - *Reactive helpers:* `sync_signal`, `use_toggle`, `use_cycle_list`, `use_css_var`, `use_textarea_autosize`, `use_supported`, `is_err`, `is_none`, `is_ok`, `is_some`
- **Hooks relevant to project boilerplate:**
  - No `use_hydrated` / `use_mounted` hook exists — the `signal(false) + Effect::new` hydration gate cannot be replaced by this library
  - `signal_debounced` / `use_debounce_fn` — directly applicable if search-as-you-type is added later
  - `use_media_query` — responsive queries without writing browser API boilerplate
  - `use_preferred_dark` — system dark mode preference detection
  - `use_breakpoints` — viewport breakpoints with Tailwind preset (`Breakpoints::tailwind()`)
  - `use_window` / `use_document` — SSR-safe wrappers to avoid `window()` panics
  - `use_textarea_autosize` — auto-sizing textareas
  - **Note:** Already a transitive dependency via `leptos_i18n 0.6.1` — zero additional cost to use
- **Last commit:** 2026-02-26 (v0.18.3)
- **SSR + hydration:** Full SSR support via `ssr` feature; `axum` and `actix` feature flags for backend integration
- **Notes:** 89 functions with per-function feature flags. Currently pulled in transitively. The `leptos_i18n` dep already activates it, but feature flags must be explicitly opted in to use additional hooks.

---

### leptosfmt

- **crates.io:** https://crates.io/crates/leptosfmt
- **GitHub:** https://github.com/bram209/leptosfmt
- **Leptos dep in Cargo.toml:** No Leptos runtime dependency — it is a standalone CLI/formatter tool that parses `view!` macro syntax. It does NOT need Leptos as a Cargo dependency.
- **Category:** formatting / tooling
- **What it provides:** `view!` macro formatter for Leptos. Integrates with `rustfmt` as a post-processor or as a standalone CLI. Configurable via `leptosfmt.toml` (line width, tab spaces, indentation style, newline style, macro names). Editor integrations exist for VSCode, Neovim, Emacs. GitHub Action available (`leptosfmt-action`).
- **Hooks relevant to project boilerplate:** N/A — formatter only
- **Last commit:** 2025-01-30 (v0.1.33)
- **SSR + hydration:** N/A
- **Notes:** 21 releases since March 2023. 86,976 total downloads. Does not declare a Leptos version dependency — it operates as a syntax transformer on the macro AST. Works with any Leptos version including 0.8. No runtime coupling.

---

### leptos-mview

- **crates.io:** https://crates.io/crates/leptos-mview
- **GitHub:** https://github.com/blorbb/leptos-mview
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8.12", features = ["ssr", "nightly"] }` (dev dep; runtime users supply their own leptos)
- **Category:** alternate macro
- **What it provides:** A concise alternative to the standard `view!` macro, inspired by maud. Key differences: CSS selector shorthand (`h1.title` for class, `nav #id`), bracketed reactive values (`when=[signal]`), format shorthand (`f["{}",val]`), boolean attribute shorthand, semicolons for empty elements. Explicitly compatible with Leptos 0.5 through 0.8 via version matching table.
- **Hooks relevant to project boilerplate:** N/A — macro alternative only
- **Last commit:** 2026-02-26 (v0.5.0 for Leptos 0.8)
- **SSR + hydration:** Compatible with SSR (the macro generates standard Leptos view code)
- **Notes:** 14 releases, 16,209 total downloads. Very recent v0.5.0 published same day as leptos-use 0.18.3 (2026-02-26). Compatibility matrix: v0.5 → Leptos 0.8.

---

### tracing-subscriber-wasm

- **crates.io:** https://crates.io/crates/tracing-subscriber-wasm
- **GitHub:** https://github.com/jquesada2016/tracing_subscriber_wasm
- **Leptos dep in Cargo.toml:** None — Leptos-agnostic WASM tracing crate
- **Category:** tracing / WASM logging
- **What it provides:** A `MakeWriter` implementation for `tracing_subscriber` that routes log output to `console.log`/`console.error`/`console.warn` in the browser. Works with the standard `tracing` + `tracing_subscriber` ecosystem. Depends on `gloo 0.8`, `tracing 0.1`, `tracing-subscriber 0.3`. Usage: `MakeConsoleWriter::map_trace_level_to(...)` + `.without_time()` in initialization.
- **Hooks relevant to project boilerplate:**
  - Directly addresses "zero client-side logging" — would enable `tracing::info!` calls in WASM/hydration code to appear in browser console
  - Enables visibility into hydration issues, signal updates, server function errors on the client side
- **Last commit:** January 2023 (only 4 commits total)
- **SSR + hydration:** WASM-only; must be initialized in the WASM entry point, not the server binary
- **Notes:** Single version (v0.1.0), 2 open issues, 1 open PR — **inactive project**. Depends on `gloo 0.8` which may conflict with newer web-sys/gloo versions. 176,065 total downloads (historically used as the default WASM tracing solution). See `wasm-tracing` for an actively maintained fork.

---

### wasm-tracing

- **crates.io:** https://crates.io/crates/wasm-tracing
- **GitHub:** https://github.com/dsgallups/wasm-tracing
- **Leptos dep in Cargo.toml:** None — Leptos-agnostic WASM tracing crate
- **Category:** tracing / WASM logging
- **What it provides:** Actively maintained fork of `tracing-wasm`. Routes `tracing` spans/events to browser console AND browser `performance` API (for profiling in DevTools). Initialization: `wasm_tracing::set_as_global_default()`. Depends on `tracing 0.1`, `tracing-subscriber 0.3`, `wasm-bindgen 0.2`. Optional `mark-with-rayon-thread-index` feature for Rayon thread identification.
- **Hooks relevant to project boilerplate:**
  - Same use-case as `tracing-subscriber-wasm` but actively maintained
  - Browser performance API integration enables timing analysis of hydration
  - **Limitation:** Requires `console` and `performance` globals — does not work in Node.js or Cloudflare Workers
- **Last commit:** 2025-08-04 (v2.1.0)
- **SSR + hydration:** WASM-only
- **Notes:** 7 releases since September 2024, 57,159 total downloads. More recent and actively maintained than `tracing-subscriber-wasm`. Latest version 2.1.0 is the most downloaded at 29,959.

---

### leptos-fetch

- **crates.io:** https://crates.io/crates/leptos-fetch
- **GitHub:** https://github.com/zakstucke/leptos-fetch
- **Leptos dep in Cargo.toml:** `leptos = "0.8.13"` (workspace)
- **Category:** async data fetching / query manager
- **What it provides:** Async query manager with centralized `QueryClient` cache. Key features vs. standard `Resource`: request deduplication (multiple components sharing same query fire only one request), automatic background refetching, configurable cache lifetimes, optimistic updates, pagination helpers, optional devtools widget (`QueryDevtools` component visible in bottom-right corner in dev mode).
- **Hooks relevant to project boilerplate:**
  - Project uses `Resource::new(move || action.version().get(), ...)` for cache invalidation — leptos-fetch is a more feature-rich alternative for complex data requirements
  - Not a clear boilerplate reducer for the current 26 server functions; more applicable if query sharing across components is needed
- **Last commit:** 2026-03-02 (v0.4.10)
- **SSR + hydration:** `ssr` feature flag; thread-safe and thread-local cache variants for SSR
- **Notes:** 16 releases since March 2025, 15,259 total downloads. Active development. Features: `ssr`, `devtools`, `devtools-always`, `rkyv`.

---

### leptos_form_tool

- **crates.io:** https://crates.io/crates/leptos_form_tool
- **GitHub:** https://github.com/MitchellMarinoDev/leptos_form_tool
- **Leptos dep in Cargo.toml:** `leptos = "0.8"`
- **Category:** forms
- **What it provides:** Declarative, builder-pattern form construction with a `FormStyle` trait for rendering. Separates form structure from rendering — swap styles without touching logic. Supports cross-field validation, conditional field visibility, context-aware fields, custom components. Validation runs both client and server side.
- **Hooks relevant to project boilerplate:**
  - Project has 8 forms (~12 lines each of label+input+error+ARIA markup). This library replaces that markup with a builder API
  - Validation hook pattern similar to what project implements manually for some forms
  - **Important:** Uses its own state management internally, not `ActionForm` pattern. Unclear if it integrates with `FormData`-based ActionForm submissions
- **Last commit:** 2025-08-13 (v0.4.1)
- **SSR + hydration:** Targets Leptos 0.8 SSR ecosystem but no explicit SSR feature flag documented
- **Notes:** 8 releases since June 2024, 7,818 total downloads. Active but modest adoption.

---

### borang

- **crates.io:** https://crates.io/crates/borang
- **GitHub:** https://github.com/jonsaw/borang
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8" }`
- **Category:** forms + validation
- **What it provides:** Leptos form library with `#[derive(Validation)]` macro. Components: `Form`, `FormComponent`, `Field`, `Input`. Built-in validators: `required`, `email`. Form state management with field-level validation and reactive state binding.
- **Hooks relevant to project boilerplate:**
  - Addresses form boilerplate (label + input + error ARIA) in the project
  - Simple validator composition with derive macro
- **Last commit:** 2025-11-12 (v0.1.1)
- **SSR + hydration:** Targets Leptos 0.8
- **Notes:** 2 releases (v0.1.0, v0.1.1), very small project. Live demo at borang-leptos.vercel.app.

---

### formidable

- **crates.io:** https://crates.io/crates/formidable
- **GitHub:** https://github.com/fabianboesiger/formidable
- **Leptos dep in Cargo.toml:** `leptos = "0.8.10"` (workspace)
- **Category:** forms (derive)
- **What it provides:** Derive-based form generation from Rust structs and enums. Supports server actions, `leptos_i18n` integration, `time`, `url`, `color`, `bigdecimal` type support, custom email/phone/non-empty string types, Vec for repeating fields, enum-to-radio/select derivation.
- **Hooks relevant to project boilerplate:**
  - Struct-derived form generation could eliminate the 8 hand-written form field blocks
  - Built-in i18n integration aligns with the project's leptos_i18n usage
- **Last commit:** 2025-11-04 (v0.1.0)
- **SSR + hydration:** Targets Leptos 0.8 SSR (uses `#[server]` macro integration)
- **Notes:** Single release, brand new. No adoption data yet.

---

### leptos-forms-rs

- **crates.io:** https://crates.io/crates/leptos-forms-rs
- **GitHub:** https://github.com/cloud-shuttle/leptos-forms-rs
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8", features = ["csr"] }`
- **Category:** forms + validation
- **What it provides:** Signal-driven reactive form handling with per-field validation. Validators: `Required`, `Email`, `MinLength`, and custom validators. Field arrays, dynamic forms, `localStorage` persistence, 144 passing tests, pre-built component library.
- **Hooks relevant to project boilerplate:**
  - Uses signal-driven input (not ActionForm/FormData pattern)
  - **Critical compatibility concern:** Project guidance explicitly bans signal-driven form input for server function submission (see `guidance/leptos-idioms.md` § BANNED Pattern) — this library's approach conflicts with the project's mandatory ActionForm pattern
- **Last commit:** 2025-09-20 (v1.3.0)
- **SSR + hydration:** Features: `csr`, `ssr`, `hydrate`, `minimal`, `wasm-opt`
- **Notes:** 10 releases in 18 days (Sept 2025). Cloud-shuttle organization. Conflicts with project's ActionForm architectural constraint.

---

### leptos_form (tlowerison)

- **crates.io:** https://crates.io/crates/leptos_form
- **GitHub:** https://github.com/tlowerison/leptos_form
- **Leptos dep in Cargo.toml:** `leptos = "0.6.5"` (workspace)
- **Category:** forms (derive)
- **What it provides:** Derive forms from structs. Proc macro generates Leptos form components.
- **Hooks relevant to project boilerplate:** N/A — **Leptos 0.6 only, incompatible**
- **Last commit:** 2024-02-05 (v0.2.0-rc1, still RC)
- **SSR + hydration:** N/A
- **Notes:** Last release February 2024. Targets Leptos 0.6.5. **Do not use — incompatible with 0.8.**

---

### vld + vld-leptos

- **crates.io:** https://crates.io/crates/vld, https://crates.io/crates/vld-leptos
- **GitHub:** https://github.com/s00d/vld
- **Leptos dep in Cargo.toml (vld-leptos):** Not directly visible in root workspace — reported as 0.8 compatible per description ("shared validation for server functions and WASM clients")
- **Category:** validation (Zod-inspired, Leptos integration)
- **What it provides:** `vld`: Type-safe runtime validation inspired by Zod — `schema!` macro generates structs with `parse()`. Validators: email, URL, UUID, IPv4/v6, Base64, ISO datetime, CUID2, ULID, Nano ID. Error accumulation (all failures collected simultaneously). `vld-leptos`: Integration layer providing shared validation between server functions and WASM clients.
- **Hooks relevant to project boilerplate:**
  - Shared validation between server functions and WASM — no separate validation logic needed for server vs. client
  - 15 total downloads across both versions — extremely low adoption
- **Last commit:** 2026-03-19 (actively developed)
- **SSR + hydration:** Explicitly targets both server (server function) and WASM (client) validation
- **Notes:** `vld` is the validation core; 27 workspace integrations (axum, actix, diesel, tower, etc.) plus `vld-leptos`. Very new and very low adoption (15 downloads total for vld-leptos).

---

### leptos_async_signal

- **crates.io:** https://crates.io/crates/leptos_async_signal
- **GitHub:** https://github.com/demiurg-dev/leptos_async_signal
- **Leptos dep in Cargo.toml:** `leptos = "0.8.2"` (workspace)
- **Category:** async reactive signal
- **What it provides:** Combines a writable signal with a resource that resolves asynchronously. Designed for SSR: ensures resources are fully resolved before rendering, enabling elements like breadcrumbs to be generated before page render completes. Similar to `leptos_meta` but for arbitrary component data.
- **Hooks relevant to project boilerplate:** Not directly applicable to the project's current 6-phase implementation
- **Last commit:** 2025-05-11 (v0.6.0)
- **SSR + hydration:** SSR-focused design
- **Notes:** 8 releases since December 2024, 5,543 total downloads.

---

### leptos-store

- **crates.io:** https://crates.io/crates/leptos-store
- **GitHub:** https://github.com/nyvorin/leptos-store
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8", default-features = false }`
- **Category:** state management
- **What it provides:** Enterprise-grade type-enforced state management. Features: `ssr`, `csr`, `middleware`, `persistence`, devtools.
- **Hooks relevant to project boilerplate:** Not applicable — project has simple per-component state via signals and Resources
- **Last commit:** 2026-03-18 (v0.11.0)
- **SSR + hydration:** Full SSR support
- **Notes:** 14 releases. Very recently updated (2026-03-18). Relatively low downloads (529 recent).

---

### leptos_reactive_axum

- **crates.io:** https://crates.io/crates/leptos_reactive_axum
- **GitHub:** https://github.com/davidon-top/leptos_reactive_axum.git
- **Leptos dep in Cargo.toml:** Not explicitly in root Cargo.toml (404); per description targets Leptos+Axum integration
- **Category:** server-side context / Axum integration
- **What it provides:** Reactive context for Axum handlers — integrates Leptos reactive system within Axum request handlers. Features: `middleware` (default), `macros`, `nightly`. 76 lines across 3 files.
- **Hooks relevant to project boilerplate:**
  - Project does context extraction in 19 server functions (3 lines each). This library targets that pattern but appears focused on making Axum handlers reactive, not simplifying Leptos `expect_context` calls
- **Last commit:** 2024-10-20 (v1.0.1)
- **SSR + hydration:** Server-side only
- **Notes:** 3 releases, 3,330 total downloads. Last update October 2024. Small codebase (76 lines).

---

### leptos-unique-ids

- **crates.io:** https://crates.io/crates/leptos-unique-ids
- **GitHub:** Not provided in crate metadata
- **Leptos dep in Cargo.toml:** `">=0.8"` (minimum version 0.8)
- **Category:** accessibility / ARIA helpers
- **What it provides:** Generates globally unique DOM `id` attributes across the application. Prevents ID collisions that break ARIA relationships (`aria-labelledby`, `aria-describedby`). Features: `into-str`, `into-attribute-value`, `convert-case`.
- **Hooks relevant to project boilerplate:**
  - Project's form fields use `aria-describedby` for error messages — this library would guarantee uniqueness of those IDs, especially if form components are reused
  - 8 forms × multiple ARIA IDs = potential collision if components are instantiated more than once
- **Last commit:** 2025-06-16 (v0.1.1)
- **SSR + hydration:** Compatible with Leptos 0.8+ (including SSR)
- **Notes:** 2 releases, 1,346 total downloads. Published by mondeja (also author of `leptos-fluent`).

---

### leptos-animate

- **crates.io:** https://crates.io/crates/leptos_animate
- **GitHub:** https://github.com/brofrain/leptos-animate
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8.17" }` (workspace)
- **Category:** animation utilities
- **What it provides:** FLIP animations, in/out transitions, custom animation helpers for Leptos.
- **Hooks relevant to project boilerplate:** Not applicable — project has no animations (design system specifies `120ms ease` transitions only)
- **Last commit:** 2026-03-14 (v0.1.12)
- **SSR + hydration:** Targets Leptos 0.8.17
- **Notes:** 13 releases, 4,258 total downloads. Actively maintained.

---

### leptoaster

- **crates.io:** https://crates.io/crates/leptoaster
- **GitHub:** https://github.com/KiaShakiba/leptoaster
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8" }`
- **Category:** toast notifications
- **What it provides:** Minimal toast notification library for Leptos.
- **Hooks relevant to project boilerplate:** Not applicable — project has no modals, toasts, or notifications (confirmed absent in spec)
- **Last commit:** 2025-05-23 (v0.2.3)
- **SSR + hydration:** Targets Leptos 0.8
- **Notes:** 12 releases, 14,690 total downloads.

---

### tailwind-fuse

- **crates.io:** https://crates.io/crates/tailwind-fuse
- **GitHub:** https://github.com/gaucho-labs/tailwind-fuse
- **Leptos dep in Cargo.toml:** None — Tailwind utility crate with no Leptos runtime dependency (Leptos is only in keywords)
- **Category:** Tailwind CSS utilities
- **What it provides:** Merge and deduplicate Tailwind CSS class strings (like `tailwind-merge` for JS), plus variant generation similar to `cva`. Works with any Leptos version or no Leptos at all.
- **Hooks relevant to project boilerplate:**
  - Project uses `data-*` attribute + CSS custom property variant pattern (`.btn[data-variant="secondary"]`) rather than class-based variants — this library is less relevant
  - Useful if conditional class merging is needed (e.g., `class="btn {extra_classes}"` where conflicts need resolution)
- **Last commit:** 2025-01-19 (v0.3.2)
- **SSR + hydration:** N/A — pure string utility
- **Notes:** 47,731 total downloads. No Leptos version dependency. Works independently.

---

### leptos-darkmode (kerkmann)

- **crates.io:** https://crates.io/crates/leptos_darkmode (note: crate name uses underscore)
- **GitHub:** https://gitlab.com/kerkmann/leptos_darkmode
- **Leptos dep in Cargo.toml:** `leptos = { version = "0.8", default-features = false }`
- **Category:** dark mode / theme
- **What it provides:** Helper for managing Tailwind CSS dark mode in Leptos apps. Stores preference in localStorage. Tailwind-specific (adds/removes `dark` class on `<html>`).
- **Hooks relevant to project boilerplate:**
  - Project uses CSS `prefers-color-scheme: dark` only (system preference, no toggle) — this library handles the manual toggle case which the project explicitly does NOT implement
- **Last commit:** 2025-06-23 (v0.4.0)
- **SSR + hydration:** Leptos 0.8
- **Notes:** 12,100 total downloads. Redundant with `use_color_mode` from leptos-use for the toggle use case.

---

### leptos_query (nicoburniske) — STALE

- **crates.io:** https://crates.io/crates/leptos_query
- **GitHub:** https://github.com/nicoburniske/leptos_query
- **Leptos dep in Cargo.toml:** `leptos = "0.6"` (workspace)
- **Category:** async query manager — **Leptos 0.6, incompatible with 0.8**
- **What it provides:** Query management with caching, devtools, storage. Superseded by `leptos-fetch` for Leptos 0.8.
- **Hooks relevant to project boilerplate:** N/A — **incompatible**
- **Last commit:** 2024-03-09 (last release)
- **SSR + hydration:** N/A
- **Notes:** 47,500 total downloads but **targets Leptos 0.6**. **Do not use — incompatible with 0.8.** Use `leptos-fetch` instead.

---

### leptos-tracked — STALE

- **crates.io:** https://crates.io/crates/leptos-tracked
- **GitHub:** https://github.com/remkop22/leptos-tracked
- **Leptos dep in Cargo.toml:** `leptos_reactive = { version = "0.1" }` — targets Leptos 0.1-era reactive internals
- **Category:** signal utilities — **pre-0.8, incompatible**
- **What it provides:** Utility traits for composing Leptos signals with fewer nested closures.
- **Hooks relevant to project boilerplate:** N/A — **incompatible**
- **Last commit:** 2023-03-07 (all 5 releases on same day)
- **SSR + hydration:** N/A
- **Notes:** **Do not use — targets Leptos 0.1-era reactive primitives.**

---

### leptos-signals (akesson) — STALE

- **crates.io:** https://crates.io/crates/leptos-signals
- **GitHub:** https://github.com/akesson/leptos-signals
- **Leptos dep in Cargo.toml:** `leptos_reactive = { version = "0.1", default-features = false }` — targets Leptos 0.1 era
- **Category:** signal utilities — **pre-0.8, incompatible**
- **What it provides:** Various signal primitives.
- **Hooks relevant to project boilerplate:** N/A — **incompatible**
- **Last commit:** 2023-01-13
- **Notes:** **Do not use — targets Leptos 0.1-era primitives.**

---

### leptos_toaster (SorenHolstHansen) — WRONG VERSION

- **crates.io:** https://crates.io/crates/leptos_toaster
- **GitHub:** https://github.com/SorenHolstHansen/leptos_toaster
- **Leptos dep in Cargo.toml:** `leptos = "0.7"`
- **Category:** toast — Leptos 0.7, incompatible with 0.8
- **Notes:** **Do not use — targets Leptos 0.7.**

---

### leptos-hotkeys — WRONG VERSION

- **crates.io:** https://crates.io/crates/leptos-hotkeys
- **GitHub:** https://github.com/gaucho-labs/leptos-hotkeys
- **Leptos dep in Cargo.toml:** Workspace root has no leptos; individual crate dep: `leptos = "0.6"` per workspace root analysis
- **Category:** keyboard shortcuts — Leptos 0.6
- **Last commit:** 2024-07-04 (v0.2.2)
- **Notes:** **Do not use — targets Leptos 0.6.**

---

### tracing-wasm (storyai) — ABANDONED

- **crates.io:** https://crates.io/crates/tracing-wasm
- **GitHub:** https://github.com/storyai/tracing-wasm
- **Leptos dep in Cargo.toml:** None
- **Category:** tracing / WASM — abandoned original
- **Last commit:** 2021-11-07 (v0.2.1)
- **Notes:** **Abandoned.** `wasm-tracing` is the maintained fork.

---

### leptos-routes (lpotthast)

- **crates.io:** https://crates.io/crates/leptos-routes
- **GitHub:** https://github.com/lpotthast/leptos-routes
- **Leptos dep in Cargo.toml:** Dev dep only: `leptos = { version = "0.7.7", features = ["ssr"] }` — no runtime leptos dependency; the macro generates code that references leptos symbols in the user's project
- **Category:** type-safe routing / code generation
- **What it provides:** Proc macro generating structs for each route from Rust module hierarchy. Eliminates string-based route references. `materialize()` method for URL generation with dynamic segments.
- **Hooks relevant to project boilerplate:** Project has 7 routes with hardcoded string paths. This library would generate type-safe route references.
- **Last commit:** 2025-02-23 (v0.3.1)
- **SSR + hydration:** MSRV 1.85.0 (Rust 2024 edition). No runtime leptos version dependency.
- **Notes:** 3 releases, 2,283 total downloads. Dev dep targets Leptos 0.7 for tests, but generated code is version-agnostic. Low adoption.

---

### leptos-routable

- **crates.io:** https://crates.io/crates/leptos-routable
- **GitHub:** Not provided in crate metadata
- **Leptos dep in Cargo.toml:** Not available
- **Category:** type-safe routing
- **What it provides:** "Type-safe routing utility for Leptos with zero-string path generation"
- **Hooks relevant to project boilerplate:** Similar to leptos-routes — eliminates string path literals
- **Last commit:** 2025-03-13 (v0.1.1)
- **SSR + hydration:** Unknown — no repository accessible
- **Notes:** 3,548 total downloads, 2 releases. No repository link in crate metadata.

---

### leptos_animate (brofrain) — already listed above

### leptos-fetch (zakstucke) — already listed above

### leptos_async_signal (demiurg-dev) — already listed above

---

## Landscape Summary

### Verified Leptos 0.8 Compatible (active)

| Library | Category | Last Release | Key Value |
|---------|----------|--------------|-----------|
| leptos-use 0.18.3 | hooks / reactive | 2026-02-26 | 89 hooks; already transitive dep via leptos_i18n |
| leptosfmt 0.1.33 | formatting | 2025-01-30 | view! macro formatter; no Leptos runtime dep |
| leptos-mview 0.5.0 | alternate macro | 2026-02-26 | Concise view! alternative |
| wasm-tracing 2.1.0 | WASM logging | 2025-08-04 | Active tracing-wasm fork; browser console + perf API |
| tracing-subscriber-wasm 0.1.0 | WASM logging | 2023-01-31 | Inactive; alternative to wasm-tracing |
| leptos-fetch 0.4.10 | async data | 2026-03-02 | Query cache; deduplication; devtools |
| leptos_form_tool 0.4.1 | forms | 2025-08-13 | Builder-pattern forms with validation |
| borang 0.1.1 | forms + validation | 2025-11-12 | Derive validation + form components |
| formidable 0.1.0 | forms (derive) | 2025-11-04 | Struct-derived forms; i18n integration |
| leptos-forms-rs 1.3.0 | forms + validation | 2025-09-20 | Signal-driven (conflicts with ActionForm mandate) |
| vld + vld-leptos 0.2.0 | validation | 2026-03-19 | Zod-inspired; 15 total downloads |
| leptos_async_signal 0.6.0 | async signal | 2025-05-11 | SSR-resolved async signals |
| leptos-store 0.11.0 | state mgmt | 2026-03-18 | Enterprise state management |
| leptoaster 0.2.3 | toasts | 2025-05-23 | Minimal toast (project has no toasts) |
| tailwind-fuse 0.3.2 | Tailwind utilities | 2025-01-19 | Class merge + variants (no Leptos dep) |
| leptos-unique-ids 0.1.1 | ARIA / accessibility | 2025-06-16 | Unique DOM IDs for aria-describedby |
| leptos-animate 0.1.12 | animation | 2026-03-14 | FLIP/transitions (project has no animations) |
| leptos-routes 0.3.1 | routing | 2025-02-23 | Type-safe route structs (low adoption) |
| leptos-darkmode 0.4.0 | dark mode | 2025-06-23 | Manual toggle (project uses system pref only) |
| leptos-unique-ids 0.1.1 | accessibility | 2025-06-16 | Global unique IDs for ARIA |

### Incompatible (wrong Leptos version)

| Library | Targets | Status |
|---------|---------|--------|
| leptos_query (nicoburniske) | Leptos 0.6 | Abandoned — use leptos-fetch |
| leptos_form (tlowerison) | Leptos 0.6.5 | Stale RC |
| leptos-tracked | Leptos 0.1 | Abandoned |
| leptos-signals (akesson) | Leptos 0.1 | Abandoned |
| leptos_toaster (SorenHolstHansen) | Leptos 0.7 | Needs update |
| leptos-hotkeys | Leptos 0.6 | Stale |
| tracing-wasm (storyai) | Any (abandoned) | Abandoned original |

### Hydration Gate Finding

No library in the Leptos ecosystem (including leptos-use 0.18.3 with 89 hooks) provides a `use_hydrated()` or `use_mounted()` hook that could replace the project's `signal(false) + Effect::new(move |_| set_hydrated.set(true))` pattern (9 instances). This pattern is the standard Leptos idiom for SSR/CSR hydration detection and must be implemented manually. `leptos-use` confirms this gap: its closest utilities (`use_window`, `use_document`, `use_supported`) detect browser API availability but not the hydration transition specifically.

### Server Function Context Extraction Finding

No dedicated library exists for simplifying Leptos server function context extraction (`expect_context::<sqlx::PgPool>()`, `expect_context::<Config>()`). This is idiomatic Leptos — `provide_context`/`expect_context` is the built-in DI mechanism. The project's 19 × 3-line context extraction pattern is the correct approach with no library alternative found.
