# Radix-Leptos — Existence Verification

**Last Verified:** 2026-03-19

---

## TL;DR — Verdict First

**There ARE two distinct Radix implementations in the Rust/Leptos ecosystem:**

1. **`radix-leptos` (cloud-shuttle)** — ACTIVE, production-ready, Leptos 0.8.8, 57+ components, 1,792+ tests. Latest release Sept 22, 2025. Last commit Feb 26, 2026. **THIS ONE IS REAL AND ACTIVELY MAINTAINED.**

2. **`radix` (RustForWeb)** — ARCHIVED on Feb 2, 2026. Multi-framework (Dioxus, Leptos, Yew). Historical, not maintained. **THIS ONE IS ARCHIVED.**

The prior agent claims are CONTRADICTORY because they conflate these two projects. One is alive; the other is archived.

---

## Search Results

### Web Search 1: "radix leptos components library"

Found:
- [radix-leptos docs.rs](https://docs.rs/radix-leptos/latest/radix_leptos/)
- [radix-leptos crates.io](https://crates.io/crates/radix-leptos)
- [GitHub cloud-shuttle/radix-leptos](https://github.com/cloud-shuttle/radix-leptos/)
- [GitHub RustForWeb/radix](https://github.com/RustForWeb/radix)
- [awesome-leptos collection](https://github.com/leptos-rs/awesome-leptos)

### Web Search 2: "radix-leptos crate rust"

Confirmed 57+ components and 1,200+ tests claim via multiple secondary sources (libraries.io, crates.io metadata).

### Web Search 3: "cloud-shuttle radix leptos"

Confirmed:
- 57+ components
- 1,200+ tests (web search) / 1,792+ tests (GitHub README)
- 538KB optimized bundle
- Leptos 0.8 compatible
- Active development

### Web Search 4: '"rust radix" archived 2026'

Confirmed: **RustForWeb/radix archived Feb 2, 2026** — This is the truth that contradicted the first agent claim.

---

## Crates.io API Checks

### `radix-leptos`

**Status:** ✅ EXISTS

| Property | Value |
|----------|-------|
| Current Version | 0.9.0 |
| Released | September 22, 2025 |
| Last Updated | September 22, 2025 02:58:40 UTC |
| Total Versions | 15 |
| Total Downloads | 4,666 |
| Recent Downloads | 349 |
| License | MIT |
| Repository | https://github.com/cloud-shuttle/radix-leptos |
| Description | "Accessible, unstyled UI primitives for Leptos" |

**Leptos Version Target:** 0.8.8 (from Cargo.toml source)

### `leptos-shadcn-ui`

**Status:** ✅ EXISTS (separate library from cloud-shuttle)

| Property | Value |
|----------|-------|
| Current Version | 0.9.0 |
| Released | September 20, 2025 |
| Last Updated | September 20, 2025 02:40:32 UTC |
| Total Versions | 16 |
| Total Downloads | 5,107 |
| Recent Downloads | 513 |
| License | MIT |
| Repository | https://github.com/cloud-shuttle/leptos-shadcn-ui |
| Description | "Comprehensive collection of beautiful, accessible UI components built for Leptos v0.8+" |

**Note:** This is NOT the same as radix-leptos. It's a separate shadcn/ui port.

### `radix-leptos-primitives`

**Status:** ✅ EXISTS (modularity crate from radix-leptos workspace)

This is `crates/radix-leptos-primitives` in the cloud-shuttle/radix-leptos workspace.

### `radix-leptos-core`

**Status:** ✅ EXISTS (core utilities from radix-leptos workspace)

This is `crates/radix-leptos-core` in the cloud-shuttle/radix-leptos workspace.

---

## GitHub Checks

### `cloud-shuttle/radix-leptos`

**Status:** ✅ ACTIVE, NOT ARCHIVED

| Property | Value |
|----------|-------|
| Exists | Yes |
| Archived | No (`"archived": false`) |
| Last Commit | February 26, 2026 (pushed_at: "2026-02-26T20:07:32Z") |
| License | MIT |
| Stars | 16 |
| Open Issues | 5 |
| Description | "High-performance, accessible UI components for Leptos - Rust port of Radix UI primitives with 57+ components, 1200+ tests, and 538KB optimized bundle" |

**README Evidence:**
- 57+ components documented
- 1,792+ passing tests (from README, not 1,200+)
- Leptos 0.8.8 compatible (v0.8.5+)
- Type safety, zero JS runtime overhead
- Server-side rendering and hydration support
- Feature flags: `core`, `experimental`, `full` for bundle optimization

### `RustForWeb/radix`

**Status:** ✅ EXISTS BUT ARCHIVED (Feb 2, 2026)

| Property | Value |
|----------|-------|
| Exists | Yes |
| Archived | Yes (`"archived": true`) |
| Last Commit | February 2, 2026 (pushed_at: "2026-02-02T09:03:30Z") |
| License | MIT |
| Stars | 66 |
| Forks | 16 |
| Open Issues | 0 (frozen) |
| Description | "Rust port of Radix with support for Leptos and Yew." |
| Framework Support | Dioxus, Leptos, Yew (three frameworks) |

**Status Note:** Marked as "unmaintained" in README. Repository is read-only.

**When it was archived:** Exactly Feb 2, 2026 — before this conversation date of Mar 19, 2026.

---

## Actual State of Radix Primitives in Leptos Ecosystem

### Active Production Library

**[radix-leptos](https://crates.io/crates/radix-leptos)** by cloud-shuttle

- **Status:** PRODUCTION-READY, actively maintained
- **Leptos Version:** 0.8.8 (supports 0.8.5+)
- **Components:** 57+
- **Tests:** 1,792+ passing tests
- **Bundle Size:** 538KB optimized WASM
- **Accessibility:** WCAG 2.1 compliant, full ARIA support, keyboard navigation
- **Styling:** Unstyled primitives (like Radix UI) — you bring your own CSS
- **Bundle Size Control:** Feature flags (`core`, `experimental`, `full`)
- **SSR/Hydration:** Full support
- **Repository:** https://github.com/cloud-shuttle/radix-leptos
- **Latest Crate Release:** September 22, 2025
- **Latest Commit:** February 26, 2026
- **License:** MIT

**Components include:**
- Form components (buttons, inputs, selects, checkboxes, radio, etc.)
- Navigation elements (dropdowns, menus, popovers)
- Data display (tables, charts)
- Advanced features (drag-and-drop, rich text editing)
- Modals, drawers, dialogs
- Tabs, accordion, collapsible
- Context menus, tooltips
- Avatar, separator, label
- Toast notifications

### Archived / Unmaintained Library

**[radix](https://github.com/RustForWeb/radix)** by RustForWeb

- **Status:** ARCHIVED (Feb 2, 2026) — unmaintained, read-only
- **Framework Support:** Dioxus, Leptos, Yew (multi-framework approach)
- **Last Commit:** February 2, 2026
- **License:** MIT
- **Component Count:** Undocumented in README

**Why it matters:** This was the "Rust Radix" that some agents may have referenced. It's now archived. Use `radix-leptos` instead.

---

## Contradiction Resolution

**The conflicting claims were:**
1. "Radix-Leptos exists, v0.9, 57+ components, 1792+ tests" — ✅ TRUE (cloud-shuttle/radix-leptos)
2. "Rust Radix was archived on Feb 2, 2026" — ✅ TRUE (RustForWeb/radix)

**They are not contradictory.** They refer to two different projects:
- **radix-leptos** (cloud-shuttle) = the ACTIVE, production-ready library
- **radix** (RustForWeb) = the ARCHIVED, now-unmaintained library

An agent likely confused them or conflated the two while researching. The cloud-shuttle project is the one to use for new Leptos 0.8 applications.

---

## Verification Evidence URLs

### Primary Source — Cloud-Shuttle Radix-Leptos

| Resource | URL |
|----------|-----|
| GitHub Repository | https://github.com/cloud-shuttle/radix-leptos |
| Crates.io | https://crates.io/crates/radix-leptos |
| Docs.rs | https://docs.rs/radix-leptos/latest/radix_leptos/ |
| README | https://raw.githubusercontent.com/cloud-shuttle/radix-leptos/main/README.md |
| Cargo.toml | https://raw.githubusercontent.com/cloud-shuttle/radix-leptos/main/Cargo.toml |

### Secondary Source — RustForWeb Radix (Archived)

| Resource | URL |
|----------|-----|
| GitHub Repository | https://github.com/RustForWeb/radix |
| Crates.io | https://crates.io/crates/radix-rust |

### Discovery

| Resource | URL |
|----------|-----|
| awesome-leptos | https://github.com/leptos-rs/awesome-leptos |
| cloud-shuttle/leptos-shadcn-ui | https://github.com/cloud-shuttle/leptos-shadcn-ui (separate library) |

---

## Final Verdict

**Is there a production-ready Radix-like headless component library for Leptos 0.8?**

**YES. Definitely YES.**

**Library:** [radix-leptos](https://crates.io/crates/radix-leptos) by cloud-shuttle
- **Leptos 0.8.8 compatible** ✅
- **57+ components** ✅
- **1,792+ tests** ✅
- **Active maintenance** (last commit Feb 26, 2026) ✅
- **Production-ready** (v0.9.0, released Sept 22, 2025) ✅
- **Unstyled primitives** (like Radix UI) ✅
- **538KB WASM bundle** ✅
- **MIT licensed** ✅

This is the canonical Radix implementation for Leptos in 2026. Use it with confidence.
